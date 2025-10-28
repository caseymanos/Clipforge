# Module 2: File System & Media Management

**Owner:** TBD  
**Dependencies:** Module 1 (Application Shell)  
**Phase:** 1 (Weeks 1-2)  
**Estimated Effort:** 4-5 days

## Overview

Handles video file import, media library management, metadata extraction, and thumbnail generation. This module is the **data foundation** for the timeline and preview systems.

## Responsibilities

- Import video files (drag-drop, file picker)
- Store media metadata in SQLite
- Generate thumbnails for clips
- Manage media library catalog
- Provide file access via custom protocol
- Handle file deduplication
- Extract video metadata (duration, resolution, codec)

## File Structure

```
src-tauri/src/
├── file_service.rs           # File I/O operations
├── media_library.rs          # Media catalog management
├── metadata.rs               # FFprobe integration
├── thumbnail.rs              # Thumbnail generation
├── database/
│   ├── mod.rs               # Database setup
│   ├── schema.sql           # SQLite schema
│   └── queries.rs           # Database queries
└── commands/
    └── file_commands.rs     # Tauri commands
```

## Data Structures

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,              // UUID
    pub path: PathBuf,
    pub filename: String,
    pub duration: f64,           // seconds
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub file_size: u64,          // bytes
    pub thumbnail_path: Option<PathBuf>,
    pub hash: String,            // SHA-256 for deduplication
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCodec {
    pub video: String,   // e.g., "h264", "hevc"
    pub audio: String,   // e.g., "aac", "mp3"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub duration: f64,
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub bitrate: u64,
    pub framerate: f64,
    pub has_audio: bool,
}
```

## Implementation

### Architecture Decision: SQLite + In-Memory Cache

**Why SQLite over pure HashMap?**
- ✅ **Persistence:** Media library survives app restarts
- ✅ **Deduplication:** Hash-based lookups via SQL index (O(log n))
- ✅ **Scalability:** Handles 1000+ files without eating RAM
- ✅ **Queries:** Complex filtering (by date, codec, size)
- ✅ **No dependencies:** Bundled SQLite (no system install)

**Hybrid Approach (Best of Both Worlds):**
```rust
pub struct FileService {
    db: Database,                                       // Persistent storage
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,    // Fast lookups (optional)
    thumbnail_generator: ThumbnailGenerator,
}
```

For **MVP (Weeks 1-4):** Use SQLite only (simpler)  
For **Optimization (Weeks 7-8):** Add HashMap cache if profiling shows need

### 1. Media File Import (SQLite-backed)

```rust
// src-tauri/src/file_service.rs
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;

pub struct FileService {
    db: Database,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,  // Optional cache
    thumbnail_generator: ThumbnailGenerator,
}

impl FileService {
    pub fn new(db: Database, thumbnail_generator: ThumbnailGenerator) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(HashMap::new())),
            thumbnail_generator,
        }
    }
    
    pub async fn import_file(&self, path: PathBuf) -> Result<MediaFile, FileError> {
        // 1. Validate file exists and is readable
        if !path.exists() {
            return Err(FileError::FileNotFound(path));
        }
        
        // 2. Calculate hash for deduplication
        let hash = self.calculate_hash(&path)?;
        
        // 3. Check database for duplicate (indexed lookup is fast)
        if let Some(existing) = self.db.find_by_hash(&hash).await? {
            // Warm cache with existing file
            self.cache.write().await.insert(existing.id.clone(), existing.clone());
            return Ok(existing);
        }
        
        // 4. Extract metadata using FFprobe
        let metadata = self.extract_metadata(&path).await?;
        
        // 5. Generate thumbnail
        let thumbnail_path = self.thumbnail_generator
            .generate(&path, 5.0)  // Frame at 5 seconds
            .await?;
        
        // 6. Create MediaFile object
        let media_file = MediaFile {
            id: Uuid::new_v4().to_string(),
            path: path.clone(),
            filename: path.file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            duration: metadata.duration,
            resolution: metadata.resolution,
            codec: metadata.codec,
            file_size: std::fs::metadata(&path)?.len(),
            thumbnail_path: Some(thumbnail_path),
            hash,
            imported_at: Utc::now(),
        };
        
        // 7. Save to database (persistent)
        self.db.insert_media_file(&media_file).await?;
        
        // 8. Cache for fast subsequent access
        self.cache.write().await.insert(media_file.id.clone(), media_file.clone());
        
        Ok(media_file)
    }
    
    pub async fn get_all_media(&self) -> Result<Vec<MediaFile>, FileError> {
        // Load from database
        let files = self.db.get_all().await?;
        
        // Populate cache for subsequent lookups
        let mut cache = self.cache.write().await;
        for file in &files {
            cache.insert(file.id.clone(), file.clone());
        }
        
        Ok(files)
    }
    
    pub async fn get_by_id(&self, id: &str) -> Result<Option<MediaFile>, FileError> {
        // 1. Check cache first (nanoseconds)
        {
            let cache = self.cache.read().await;
            if let Some(file) = cache.get(id) {
                return Ok(Some(file.clone()));
            }
        }
        
        // 2. Load from database and cache
        if let Some(file) = self.db.get_by_id(id).await? {
            self.cache.write().await.insert(file.id.clone(), file.clone());
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }
    
    pub async fn delete_media(&self, id: String) -> Result<(), FileError> {
        // Remove from database
        self.db.delete_media_file(&id).await?;
        
        // Invalidate cache
        self.cache.write().await.remove(&id);
        
        Ok(())
    }
    
    fn calculate_hash(&self, path: &Path) -> Result<String, FileError> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 8192];
        
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    async fn extract_metadata(&self, path: &Path) -> Result<FileMetadata, FileError> {
        // Delegate to metadata module (see section 2 below)
        extract_metadata(path).await
    }
}
```

### 2. Metadata Extraction with FFprobe

```rust
// src-tauri/src/metadata.rs
use std::process::Command;
use serde_json::Value;

