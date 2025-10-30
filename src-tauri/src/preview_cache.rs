use log::{debug, info};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Frame cache for preview rendering
/// Stores rendered frames as JPEG bytes to speed up scrubbing
pub struct PreviewCache {
    /// LRU cache mapping timestamp (in milliseconds) to JPEG frame data
    frames: Arc<Mutex<LruCache<u64, Vec<u8>>>>,
    /// Maximum number of frames to cache
    capacity: usize,
}

impl PreviewCache {
    /// Create a new preview cache
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of frames to cache (default: 100)
    ///   - Typical 1080p JPEG: ~150KB
    ///   - 100 frames = ~15MB memory usage
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        info!("Initializing preview cache with capacity: {}", capacity);

        Self {
            frames: Arc::new(Mutex::new(LruCache::new(cap))),
            capacity,
        }
    }

    /// Get a frame from cache
    ///
    /// # Arguments
    /// * `timestamp` - Time in seconds
    ///
    /// # Returns
    /// * `Some(Vec<u8>)` - JPEG frame data if cached
    /// * `None` - If frame not in cache
    pub async fn get(&self, timestamp: f64) -> Option<Vec<u8>> {
        let key = self.timestamp_to_key(timestamp);
        let mut cache = self.frames.lock().await;

        if let Some(frame) = cache.get(&key) {
            debug!("Cache hit for timestamp: {}s (key: {}ms)", timestamp, key);
            Some(frame.clone())
        } else {
            debug!("Cache miss for timestamp: {}s (key: {}ms)", timestamp, key);
            None
        }
    }

    /// Put a frame into the cache
    ///
    /// # Arguments
    /// * `timestamp` - Time in seconds
    /// * `frame_data` - JPEG encoded frame bytes
    pub async fn put(&self, timestamp: f64, frame_data: Vec<u8>) {
        let key = self.timestamp_to_key(timestamp);
        let frame_size_kb = frame_data.len() / 1024;

        debug!(
            "Caching frame at {}s (key: {}ms, size: {}KB)",
            timestamp, key, frame_size_kb
        );

        let mut cache = self.frames.lock().await;
        cache.put(key, frame_data);
    }

    /// Clear all cached frames
    pub async fn clear(&self) {
        info!("Clearing preview cache");
        let mut cache = self.frames.lock().await;
        cache.clear();
    }

    /// Get current cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.frames.lock().await;
        let current_size = cache.len();

        CacheStats {
            capacity: self.capacity,
            current_size,
            hit_rate: 0.0, // TODO: Track hits/misses for accurate hit rate
        }
    }

    /// Convert timestamp (seconds) to cache key (milliseconds)
    /// Rounds to nearest 100ms for better cache hit rate
    fn timestamp_to_key(&self, timestamp: f64) -> u64 {
        // Round to nearest 100ms to improve cache hit rate
        ((timestamp * 10.0).round() * 100.0) as u64
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub capacity: usize,
    pub current_size: usize,
    pub hit_rate: f64,
}

impl Default for PreviewCache {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let cache = PreviewCache::new(10);
        let test_data = vec![1, 2, 3, 4, 5];

        cache.put(1.5, test_data.clone()).await;

        let retrieved = cache.get(1.5).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_data);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = PreviewCache::new(10);
        let retrieved = cache.get(5.0).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_rounding() {
        let cache = PreviewCache::new(10);
        let test_data = vec![1, 2, 3];

        // Put at 1.54 seconds
        cache.put(1.54, test_data.clone()).await;

        // Should retrieve at 1.55 (rounds to same 100ms bucket)
        let retrieved = cache.get(1.55).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = PreviewCache::new(3);

        cache.put(1.0, vec![1]).await;
        cache.put(2.0, vec![2]).await;
        cache.put(3.0, vec![3]).await;
        cache.put(4.0, vec![4]).await; // Should evict 1.0

        assert!(cache.get(1.0).await.is_none());
        assert!(cache.get(4.0).await.is_some());
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = PreviewCache::new(10);

        cache.put(1.0, vec![1]).await;
        cache.put(2.0, vec![2]).await;

        cache.clear().await;

        assert!(cache.get(1.0).await.is_none());
        assert!(cache.get(2.0).await.is_none());
    }

    #[tokio::test]
    async fn test_stats() {
        let cache = PreviewCache::new(10);

        cache.put(1.0, vec![1]).await;
        cache.put(2.0, vec![2]).await;

        let stats = cache.stats().await;
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.current_size, 2);
    }
}
