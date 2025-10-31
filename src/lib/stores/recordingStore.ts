import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { videoDir } from '@tauri-apps/api/path';

// Types matching backend data structures
export type RecordingStateType = 'Idle' | 'Recording' | 'Paused' | 'Finalizing' | 'Error';

export interface RecordingSource {
    type: 'screen' | 'window' | 'webcam';  // Matches Rust serde tag
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
export type RecordingMode = 'ScreenOnly' | 'WebcamOnly' | 'ScreenAndWebcam';

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
    recording_mode?: RecordingMode;
    webcam_source?: RecordingSource;
    webcam_output_path?: string;
    webcam_overlay_config?: WebcamOverlayConfig;
}

export interface RecordingState {
    state: RecordingStateType;
    duration: number;
    output_path: string | null;
    error_message: string | null;
}

// Webcam overlay types
export type WebcamPosition = 'TopLeft' | 'TopRight' | 'BottomLeft' | 'BottomRight';
export type WebcamShape = 'Square' | 'Circle';

export interface WebcamOverlayConfig {
    position: WebcamPosition;
    shape: WebcamShape;
    size: number;
    margin: number;
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
    fps: 30,
    quality: 7,
    show_cursor: true,
    audio_input: 'None',
});

export const isLoadingSources = writable<boolean>(false);
export const hasPermissions = writable<boolean | null>(null);
export const recordingError = writable<string | null>(null);

// Webcam recording stores
export const recordingMode = writable<RecordingMode>('ScreenOnly');
export const selectedWebcam = writable<string | null>(null);

// Webcam overlay configuration store
export const webcamOverlayConfig = writable<WebcamOverlayConfig>({
    position: 'BottomRight',
    shape: 'Square',  // Changed from Circle to Square to avoid FFmpeg filter parsing issues
    size: 320,
    margin: 20,
});

// Compositing progress tracking
export const isCompositing = writable<boolean>(false);
export const compositingProgress = writable<number>(0);

// Track webcam output path during recording (for auto-compositing)
export const currentWebcamPath = writable<string | null>(null);

// Derived stores
export const isRecording = derived(recordingState, $state => $state.state === 'Recording');
export const isPaused = derived(recordingState, $state => $state.state === 'Paused');
export const isIdle = derived(recordingState, $state => $state.state === 'Idle');
export const isFinalizing = derived(recordingState, $state => $state.state === 'Finalizing');

export const isDualMode = derived(recordingMode, $mode => $mode === 'ScreenAndWebcam');
export const isWebcamMode = derived(recordingMode, $mode => $mode === 'WebcamOnly' || $mode === 'ScreenAndWebcam');

export const webcamSources = derived(availableSources, $sources =>
    $sources.filter((s: RecordingSource) => s.type === 'webcam')
);

export const screenSources = derived(availableSources, $sources =>
    $sources.filter((s: RecordingSource) => s.type === 'screen' || s.type === 'window')
);

// Event listeners
let unlistenStarted: UnlistenFn | null = null;
let unlistenDuration: UnlistenFn | null = null;
let unlistenStopped: UnlistenFn | null = null;

/**
 * Initialize event listeners for recording updates
 */