pub async fn extract_metadata(path: &Path) -> Result<FileMetadata, MetadataError> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            "-show_format",
            path.to_str().unwrap(),
        ])
        .output()?;
    
    if !output.status.success() {
        return Err(MetadataError::FFprobeError);
    }
    
    let json: Value = serde_json::from_slice(&output.stdout)?;
    
    // Extract video stream info
    let video_stream = json["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or(MetadataError::NoVideoStream)?;
    
    let audio_stream = json["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["codec_type"] == "audio");
    
    let format = &json["format"];
    
    Ok(FileMetadata {
        duration: format["duration"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        resolution: Resolution {
            width: video_stream["width"].as_u64().unwrap() as u32,
            height: video_stream["height"].as_u64().unwrap() as u32,
        },
        codec: VideoCodec {
            video: video_stream["codec_name"]
                .as_str()
                .unwrap()
                .to_string(),
            audio: audio_stream
                .map(|s| s["codec_name"].as_str().unwrap().to_string())
                .unwrap_or_else(|| "none".to_string()),
        },
        bitrate: format["bit_rate"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        framerate: parse_framerate(video_stream["r_frame_rate"].as_str().unwrap()),
        has_audio: audio_stream.is_some(),
    })
}

fn parse_framerate(fps_str: &str) -> f64 {
    // Parse "30/1" or "24000/1001" format
    let parts: Vec<&str> = fps_str.split('/').collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().unwrap_or(30.0);
        let den: f64 = parts[1].parse().unwrap_or(1.0);
        num / den
    } else {
        30.0
    }
}
```

### 3. Thumbnail Generation

```rust
// src-tauri/src/thumbnail.rs
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

pub struct ThumbnailGenerator {
    cache_dir: PathBuf,
}

impl ThumbnailGenerator {
    pub fn new() -> Result<Self, std::io::Error> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cache directory not found"
            ))?
            .join("clipforge")
            .join("thumbnails");
        
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self { cache_dir })
    }
    
    pub async fn generate(
        &self,
        video_path: &Path,
        timestamp: f64
    ) -> Result<PathBuf, ThumbnailError> {
        let thumb_filename = format!("{}.jpg", Uuid::new_v4());
        let output_path = self.cache_dir.join(&thumb_filename);
        
        let status = Command::new("ffmpeg")
            .args(&[
                "-ss", &timestamp.to_string(),
                "-i", video_path.to_str().unwrap(),
                "-vframes", "1",
                "-vf", "scale=320:-1",
                "-q:v", "2",
                "-y",
                output_path.to_str().unwrap(),
            ])
            .status()?;
        
        if !status.success() {
            return Err(ThumbnailError::GenerationFailed);
        }
        
        Ok(output_path)
    }
    
    pub async fn generate_sequence(
        &self,
        video_path: &Path,
        duration: f64,
        count: usize
    ) -> Result<Vec<PathBuf>, ThumbnailError> {
        let mut thumbnails = Vec::new();
        let interval = duration / count as f64;
        
        for i in 0..count {
            let timestamp = i as f64 * interval;
            let thumb = self.generate(video_path, timestamp).await?;
            thumbnails.push(thumb);
        }
        
        Ok(thumbnails)
    }
}
```

### 4. SQLite Database Schema (Optimized)

```sql
-- src-tauri/src/database/schema.sql
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
    hash TEXT NOT NULL UNIQUE,      -- UNIQUE constraint = automatic index
    imported_at TEXT NOT NULL
);

