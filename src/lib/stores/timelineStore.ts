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
        // Sync with backend
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
    mediaFile: { id: string; duration: number },
    trackId?: string,
    position?: number
): Promise<void> {
    // Generate unique clip ID
    const clipId = `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    // Get current timeline to find default track
    const currentTimeline = await getTimelineState();

    // Use provided trackId or find first video track
    let targetTrackId = trackId;
    if (!targetTrackId) {
        const videoTrack = currentTimeline.tracks.find(t => t.track_type === 'Video');
        if (!videoTrack) {
            throw new Error('No video track found in timeline');
        }
        targetTrackId = videoTrack.id;
    }

    // Calculate position (default to end of timeline)
    const targetPosition = position ?? currentTimeline.duration;

    // Create clip from media file
    const clip: Clip = {
        id: clipId,
        media_file_id: mediaFile.id,
        track_position: targetPosition,
        duration: mediaFile.duration,
        trim_start: 0,
        trim_end: mediaFile.duration,
        effects: [],
        volume: 1.0,
        speed: 1.0,
    };

    // Use existing addClipToTrack function (already has backend sync)
    await addClipToTrack(targetTrackId, clip);
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
        // If no timeline exists, create one
        try {
            await invoke('create_timeline', { name: 'Untitled Timeline' });
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
