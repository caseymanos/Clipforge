# Documentation Changelog

## October 27, 2025 - Storage Architecture Updates

### Changes Made

Updated documentation to clarify the **SQLite vs HashMap** decision for media library storage, based on developer feedback about in-memory vs persistent storage trade-offs.

### Files Updated

1. **modules/module-02-file-system-media.md**
   - Added "Architecture Decision" section explaining SQLite + cache hybrid
   - Updated `FileService` implementation with cache-aware code
   - Added `get_by_id()` and `delete_media()` methods
   - Enhanced SQLite schema with better indexing strategy
   - Updated database operations with bulk insert example
   - Expanded testing section with cache consistency tests
   - Added performance benchmarks for hash index lookups

2. **architecture/02-technical-architecture.md**
   - Added new section: "Data Storage: SQLite vs In-Memory HashMap"
   - Performance comparison table (HashMap vs SQLite vs SQLite+Index)
   - Implementation strategy by phase
   - Decision criteria for when to use each approach

3. **architecture/dependencies.md**
   - Added optional caching libraries (`lru`, `dashmap`)
   - Notes on when to add cache dependencies (Week 7-8)
   - Rationale for bundled SQLite feature

4. **architecture/storage-decision.md** (NEW)
   - Comprehensive reference document
   - Detailed pros/cons comparison
   - Performance benchmarks with real numbers
   - Implementation path by project phase
   - Common mistakes to avoid
   - Real-world examples from professional tools

### Key Decisions Documented

**For MVP (Weeks 1-4):**
```rust
pub struct FileService {
    db: Database,  // SQLite only - simple, sufficient
    thumbnail_generator: ThumbnailGenerator,
}
```

**For Optimization (Weeks 7-8 if needed):**
```rust
pub struct FileService {
    db: Database,
    cache: Arc<RwLock<HashMap<String, MediaFile>>>,  // Add if profiling shows need
    thumbnail_generator: ThumbnailGenerator,
}
```

### Rationale

**Why SQLite:**
- ✅ Persistent across restarts (critical for UX)
- ✅ O(log n) hash lookups with indexes
- ✅ Handles 1000+ files without RAM issues
- ✅ SQL queries for filtering/sorting
- ✅ No external dependencies (bundled)

**Why not pure HashMap:**
- ❌ Lost on restart (poor UX)
- ❌ O(n) duplicate detection
- ❌ Limited by RAM
- ❌ No persistence

**Why hybrid is optional:**
- Add cache only if profiling shows >100ms lookups
- Most apps won't need it for MVP
- Start simple, optimize later

### Developer Impact

**What developers should do:**
1. Implement SQLite-backed storage first (simpler)
2. Use provided code examples from Module 02
3. Profile during Week 7
4. Add cache only if measurements show need

**What developers should NOT do:**
- Don't use pure HashMap for production
- Don't add cache prematurely
- Don't skip database indexing

### Testing Additions

New tests added to Module 02:
- `test_cache_consistency()` - Cache/DB sync
- `test_delete_invalidates_cache()` - Cache invalidation
- `test_bulk_import_performance()` - Batch operations
- `test_hash_index_performance()` - Index effectiveness

### Documentation Stats

- **Lines added:** ~500 lines across 4 files
- **Code examples:** 6 new implementations
- **Performance data:** 3 comparison tables
- **Decision guidance:** 1 comprehensive reference doc

### Next Steps

No action required. Documentation is complete and ready for development team.

---

**Updated by:** Claude  
**Date:** October 27, 2025  
**Version:** 1.1  
**Review:** Not required (clarification update)
