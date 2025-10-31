# FFmpeg Bundling Documentation

## Overview

ClipForge now bundles a minimal custom-built FFmpeg with the application, eliminating the need for users to install FFmpeg separately. This provides a self-contained, user-friendly installation experience while keeping the bundle size small.

## Bundle Information

- **FFmpeg Version**: 7.1 (stable)
- **Target Architecture**: Apple Silicon (aarch64) only
- **Build Type**: Static (no external dependencies)
- **Bundle Size Impact**:
  - FFmpeg binary: 5.9 MB
  - FFprobe binary: 5.9 MB
  - **Total DMG size: ~10 MB** (down from 5.1 MB without FFmpeg)

## What's Included

### Codecs

**Video Encoders:**
- libx264 (H.264) - Primary video codec
- mjpeg - JPEG encoding for thumbnails

**Video Decoders:**
- H.264, HEVC (H.265)
- MPEG-4
- MJPEG, PNG
- Raw video

**Audio Encoders:**
- AAC - Primary audio codec
- PCM (16-bit)

**Audio Decoders:**
- AAC, MP3
- PCM (16-bit, 32-bit float)

### Filters

**Video Filters:**
- scale, crop, trim, concat
- fade, eq (brightness/contrast/saturation)
- boxblur, unsharp
- split, setpts, format, null, color, fps, overlay

**Audio Filters:**
- volume, aformat, anull
- atrim, asetpts, asplit
- loudnorm (audio normalization)
- aresample

### Formats

**Input Formats (Demuxers):**
- MP4, MOV, M4V
- Concat demuxer
- Image2 (JPEG, PNG)
- WAV, MP3, AAC

**Output Formats (Muxers):**
- MP4, MOV
- Image2 (JPEG)
- WAV

### macOS-Specific Features

- **AVFoundation** - Screen capture support
- **VideoToolbox** - Hardware acceleration
- **AudioToolbox** - Audio processing

## Building FFmpeg from Source

The minimal FFmpeg build is created using the script:

```bash
./scripts/build-ffmpeg-minimal.sh
```

### Build Requirements

- Xcode Command Line Tools
- Homebrew
- libx264 (`brew install x264`)
- pkg-config (`brew install pkg-config`)

### Build Process

1. Clones FFmpeg 7.1 from official repository
2. Configures with minimal feature set (only what ClipForge needs)
3. Compiles as static binary (no dynamic library dependencies)
4. Outputs to `src-tauri/binaries/`

### Build Time

- Approximately 10-15 minutes on Apple Silicon Mac

## How It Works

### Tauri Integration

ClipForge uses Tauri's `externalBin` feature to bundle FFmpeg:

1. **Configuration** (`tauri.conf.json`):
   ```json
   {
     "bundle": {
       "externalBin": [
         "binaries/ffmpeg",
         "binaries/ffprobe"
       ]
     }
   }
   ```

2. **Runtime Detection** (`src-tauri/src/ffmpeg_utils.rs`):
   - App locates bundled binaries in same directory as executable
   - Tauri automatically adds platform suffix (`-aarch64-apple-darwin`)
   - No fallback to system FFmpeg (bundled-only approach)

3. **Bundle Location**:
   ```
   ClipForge.app/
     Contents/
       MacOS/
         ClipForge          (main executable)
         ffmpeg             (bundled FFmpeg)
         ffprobe            (bundled FFprobe)
   ```

## Size Comparison

| Configuration | FFmpeg Size | Total Bundle | Notes |
|--------------|-------------|--------------|-------|
| **System FFmpeg** (previous) | 0 MB (user installs) | ~5 MB | Required user to install Homebrew + FFmpeg |
| **Full Static FFmpeg** | 60-80 MB | ~85 MB | Includes all codecs/filters |
| **Minimal Custom Build** (current) | 12 MB | **~10 MB** | Only features ClipForge needs |

## Rebuilding FFmpeg

If you need to rebuild FFmpeg (e.g., to add features or update version):

```bash
# 1. Modify the build script if needed
nano scripts/build-ffmpeg-minimal.sh

# 2. Run the build script
./scripts/build-ffmpeg-minimal.sh

# 3. Rebuild ClipForge
npm run tauri build -- --target aarch64-apple-darwin
```

## Features NOT Included

To keep the binary small, these features are **disabled**:

- Network protocols (HTTP, RTMP, etc.)
- Most video codecs (VP8/9, AV1, etc.)
- Most audio codecs (Opus, Vorbis, FLAC, etc.)
- Hardware encoders (only decoders enabled)
- Advanced filters (stabilization, denoise, etc.)
- Subtitle support
- Postprocessing filters

## Development Mode

In development mode (`npm run tauri dev`), the bundled FFmpeg may not be available since binaries are only packaged during release builds. For development, you can:

1. Install FFmpeg via Homebrew: `brew install ffmpeg`
2. Or temporarily copy binaries to the dev build directory

## Troubleshooting

### "FFmpeg not found" error

If you see this error after installing ClipForge:

1. **Check binary exists**:
   ```bash
   ls -la /Applications/ClipForge.app/Contents/MacOS/
   ```
   Should show `ffmpeg` and `ffprobe` files

2. **Check permissions**:
   ```bash
   xattr -cr /Applications/ClipForge.app
   ```

3. **Reinstall** the application from DMG

### Build errors

If the FFmpeg build fails:

1. Ensure Xcode Command Line Tools are installed:
   ```bash
   xcode-select --install
   ```

2. Install required dependencies:
   ```bash
   brew install x264 pkg-config
   ```

3. Check build logs at `/tmp/ffmpeg-clipforge-build`

## Future Improvements

Potential enhancements for future releases:

1. **Universal Binary**: Add Intel (x86_64) support for older Macs
2. **AVFoundation Direct**: Use native macOS APIs for more operations (further reduce FFmpeg dependency)
3. **On-Demand Download**: Download FFmpeg on first run instead of bundling (smallest initial download)
4. **Custom Builds**: Allow users to rebuild with additional codecs if needed

## License

FFmpeg is licensed under **GPL v2+**. The bundled FFmpeg binary is built with:
- `--enable-gpl` flag (required for libx264)
- Static linking only
- No proprietary codecs

ClipForge's use of FFmpeg complies with GPL requirements.
