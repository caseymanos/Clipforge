use crate::models::{Timeline, Track, Clip, TrackType, Resolution};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;
use log::{info, warn, error};

/// Timeline service for managing non-destructive video editing
pub struct TimelineService {
    current_timeline: Option<Timeline>,
    project_path: Option<PathBuf>,
}

/// Timeline operation errors
#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
    #[error("Timeline not found")]
    TimelineNotFound,

    #[error("Track not found: {0}")]
    TrackNotFound(String),

    #[error("Clip not found: {0}")]
    ClipNotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Overlap detected: {0}")]
    OverlapError(String),
}

/// Project file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub version: String,
    pub timeline: Timeline,
    pub created_at: String,
    pub modified_at: String,
}

impl TimelineService {
    /// Create a new timeline service
    pub fn new() -> Self {
        Self {
            current_timeline: None,
            project_path: None,
        }
    }

    /// Create a new timeline
    pub fn create_timeline(
        &mut self,
        name: String,
        framerate: f64,
        resolution: Resolution,
    ) -> Result<Timeline, TimelineError> {
        let timeline = Timeline {
            id: Uuid::new_v4().to_string(),
            name,
            framerate,
            resolution,
            tracks: vec![
                // Default video track
                Track {
                    id: Uuid::new_v4().to_string(),
                    track_type: TrackType::Video,
                    clips: Vec::new(),
                    muted: false,
                    locked: false,
                },
                // Default audio track
                Track {
                    id: Uuid::new_v4().to_string(),
                    track_type: TrackType::Audio,
                    clips: Vec::new(),
                    muted: false,
                    locked: false,
                },
            ],
            duration: 0.0,
            subtitle_track: None,
            subtitle_enabled: false,
        };

        self.current_timeline = Some(timeline.clone());
        info!("Created new timeline: {}", timeline.name);

        Ok(timeline)
    }

    /// Get current timeline
    pub fn get_timeline(&self) -> Result<&Timeline, TimelineError> {
        self.current_timeline.as_ref()
            .ok_or(TimelineError::TimelineNotFound)
    }

    /// Get mutable timeline
    fn get_timeline_mut(&mut self) -> Result<&mut Timeline, TimelineError> {
        self.current_timeline.as_mut()
            .ok_or(TimelineError::TimelineNotFound)
    }

    /// Add a new track to the timeline
    pub fn add_track(
        &mut self,
        track_type: TrackType,
    ) -> Result<String, TimelineError> {
        let timeline = self.get_timeline_mut()?;

        let track = Track {
            id: Uuid::new_v4().to_string(),
            track_type,
            clips: Vec::new(),
            muted: false,
            locked: false,
        };

        let track_id = track.id.clone();
        timeline.tracks.push(track);

        info!("Added new {:?} track: {}", track_type, track_id);
        Ok(track_id)
    }

    /// Remove a track from the timeline
    pub fn remove_track(&mut self, track_id: &str) -> Result<(), TimelineError> {
        let timeline = self.get_timeline_mut()?;

        let index = timeline.tracks.iter()
            .position(|t| t.id == track_id)
            .ok_or_else(|| TimelineError::TrackNotFound(track_id.to_string()))?;

        timeline.tracks.remove(index);
        info!("Removed track: {}", track_id);

        Ok(())
    }

    /// Add a clip to a track
    pub fn add_clip(
        &mut self,
        track_id: &str,
        clip: Clip,
    ) -> Result<(), TimelineError> {
        let timeline = self.get_timeline_mut()?;

        let track = timeline.tracks.iter_mut()
            .find(|t| t.id == track_id)
            .ok_or_else(|| TimelineError::TrackNotFound(track_id.to_string()))?;

        if track.locked {
            return Err(TimelineError::InvalidOperation(
                format!("Track {} is locked", track_id)
            ));
        }

        // Check for overlaps
        if let Some(overlap) = Self::check_overlap(track, &clip) {
            warn!("Overlap detected when adding clip");
            return Err(TimelineError::OverlapError(overlap));
        }

        track.clips.push(clip.clone());

        // Update timeline duration
        let clip_end = clip.track_position + clip.duration;
        if clip_end > timeline.duration {
            timeline.duration = clip_end;
        }

        info!("Added clip {} to track {}", clip.id, track_id);
        Ok(())
    }

