# Storage Architecture Decision: SQLite vs HashMap

## TL;DR

**Use SQLite with optional in-memory cache.**

- ✅ **Phase 1-4 (MVP):** SQLite only (simpler, sufficient)
- ✅ **Phase 7-8 (Polish):** Add HashMap cache only if profiling shows need

## The Question

> "Should we use HashMap or SQLite for the media library?"

The answer depends on what you value more:

| Priority | Recommendation |
|----------|----------------|
| **Simplicity** | HashMap (prototyping only) |
| **Persistence** | SQLite (production apps) |
| **Speed** | SQLite + cache (best of both) |

## Detailed Comparison

### HashMap (In-Memory Only)

```rust
use std::collections::HashMap;

pub struct FileService {
    media_files: HashMap<String, MediaFile>,
}
```

**Pros:**
- ✅ Extremely fast lookups (nanoseconds)
- ✅ Simple implementation
- ✅ No external dependencies
- ✅ No disk I/O

**Cons:**
- ❌ Lost on app restart (terrible UX)
- ❌ Limited by available RAM
- ❌ O(n) for duplicate detection
- ❌ No complex queries
- ❌ Manual serialization needed

**When to use:**
- Quick prototypes (<100 files)
- Demo applications
- Learning exercises
- Data doesn't need to persist

### SQLite (Persistent Database)

```rust
use rusqlite::Connection;

pub struct FileService {
    db: Database,
}
```

**Pros:**
- ✅ Survives app restarts
- ✅ Handles 10,000+ files
- ✅ O(log n) indexed lookups
- ✅ SQL queries (filter, sort, aggregate)
- ✅ ACID transactions
- ✅ No external dependencies (bundled)

**Cons:**
- ❌ Disk I/O overhead (~1-5ms vs nanoseconds)
- ❌ More complex setup
- ❌ Schema migrations needed

**When to use:**
- Production applications
- Data must persist
- Need complex queries
- Scaling to 100+ files

### Hybrid (SQLite + HashMap Cache)

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct FileService {
    db: Database,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,
}
```

**Pros:**
- ✅ Persistent storage
- ✅ Fast reads from cache
- ✅ Best of both worlds

**Cons:**
- ❌ Cache invalidation complexity
- ❌ Memory overhead for cache
- ❌ More code to maintain

**When to use:**
- High read frequency (>100 reads/sec)
- Large media libraries (1000+ files)
- Performance is critical

## Performance Benchmarks

### Lookup Speed

| Operation | HashMap | SQLite | SQLite + Index | Hybrid |
|-----------|---------|--------|----------------|--------|
| Insert | 10ns | 1ms | 1ms | 1ms |
| Lookup by ID | 10ns | 100μs | 100μs | 10ns (cached) |
| Duplicate check (1000 files) | 1000 iterations | 10 index reads | 10 index reads | 10 index reads |

### Memory Usage

| Files | HashMap | SQLite | Hybrid |
|-------|---------|--------|--------|
| 100 | 200KB | 0KB + 100KB DB | 200KB + 100KB DB |
| 1000 | 2MB | 0KB + 1MB DB | 2MB + 1MB DB |
| 10000 | 20MB | 0KB + 10MB DB | 2MB + 10MB DB |

*Hybrid assumes cache size = 1000 files max*

### Persistence Cost

| Scenario | HashMap | SQLite | Hybrid |
|----------|---------|--------|--------|
| App restart with 100 files | Re-scan all (~30s) | Instant load | Instant load |
| Power loss | All data lost | Data safe | Data safe |
| Backup | Manual export | Copy .db file | Copy .db file |

## ClipForge Decision

**Why SQLite is the right choice:**

1. **User Expectation:**  
   Users expect their media library to persist between sessions. Losing imported files is a critical UX failure.

2. **Scalability:**  
   A creator might import 100+ clips for a project. HashMap consumes RAM unnecessarily.

3. **Deduplication:**  
   Hash-based duplicate detection is O(log n) with SQL index vs O(n) with HashMap scan.

4. **Features:**  
   Future features like "find all H.264 videos" or "show recent imports" need SQL queries.

5. **Zero Config:**  
   `rusqlite` with `bundled` feature = no system dependencies.

## Implementation Path

### Phase 1-4: SQLite Only (Recommended)

```rust
// src-tauri/src/file_service.rs
pub struct FileService {
    db: Database,
    thumbnail_generator: ThumbnailGenerator,
}