export async function initializeRecordingListeners() {
    // Clean up any existing listeners first to prevent duplicates
    cleanupRecordingListeners();

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

export type SourceTypeFilter = 'screen' | 'webcam' | 'window' | 'all';

/**
 * Load available recording sources with optional filtering
 * @param filter - Filter type: 'screen', 'webcam', 'window', or 'all' (default: 'all')
 */
export async function loadRecordingSources(filter: SourceTypeFilter = 'all'): Promise<void> {
    console.log('[loadRecordingSources] Starting load with filter:', filter);
    isLoadingSources.set(true);
    recordingError.set(null);

    try {
        console.log('[loadRecordingSources] Calling backend list_recording_sources...');
        const sources = await invoke<RecordingSource[]>('list_recording_sources', { filter });
        console.log('[loadRecordingSources] Received', sources.length, 'sources:', sources);
        availableSources.set(sources);

        // Auto-select first source if none selected
        if (sources.length > 0) {
            selectedSource.update(current => current || sources[0].id);
        }
        console.log('[loadRecordingSources] Load completed successfully');
    } catch (error) {
        console.error('[loadRecordingSources] Failed to load recording sources:', error);
        recordingError.set(error as string);
    } finally {
        console.log('[loadRecordingSources] Setting isLoadingSources to false');
        isLoadingSources.set(false);
    }
}

/**
 * Load screen sources only (screens and windows)
 */
export async function loadScreenSources(): Promise<void> {
    return loadRecordingSources('screen');
}

/**
 * Load webcam sources only
 */
export async function loadWebcamSources(): Promise<void> {
    return loadRecordingSources('webcam');
}

/**
 * Load available audio input devices (microphones)
 */
export async function loadAudioDevices(): Promise<void> {
    console.log('[loadAudioDevices] Loading audio devices...');
    recordingError.set(null);

    try {
        const devices = await invoke<[string, string][]>('list_audio_devices');
        console.log('[loadAudioDevices] Received', devices.length, 'audio devices:', devices);

        // Convert tuples to AudioDevice objects
        const audioDevices: AudioDevice[] = devices.map(([id, name]) => ({ id, name }));
        availableAudioDevices.set(audioDevices);

        console.log('[loadAudioDevices] Load completed successfully');
    } catch (error) {
        console.error('[loadAudioDevices] Failed to load audio devices:', error);
        recordingError.set(error as string);
        // Set empty array on error
        availableAudioDevices.set([]);
    }
}

/**
 * Start recording with the current configuration
 */
export async function startRecording(): Promise<boolean> {
    recordingError.set(null);

    try {
        // Get current values from stores using get() to avoid closure bugs
        const sourceId = get(selectedSource);
        const sources = get(availableSources);
        const config = get(recordingConfig);

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

        // Get recording mode and webcam selection
        const mode = get(recordingMode);
        const webcamId = get(selectedWebcam);

        // Validate mode-specific requirements
        if ((mode === 'WebcamOnly' || mode === 'ScreenAndWebcam') && !webcamId) {
            recordingError.set('Please select a webcam source');
            return false;
        }

        // Generate output paths
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
        const videoDirPath = await videoDir();
        const outputPath = `${videoDirPath}/ClipForge/screen-${timestamp}.mp4`;
        const webcamPath = `${videoDirPath}/ClipForge/webcam-${timestamp}.mp4`;

        // Convert quality from string to 1-10 scale
        const qualityMap = {
            'Low': 3,
            'Medium': 5,
            'High': 7,
            'Ultra': 9
        };
        const qualityValue = qualityMap[config.quality as keyof typeof qualityMap] || 7;

        // Log audio configuration for debugging
        console.log('[AUDIO] Recording audio config:', {
            audio_input: config.audio_input || 'None',
            audio_device_id: config.audio_device_id,
            recording_mode: mode
        });

        // Get webcam source object if needed
        let webcamSource: RecordingSource | undefined;
        if (mode === 'WebcamOnly' || mode === 'ScreenAndWebcam') {
            webcamSource = sources.find(s => s.id === webcamId && s.type === 'webcam');
            if (!webcamSource) {
                recordingError.set('Selected webcam not found');
                return false;
            }
        }

        const fullConfig: RecordingConfig = {
            output_path: outputPath,
            fps: config.fps || 30,
            quality: typeof config.quality === 'number' ? config.quality : qualityValue,
            show_cursor: config.show_cursor !== undefined ? config.show_cursor : true,
            audio_input: config.audio_input || 'None',
            audio_device_id: config.audio_device_id,
            crop_region: config.crop_region,
            recording_mode: mode,
            webcam_source: webcamSource,
            webcam_output_path: (mode === 'ScreenAndWebcam') ? webcamPath : undefined,
        };

        // Store webcam path for auto-compositing when in dual mode
        if (mode === 'ScreenAndWebcam') {
            currentWebcamPath.set(webcamPath);
            console.log('[startRecording] Stored webcam path for auto-compositing:', webcamPath);
        } else {
            currentWebcamPath.set(null);
        }

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
 * If webcam was recorded in dual mode, automatically composites them
 */
export async function stopRecording(): Promise<string | null> {
    recordingError.set(null);

    try {
        recordingState.update(state => ({
            ...state,
            state: 'Finalizing',
        }));

        const screenPath = await invoke<string>('stop_recording');
        console.log('[stopRecording] Screen recording saved to:', screenPath);

        // Check if we need to auto-composite webcam overlay
        const webcamPath = get(currentWebcamPath);
        const mode = get(recordingMode);

        if (mode === 'ScreenAndWebcam' && webcamPath) {
            console.log('[stopRecording] Dual mode detected, auto-compositing webcam overlay...');
            console.log('[stopRecording] Screen path:', screenPath);
            console.log('[stopRecording] Webcam path:', webcamPath);

            // Generate composite output path
            const videoDirPath = await videoDir();
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-').substring(0, 19);
            const compositePath = `${videoDirPath}/ClipForge/composite-${timestamp}.mp4`;

            // Get webcam overlay config
            const overlayConfig = get(webcamOverlayConfig);

            // Composite the videos
            const compositeResult = await compositeWebcamRecording(
                screenPath,
                webcamPath,
                compositePath,
                overlayConfig
            );

            if (compositeResult) {
                console.log('[stopRecording] Compositing successful:', compositeResult);

                // Clear webcam path
                currentWebcamPath.set(null);

                recordingState.update(state => ({
                    ...state,
                    state: 'Idle',
                    output_path: compositeResult, // Return composite path instead of screen path
                    duration: 0,
                }));

                return compositeResult;
            } else {
                // Compositing failed - check if source files still exist
                const screenExists = await invoke<boolean>('file_exists', { path: screenPath });
                const webcamExists = await invoke<boolean>('file_exists', { path: webcamPath });

                console.error('[stopRecording] Compositing failed!');
                console.error('  Screen file exists:', screenExists);
                console.error('  Webcam file exists:', webcamExists);

                const errorMsg = 'Failed to composite screen and webcam recordings. ' +
                    (screenExists ? 'Screen recording saved separately.' : 'Source files may have been lost.');
                recordingError.set(errorMsg);

                if (screenExists) {
                    // Return screen recording if it still exists
                    console.log('[stopRecording] Returning screen recording as fallback');
                    currentWebcamPath.set(null);
                    recordingState.update(state => ({
                        ...state,
                        state: 'Idle',
                        output_path: screenPath,
                        duration: 0,
                        error_message: errorMsg,
                    }));
                    return screenPath;
                } else {
                    // Both files lost
                    throw new Error('Compositing failed and source recordings were lost');
                }
            }
        }

        // Clear webcam path
        currentWebcamPath.set(null);

        recordingState.update(state => ({
            ...state,
            state: 'Idle',
            output_path: screenPath,
            duration: 0,
        }));

        return screenPath;
    } catch (error) {
        console.error('Failed to stop recording:', error);
        recordingError.set(error as string);
        currentWebcamPath.set(null); // Clear on error
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

/**
 * Composite webcam overlay onto screen recording
 *
 * @param screenPath - Path to screen recording file
 * @param webcamPath - Path to webcam recording file
 * @param outputPath - Path where composite video should be saved
 * @param overlayConfig - Webcam overlay configuration
 * @returns Path to the composite video file
 */
export async function compositeWebcamRecording(
    screenPath: string,
    webcamPath: string,
    outputPath: string,
    overlayConfig: WebcamOverlayConfig
): Promise<string | null> {
    console.log('[compositeWebcamRecording] Starting compositing...');
    isCompositing.set(true);
    compositingProgress.set(0);
    recordingError.set(null);

    // Set up progress listener
    const unlistenProgress = await listen<{ progress: number }>('compositing:progress', (event) => {
        compositingProgress.set(event.payload.progress);
        console.log('[compositeWebcamRecording] Progress:', event.payload.progress);
    });

    // Set up completion listener
    const unlistenComplete = await listen<{ output_path: string }>('compositing:complete', (event) => {
        console.log('[compositeWebcamRecording] Complete:', event.payload.output_path);
        isCompositing.set(false);
        compositingProgress.set(100);
    });

    try {
        console.log('[compositeWebcamRecording] Invoking backend composite command...');
        const result = await invoke<string>('composite_webcam_recording', {
            screenPath,
            webcamPath,
            outputPath,
            overlayConfig,
        });

        console.log('[compositeWebcamRecording] Compositing complete:', result);

        // Verify the composite file exists
        const compositeExists = await invoke<boolean>('file_exists', { path: result });
        if (!compositeExists) {
            const errorMsg = `Composite command succeeded but file not found: ${result}`;
            console.error('[compositeWebcamRecording]', errorMsg);
            recordingError.set(errorMsg);
            return null;
        }

        console.log('[compositeWebcamRecording] Composite file verified:', result);
        return result;
    } catch (error) {
        const errorMsg = `Failed to composite webcam recording: ${error}`;
        console.error('[compositeWebcamRecording]', errorMsg);
        console.error('[compositeWebcamRecording] Details:', {
            screenPath,
            webcamPath,
            outputPath,
            overlayConfig,
            error,
        });
        recordingError.set(errorMsg);
        return null;
    } finally {
        // Clean up listeners
        unlistenProgress();
        unlistenComplete();
        isCompositing.set(false);
    }
}