-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_hash ON media_files(hash);           -- O(log n) duplicate detection
CREATE INDEX IF NOT EXISTS idx_imported_at ON media_files(imported_at DESC);  -- Recent files first
CREATE INDEX IF NOT EXISTS idx_path ON media_files(path);           -- Find by filesystem path

-- Optional: Full-text search on filename
-- CREATE VIRTUAL TABLE IF NOT EXISTS media_files_fts USING fts5(filename, content='media_files', content_rowid='rowid');
```

**Index Strategy Explained:**
- `hash` index: Fast duplicate detection (critical for imports)
- `imported_at` index: Recent files queries (common UI pattern)
- `path` index: Check if file already imported by path
- Primary key on `id`: Fast lookups by UUID

### 5. Database Operations (with Cache-Aware Patterns)

```rust
// src-tauri/src/database/mod.rs
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, rusqlite::Error> {
        let db_path = dirs::data_local_dir()
            .unwrap()
            .join("clipforge")
            .join("media.db");
        
        std::fs::create_dir_all(db_path.parent().unwrap())?;
        
        let conn = Connection::open(db_path)?;
        Self::init_schema(&conn)?;
        
        Ok(Self { conn })
    }
    
    fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(include_str!("schema.sql"))?;
        Ok(())
    }
    
    pub async fn insert_media_file(&self, file: &MediaFile) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO media_files VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                file.id,
                file.path.to_str().unwrap(),
                file.filename,
                file.duration,
                file.resolution.width,
                file.resolution.height,
                file.codec.video,
                file.codec.audio,
                file.file_size,
                file.thumbnail_path.as_ref().map(|p| p.to_str().unwrap()),
                file.hash,
                file.imported_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
    
    pub async fn find_by_hash(&self, hash: &str) -> Result<Option<MediaFile>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM media_files WHERE hash = ?1 LIMIT 1"
        )?;
        
        let mut rows = stmt.query(params![hash])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_media_file(row)?))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_by_id(&self, id: &str) -> Result<Option<MediaFile>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM media_files WHERE id = ?1"
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_media_file(row)?))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_all(&self) -> Result<Vec<MediaFile>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM media_files ORDER BY imported_at DESC"
        )?;
        
        let rows = stmt.query_map([], Self::row_to_media_file)?;
        
        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }
        
        Ok(files)
    }
    
    pub async fn delete_media_file(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM media_files WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
    
    // Query by filters (leverages indexes)
    pub async fn find_by_codec(&self, codec: &str) -> Result<Vec<MediaFile>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM media_files WHERE video_codec = ?1 ORDER BY imported_at DESC"
        )?;
        
        let rows = stmt.query_map(params![codec], Self::row_to_media_file)?;
        
        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }
        
        Ok(files)
    }
    
    fn row_to_media_file(row: &rusqlite::Row) -> Result<MediaFile, rusqlite::Error> {
        Ok(MediaFile {
            id: row.get(0)?,
            path: PathBuf::from(row.get::<_, String>(1)?),
            filename: row.get(2)?,
            duration: row.get(3)?,
            resolution: Resolution {
                width: row.get(4)?,
                height: row.get(5)?,
            },
            codec: VideoCodec {
                video: row.get(6)?,
                audio: row.get(7)?,
            },
            file_size: row.get(8)?,
            thumbnail_path: row.get::<_, Option<String>>(9)?.map(PathBuf::from),
            hash: row.get(10)?,
            imported_at: row.get::<_, String>(11)?.parse().unwrap(),
        })
    }
}

