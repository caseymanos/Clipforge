# ClipForge Troubleshooting Guide

**Common issues and solutions for ClipForge video editor**

Version: 0.1.0 (MVP)
Last Updated: October 28, 2025

---

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [FFmpeg Problems](#ffmpeg-problems)
3. [Import Failures](#import-failures)
4. [Timeline Performance](#timeline-performance)
5. [Export Errors](#export-errors)
6. [Screen Recording](#screen-recording)
7. [Memory Issues](#memory-issues)
8. [Platform-Specific](#platform-specific)

---

## Installation Issues

### App won't launch (macOS)

**Symptoms:**
- Double-clicking ClipForge does nothing
- Error: "ClipForge is damaged and can't be opened"

**Solutions:**

**1. Quarantine Attribute:**
```bash
# Remove quarantine flag
xattr -d com.apple.quarantine /Applications/ClipForge.app
```

**2. Gatekeeper:**
```bash
# Allow app from unidentified developer
sudo spctl --master-disable
# Open app, then re-enable:
sudo spctl --master-enable
```

**3. Permissions:**
```bash
# Fix app permissions
chmod -R 755 /Applications/ClipForge.app
```

---

### App won't launch (Windows)

**Symptoms:**
- Error: "VCRUNTIME140.dll is missing"
- Immediate crash on launch

**Solutions:**

**1. Install Visual C++ Redistributable:**
- Download from [Microsoft](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)
- Install both x64 and x86 versions

**2. Windows Defender:**
- Add ClipForge to exclusions
- Settings > Update & Security > Windows Security > Virus & threat protection
- Manage settings > Add exclusion

---

### App won't launch (Linux)

**Symptoms:**
- Error: "Permission denied"
- AppImage doesn't execute

**Solutions:**

**1. Make executable:**
```bash
chmod +x ClipForge.AppImage
```

**2. Missing dependencies:**
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.0-37 libgtk-3-0

# Fedora
sudo dnf install webkit2gtk3 gtk3
```

**3. Run with terminal to see errors:**
```bash
./ClipForge.AppImage
```

---

## FFmpeg Problems

### FFmpeg not found

**Error Message:**
```
FFmpeg not found in system PATH
```

**Diagnosis:**
```bash
# Check if FFmpeg is installed
ffmpeg -version

# Check PATH
echo $PATH  # macOS/Linux
echo %PATH%  # Windows
```

**Solutions by Platform:**

**macOS:**
```bash
# Install via Homebrew
brew install ffmpeg

# Verify installation
which ffmpeg
# Should output: /opt/homebrew/bin/ffmpeg or /usr/local/bin/ffmpeg
```

**Windows:**
1. Download from [ffmpeg.org](https://ffmpeg.org/download.html)
2. Extract to `C:\Program Files\ffmpeg`
3. Add to PATH:
   - System Properties > Environment Variables
   - Edit "Path" variable
   - Add `C:\Program Files\ffmpeg\bin`
4. Restart ClipForge

**Linux (Ubuntu):**
```bash
sudo apt update
sudo apt install ffmpeg

# Verify
which ffmpeg
```

---

### FFmpeg version too old

**Error Message:**
```
FFmpeg version 3.x detected, version 4.0+ required
```

**Solution:**

**macOS:**
```bash
brew upgrade ffmpeg
```

**Ubuntu:**
```bash
# Add PPA for latest version
sudo add-apt-repository ppa:savoury1/ffmpeg4
sudo apt update
sudo apt install ffmpeg
```

**Windows:**
- Download latest from [ffmpeg.org](https://ffmpeg.org/download.html)
- Replace old installation

---

### FFmpeg crashes during export

**Symptoms:**
- Export starts but fails midway
- Error: "FFmpeg process failed"

**Solutions:**

**1. Check FFmpeg directly:**
```bash
# Test FFmpeg with simple command
ffmpeg -i input.mp4 -c copy output.mp4
```

**2. Check disk space:**
```bash
# macOS/Linux
df -h

# Windows
dir
```

**3. Check file permissions:**
```bash
# Ensure output directory is writable
ls -la /path/to/output/
```

**4. View FFmpeg logs:**
- Check ClipForge console output
- Look for FFmpeg stderr messages

---

## Import Failures

### Video file won't import

**Symptoms:**
- Import dialog closes but file doesn't appear
- Error: "Failed to import media file"

**Diagnosis:**

**1. Check file format:**
```bash
ffprobe video.mp4
# Should show codec information
```

**2. Verify file integrity:**
- Try playing in VLC or other player
- File might be corrupted

**3. Check file permissions:**
```bash
ls -la video.mp4
# Ensure you have read permissions
```

**Solutions:**

**Convert to compatible format:**
```bash
# Convert to MP4 H.264
ffmpeg -i input.mov -c:v libx264 -c:a aac output.mp4
```

**Re-download file:**
- Original download might be incomplete
- Verify file size matches expected

**Check supported formats:**
- Supported: MP4, MOV, WebM, AVI, MKV
- Unsupported: FLV, WMV (may work but not tested)

---

### Duplicate detection prevents import

**Symptoms:**
- Import succeeds but file doesn't appear
- File already in library

**Explanation:**
- ClipForge uses SHA-256 hashing
- Detects if file already imported
- Even with different filename

**Solution:**
- File is already in Media Library
- Search for it by original name
- Or check import date sorting

---

### Thumbnail generation fails

**Symptoms:**
- File imports but shows blank thumbnail
- Console error: "Thumbnail generation failed"

**Solutions:**

**1. Check FFmpeg:**
```bash
# Test thumbnail extraction
ffmpeg -i video.mp4 -ss 00:00:05 -frames:v 1 thumb.jpg
```

**2. Video codec issues:**
- Some rare codecs unsupported
- Convert to H.264 (see above)

**3. Disk space:**
- Thumbnails stored in app data directory
- Ensure sufficient space

---

## Timeline Performance

### Timeline is laggy (< 30 FPS)

**Symptoms:**
- Dragging clips is slow
- Scrolling stutters
- Zoom is unresponsive

**Diagnosis:**

**1. Check FPS:**
- Open DevTools (Right-click > Inspect Element)
- Cmd/Ctrl + Shift + P > "Show FPS meter"
- Observe FPS during interaction

**2. Check clip count:**
- Performance target: 30 FPS with 20 clips
- More clips = lower FPS

**Solutions:**

**Short-term:**
- Close other apps (free RAM)
- Restart ClipForge
- Reduce clip count on timeline
- Disable effects temporarily

**Long-term:**
- Upgrade RAM (16GB recommended)
- Use SSD instead of HDD
- Update graphics drivers
- Wait for performance optimizations (Week 5-6)

---

### Clips don't drag smoothly

**Symptoms:**
- Clips jump instead of smooth drag
- Drop zones not showing

**Solutions:**

**1. Zoom level:**
- Zoom in for fine control
- Zoomed-out view may feel jumpy

**2. Mouse sensitivity:**
- System mouse acceleration can affect drag
- Disable in System Settings (macOS) or Control Panel (Windows)

**3. GPU acceleration:**
- Ensure hardware acceleration enabled in browser
- DevTools > Settings > Preferences > Enable hardware acceleration

---

## Export Errors

### Export fails immediately

**Error Message:**
```
Timeline validation error: Timeline must have at least one enabled video track with clips
```

**Cause:**
- Timeline is empty
- All tracks are muted
- No video clips on timeline

**Solution:**
- Add at least one video clip to timeline
- Ensure track is not muted (check backend)
- Verify clip has valid duration

---

### Export fails mid-process

**Error Messages:**
- "FFmpeg exited with status: 1"
- "Output file was not created"

**Diagnosis:**

**1. Check FFmpeg command:**
- View console logs for exact FFmpeg command
- Test command manually

**2. Check output path:**
```bash
# Ensure directory exists and is writable
mkdir -p /path/to/output/
touch /path/to/output/test.txt
```

**Solutions:**

**1. Disk space:**
```bash
# Ensure 2x video size available
# Example: 1GB video needs 2GB free space
df -h /path/to/output/
```

**2. File permissions:**
```bash
# macOS/Linux
chmod 755 /path/to/output/
```

**3. Output path length:**
- Windows: Path must be <260 characters
- Use shorter filename/path

**4. Special characters:**
- Avoid: `<>:"|?*` in filename
- Use alphanumeric and `-_` only

---

### Export is too slow (< 1.0x real-time)

**Symptoms:**
- 60-second video takes >60 seconds to export
- Progress crawls

**Expected Performance:**
- Simple timeline (3 clips, no effects): ≥1.0x
- Complex timeline (20 clips, effects): ~0.5x

**Solutions:**

**1. Close other apps:**
- Free CPU resources for FFmpeg
- Especially browsers, IDEs

**2. Check CPU usage:**
- Activity Monitor (macOS) / Task Manager (Windows)
- FFmpeg should use 100-200% CPU

**3. Simplify timeline:**
- Reduce clip count
- Remove effects
- Lower output resolution

**4. Hardware:**
- Export on faster machine
- SSDs significantly faster than HDDs

---

## Screen Recording

### Permission denied (macOS)

**Error Message:**
```
Recording permission denied
```

**Solution:**

**1. Grant permission:**
1. System Settings > Privacy & Security > Screen Recording
2. Toggle ClipForge ON
3. Restart ClipForge

**2. If ClipForge not in list:**
```bash
# Reset TCC database (requires restart)
tccutil reset ScreenCapture
```

**3. macOS Ventura+ security:**
- May require Full Disk Access as well
- System Settings > Privacy & Security > Full Disk Access

---

### Recording doesn't start (macOS)

**Symptoms:**
- "Start Recording" button does nothing
- No error message

**Solutions:**

**1. Check permissions (see above)**

**2. AVFoundation framework:**
```bash
# Verify system frameworks
ls /System/Library/Frameworks/AVFoundation.framework
```

**3. Restart macOS:**
- Some permission changes require reboot

---

### Windows/Linux recording not working

**Status:**
- Windows: Stub implementation (coming soon)
- Linux: Stub implementation (coming soon)

**Workaround:**
- Use system screen recorder:
  - macOS: QuickTime Player
  - Windows: Xbox Game Bar (Win + G)
  - Linux: OBS Studio
- Import recording into ClipForge

---

## Memory Issues

### Memory usage grows over time

**Symptoms:**
- ClipForge starts at 100MB, grows to 500MB+
- System becomes slow
- Warnings about low memory

**Diagnosis:**

**1. Check memory:**
- Activity Monitor (macOS)
- Task Manager (Windows)
- `htop` (Linux)

**2. Timeline complexity:**
- More clips = more memory
- Preview cache stores rendered frames

**Solutions:**

**Short-term:**
- Restart ClipForge
- Clear preview cache: invoke `clear_preview_cache`
- Close/reload project

**Long-term:**
- Report memory leak if memory never decreases
- Wait for optimization (Week 5-6 sprint)
- Upgrade RAM

---

### Out of memory crash

**Symptoms:**
- ClipForge suddenly closes
- Error: "Out of memory"

**Causes:**
- Timeline too complex
- Preview cache too large (100 frames default)
- Memory leak

**Solutions:**

**1. Increase available memory:**
- Close other apps
- Restart computer
- Upgrade RAM (16GB recommended)

**2. Reduce timeline complexity:**
- Split into multiple projects
- Export segments separately
- Merge in post

**3. Tune preview cache:** *(Developer option)*
```rust
// In preview_cache.rs
const MAX_CACHE_SIZE: usize = 50;  // Reduce from 100
```

---

## Platform-Specific

### macOS Specific

**Gatekeeper blocks app:**
```bash
xattr -d com.apple.quarantine /Applications/ClipForge.app
```

**Notarization warning:**
- ClipForge is not yet notarized
- Will be notarized for v1.0 release

**Rosetta 2 (Apple Silicon):**
- ClipForge compiles for both Intel and ARM
- Should run natively on M1/M2/M3 Macs

---

### Windows Specific

**Antivirus false positive:**
- Add ClipForge to exclusions
- Tauri apps sometimes flagged

**DPI scaling issues:**
- Right-click ClipForge.exe > Properties
- Compatibility > Change high DPI settings
- Override high DPI scaling behavior

---

### Linux Specific

**Wayland vs X11:**
- ClipForge works on both
- Some features may require X11

**Missing system libraries:**
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.0-37 libgtk-3-0

# Fedora
sudo dnf install webkit2gtk3 gtk3

# Arch
sudo pacman -S webkit2gtk gtk3
```

---

## Getting More Help

### Before Reporting Issues

**Gather information:**
1. ClipForge version (`get_app_version` command)
2. Operating system and version
3. FFmpeg version (`ffmpeg -version`)
4. Steps to reproduce
5. Error messages (check console)
6. Screenshots

**Check console logs:**
- Open DevTools (Right-click > Inspect Element)
- Console tab shows errors
- Copy relevant error messages

---

### Reporting Bugs

**GitHub Issues:**
1. Go to [github.com/clipforge/clipforge/issues](https://github.com/clipforge/clipforge/issues)
2. Click "New Issue"
3. Use bug report template
4. Include gathered information above
5. Attach screenshots if helpful

**Include:**
- Clear description of problem
- Expected vs actual behavior
- Steps to reproduce
- System information
- Console errors

---

### Community Support

**Resources:**
- [GitHub Discussions](https://github.com/clipforge/clipforge/discussions)
- [User Guide](user-guide.md)
- [Quickstart Tutorial](quickstart.md)
- [API Reference](api-reference.md)

**Developer docs:**
- [Technical Architecture](../clipforges/02-technical-architecture.md)
- [CLAUDE.md](../CLAUDE.md)

---

## Common Error Messages

### "FFmpeg not found in system PATH"
→ See [FFmpeg Problems](#ffmpeg-problems)

### "Timeline must have at least one enabled video track with clips"
→ Add clips to timeline, ensure track not muted

### "Failed to import media file"
→ See [Import Failures](#import-failures)

### "Output file was not created"
→ See [Export Errors](#export-errors)

### "Recording permission denied"
→ See [Screen Recording](#screen-recording)

---

**Last Updated:** October 28, 2025
**Version:** 0.1.0 MVP
**Submit improvements:** [GitHub Issues](https://github.com/clipforge/clipforge/issues)
