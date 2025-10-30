import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { playheadTime } from './timelineStore';

// Types matching Rust backend
export interface SubtitleSegment {
    id: number;
    start_time: number;
    end_time: number;
    text: string;
}

export interface SubtitleSource {
    type: 'Transcribed' | 'Imported' | 'Manual';
    media_file_id?: string;
    provider?: string;
    file_path?: string;
}

export interface SubtitleStyle {
    font_size: number;
    font_name: string;
    margin_v: number;
}

export interface SubtitleTrack {
    segments: SubtitleSegment[];
    language: string;
    source: SubtitleSource;
    style: SubtitleStyle;
}

export interface SubtitleProgress {
    stage: string;
    progress: number;  // 0.0 to 1.0
}

// Store state
export interface SubtitleState {
    currentTrack: SubtitleTrack | null;
    isTranscribing: boolean;
    transcriptionProgress: SubtitleProgress | null;
    enabled: boolean;
    editingSegmentId: number | null;
    apiKeyConfigured: boolean;
}

// Main subtitle store
export const subtitleStore = writable<SubtitleState>({
    currentTrack: null,
    isTranscribing: false,
    transcriptionProgress: null,
    enabled: false,
    editingSegmentId: null,
    apiKeyConfigured: false,
});

// Derived: Get active subtitle at current playhead time
export const currentSubtitle = derived(
    [subtitleStore, playheadTime],
    ([$subtitles, $playhead]) => {
        if (!$subtitles.enabled || !$subtitles.currentTrack) {
            return null;
        }

        return $subtitles.currentTrack.segments.find(
            seg => $playhead >= seg.start_time && $playhead < seg.end_time
        ) || null;
    }
);

// Setup event listeners for transcription progress
let progressUnlisten: (() => void) | null = null;

export async function setupSubtitleEventListeners() {
    // Listen for transcription progress
    if (progressUnlisten) {
        progressUnlisten();
    }

    progressUnlisten = await listen<SubtitleProgress>('subtitle:progress', (event) => {
        subtitleStore.update(state => ({
            ...state,
            transcriptionProgress: event.payload,
        }));
    });
}

// Initialize listeners on module load
setupSubtitleEventListeners();

// Actions

/**
 * Set OpenAI API key
 */
export async function setOpenAIApiKey(apiKey: string): Promise<void> {
    try {
        await invoke('set_openai_api_key', { apiKey });
        subtitleStore.update(state => ({
            ...state,
            apiKeyConfigured: true,
        }));
    } catch (error) {
        console.error('Failed to set API key:', error);
        throw error;
    }
}

/**
 * Check if subtitle service is available
 */
export async function checkSubtitleAvailable(): Promise<boolean> {
    try {
        const available = await invoke<boolean>('check_subtitle_available');
        subtitleStore.update(state => ({
            ...state,
            apiKeyConfigured: available,
        }));
        return available;
    } catch (error) {
        console.error('Failed to check subtitle availability:', error);
        return false;
    }
}

/**
 * Transcribe timeline audio to generate subtitles
 */
export async function transcribeTimelineAudio(
    timelineId: string,
    mediaFiles: any[],
    language?: string
): Promise<void> {
    const previousState = get(subtitleStore);

    try {
        // Optimistically set transcribing state
        subtitleStore.update(state => ({
            ...state,
            isTranscribing: true,
            transcriptionProgress: {
                stage: 'Starting...',
                progress: 0,
            },
        }));

        const track = await invoke<SubtitleTrack>('transcribe_timeline_audio', {
            timelineId,
            mediaFiles,
            language: language || null,
        });

        // Update with transcribed track
        subtitleStore.update(state => ({
            ...state,
            currentTrack: track,
            isTranscribing: false,
            transcriptionProgress: null,
            enabled: true,  // Auto-enable after transcription
        }));

    } catch (error) {
        console.error('Transcription failed:', error);
        // Rollback on error
        subtitleStore.set({
            ...previousState,
            isTranscribing: false,
            transcriptionProgress: null,
        });
        throw error;
    }
}

/**
 * Update a subtitle segment
 */
