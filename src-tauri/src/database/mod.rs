use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::models::{MediaFile, MediaType, Resolution, MediaCodec, ProxyStatus};

/// Database wrapper for media library storage
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Helper to handle mutex lock with proper error handling for poisoned locks
    fn lock_conn(&self) -> Result<std::sync::MutexGuard<Connection>, rusqlite::Error> {
        self.conn.lock().map_err(|e| {
            log::error!("Database mutex poisoned: {}", e);
            rusqlite::Error::InvalidPath(
                format!("Database connection unavailable (mutex poisoned): {}", e).into()
            )
        })
    }

    /// Create a new database connection
    pub fn new() -> Result<Self, rusqlite::Error> {
        let db_path = dirs::data_local_dir()
            .ok_or_else(|| rusqlite::Error::InvalidPath("Data directory not found".into()))?
            .join("clipforge")
            .join("media.db");

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::InvalidPath(format!("Failed to create directory: {}", e).into())
            })?;
        }

        let conn = Connection::open(&db_path)?;
        Self::init_schema(&conn)?;

        log::info!("Database initialized at: {:?}", db_path);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Initialize database schema
    fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(include_str!("schema.sql"))?;

        // Run migrations for existing databases
        Self::run_migrations(conn)?;

        Ok(())
    }

    /// Run database migrations to add new columns to existing tables
    fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
        // Check if proxy columns exist, if not add them
        let has_proxy_path = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('media_files') WHERE name='proxy_path'",
            [],
            |row| row.get::<_, i32>(0)
        )? > 0;

        if !has_proxy_path {
            log::info!("Running migration: Adding proxy support columns");
            conn.execute_batch(
                "ALTER TABLE media_files ADD COLUMN proxy_path TEXT;
                 ALTER TABLE media_files ADD COLUMN has_proxy INTEGER DEFAULT 0;
                 ALTER TABLE media_files ADD COLUMN proxy_status TEXT DEFAULT 'none';"
            )?;
            log::info!("Migration complete: Proxy columns added");
        }

        // Check if media_type column exists, if not add it
        let has_media_type = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('media_files') WHERE name='media_type'",
            [],
            |row| row.get::<_, i32>(0)
        )? > 0;

        if !has_media_type {
            log::info!("Running migration: Adding media_type column for audio support");
            conn.execute_batch(
                "ALTER TABLE media_files ADD COLUMN media_type TEXT NOT NULL DEFAULT 'video';"
            )?;
            log::info!("Migration complete: media_type column added");
        }

        Ok(())
    }

    /// Insert a new media file
    pub fn insert_media_file(&self, file: &MediaFile) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn()?;

        let path_str = file.path.to_str()
            .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path contains invalid UTF-8 characters"
                ))
            ))?;

        let thumbnail_str = file.thumbnail_path.as_ref()
            .map(|p| p.to_str()
                .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Thumbnail path contains invalid UTF-8 characters"
                    ))
                ))
            )
            .transpose()?;

        let proxy_str = file.proxy_path.as_ref()
            .map(|p| p.to_str()
                .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Proxy path contains invalid UTF-8 characters"
                    ))
                ))
            )
            .transpose()?;

        let proxy_status_str = match file.proxy_status {
            ProxyStatus::None => "none",
            ProxyStatus::Generating => "generating",
            ProxyStatus::Ready => "ready",
            ProxyStatus::Failed => "failed",
        };

        let media_type_str = match file.media_type {
            MediaType::Video => "video",
            MediaType::Audio => "audio",
            MediaType::Image => "image",
        };

        conn.execute(
            "INSERT INTO media_files VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                file.id,
                path_str,
                file.filename,
                media_type_str,
                file.duration,
                file.resolution.as_ref().map(|r| r.width),
                file.resolution.as_ref().map(|r| r.height),
                file.codec.video,
                file.codec.audio,
                file.file_size,
                thumbnail_str,
                file.hash,
                file.imported_at.to_rfc3339(),
                proxy_str,
                file.has_proxy as i32,
                proxy_status_str,
            ],
        )?;
        Ok(())
    }

    /// Find media file by hash (for deduplication)
    pub fn find_by_hash(&self, hash: &str) -> Result<Option<MediaFile>, rusqlite::Error> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM media_files WHERE hash = ?1 LIMIT 1"
        )?;

        let mut rows = stmt.query(params![hash])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_media_file(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get media file by ID
    pub fn get_by_id(&self, id: &str) -> Result<Option<MediaFile>, rusqlite::Error> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM media_files WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_media_file(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all media files, sorted by import date (newest first)
    pub fn get_all(&self) -> Result<Vec<MediaFile>, rusqlite::Error> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM media_files ORDER BY imported_at DESC"
        )?;

        let rows = stmt.query_map([], Self::row_to_media_file)?;

        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }

        Ok(files)
    }

    /// Delete media file by ID
    pub fn delete_media_file(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM media_files WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Find media files by codec
    #[allow(dead_code)]
    pub fn find_by_codec(&self, codec: &str) -> Result<Vec<MediaFile>, rusqlite::Error> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM media_files WHERE video_codec = ?1 ORDER BY imported_at DESC"
        )?;

        let rows = stmt.query_map(params![codec], Self::row_to_media_file)?;

        let mut files = Vec::new();
        for file in rows {
            files.push(file?);
        }

        Ok(files)
    }

    /// Convert database row to MediaFile struct
    fn row_to_media_file(row: &rusqlite::Row) -> Result<MediaFile, rusqlite::Error> {
        let proxy_status_str: Option<String> = row.get(15).ok();
        let proxy_status = match proxy_status_str.as_deref() {
            Some("generating") => ProxyStatus::Generating,
            Some("ready") => ProxyStatus::Ready,
            Some("failed") => ProxyStatus::Failed,
            _ => ProxyStatus::None,
        };

        let media_type_str: String = row.get(3)?;
        let media_type = match media_type_str.as_str() {
            "audio" => MediaType::Audio,
            "image" => MediaType::Image,
            _ => MediaType::Video,
        };

        // Resolution is optional for audio files
        let width: Option<u32> = row.get(5)?;
        let height: Option<u32> = row.get(6)?;
        let resolution = match (width, height) {
            (Some(w), Some(h)) => Some(Resolution { width: w, height: h }),
            _ => None,
        };

        Ok(MediaFile {
            id: row.get(0)?,
            path: PathBuf::from(row.get::<_, String>(1)?),
            filename: row.get(2)?,
            media_type,
            duration: row.get(4)?,
            resolution,
            codec: MediaCodec {
                video: row.get(7)?,
                audio: row.get(8)?,
            },
            file_size: row.get(9)?,
            thumbnail_path: row.get::<_, Option<String>>(10)?.map(PathBuf::from),
            hash: row.get(11)?,
            imported_at: row.get::<_, String>(12)?
                .parse()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    12,
                    rusqlite::types::Type::Text,
                    Box::new(e)
                ))?,
            proxy_path: row.get::<_, Option<String>>(13).ok().flatten().map(PathBuf::from),
            has_proxy: row.get::<_, Option<i32>>(14).ok().flatten().unwrap_or(0) != 0,
            proxy_status,
        })
    }

    /// Bulk insert multiple files (using transaction for performance)
    #[allow(dead_code)]
    pub fn insert_multiple(&self, files: &[MediaFile]) -> Result<(), rusqlite::Error> {
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction()?;

        for file in files {
            let path_str = file.path.to_str()
                .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Path contains invalid UTF-8 characters"
                    ))
                ))?;

            let thumbnail_str = file.thumbnail_path.as_ref()
                .map(|p| p.to_str()
                    .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Thumbnail path contains invalid UTF-8 characters"
                        ))
                    ))
                )
                .transpose()?;

            let proxy_str = file.proxy_path.as_ref()
                .map(|p| p.to_str()
                    .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Proxy path contains invalid UTF-8 characters"
                        ))
                    ))
                )
                .transpose()?;

            let proxy_status_str = match file.proxy_status {
                ProxyStatus::None => "none",
                ProxyStatus::Generating => "generating",
                ProxyStatus::Ready => "ready",
                ProxyStatus::Failed => "failed",
            };

            let media_type_str = match file.media_type {
                MediaType::Video => "video",
                MediaType::Audio => "audio",
                MediaType::Image => "image",
            };

            tx.execute(
                "INSERT INTO media_files VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    file.id,
                    path_str,
                    file.filename,
                    media_type_str,
                    file.duration,
                    file.resolution.as_ref().map(|r| r.width),
                    file.resolution.as_ref().map(|r| r.height),
                    file.codec.video,
                    file.codec.audio,
                    file.file_size,
                    thumbnail_str,
                    file.hash,
                    file.imported_at.to_rfc3339(),
                    proxy_str,
                    file.has_proxy as i32,
                    proxy_status_str,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Update proxy status and path for a media file
    pub fn update_proxy_status(
        &self,
        file_id: &str,
        proxy_path: Option<PathBuf>,
        status: ProxyStatus,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn()?;

        let proxy_str = proxy_path.as_ref()
            .map(|p| p.to_str()
                .ok_or_else(|| rusqlite::Error::ToSqlConversionFailure(
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Proxy path contains invalid UTF-8 characters"
                    ))
                ))
            )
            .transpose()?;

        let status_str = match status {
            ProxyStatus::None => "none",
            ProxyStatus::Generating => "generating",
            ProxyStatus::Ready => "ready",
            ProxyStatus::Failed => "failed",
        };

        let has_proxy = matches!(status, ProxyStatus::Ready) as i32;

        conn.execute(
            "UPDATE media_files SET proxy_path = ?1, has_proxy = ?2, proxy_status = ?3 WHERE id = ?4",
            params![proxy_str, has_proxy, status_str, file_id],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new();
        assert!(db.is_ok());
    }
}
