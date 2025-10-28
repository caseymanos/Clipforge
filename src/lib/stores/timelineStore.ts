import { writable, derived } from 'svelte/store';

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
    enabled: boolean;
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

export interface Effect {
    effect_type: 'Brightness' | 'Contrast' | 'Saturation' | 'Blur' | 'FadeIn' | 'FadeOut';
    intensity: number;
}

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
            enabled: true,
            locked: false,
        },
        {
            id: 'audio-track-1',
            track_type: 'Audio',
            clips: [],
            enabled: true,
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

// Actions
export function addClipToTrack(trackId: string, clip: Clip) {
    timelineStore.update(timeline => {
        const track = timeline.tracks.find(t => t.id === trackId);
        if (track && !track.locked) {
            track.clips.push(clip);

            // Update timeline duration
            const clipEnd = clip.track_position + clip.duration;
            if (clipEnd > timeline.duration) {
                timeline.duration = clipEnd;
            }
        }
        return timeline;
    });
}

export function moveClip(clipId: string, newTrackId: string, newPosition: number) {
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
}

export function removeClip(clipId: string) {
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
}

export function updateClipDuration(clipId: string, newDuration: number) {
    timelineStore.update(timeline => {
        for (const track of timeline.tracks) {
            const clip = track.clips.find(c => c.id === clipId);
            if (clip) {
                clip.duration = newDuration;
                break;
            }
        }
        return timeline;
    });
}

export function setPlayheadTime(time: number) {
    playheadTime.set(Math.max(0, time));
}

export function selectClip(clipId: string | null) {
    selectedClipId.set(clipId);
}
