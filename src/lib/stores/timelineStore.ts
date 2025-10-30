import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types matching backend data structures
export interface Timeline {
    id: string;
    name: string;
    framerate: number;
    resolution: {
        width: number;
        height: number;
    };
    tracks: Track[];
    duration: number;
}

export interface Track {
    id: string;
    track_type: 'Video' | 'Audio' | 'Overlay';
    clips: Clip[];
    muted: boolean;  // Backend uses 'muted' not 'enabled'
    locked: boolean;
}

export interface Clip {
    id: string;
    media_file_id: string;
    name?: string;           // Display name (typically filename)
    track_position: number;  // Start time in seconds
    duration: number;        // Clip duration in seconds
    trim_start: number;      // Trim from source start
    trim_end: number;        // Trim from source end
    effects: Effect[];
    volume: number;
    speed: number;
}

// Effect structure matching backend models.rs
export interface Effect {
    id: string;
    effect_type: EffectType;
    enabled: boolean;
}

// EffectType enum matching backend (tagged union)
export type EffectType =
    | { type: 'Brightness'; value: number }
    | { type: 'Contrast'; value: number }
    | { type: 'Saturation'; value: number }
    | { type: 'Blur'; radius: number }
    | { type: 'Sharpen'; amount: number }
    | { type: 'Normalize' }
    | { type: 'FadeIn'; duration: number }
    | { type: 'FadeOut'; duration: number };

// Timeline state
export const timelineStore = writable<Timeline>({
    id: 'timeline-1',
    name: 'Untitled Timeline',
    framerate: 30.0,
    resolution: {
        width: 1920,
        height: 1080,
    },
    tracks: [
        {
            id: 'video-track-1',
            track_type: 'Video',
            clips: [],
            muted: false,
            locked: false,
        },
        {
            id: 'audio-track-1',
            track_type: 'Audio',
            clips: [],
            muted: false,
            locked: false,
        },
    ],
    duration: 0,
});

// Playhead position (in seconds)
export const playheadTime = writable<number>(0);

// Selected clip ID
export const selectedClipId = writable<string | null>(null);

// Zoom level (pixels per second)
export const pixelsPerSecond = writable<number>(50);

// Scroll position
export const scrollOffset = writable<number>(0);

// Derived store: total timeline duration
export const timelineDuration = derived(
    timelineStore,
    $timeline => {
        if ($timeline.tracks.length === 0) return 0;

        let maxDuration = 0;
        for (const track of $timeline.tracks) {
            for (const clip of track.clips) {
                const clipEnd = clip.track_position + clip.duration;
                if (clipEnd > maxDuration) {
                    maxDuration = clipEnd;
                }
            }
        }
        return maxDuration;
    }
);

// Actions with backend integration

/**
 * Add a clip to a track (with backend sync)
 */
export async function addClipToTrack(trackId: string, clip: Clip): Promise<void> {
    // Optimistic UI update
    const previousState = await getTimelineState();
    timelineStore.update(timeline => {
        const track = timeline.tracks.find(t => t.id === trackId);
        if (track && !track.locked) {
            track.clips.push(clip);
            const clipEnd = clip.track_position + clip.duration;
            if (clipEnd > timeline.duration) {
                timeline.duration = clipEnd;
            }
        }
        return timeline;
    });

    try {
        // Sync with backend (Tauri converts Rust snake_case to JS camelCase)
        await invoke('add_clip_to_timeline', { trackId, clip });
    } catch (error) {
        // Rollback on error
        console.error('Failed to add clip to backend:', error);
        timelineStore.set(previousState);
        throw error;
    }
}

// Helper to get current timeline state
async function getTimelineState(): Promise<Timeline> {
    return new Promise(resolve => {
        timelineStore.subscribe(value => {
            resolve(JSON.parse(JSON.stringify(value))); // Deep copy
        })();
    });
}

/**
 * Move a clip to a new track and position (with backend sync)
 */
