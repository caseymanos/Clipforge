import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types matching backend data structures
export type MediaType = 'video' | 'audio' | 'image';

export interface MediaFile {
    id: string;
    path: string;
    filename: string;
    media_type: MediaType;
    duration: number;
    resolution: {
        width: number;
        height: number;
    } | null;
    codec: {
        video: string | null;
        audio: string | null;
    };
    file_size: number;
    thumbnail_path: string | null;
    hash: string;
    imported_at: string;
}

export interface FileMetadata {
    media_type: MediaType;
    duration: number;
    resolution: {
        width: number;
        height: number;
    } | null;
    codec: {
        video: string | null;
        audio: string | null;
    };
    bitrate: number;
    framerate: number | null;
    has_audio: boolean;
    has_video: boolean;
}

// Media library state
export const mediaLibraryStore = writable<MediaFile[]>([]);

// Loading state
export const isLoadingLibrary = writable<boolean>(false);

// Selected media file
export const selectedMediaFile = writable<string | null>(null);

// Error state
export const mediaLibraryError = writable<string | null>(null);

// Actions

/**
 * Load all media files from the backend
 */
export async function loadMediaLibrary() {
    isLoadingLibrary.set(true);
    mediaLibraryError.set(null);

    try {
        const files = await invoke<MediaFile[]>('get_media_library');
        mediaLibraryStore.set(files);
    } catch (error) {
        console.error('Failed to load media library:', error);
        mediaLibraryError.set(error as string);
    } finally {
        isLoadingLibrary.set(false);
    }
}

/**
 * Import a media file into the library
 */
export async function importMediaFile(path: string): Promise<MediaFile | null> {
    isLoadingLibrary.set(true);
    mediaLibraryError.set(null);

    try {
        const mediaFile = await invoke<MediaFile>('import_media_file', { path });

        // Add to store (optimistic update)
        mediaLibraryStore.update(files => {
            // Check if already exists (deduplication)
            const exists = files.some(f => f.id === mediaFile.id);
            if (!exists) {
                return [...files, mediaFile];
            }
            return files;
        });

        return mediaFile;
    } catch (error) {
        console.error('Failed to import media file:', error);
        mediaLibraryError.set(error as string);
        return null;
    } finally {
        isLoadingLibrary.set(false);
    }
}

/**
 * Delete a media file from the library
 */
export async function deleteMediaFile(id: string): Promise<boolean> {
    mediaLibraryError.set(null);

    try {
        await invoke('delete_media_file', { id });

        // Remove from store
        mediaLibraryStore.update(files => files.filter(f => f.id !== id));

        // Clear selection if deleted file was selected
        selectedMediaFile.update(selected => selected === id ? null : selected);

        return true;
    } catch (error) {
        console.error('Failed to delete media file:', error);
        mediaLibraryError.set(error as string);
        return false;
    }
}

/**
 * Get a specific media file by ID
 */
export async function getMediaFile(id: string): Promise<MediaFile | null> {
    try {
        const file = await invoke<MediaFile | null>('get_media_file', { id });
        return file;
    } catch (error) {
        console.error('Failed to get media file:', error);
        return null;
    }
}

/**
 * Extract metadata from a video file
 */
export async function getFileMetadata(path: string): Promise<FileMetadata | null> {
    try {
        const metadata = await invoke<FileMetadata>('get_file_metadata', { path });
        return metadata;
    } catch (error) {
        console.error('Failed to get file metadata:', error);
        return null;
    }
}

/**
 * Generate a thumbnail for a video file
 */
export async function generateThumbnail(videoPath: string, timestamp: number): Promise<string | null> {
    try {
        const thumbnailPath = await invoke<string>('generate_thumbnail', {
            videoPath,
            timestamp,
        });
        return thumbnailPath;
    } catch (error) {
        console.error('Failed to generate thumbnail:', error);
        return null;
    }
}

/**
 * Select a media file
 */
export function selectMediaFile(id: string | null) {
    selectedMediaFile.set(id);
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Format duration for display (HH:MM:SS or MM:SS)
 */
export function formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (hours > 0) {
        return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }

    return `${minutes}:${secs.toString().padStart(2, '0')}`;
}