export async function updateSubtitleSegment(
    timelineId: string,
    segmentId: number,
    newText: string,
    newStart?: number,
    newEnd?: number
): Promise<void> {
    const previousState = get(subtitleStore);

    try {
        // Optimistically update UI
        subtitleStore.update(state => {
            if (!state.currentTrack) return state;

            const segments = state.currentTrack.segments.map(seg => {
                if (seg.id === segmentId) {
                    return {
                        ...seg,
                        text: newText,
                        start_time: newStart !== undefined ? newStart : seg.start_time,
                        end_time: newEnd !== undefined ? newEnd : seg.end_time,
                    };
                }
                return seg;
            });

            return {
                ...state,
                currentTrack: {
                    ...state.currentTrack,
                    segments,
                },
            };
        });

        // Sync backend
        await invoke('update_subtitle_segment', {
            timelineId,
            segmentId,
            newText,
            newStart: newStart || null,
            newEnd: newEnd || null,
        });

    } catch (error) {
        console.error('Failed to update subtitle segment:', error);
        // Rollback on error
        subtitleStore.set(previousState);
        throw error;
    }
}

/**
 * Delete a subtitle segment
 */
export async function deleteSubtitleSegment(
    timelineId: string,
    segmentId: number
): Promise<void> {
    const previousState = get(subtitleStore);

    try {
        // Optimistically update UI
        subtitleStore.update(state => {
            if (!state.currentTrack) return state;

            const segments = state.currentTrack.segments.filter(seg => seg.id !== segmentId);

            return {
                ...state,
                currentTrack: {
                    ...state.currentTrack,
                    segments,
                },
            };
        });

        // Note: Backend doesn't have delete command yet, would use update_subtitle_segment
        // For now, this is frontend-only until saved to timeline

    } catch (error) {
        console.error('Failed to delete subtitle segment:', error);
        // Rollback on error
        subtitleStore.set(previousState);
        throw error;
    }
}

/**
 * Toggle subtitles on/off
 */
export async function toggleSubtitles(
    timelineId: string,
    enabled: boolean
): Promise<void> {
    const previousState = get(subtitleStore);

    try {
        // Optimistically update UI
        subtitleStore.update(state => ({
            ...state,
            enabled,
        }));

        // Sync backend
        await invoke('toggle_subtitles', {
            timelineId,
            enabled,
        });

    } catch (error) {
        console.error('Failed to toggle subtitles:', error);
        // Rollback on error
        subtitleStore.set(previousState);
        throw error;
    }
}

/**
 * Export subtitles to SRT file
 */
export async function exportSubtitlesSRT(outputPath: string): Promise<void> {
    const state = get(subtitleStore);

    if (!state.currentTrack) {
        throw new Error('No subtitle track to export');
    }

    try {
        await invoke('export_subtitles_srt', {
            track: state.currentTrack,
            outputPath,
        });
    } catch (error) {
        console.error('Failed to export SRT:', error);
        throw error;
    }
}

/**
 * Import subtitles from SRT file
 */
export async function importSubtitlesSRT(
    filePath: string,
    language?: string
): Promise<void> {
    const previousState = get(subtitleStore);

    try {
        const track = await invoke<SubtitleTrack>('import_subtitles_srt', {
            filePath,
            language: language || null,
        });

        subtitleStore.update(state => ({
            ...state,
            currentTrack: track,
            enabled: true,
        }));

    } catch (error) {
        console.error('Failed to import SRT:', error);
        subtitleStore.set(previousState);
        throw error;
    }
}

/**
 * Set which segment is being edited
 */
export function setEditingSegment(segmentId: number | null): void {
    subtitleStore.update(state => ({
        ...state,
        editingSegmentId: segmentId,
    }));
}

/**
 * Update subtitle style
 */
export function updateSubtitleStyle(style: Partial<SubtitleStyle>): void {
    subtitleStore.update(state => {
        if (!state.currentTrack) return state;

        return {
            ...state,
            currentTrack: {
                ...state.currentTrack,
                style: {
                    ...state.currentTrack.style,
                    ...style,
                },
            },
        };
    });
}

/**
 * Clear subtitle track
 */
export function clearSubtitles(): void {
    subtitleStore.update(state => ({
        ...state,
        currentTrack: null,
        enabled: false,
    }));
}