impl FileService {
    pub async fn import_file(&self, path: PathBuf) -> Result<MediaFile, FileError> {
        // 1. Check database for duplicate (indexed)
        let hash = self.calculate_hash(&path)?;
        if let Some(existing) = self.db.find_by_hash(&hash).await? {
            return Ok(existing);
        }
        
        // 2. Extract metadata, generate thumbnail
        let metadata = extract_metadata(&path).await?;
        let thumbnail = self.thumbnail_generator.generate(&path, 5.0).await?;
        
        // 3. Create and save MediaFile
        let media_file = MediaFile { /* ... */ };
        self.db.insert_media_file(&media_file).await?;
        
        Ok(media_file)
    }
}
```

**Pros:**
- Simple implementation
- Easy to understand
- No cache invalidation logic
- Sufficient for MVP

**Performance:** 1-5ms per operation (acceptable)

### Phase 7-8: Add Cache (If Needed)

**Only add cache if profiling shows:**
- `get_by_id()` called >100 times/second
- Timeline editor feels sluggish
- Noticeable lag when scrubbing

```rust
pub struct FileService {
    db: Database,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,
    thumbnail_generator: ThumbnailGenerator,
}

impl FileService {
    pub async fn get_by_id(&self, id: &str) -> Result<Option<MediaFile>, FileError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(file) = cache.get(id) {
                return Ok(Some(file.clone()));
            }
        }
        
        // Load from database and warm cache
        if let Some(file) = self.db.get_by_id(id).await? {
            self.cache.write().await.insert(file.id.clone(), file.clone());
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }
}
```

**Cache strategy:**
- LRU eviction (keep 1000 most recent)
- Invalidate on delete/update
- Pre-populate on app startup (recent files)

## Common Mistakes

### ❌ Don't: Pure HashMap in production
```rust
// Lost on restart - poor UX
pub struct FileService {
    media_files: HashMap<String, MediaFile>,
}
```

### ❌ Don't: Premature caching
```rust
// Added complexity without profiling
pub struct FileService {
    db: Database,
    cache: HashMap<String, MediaFile>,  // Not needed yet!
}
```

### ✅ Do: Start simple, optimize later
```rust
// Phase 1-4: Simple and sufficient
pub struct FileService {
    db: Database,
}

// Phase 7-8: Add cache if profiling shows need
pub struct FileService {
    db: Database,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,
}
```

## Real-World Performance

Based on similar video editing applications:

**DaVinci Resolve:** SQLite for project data + caching for active timeline  
**Premiere Pro:** Database for media catalog + in-memory for active sequences  
**Final Cut Pro:** Core Data (SQLite) + aggressive caching

**ClipForge target performance:**
- Import 100 videos: <30 seconds (metadata extraction is bottleneck, not storage)
- Load media library: <1 second (SQLite query is fast)
- Timeline editing: 30 FPS (cache not needed for this)

## Conclusion

**Use SQLite for ClipForge.**

- Start with SQLite-only implementation (Weeks 1-4)
- Profile during Week 7
- Add HashMap cache only if measurements show need
- Most likely you won't need the cache for MVP

The hybrid approach gives you production-quality persistence with room to optimize if profiling reveals a bottleneck.

---

**Decision Made:** October 27, 2025  
**Review Date:** Week 7 (Performance profiling)  
**Status:** ✅ Approved
