# Data Structures Reference

## Core Types

### MediaFile
```rust
pub struct MediaFile {
    pub id: String,
    pub path: PathBuf,
    pub filename: String,
    pub duration: f64,
    pub resolution: Resolution,
    pub codec: VideoCodec,
    pub file_size: u64,
    pub thumbnail_path: Option<PathBuf>,
    pub hash: String,
    pub imported_at: DateTime<Utc>,
}
```

### Timeline
```rust
pub struct Timeline {
    pub id: String,
    pub name: String,
    pub framerate: f64,
    pub resolution: Resolution,
    pub tracks: Vec<Track>,
    pub duration: f64,
}
```

### Track
```rust
pub struct Track {
    pub id: String,
    pub track_type: TrackType,
    pub clips: Vec<Clip>,
    pub enabled: bool,
    pub locked: bool,
}
```

### Clip
```rust
pub struct Clip {
    pub id: String,
    pub media_file_id: String,
    pub track_position: f64,
    pub duration: f64,
    pub trim_start: f64,
    pub trim_end: f64,
    pub effects: Vec<Effect>,
    pub volume: f64,
    pub speed: f64,
}
```

### Resolution
```rust
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}
```

### VideoCodec
```rust
pub struct VideoCodec {
    pub video: String,
    pub audio: String,
}
```

## Enums

### TrackType
```rust
pub enum TrackType {
    Video,
    Audio,
    Overlay,
}
```

### EffectType
```rust
pub enum EffectType {
    Brightness,
    Contrast,
    Saturation,
    Blur,
    FadeIn,
    FadeOut,
}
```
