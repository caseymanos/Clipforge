import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { videoDir } from '@tauri-apps/api/path';

// Types matching backend data structures
export type RecordingStateType = 'Idle' | 'Recording' | 'Paused' | 'Finalizing' | 'Error';

export interface RecordingSource {
    type: 'screen' | 'window';  // Matches Rust serde tag
    id: string;
    name: string;
    width?: number;  // For Screen type
    height?: number; // For Screen type
    app_name?: string; // For Window type
    preview_path?: string; // Path to preview thumbnail image
}

export interface AudioDevice {
    id: string;
    name: string;
}

export type AudioInputType = 'None' | 'SystemAudio' | 'Microphone' | 'Both';

export interface CropRegion {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface RecordingConfig {
    output_path: string;
    fps: number;
    quality: number; // 1-10 scale in backend
    audio_input: AudioInputType;
    audio_device_id?: string;
    show_cursor: boolean;
    crop_region?: CropRegion;
}

export interface RecordingState {
    state: RecordingStateType;
    duration: number;
    output_path: string | null;
    error_message: string | null;
}

// Store state
export const recordingState = writable<RecordingState>({
    state: 'Idle',
    duration: 0,
    output_path: null,
    error_message: null,
});

export const availableSources = writable<RecordingSource[]>([]);
export const availableAudioDevices = writable<AudioDevice[]>([]);
export const selectedSource = writable<string | null>(null);
export const recordingConfig = writable<Partial<RecordingConfig>>({
    framerate: 30,
    quality: 'High',
    capture_cursor: true,
    audio_input: 'None',
});

export const isLoadingSources = writable<boolean>(false);
export const hasPermissions = writable<boolean | null>(null);
export const recordingError = writable<string | null>(null);

// Derived stores
export const isRecording = derived(recordingState, $state => $state.state === 'Recording');
export const isPaused = derived(recordingState, $state => $state.state === 'Paused');
export const isIdle = derived(recordingState, $state => $state.state === 'Idle');
export const isFinalizing = derived(recordingState, $state => $state.state === 'Finalizing');

// Event listeners
let unlistenStarted: UnlistenFn | null = null;
let unlistenDuration: UnlistenFn | null = null;
let unlistenStopped: UnlistenFn | null = null;

/**
 * Initialize event listeners for recording updates
 */
export async function initializeRecordingListeners() {
    // Listen for recording started event
    unlistenStarted = await listen<void>('recording:started', () => {
        console.log('Recording started');
        recordingState.update(state => ({
            ...state,
            state: 'Recording',
            duration: 0,
            error_message: null,
        }));
    });

    // Listen for duration updates (emitted every 500ms)
    unlistenDuration = await listen<{ duration: number }>('recording:duration', (event) => {
        recordingState.update(state => ({
            ...state,
            duration: event.payload.duration,
        }));
    });

    // Listen for recording stopped event
    unlistenStopped = await listen<{ file_path: string }>('recording:stopped', (event) => {
        console.log('Recording stopped:', event.payload.file_path);
        recordingState.update(state => ({
            ...state,
            state: 'Idle',
            output_path: event.payload.file_path,
        }));
    });
}

/**
 * Clean up event listeners
 */
export function cleanupRecordingListeners() {
    if (unlistenStarted) {
        unlistenStarted();
        unlistenStarted = null;
    }
    if (unlistenDuration) {
        unlistenDuration();
        unlistenDuration = null;
    }
    if (unlistenStopped) {
        unlistenStopped();
        unlistenStopped = null;
    }
}

/**
 * Check if the app has screen recording permissions
 */
export async function checkRecordingPermissions(): Promise<boolean> {
    try {
        const hasPerms = await invoke<boolean>('check_recording_permissions');
        hasPermissions.set(hasPerms);
        return hasPerms;
    } catch (error) {
        console.error('Failed to check recording permissions:', error);
        hasPermissions.set(false);
        return false;
    }
}

/**
 * Request screen recording permissions from the system
 */
export async function requestRecordingPermissions(): Promise<boolean> {
    try {
        const granted = await invoke<boolean>('request_recording_permissions');
        hasPermissions.set(granted);
        return granted;
    } catch (error) {
        console.error('Failed to request recording permissions:', error);
        recordingError.set(error as string);
        return false;
    }
}

/**
 * Load available recording sources (screens and windows)
 */
export async function loadRecordingSources(): Promise<void> {
    isLoadingSources.set(true);
    recordingError.set(null);

    try {
        const sources = await invoke<RecordingSource[]>('list_recording_sources');
        availableSources.set(sources);

        // Auto-select first source if none selected
        if (sources.length > 0) {
            selectedSource.update(current => current || sources[0].id);
        }
    } catch (error) {
        console.error('Failed to load recording sources:', error);
        recordingError.set(error as string);
    } finally {
        isLoadingSources.set(false);
    }
}

/**
 * Start recording with the current configuration
 */
export async function startRecording(): Promise<boolean> {
    recordingError.set(null);

    try {
        // Get current values from stores
        const sourceId = await new Promise<string | null>(resolve => {
            const unsubscribe = selectedSource.subscribe(value => {
                resolve(value);
                unsubscribe();
            });
        });

        const sources = await new Promise<RecordingSource[]>(resolve => {
            const unsubscribe = availableSources.subscribe(value => {
                resolve(value);
                unsubscribe();
            });
        });

        const config = await new Promise<Partial<RecordingConfig>>(resolve => {
            const unsubscribe = recordingConfig.subscribe(value => {
                resolve(value);
                unsubscribe();
            });
        });

        if (!sourceId) {
            recordingError.set('No recording source selected');
            return false;
        }

        // Find the full source object
        const source = sources.find(s => s.id === sourceId);
        if (!source) {
            recordingError.set('Selected source not found');
            return false;
        }

        // Generate output path - use Videos directory
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const videoDirPath = await videoDir();
        const outputPath = `${videoDirPath}/ClipForge/recording-${timestamp}.mp4`;

        // Convert quality from string to 1-10 scale
        const qualityMap = {
            'Low': 3,
            'Medium': 5,
            'High': 7,
            'Ultra': 9
        };
        const qualityValue = qualityMap[config.quality as keyof typeof qualityMap] || 7;

        const fullConfig: RecordingConfig = {
            output_path: outputPath,
            fps: config.framerate || 30,
            quality: qualityValue,
            show_cursor: config.capture_cursor !== undefined ? config.capture_cursor : true,
            audio_input: config.audio_input || 'None',
            audio_device_id: config.audio_device_id,
            crop_region: config.crop_region,
        };

        await invoke('start_recording', {
            source: source,
            config: fullConfig,
        });

        recordingState.update(state => ({
            ...state,
            state: 'Recording',
            duration: 0,
            output_path: null,
            error_message: null,
        }));

        return true;
    } catch (error) {
        console.error('Failed to start recording:', error);
        recordingError.set(error as string);
        recordingState.update(state => ({
            ...state,
            state: 'Error',
            error_message: error as string,
        }));
        return false;
    }
}

/**
 * Stop the current recording
 */
export async function stopRecording(): Promise<string | null> {
    recordingError.set(null);

    try {
        recordingState.update(state => ({
            ...state,
            state: 'Finalizing',
        }));

        const filePath = await invoke<string>('stop_recording');

        recordingState.update(state => ({
            ...state,
            state: 'Idle',
            output_path: filePath,
            duration: 0,
        }));

        return filePath;
    } catch (error) {
        console.error('Failed to stop recording:', error);
        recordingError.set(error as string);
        recordingState.update(state => ({
            ...state,
            state: 'Error',
            error_message: error as string,
        }));
        return null;
    }
}

/**
 * Get the current recording state from backend
 */
export async function refreshRecordingState(): Promise<void> {
    try {
        const state = await invoke<RecordingState>('get_recording_state');
        recordingState.set(state);
    } catch (error) {
        console.error('Failed to get recording state:', error);
    }
}

/**
 * Get the current recording duration from backend
 */
export async function getRecordingDuration(): Promise<number> {
    try {
        const duration = await invoke<number>('get_recording_duration');
        return duration;
    } catch (error) {
        console.error('Failed to get recording duration:', error);
        return 0;
    }
}

/**
 * Format recording duration for display (MM:SS)
 */
export function formatRecordingDuration(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Reset recording state to idle
 */
export function resetRecordingState(): void {
    recordingState.set({
        state: 'Idle',
        duration: 0,
        output_path: null,
        error_message: null,
    });
    recordingError.set(null);
}
