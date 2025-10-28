-- SQLite schema for ClipForge media library
-- Optimized with indexes for fast duplicate detection and queries

CREATE TABLE IF NOT EXISTS media_files (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    filename TEXT NOT NULL,
    duration REAL NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    video_codec TEXT NOT NULL,
    audio_codec TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    thumbnail_path TEXT,
    hash TEXT NOT NULL UNIQUE,      -- UNIQUE constraint creates automatic index
    imported_at TEXT NOT NULL
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_hash ON media_files(hash);           -- O(log n) duplicate detection
CREATE INDEX IF NOT EXISTS idx_imported_at ON media_files(imported_at DESC);  -- Recent files first
CREATE INDEX IF NOT EXISTS idx_path ON media_files(path);           -- Find by filesystem path

-- Optional: Full-text search on filename (can be added later if needed)
-- CREATE VIRTUAL TABLE IF NOT EXISTS media_files_fts USING fts5(filename, content='media_files', content_rowid='rowid');