    /// Remove a clip from the timeline
    pub fn remove_clip(
        &mut self,
        clip_id: &str,
    ) -> Result<(), TimelineError> {
        let timeline = self.get_timeline_mut()?;

        for track in &mut timeline.tracks {
            if let Some(index) = track.clips.iter().position(|c| c.id == clip_id) {
                track.clips.remove(index);

                // Recalculate timeline duration
                timeline.duration = Self::calculate_duration(&timeline.tracks);

                info!("Removed clip: {}", clip_id);
                return Ok(());
            }
        }

        Err(TimelineError::ClipNotFound(clip_id.to_string()))
    }

    /// Move a clip to a new position or track
    pub fn move_clip(
        &mut self,
        clip_id: &str,
        new_track_id: &str,
        new_position: f64,
    ) -> Result<(), TimelineError> {
        let timeline = self.get_timeline_mut()?;

        // Find and remove clip from current track
        let mut clip_to_move: Option<Clip> = None;
        for track in &mut timeline.tracks {
            if let Some(index) = track.clips.iter().position(|c| c.id == clip_id) {
                clip_to_move = Some(track.clips.remove(index));
                break;
            }
        }

        let mut clip = clip_to_move
            .ok_or_else(|| TimelineError::ClipNotFound(clip_id.to_string()))?;

        // Update position
        clip.track_position = new_position;

        // Find new track
        let new_track = timeline.tracks.iter_mut()
            .find(|t| t.id == new_track_id)
            .ok_or_else(|| TimelineError::TrackNotFound(new_track_id.to_string()))?;

        if new_track.locked {
            return Err(TimelineError::InvalidOperation(
                format!("Target track {} is locked", new_track_id)
            ));
        }

        // Check for overlaps in new track
        if let Some(overlap) = Self::check_overlap(new_track, &clip) {
            warn!("Overlap detected when moving clip");
            return Err(TimelineError::OverlapError(overlap));
        }

        new_track.clips.push(clip.clone());

        // Recalculate timeline duration
        timeline.duration = Self::calculate_duration(&timeline.tracks);

        info!("Moved clip {} to track {} at position {}", clip_id, new_track_id, new_position);
        Ok(())
    }

    /// Trim a clip (adjust in/out points)
    pub fn trim_clip(
        &mut self,
        clip_id: &str,
        trim_start: Option<f64>,
        trim_end: Option<f64>,
    ) -> Result<(), TimelineError> {
        let timeline = self.get_timeline_mut()?;

        for track in &mut timeline.tracks {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                if let Some(start) = trim_start {
                    if start < 0.0 || start >= clip.trim_end {
                        return Err(TimelineError::InvalidOperation(
                            format!("Invalid trim_start: {}", start)
                        ));
                    }
                    clip.trim_start = start;
                }

                if let Some(end) = trim_end {
                    if end <= clip.trim_start {
                        return Err(TimelineError::InvalidOperation(
                            format!("Invalid trim_end: {}", end)
                        ));
                    }
                    clip.trim_end = end;
                }

                // Update duration based on trim points
                clip.duration = (clip.trim_end - clip.trim_start) / clip.speed as f64;

                // Recalculate timeline duration
                timeline.duration = Self::calculate_duration(&timeline.tracks);

                info!("Trimmed clip {}: start={:?}, end={:?}", clip_id, trim_start, trim_end);
                return Ok(());
            }
        }