// Performance tip: For bulk operations, use transactions
impl Database {
    pub async fn insert_multiple(&self, files: &[MediaFile]) -> Result<(), rusqlite::Error> {
        let tx = self.conn.transaction()?;
        
        for file in files {
            tx.execute(
                "INSERT INTO media_files VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    file.id,
                    file.path.to_str().unwrap(),
                    file.filename,
                    file.duration,
                    file.resolution.width,
                    file.resolution.height,
                    file.codec.video,
                    file.codec.audio,
                    file.file_size,
                    file.thumbnail_path.as_ref().map(|p| p.to_str().unwrap()),
                    file.hash,
                    file.imported_at.to_rfc3339(),
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }
}
```

### 6. Tauri Commands

```rust
// src-tauri/src/commands/file_commands.rs
use crate::file_service::FileService;
use crate::media_library::MediaFile;
use tauri::State;

#[tauri::command]
pub async fn import_media_file(
    path: String,
    file_service: State<'_, FileService>
) -> Result<MediaFile, String> {
    file_service
        .import_file(PathBuf::from(path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_media_library(
    file_service: State<'_, FileService>
) -> Result<Vec<MediaFile>, String> {
    file_service
        .get_all_media()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_media_file(
    id: String,
    file_service: State<'_, FileService>
) -> Result<(), String> {
    file_service
        .delete_media(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_metadata(
    path: String
) -> Result<FileMetadata, String> {
    extract_metadata(&PathBuf::from(path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_thumbnail(
    video_path: String,
    timestamp: f64,
    thumb_service: State<'_, ThumbnailGenerator>
) -> Result<String, String> {
    let path = thumb_service
        .generate(&PathBuf::from(video_path), timestamp)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(path.to_string_lossy().to_string())
}
```

## Dependencies

```toml
[dependencies]
rusqlite = { version = "0.30", features = ["bundled"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
sha2 = "0.10"
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    async fn setup_test_service() -> FileService {
        let db = Database::new_test().unwrap();
        let thumb_gen = ThumbnailGenerator::new().unwrap();
        FileService::new(db, thumb_gen)
    }
    
    #[tokio::test]
    async fn test_import_file() {
        let service = setup_test_service().await;
        let result = service.import_file(PathBuf::from("test.mp4")).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_deduplication() {
        let service = setup_test_service().await;
        
        // Import same file twice
        let file1 = service.import_file(PathBuf::from("test.mp4")).await.unwrap();
        let file2 = service.import_file(PathBuf::from("test.mp4")).await.unwrap();
        
        // Should return same ID (deduplication via hash)
        assert_eq!(file1.id, file2.id);
        assert_eq!(file1.hash, file2.hash);
    }
    
    #[tokio::test]
    async fn test_cache_consistency() {
        let service = setup_test_service().await;
        
        // Import file
        let file1 = service.import_file(PathBuf::from("test.mp4")).await.unwrap();
        
        // Retrieve by ID (should hit cache)
        let file2 = service.get_by_id(&file1.id).await.unwrap().unwrap();
        
        assert_eq!(file1.id, file2.id);
        assert_eq!(file1.path, file2.path);
    }
    
    #[tokio::test]
    async fn test_delete_invalidates_cache() {
        let service = setup_test_service().await;
        
        // Import and cache
        let file = service.import_file(PathBuf::from("test.mp4")).await.unwrap();
        
        // Delete should remove from both DB and cache
        service.delete_media(file.id.clone()).await.unwrap();
        
        // Should not be found
        let result = service.get_by_id(&file.id).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_bulk_import_performance() {
        let service = setup_test_service().await;
        let start = std::time::Instant::now();
        
        // Simulate importing 100 files
        for i in 0..100 {
            let path = PathBuf::from(format!("test_{}.mp4", i));
            service.import_file(path).await.ok();
        }
        
        let elapsed = start.elapsed();
        // Should complete in reasonable time
        assert!(elapsed.as_secs() < 30);
    }
}

// Database-specific tests
#[cfg(test)]
mod database_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hash_index_performance() {
        let db = Database::new_test().unwrap();
        
        // Insert 1000 files
        for i in 0..1000 {
            let file = MediaFile {
                id: format!("id_{}", i),
                hash: format!("hash_{}", i),
                // ... other fields
            };
            db.insert_media_file(&file).await.unwrap();
        }
        
        // Lookup should be fast (indexed)
        let start = std::time::Instant::now();
        let result = db.find_by_hash("hash_500").await.unwrap();
        let elapsed = start.elapsed();
        
        assert!(result.is_some());
        assert!(elapsed.as_millis() < 10); // Should be <10ms
    }
}
```

## Acceptance Criteria

- [ ] Import MP4, MOV, WebM, AVI, MKV files
- [ ] Display accurate metadata (duration, resolution, codec)
- [ ] Generate thumbnails within 2 seconds
- [ ] Detect and skip duplicate files
- [ ] Handle missing/moved files gracefully
- [ ] Support files up to 100GB
- [ ] Media library persists between sessions

---

**Status:** Not Started  
**Target Completion:** Week 1, End