export async function moveClip(clipId: string, newTrackId: string, newPosition: number): Promise<void> {
    // Optimistic UI update
    const previousState = await getTimelineState();
    timelineStore.update(timeline => {
        // Find and remove clip from current track
        let clipToMove: Clip | null = null;
        for (const track of timeline.tracks) {
            const index = track.clips.findIndex(c => c.id === clipId);
            if (index !== -1) {
                clipToMove = track.clips.splice(index, 1)[0];
                break;
            }
        }

        if (!clipToMove) return timeline;

        // Add to new track at new position
        const newTrack = timeline.tracks.find(t => t.id === newTrackId);
        if (newTrack && !newTrack.locked) {
            clipToMove.track_position = newPosition;
            newTrack.clips.push(clipToMove);
        }

        return timeline;
    });

    try {
        // Sync with backend
        await invoke('move_clip_on_timeline', { clipId, newTrackId, newPosition });
    } catch (error) {
        console.error('Failed to move clip in backend:', error);
        timelineStore.set(previousState);
        throw error;
    }
}

/**
 * Remove a clip from the timeline (with backend sync)
 */
export async function removeClip(clipId: string): Promise<void> {
    // Optimistic UI update
    const previousState = await getTimelineState();
    timelineStore.update(timeline => {
        for (const track of timeline.tracks) {
            const index = track.clips.findIndex(c => c.id === clipId);
            if (index !== -1) {
                track.clips.splice(index, 1);
                break;
            }
        }
        return timeline;
    });

    try {
        // Sync with backend
        await invoke('remove_clip_from_timeline', { clipId });
    } catch (error) {
        console.error('Failed to remove clip from backend:', error);
        timelineStore.set(previousState);
        throw error;
    }
}

/**
 * Update clip duration/trim (with backend sync)
 */
export async function updateClipDuration(clipId: string, newDuration: number, trimStart?: number, trimEnd?: number): Promise<void> {
    // Optimistic UI update
    const previousState = await getTimelineState();
    timelineStore.update(timeline => {
        for (const track of timeline.tracks) {
            const clip = track.clips.find(c => c.id === clipId);
            if (clip) {
                clip.duration = newDuration;
                if (trimStart !== undefined) clip.trim_start = trimStart;
                if (trimEnd !== undefined) clip.trim_end = trimEnd;
                break;
            }
        }
        return timeline;
    });

    try {
        // Sync with backend
        await invoke('trim_clip_on_timeline', {
            clipId,
            trimStart: trimStart ?? 0,
            trimEnd: trimEnd ?? newDuration
        });
    } catch (error) {
        console.error('Failed to trim clip in backend:', error);
        timelineStore.set(previousState);
        throw error;
    }
}

/**
 * Split a clip at the playhead position
 * Returns the IDs of both new clips, or throws an error if the operation fails
 */
export async function splitClipAtPlayhead(): Promise<{ firstClipId: string; secondClipId: string }> {
    // Get current state
    let currentClipId: string | null = null;
    let currentPlayhead: number = 0;
    let clip: Clip | null = null;
    let trackId: string | null = null;

    selectedClipId.subscribe(id => currentClipId = id)();
    playheadTime.subscribe(time => currentPlayhead = time)();

    // Validation: Check if a clip is selected
    if (!currentClipId) {
        throw new Error('No clip selected. Please select a clip to split.');
    }

    // Find the selected clip and its track
    const currentTimeline = await getTimelineState();
    for (const track of currentTimeline.tracks) {
        const foundClip = track.clips.find(c => c.id === currentClipId);
        if (foundClip) {
            clip = foundClip;
            trackId = track.id;
            break;
        }
    }

    if (!clip || !trackId) {
        throw new Error('Selected clip not found in timeline.');
    }

    // Validation: Check if playhead is within clip bounds
    const clipStart = clip.track_position;
    const clipEnd = clip.track_position + clip.duration;

    if (currentPlayhead <= clipStart || currentPlayhead >= clipEnd) {
        throw new Error(`Playhead must be within the clip bounds (${clipStart.toFixed(2)}s - ${clipEnd.toFixed(2)}s). Current position: ${currentPlayhead.toFixed(2)}s`);
    }

    // Calculate split time relative to clip start
    const splitTime = currentPlayhead - clipStart;

    // Optimistic UI update
    const previousState = await getTimelineState();

    try {
        // Call backend to split clip
        const [firstClipId, secondClipId] = await invoke<[string, string]>('split_clip_at_time', {
            clipId: currentClipId,
            splitTime
        });

        // Update frontend timeline state
        await initializeTimeline(); // Refresh from backend to get accurate state

        // Select both new clips
        selectedClipId.set(firstClipId); // Primary selection is first clip
        // Note: Multi-selection would require a separate store. For now, we just select the first clip.
        // TODO: Implement multi-selection store to select both clips

        return { firstClipId, secondClipId };
    } catch (error) {
        console.error('Failed to split clip in backend:', error);
        timelineStore.set(previousState);
        throw error;
    }
}

