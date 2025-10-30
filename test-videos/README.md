# ClipForge Test Videos

This directory contains 20 sample video clips for performance testing ClipForge.

## Source

All videos downloaded from https://test-videos.co.uk/ which provides free Creative Commons test videos for development and testing purposes.

## Video Details

**Total Videos:** 20
**Total Size:** ~45MB
**Duration per video:** 10 seconds
**Formats:** MP4 (H.264 codec)
**Resolutions:** Mix of 360p, 720p, and 1080p

### File Breakdown

#### Big Buck Bunny (9 clips)
- 3x 360p (1MB, 2MB, 5MB)
- 3x 720p (1MB, 2MB, 5MB)
- 3x 1080p (1MB, 2MB, 5MB)

#### Sintel (6 clips)
- 2x 360p (1MB, 2MB)
- 2x 720p (1MB, 2MB)
- 2x 1080p (1MB, 2MB)

#### Jellyfish (5 clips)
- 1x 360p (1MB)
- 2x 720p (1MB, 2MB)
- 2x 1080p (1MB, 2MB)

## Usage

These videos are used for:

1. **Timeline FPS Testing** - Load all 20 clips onto timeline to measure rendering performance
2. **Memory Usage Testing** - Import all clips to measure memory footprint
3. **Export Speed Testing** - Create test timelines with varying complexity

## License

These are open-source test videos available for free use. Original sources:
- **Big Buck Bunny** - (c) copyright 2008, Blender Foundation / www.bigbuckbunny.org
- **Sintel** - (c) copyright Blender Foundation | www.sintel.org
- **Jellyfish** - Sample video for testing purposes

## Download Scripts

- `download-samples.sh` - Initial download of 10 videos
- `download-more.sh` - Additional 10 videos to reach 20 total

## Verification

All videos verified as valid MP4/H.264 files using ffprobe:
```bash
ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height,duration -of default=noprint_wrappers=1 <file.mp4>
```

Example output:
```
codec_name=h264
width=1280
height=720
duration=10.000000
```

---

**Created:** October 28, 2025
**Purpose:** ClipForge performance profiling and testing
