use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;
use chrono::Utc;

use crate::database::Database;
use crate::thumbnail::ThumbnailGenerator;
use crate::metadata::extract_metadata;
use crate::models::{MediaFile, FileError, ProxyStatus};

/// Service for managing media file imports and library
pub struct FileService {
    db: Arc<Database>,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,
    thumbnail_generator: Arc<ThumbnailGenerator>,
}

impl FileService {
    /// Create a new file service
    pub fn new(db: Database, thumbnail_generator: ThumbnailGenerator) -> Self {
        Self {
            db: Arc::new(db),
            cache: Arc::new(RwLock::new(HashMap::new())),
            thumbnail_generator: Arc::new(thumbnail_generator),
        }
    }

    /// Import a video file into the media library
    pub async fn import_file(&self, path: PathBuf) -> Result<MediaFile, FileError> {
        // 1. Validate file exists and is readable
        if !path.exists() {
            return Err(FileError::FileNotFound(path));
        }

        log::info!("Importing file: {:?}", path);

        // 2. Calculate hash for deduplication
        let hash = self.calculate_hash(&path)?;

        // 3. Get file size for additional collision detection
        let file_size = std::fs::metadata(&path)?.len();

        // 4. Check database for duplicate (indexed lookup)
        if let Some(existing) = self.db.find_by_hash(&hash)? {
            // Additional verification: compare file size to detect extremely rare hash collisions
            if existing.file_size == file_size {
                log::info!("File already exists in library (duplicate): {:?}", path);
                // Warm cache with existing file
                self.cache.write().await.insert(existing.id.clone(), existing.clone());
                return Ok(existing);
            } else {
                // Extremely rare: hash collision with different file sizes
                log::warn!(
                    "Hash collision detected! Same hash but different sizes: existing={} bytes, new={} bytes",
                    existing.file_size, file_size
                );
                // Continue with import - allow both files
            }
        }

        // 4. Extract metadata using FFprobe
        let metadata = extract_metadata(&path).await
            .map_err(|e| FileError::MetadataError(e.to_string()))?;

        log::debug!("Extracted metadata: duration={:.2}s, resolution={}x{}",
                   metadata.duration, metadata.resolution.width, metadata.resolution.height);

        // 5. Generate thumbnail (at 5 seconds into the video, or 0 if shorter)
        let thumb_timestamp = if metadata.duration > 5.0 { 5.0 } else { 0.0 };
        let thumbnail_path = self.thumbnail_generator
            .generate(&path, thumb_timestamp)
            .await
            .map_err(|_| FileError::ThumbnailError)?;

        // 6. Create MediaFile object
        let media_file = MediaFile {
            id: Uuid::new_v4().to_string(),
            path: path.clone(),
            filename: path.file_name()
                .ok_or(FileError::InvalidFormat)?
                .to_string_lossy()
                .to_string(),
            duration: metadata.duration,
            resolution: metadata.resolution,
            codec: metadata.codec,
            file_size,  // Use previously calculated file_size
            thumbnail_path: Some(thumbnail_path),
            hash,
            imported_at: Utc::now(),
            proxy_path: None,                   // No proxy initially
            has_proxy: false,                   // No proxy initially
            proxy_status: ProxyStatus::None,    // No proxy initially
        };

        // 7. Save to database (persistent)
        self.db.insert_media_file(&media_file)?;

        // 8. Cache for fast subsequent access
        self.cache.write().await.insert(media_file.id.clone(), media_file.clone());

        log::info!("File imported successfully: {} ({})", media_file.filename, media_file.id);

        Ok(media_file)
    }

    /// Get all media files in the library
    pub async fn get_all_media(&self) -> Result<Vec<MediaFile>, FileError> {
        // Load from database
        let files = self.db.get_all()?;

        // Populate cache for subsequent lookups
        let mut cache = self.cache.write().await;
        for file in &files {
            cache.insert(file.id.clone(), file.clone());
        }

        Ok(files)
    }

    /// Get media file by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<MediaFile>, FileError> {
        // 1. Check cache first (fast path)
        {
            let cache = self.cache.read().await;
            if let Some(file) = cache.get(id) {
                return Ok(Some(file.clone()));
            }
        }

        // 2. Load from database and cache
        if let Some(file) = self.db.get_by_id(id)? {
            self.cache.write().await.insert(file.id.clone(), file.clone());
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Delete media file from library
    pub async fn delete_media(&self, id: String) -> Result<(), FileError> {
        log::info!("Deleting media file: {}", id);

        // Remove from database
        self.db.delete_media_file(&id)?;

        // Invalidate cache
        self.cache.write().await.remove(&id);

        Ok(())
    }

    /// Calculate SHA-256 hash of a file
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

    /// Get database reference for advanced queries
    #[allow(dead_code)]
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Get thumbnail generator reference
    pub fn thumbnail_generator(&self) -> &ThumbnailGenerator {
        &self.thumbnail_generator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_service_creation() {
        let db = Database::new().unwrap();
        let thumb_gen = ThumbnailGenerator::new().unwrap();
        let service = FileService::new(db, thumb_gen);

        // Service should be created successfully
        assert!(service.db.get_all().is_ok());
    }
}
