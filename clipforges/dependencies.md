# Dependencies Reference

## Rust Backend Dependencies

```toml
[dependencies]
# Core Tauri
tauri = { version = "2.0", features = ["macos-private-api"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1", features = ["full"] }

# File system
dirs = "5.0"

# Database
rusqlite = { version = "0.30", features = ["bundled"] }

# Hashing
sha2 = "0.10"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Logging
env_logger = "0.11"
log = "0.4"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Caching (optional, add in Phase 7-8 if needed)
# lru = "0.12"  # LRU cache for preview frames
# dashmap = "5.5"  # Concurrent HashMap alternative

# Platform-specific
[target.'cfg(target_os = "windows")'.dependencies]
windows-capture = "1.5"

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
core-foundation = "0.9"

[target.'cfg(target_os = "linux")'.dependencies]
gstreamer = "0.22"
gstreamer-video = "0.22"
```

## Dependency Notes

### SQLite (`rusqlite`)
- **`bundled` feature:** Includes SQLite library (no system dependency)
- **Why:** Persistent storage, indexed queries, cross-platform
- **Alternative considered:** `sled` (pure Rust), rejected due to less SQL query power

### Caching Libraries (Optional)
**Add only if profiling shows need:**

- **`lru`**: For preview frame caching (Module 8)
  ```rust
  use lru::LruCache;
  let cache: LruCache<u64, Vec<u8>> = LruCache::new(100);
  ```

- **`dashmap`**: For concurrent HashMap (if FileService cache added)
  ```rust
  use dashmap::DashMap;
  let cache: DashMap<String, MediaFile> = DashMap::new();
  // Lock-free reads, faster than Arc<RwLock<HashMap>>
  ```

**Decision:** Start without caching libraries, add in Week 7 if needed

## Frontend Dependencies

```json
{
  "dependencies": {
    "svelte": "^4.0.0",
    "konva": "^9.0.0",
    "wavesurfer.js": "^7.0.0",
    "@tauri-apps/api": "^2.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.0.0",
    "@tauri-apps/cli": "^2.0.0"
  }
}
```