export function setPlayheadTime(time: number) {
    playheadTime.set(Math.max(0, time));
}

export function selectClip(clipId: string | null) {
    selectedClipId.set(clipId);
}

/**
 * Add a media file to the timeline (creates a Clip from MediaFile)
 */
export async function addMediaFileToTimeline(
    mediaFile: { id: string; duration: number; filename: string; codec: { video: string | null; audio: string | null }; media_type?: string },
    trackId?: string,
    position?: number
): Promise<void> {
    // Get current timeline to find tracks
    const currentTimeline = await getTimelineState();

    // Find video and audio tracks
    const videoTrack = currentTimeline.tracks.find(t => t.track_type === 'Video');
    const audioTrack = currentTimeline.tracks.find(t => t.track_type === 'Audio');

    // Calculate position (default to end of timeline)
    // Find the actual end position by checking all clips on all tracks
    let calculatedEndPosition = 0;
    for (const track of currentTimeline.tracks) {
        for (const clip of track.clips) {
            const clipEnd = clip.track_position + clip.duration;
            if (clipEnd > calculatedEndPosition) {
                calculatedEndPosition = clipEnd;
            }
        }
    }
    const targetPosition = position ?? calculatedEndPosition;

    // Determine if this is audio-only file
    const isAudioOnly = mediaFile.media_type === 'audio' || (!mediaFile.codec.video && mediaFile.codec.audio);
    const hasVideo = mediaFile.codec.video && mediaFile.codec.video.toLowerCase() !== 'none';
    const hasAudio = mediaFile.codec.audio && mediaFile.codec.audio.toLowerCase() !== 'none';

    // If it's a video file (or file with video), add video clip
    if (hasVideo && videoTrack) {
        const targetVideoTrackId = trackId || videoTrack.id;

        const videoClipId = `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        const videoClip: Clip = {
            id: videoClipId,
            media_file_id: mediaFile.id,
            name: mediaFile.filename,  // Include filename for display
            track_position: targetPosition,
            duration: mediaFile.duration,
            trim_start: 0,
            trim_end: mediaFile.duration,
            effects: [],
            volume: 1.0,
            speed: 1.0,
        };

        // Add video clip to video track
        await addClipToTrack(targetVideoTrackId, videoClip);
    }

    // Add audio clip to audio track (for both video files with audio and audio-only files)
    if (hasAudio && audioTrack) {
        const audioClipId = `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}-audio`;
        const audioClip: Clip = {
            id: audioClipId,
            media_file_id: mediaFile.id,
            name: isAudioOnly ? mediaFile.filename : `${mediaFile.filename} (audio)`,  // Don't add "(audio)" suffix for audio-only files
            track_position: targetPosition,
            duration: mediaFile.duration,
            trim_start: 0,
            trim_end: mediaFile.duration,
            effects: [],
            volume: 1.0,
            speed: 1.0,
        };

        // Add audio clip to audio track
        await addClipToTrack(audioTrack.id, audioClip);
    }
}

/**
 * Initialize timeline from backend
 */
export async function initializeTimeline(): Promise<void> {
    try {
        const timeline = await invoke<Timeline>('get_current_timeline');
        timelineStore.set(timeline);
    } catch (error) {
        console.error('Failed to load timeline from backend:', error);
        // If no timeline exists, create one with default settings
        try {
            await invoke('create_timeline', {
                name: 'Untitled Timeline',
                framerate: 30.0,
                width: 1920,
                height: 1080
            });
            const newTimeline = await invoke<Timeline>('get_current_timeline');
            timelineStore.set(newTimeline);
        } catch (createError) {
            console.error('Failed to create timeline:', createError);
            throw createError;
        }
    }
}

/**
 * Save timeline project to file
 */
export async function saveTimelineProject(filePath: string): Promise<void> {
    try {
        await invoke('save_timeline_project', { path: filePath });
    } catch (error) {
        console.error('Failed to save timeline project:', error);
        throw error;
    }
}

/**
 * Load timeline project from file
 */
export async function loadTimelineProject(filePath: string): Promise<void> {
    try {
        await invoke('load_timeline_project', { path: filePath });
        // Refresh timeline from backend
        const timeline = await invoke<Timeline>('get_current_timeline');
        timelineStore.set(timeline);
    } catch (error) {
        console.error('Failed to load timeline project:', error);
        throw error;
    }
}
