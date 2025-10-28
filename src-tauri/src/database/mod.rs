use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::models::{MediaFile, Resolution, VideoCodec};

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

        conn.execute(
            "INSERT INTO media_files VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                file.id,
                path_str,
                file.filename,
                file.duration,
                file.resolution.width,
                file.resolution.height,
                file.codec.video,
                file.codec.audio,
                file.file_size,
                thumbnail_str,
                file.hash,
                file.imported_at.to_rfc3339(),
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
            imported_at: row.get::<_, String>(11)?
                .parse()
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    Box::new(e)
                ))?,
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

            tx.execute(
                "INSERT INTO media_files VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    file.id,
                    path_str,
                    file.filename,
                    file.duration,
                    file.resolution.width,
                    file.resolution.height,
                    file.codec.video,
                    file.codec.audio,
                    file.file_size,
                    thumbnail_str,
                    file.hash,
                    file.imported_at.to_rfc3339(),
                ],
            )?;
        }

        tx.commit()?;
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