        Err(TimelineError::ClipNotFound(clip_id.to_string()))
    }

    /// Split a clip at a given time
    pub fn split_clip(
        &mut self,
        clip_id: &str,
        split_time: f64,
    ) -> Result<(String, String), TimelineError> {
        let timeline = self.get_timeline_mut()?;

        for track in &mut timeline.tracks {
            if let Some(index) = track.clips.iter().position(|c| c.id == clip_id) {
                let original_clip = &track.clips[index];

                // Validate split time
                let clip_start = original_clip.track_position;
                let clip_end = clip_start + original_clip.duration;

                if split_time <= clip_start || split_time >= clip_end {
                    return Err(TimelineError::InvalidOperation(
                        format!("Split time {} is outside clip bounds [{}, {}]",
                            split_time, clip_start, clip_end)
                    ));
                }

                let split_offset = split_time - clip_start;
                let source_split_time = original_clip.trim_start + (split_offset * original_clip.speed as f64);

                // Create first part (before split)
                let first_clip = Clip {
                    id: Uuid::new_v4().to_string(),
                    media_file_id: original_clip.media_file_id.clone(),
                    name: original_clip.name.clone(),
                    track_position: original_clip.track_position,
                    duration: split_offset,
                    trim_start: original_clip.trim_start,
                    trim_end: source_split_time,
                    effects: original_clip.effects.clone(),
                    volume: original_clip.volume,
                    speed: original_clip.speed,
                };

                // Create second part (after split)
                let second_clip = Clip {
                    id: Uuid::new_v4().to_string(),
                    media_file_id: original_clip.media_file_id.clone(),
                    name: original_clip.name.clone(),
                    track_position: split_time,
                    duration: original_clip.duration - split_offset,
                    trim_start: source_split_time,
                    trim_end: original_clip.trim_end,
                    effects: original_clip.effects.clone(),
                    volume: original_clip.volume,
                    speed: original_clip.speed,
                };

                let first_id = first_clip.id.clone();
                let second_id = second_clip.id.clone();

                // Replace original clip with split clips
                track.clips.remove(index);
                track.clips.insert(index, second_clip);
                track.clips.insert(index, first_clip);

                info!("Split clip {} at {} into {} and {}", clip_id, split_time, first_id, second_id);
                return Ok((first_id, second_id));
            }
        }

        Err(TimelineError::ClipNotFound(clip_id.to_string()))
    }

    /// Get clips at a specific time (playhead position)
    pub fn get_clips_at_time(&self, time: f64) -> Result<Vec<Clip>, TimelineError> {
        let timeline = self.get_timeline()?;

        let mut clips_at_time = Vec::new();

        for track in &timeline.tracks {
            if track.muted {
                continue;
            }

            for clip in &track.clips {
                let clip_start = clip.track_position;
                let clip_end = clip_start + clip.duration;

                if time >= clip_start && time < clip_end {
                    clips_at_time.push(clip.clone());
                }
            }
        }

        Ok(clips_at_time)
    }

    /// Save timeline to project file
    pub fn save_project(&mut self, path: PathBuf) -> Result<(), TimelineError> {
        let timeline = self.get_timeline()?.clone();

        let project = Project {
            version: "1.0.0".to_string(),
            timeline,
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string_pretty(&project)?;
        fs::write(&path, json)?;

        self.project_path = Some(path.clone());
        info!("Saved project to: {:?}", path);

        Ok(())
    }

    /// Load timeline from project file
    pub fn load_project(&mut self, path: PathBuf) -> Result<Timeline, TimelineError> {
        let json = fs::read_to_string(&path)?;
        let project: Project = serde_json::from_str(&json)?;

        self.current_timeline = Some(project.timeline.clone());
        self.project_path = Some(path.clone());

        info!("Loaded project from: {:?}", path);
        Ok(project.timeline)
    }

    /// Check if a clip overlaps with existing clips in the track
    fn check_overlap(track: &Track, new_clip: &Clip) -> Option<String> {
        let new_start = new_clip.track_position;
        let new_end = new_start + new_clip.duration;

        for existing_clip in &track.clips {
            // Skip checking against itself
            if existing_clip.id == new_clip.id {
                continue;
            }

            let existing_start = existing_clip.track_position;
            let existing_end = existing_start + existing_clip.duration;

            // Check for overlap
            if new_start < existing_end && new_end > existing_start {
                return Some(format!(
                    "Clip overlaps with existing clip {} at position {}",
                    existing_clip.id, existing_start
                ));
            }
        }

        None
    }

    /// Calculate total duration of timeline
    fn calculate_duration(tracks: &[Track]) -> f64 {
        let mut max_duration = 0.0;

        for track in tracks {
            for clip in &track.clips {
                let clip_end = clip.track_position + clip.duration;
                if clip_end > max_duration {
                    max_duration = clip_end;
                }
            }
        }

        max_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_clip(position: f64, duration: f64) -> Clip {
        Clip {
            id: Uuid::new_v4().to_string(),
            media_file_id: "test-media".to_string(),
            name: Some("test-clip.mp4".to_string()),
            track_position: position,
            duration,
            trim_start: 0.0,
            trim_end: duration,
            effects: Vec::new(),
            volume: 1.0,
            speed: 1.0,
        }
    }

    #[test]
    fn test_create_timeline() {
        let mut service = TimelineService::new();
        let timeline = service.create_timeline(
            "Test Timeline".to_string(),
            30.0,
            Resolution { width: 1920, height: 1080 },
        ).unwrap();

        assert_eq!(timeline.name, "Test Timeline");
        assert_eq!(timeline.framerate, 30.0);
        assert_eq!(timeline.tracks.len(), 2);
    }

    #[test]
    fn test_add_remove_clip() {
        let mut service = TimelineService::new();
        service.create_timeline(
            "Test".to_string(),
            30.0,
            Resolution { width: 1920, height: 1080 },
        ).unwrap();

        let track_id = service.get_timeline().unwrap().tracks[0].id.clone();
        let clip = create_test_clip(0.0, 5.0);
        let clip_id = clip.id.clone();

        // Add clip
        service.add_clip(&track_id, clip).unwrap();
        let timeline = service.get_timeline().unwrap();
        assert_eq!(timeline.tracks[0].clips.len(), 1);

        // Remove clip
        service.remove_clip(&clip_id).unwrap();
        let timeline = service.get_timeline().unwrap();
        assert_eq!(timeline.tracks[0].clips.len(), 0);
    }

    #[test]
    fn test_split_clip() {
        let mut service = TimelineService::new();
        service.create_timeline(
            "Test".to_string(),
            30.0,
            Resolution { width: 1920, height: 1080 },
        ).unwrap();

        let track_id = service.get_timeline().unwrap().tracks[0].id.clone();
        let clip = create_test_clip(0.0, 10.0);
        let clip_id = clip.id.clone();

        service.add_clip(&track_id, clip).unwrap();

        // Split at 5.0 seconds
        let (first_id, second_id) = service.split_clip(&clip_id, 5.0).unwrap();

        let timeline = service.get_timeline().unwrap();
        let clips = &timeline.tracks[0].clips;

        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].duration, 5.0);
        assert_eq!(clips[1].duration, 5.0);
        assert_eq!(clips[1].track_position, 5.0);
    }

    #[test]
    fn test_overlap_detection() {
        let mut service = TimelineService::new();
        service.create_timeline(
            "Test".to_string(),
            30.0,
            Resolution { width: 1920, height: 1080 },
        ).unwrap();

        let track_id = service.get_timeline().unwrap().tracks[0].id.clone();

        // Add first clip at 0-5
        let clip1 = create_test_clip(0.0, 5.0);
        service.add_clip(&track_id, clip1).unwrap();

        // Try to add overlapping clip at 3-8
        let clip2 = create_test_clip(3.0, 5.0);
        let result = service.add_clip(&track_id, clip2);

        assert!(result.is_err());
    }
}
