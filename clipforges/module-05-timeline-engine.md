# Module 5: Timeline Engine & Edit Decision List

**Owner:** TBD  
**Dependencies:** Module 2 (File System)  
**Phase:** 1 (Weeks 1-2)  
**Estimated Effort:** 5-6 days

## Overview

Non-destructive editing engine that stores all edit decisions without modifying source files.

## Data Structures

```rust
pub struct Timeline {
    pub id: String,
    pub tracks: Vec<Track>,
    pub duration: f64,
}

pub struct Track {
    pub id: String,
    pub track_type: TrackType,
    pub clips: Vec<Clip>,
}

pub struct Clip {
    pub id: String,
    pub media_file_id: String,
    pub track_position: f64,
    pub duration: f64,
    pub trim_start: f64,
    pub trim_end: f64,
}
```

## Core Operations

- Add clip to timeline
- Remove clip
- Move clip (position/track)
- Trim clip (adjust in/out points)
- Split clip at time
- Get clips at playhead

## Serialization

Projects saved as JSON:
```json
{
  "timeline": {
    "id": "uuid",
    "tracks": [...]
  }
}
```

## Acceptance Criteria

- [ ] Add/remove clips from timeline
- [ ] Trim clips non-destructively
- [ ] Split clips at playhead
- [ ] Save/load projects as JSON
- [ ] Handle overlapping clips

---

**Status:** Not Started  
**Target Completion:** Week 2, End
