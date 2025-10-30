-- SQLite schema for ClipForge media library
-- Optimized with indexes for fast duplicate detection and queries

CREATE TABLE IF NOT EXISTS media_files (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    filename TEXT NOT NULL,
    media_type TEXT NOT NULL DEFAULT 'video',  -- video | audio | image
    duration REAL NOT NULL,
    width INTEGER,                  -- NULL for audio files
    height INTEGER,                 -- NULL for audio files
    video_codec TEXT,               -- NULL for audio-only files
    audio_codec TEXT,               -- NULL for video-only files without audio
    file_size INTEGER NOT NULL,
    thumbnail_path TEXT,
    hash TEXT NOT NULL UNIQUE,      -- UNIQUE constraint creates automatic index
    imported_at TEXT NOT NULL,
    proxy_path TEXT,                -- Path to H.264 proxy file for smooth editing
    has_proxy INTEGER DEFAULT 0,    -- Boolean: 1 if proxy exists, 0 otherwise
    proxy_status TEXT DEFAULT 'none' -- none | generating | ready | failed
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_hash ON media_files(hash);           -- O(log n) duplicate detection
CREATE INDEX IF NOT EXISTS idx_imported_at ON media_files(imported_at DESC);  -- Recent files first
CREATE INDEX IF NOT EXISTS idx_path ON media_files(path);           -- Find by filesystem path

-- Optional: Full-text search on filename (can be added later if needed)
-- CREATE VIRTUAL TABLE IF NOT EXISTS media_files_fts USING fts5(filename, content='media_files', content_rowid='rowid');
