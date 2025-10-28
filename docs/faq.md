# ClipForge FAQ (Frequently Asked Questions)

**Common questions about ClipForge video editor**

Version: 0.1.0 (MVP)
Last Updated: October 28, 2025

---

## General Questions

### What is ClipForge?

ClipForge is a free, open-source desktop video editor built with Rust and Svelte. It provides fast, non-destructive video editing with a modern interface, suitable for content creators, educators, and professionals.

**Key features:**
- Visual timeline editor with drag-and-drop
- Screen recording (macOS)
- Multi-format support (MP4, MOV, WebM, AVI, MKV)
- Fast export with FFmpeg
- Project save/load
- Cross-platform (macOS, Windows, Linux)

---

### Is ClipForge free?

**Yes, completely free and open-source.**

- No subscription fees
- No watermarks
- No feature limitations
- MIT licensed (permissive)
- Source code available on GitHub

---

### How does ClipForge compare to other editors?

| Feature | ClipForge | Adobe Premiere | DaVinci Resolve | iMovie |
|---------|-----------|----------------|-----------------|--------|
| **Price** | Free | $22.99/month | Free/Paid | Free (macOS) |
| **Open Source** | ✅ Yes | ❌ No | ❌ No | ❌ No |
| **Timeline Editing** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| **Effects** | 🟡 Basic | ✅ Advanced | ✅ Advanced | ✅ Basic |
| **Export Presets** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| **Cross-Platform** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ macOS only |
| **Learning Curve** | Easy | Hard | Medium | Easy |
| **Performance** | Fast (Rust) | Good | Good | Good |

**ClipForge is best for:**
- Simple to moderate editing tasks
- Users who want free, open-source software
- Quick exports without complex workflows
- Learning video editing basics

---

## Installation & Setup

### What platforms does ClipForge support?

**Supported:**
- ✅ **macOS** 11.0+ (Big Sur and later)
- ✅ **Windows** 10+ (with planned installer)
- ✅ **Linux** Ubuntu 20.04+, Fedora 34+, Arch Linux

**Tested on:**
- macOS 14 Sonoma (primary development platform)
- Windows 11
- Ubuntu 22.04 LTS

---

### Do I need FFmpeg?

**Yes, FFmpeg is required** for video processing.

**Installation:**

**macOS:**
```bash
brew install ffmpeg
```

**Windows:**
- FFmpeg bundling planned for v1.0
- Manual installation: [ffmpeg.org/download.html](https://ffmpeg.org/download.html)

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# Fedora
sudo dnf install ffmpeg

# Arch
sudo pacman -S ffmpeg
```

**Why FFmpeg?**
- Industry-standard video processing
- Supports all formats
- Fast encoding/decoding
- Open source and free

---

### How much disk space do I need?

**Application:**
- ClipForge app: ~15MB (without FFmpeg)
- FFmpeg: ~50-100MB

**Working space:**
- Imported videos: No copies (references original files)
- Thumbnails: ~50KB per video
- Preview cache: ~10-50MB
- Projects: <1MB per project
- Exports: Same size as output video

**Recommended:**
- 500MB for app and thumbnails
- 10-50GB for video projects and exports

---

### What are the system requirements?

**Minimum:**
- CPU: Dual-core (Intel i3 or equivalent)
- RAM: 8GB
- Storage: 500MB + project space
- OS: macOS 11+, Windows 10+, Ubuntu 20.04+

**Recommended:**
- CPU: Quad-core (Intel i5/AMD Ryzen 5 or better)
- RAM: 16GB
- Storage: SSD with 50GB+ free
- GPU: Dedicated graphics card
- Display: 1920x1080 or higher

**For 4K editing:**
- CPU: 6+ cores
- RAM: 32GB
- GPU: Dedicated with 4GB+ VRAM
- Storage: Fast NVMe SSD

---

## File Formats & Compatibility

### What video formats are supported?

**Fully supported:**
- MP4 (H.264/H.265)
- MOV (QuickTime)
- WebM (VP8/VP9)
- AVI
- MKV (Matroska)

**Experimental:**
- FLV (may work)
- WMV (may work)
- MPEG (may work)

**Not supported:**
- Proprietary formats (e.g., Sony XAVC, Canon CR3)
- DRM-protected videos
- Damaged/corrupted files

---

### Can I edit 4K video?

**Yes, with caveats:**

**Current status (v0.1.0):**
- 4K import: ✅ Supported
- 4K timeline: ✅ Supported
- 4K export: ✅ Supported
- 4K preview: 🟡 May be slow

**Performance:**
- Expect slower preview with 4K clips
- Export speed: ~0.3-0.5x real-time for 4K
- Requires powerful CPU

**Optimization tips:**
- Use 1080p timeline, export to 4K
- Generate proxy files (planned feature)
- Close other apps during editing

**Best for 4K:**
- Short videos (<5 minutes)
- Powerful hardware (see system requirements)
- Simple edits (few effects)

---

### Can I import photos/images?

**Not yet in v0.1.0.**

**Planned for v1.1:**
- JPEG/PNG import
- Static duration (e.g., 5 seconds per image)
- Ken Burns effect (zoom/pan)
- Slideshows

**Workaround:**
- Create video from images using FFmpeg:
```bash
ffmpeg -loop 1 -i image.jpg -t 5 -pix_fmt yuv420p output.mp4
```
- Import the generated video

---

### What audio formats are supported?

**Currently:**
- Audio embedded in video files (MP4, MOV, etc.)

**Standalone audio (planned v1.1):**
- MP3
- WAV
- AAC
- OGG

**Audio editing features (planned):**
- Volume adjustment
- Fade in/out
- Audio track muting
- Separate audio extraction

---

## Editing Features

### Is editing non-destructive?

**Yes, completely non-destructive.**

**What this means:**
- Original files are NEVER modified
- Edits stored in project file (.cfp)
- Can undo any edit by reloading project
- Safe to experiment without losing original

**How it works:**
- ClipForge references original file paths
- Trim points stored as metadata
- Export generates new file with edits applied

---

### Can I undo/redo edits?

**Partial support in v0.1.0:**
- ✅ Project save/load = full undo (reload last save)
- ❌ Real-time undo/redo not yet implemented

**Planned for v1.0:**
- Cmd+Z / Ctrl+Z: Undo
- Cmd+Shift+Z / Ctrl+Shift+Z: Redo
- Unlimited undo history
- Undo/redo for all operations

**Workaround:**
- Save project frequently
- Use "Save As" before major changes
- Reload project to revert

---

### How do I split a clip?

**Current method (v0.1.0):**
```typescript
// Via backend command
await invoke('split_clip_at_time', {
  clip_id: 'your-clip-id',
  split_time: 5.0  // Split at 5 seconds
});
```

**Planned UI (v1.0):**
1. Position playhead where you want to split
2. Select clip
3. Press Cmd+K / Ctrl+K (or click "Split" button)
4. Clip splits into two

---

### Can I add transitions?

**Not yet in v0.1.0.**

**Planned for v1.1:**
- Fade in/out
- Cross-dissolve
- Wipes (left, right, up, down)
- Custom duration

**Workaround for fade:**
- Apply FadeIn/FadeOut effects (when effect UI added)

---

### Can I add text/titles?

**Not yet in v0.1.0.**

**Planned for v1.2:**
- Text overlay track
- Font selection
- Position/size/rotation
- Animation (fade, slide)

**Workaround:**
- Create title card in image editor
- Convert to video (see photo import workaround)
- Import as clip

---

### Can I color grade?

**Basic effects available (v0.1.0):**
- Brightness
- Contrast
- Saturation

**Advanced color grading (planned v2.0):**
- Curves
- Color wheels
- LUTs (lookup tables)
- Scopes (waveform, vectorscope)

---

## Performance & Export

### How fast is the export?

**Target performance:**
- Simple timeline (3 clips, no effects): ≥1.0x real-time
- Medium complexity: ~0.8x real-time
- Complex (20 clips, effects): ~0.5x real-time

**Example:**
- 60-second video, simple timeline: exports in <60 seconds
- 60-second video, complex: exports in ~120 seconds

**Depends on:**
- CPU speed (more cores = faster)
- Clip count
- Effects applied
- Output resolution
- Disk speed (SSD vs HDD)

---

### Why is my timeline laggy?

**Common causes:**
1. Too many clips (>20)
2. High resolution clips (4K)
3. Insufficient RAM (<8GB)
4. Slow disk (HDD vs SSD)
5. Other apps consuming CPU

**Solutions:**
- Close other applications
- Reduce clip count
- Restart ClipForge
- Upgrade to 16GB RAM
- Use SSD for video files
- Lower preview quality (planned feature)

See [Troubleshooting: Timeline Performance](troubleshooting.md#timeline-performance)

---

### Can I export to different formats?

**Current (v0.1.0):**
- MP4 only (H.264 codec)

**Presets available:**
- YouTube 1080p
- Instagram Post (square 1080x1080)
- Twitter Video (720p)

**Planned (v1.0):**
- MOV export
- WebM export
- Custom resolution
- Custom codec selection
- Animated GIF export

---

### What's the maximum video length?

**No hard limit.**

**Practical limits:**
- Memory: Longer timelines use more RAM
- Export time: Linear with duration
- File size: Depends on bitrate and resolution

**Tested:**
- 2 hour timeline: Works
- 20+ clips: Works
- Multiple exports: Works

**Recommendations:**
- For very long videos (>2 hours): Split into segments
- Export segments separately
- Concatenate final exports

---

## Screen Recording

### Does screen recording work on my platform?

**macOS:** ✅ Fully supported (AVFoundation)
**Windows:** 🟡 Stub (coming in v1.1)
**Linux:** 🟡 Stub (coming in v1.1)

---

### How do I grant screen recording permission (macOS)?

**First time:**
1. ClipForge requests permission
2. System Settings opens automatically
3. Navigate to: Privacy & Security > Screen Recording
4. Toggle ClipForge ON
5. Restart ClipForge

**Manual:**
1. System Settings > Privacy & Security
2. Click "Screen Recording" in sidebar
3. Enable ClipForge
4. Restart ClipForge

**Still not working?**
- Full Disk Access may also be needed (Ventura+)
- Try restarting macOS

---

### Can I record specific windows?

**Yes on macOS:**
- `list_recording_sources` shows available windows
- Select specific window or full screen
- Choose at recording start

**Windows/Linux:**
- Coming in v1.1 with proper implementation

---

## Projects & Files

### What's the .cfp file format?

**.cfp = ClipForge Project**

**Format:** JSON (human-readable)

**Contains:**
- Timeline structure
- Track configuration
- Clip positions and trim points
- Effects (if applied)
- Project metadata

**Does NOT contain:**
- Original video files (referenced by path)
- Generated thumbnails
- Preview cache

**Compatibility:**
- Cross-platform (open on any OS)
- Forward compatible (v1.0 opens v0.1.0 projects)
- Text editor readable (for debugging)

---

### Can I share projects with others?

**Yes, but with caveats:**

**To share:**
1. Export .cfp project file
2. Also share original video files
3. Recipient must have same file paths OR edit paths in .cfp

**Better method (planned v1.1):**
- "Package project" feature
- Bundles .cfp + all media files
- Portable project folder

**For collaboration:**
- Currently: Manual file sharing
- Future (v2.0): Cloud sync, collaborative editing

---

### Where are my projects saved?

**Project files (.cfp):**
- Wherever you choose during "Save Project"
- Default: Documents folder

**Application data:**
- **macOS:** `~/Library/Application Support/com.clipforge.app/`
- **Windows:** `%APPDATA%\com.clipforge.app\`
- **Linux:** `~/.local/share/com.clipforge.app/`

**Contains:**
- SQLite database (media library)
- Generated thumbnails
- App preferences
- Preview cache

---

## Troubleshooting

### The app won't launch. What do I do?

See [Troubleshooting: Installation Issues](troubleshooting.md#installation-issues)

**Quick fixes:**
- **macOS:** Remove quarantine: `xattr -d com.apple.quarantine /Applications/ClipForge.app`
- **Windows:** Install Visual C++ Redistributable
- **Linux:** Make executable: `chmod +x ClipForge.AppImage`

---

### FFmpeg not found error?

**Install FFmpeg:**
- **macOS:** `brew install ffmpeg`
- **Windows:** Download from [ffmpeg.org](https://ffmpeg.org/download.html)
- **Linux:** `sudo apt install ffmpeg` (Ubuntu)

**Verify:**
```bash
ffmpeg -version
```

See [Troubleshooting: FFmpeg Problems](troubleshooting.md#ffmpeg-problems)

---

### Video won't import?

**Check:**
1. File format supported? (MP4, MOV, WebM, AVI, MKV)
2. File plays in other apps? (might be corrupted)
3. Sufficient disk space?
4. File permissions readable?

**Convert to MP4:**
```bash
ffmpeg -i input.mov -c:v libx264 -c:a aac output.mp4
```

See [Troubleshooting: Import Failures](troubleshooting.md#import-failures)

---

### Export fails?

**Common causes:**
1. Insufficient disk space
2. Output path not writable
3. FFmpeg not installed
4. Missing media files

**Solutions:**
- Check free disk space (need 2x video size)
- Verify output directory exists
- Test FFmpeg: `ffmpeg -version`
- Verify all clips' source files exist

See [Troubleshooting: Export Errors](troubleshooting.md#export-errors)

---

## Future Features

### What's coming in v1.0?

**Planned features:**
- ✅ Keyboard shortcuts (Space, J/K/L, etc.)
- ✅ Undo/redo system
- ✅ Export UI dialog
- ✅ Performance optimizations
- ✅ Cross-platform testing
- ✅ FFmpeg bundling (Windows)
- ✅ Complete documentation

**Release date:** End of Week 8 (target)

---

### What's coming in v1.1?

**Planned features:**
- Video transitions (fade, dissolve, wipes)
- Audio tracks support
- Standalone audio import (MP3, WAV)
- Image import (JPEG, PNG)
- Proxy files for better performance
- Package project (bundle media files)
- More video effects
- Batch export

---

### What's coming in v2.0?

**Planned features:**
- GPU-accelerated rendering
- 4K performance improvements
- Color grading tools (curves, color wheels)
- Text/title overlays
- Plugin system
- Cloud project sync
- Collaborative editing
- Multi-cam editing
- Motion tracking

---

## Getting Help

### Where can I get support?

**Documentation:**
- [User Guide](user-guide.md) - Complete manual
- [Quickstart Tutorial](quickstart.md) - 5-minute guide
- [Troubleshooting](troubleshooting.md) - Common issues
- [API Reference](api-reference.md) - For developers

**Community:**
- [GitHub Issues](https://github.com/clipforge/clipforge/issues) - Bug reports
- [GitHub Discussions](https://github.com/clipforge/clipforge/discussions) - Questions
- Email: support@clipforge.dev (if applicable)

---

### How can I contribute?

**Ways to contribute:**
- 🐛 Report bugs
- 💡 Suggest features
- 📝 Improve documentation
- 💻 Submit code (PRs welcome!)
- 🎨 Design icons/UI
- 🌐 Translate to other languages (planned)

**Get started:**
1. Read [CONTRIBUTING.md](../CONTRIBUTING.md) (coming soon)
2. Check open issues
3. Join discussions
4. Fork repository
5. Submit pull request

**No coding experience?**
- Test new features
- Report bugs with detailed steps
- Improve documentation
- Help other users in discussions

---

### Where can I request features?

**Feature requests:**
- [GitHub Issues](https://github.com/clipforge/clipforge/issues) with "enhancement" label
- [GitHub Discussions](https://github.com/clipforge/clipforge/discussions) for ideas

**Before requesting:**
- Search existing issues/discussions
- Explain use case and why it's useful
- Provide examples or mockups if possible

**Popular requests:**
- Multi-cam editing
- Green screen (chroma key)
- Motion tracking
- Audio effects
- Mobile app

---

## About the Project

### Who makes ClipForge?

ClipForge is an open-source project.

**Technology stack:**
- Rust (backend)
- Tauri v2 (desktop framework)
- Svelte 4 (frontend)
- FFmpeg (video processing)
- SQLite (database)

**Inspired by:**
- Adobe Premiere Pro
- DaVinci Resolve
- Final Cut Pro
- Open-source ethos

---

### Is ClipForge stable?

**Current status (v0.1.0 MVP):**
- ✅ Core features working
- ✅ Most operations stable
- 🟡 Some features incomplete (effects UI, keyboard shortcuts)
- 🟡 Limited testing (macOS primary, Windows/Linux partial)

**Production ready:** Not yet (target v1.0)

**Safe to use for:**
- Learning video editing
- Simple projects
- Testing and feedback
- Non-critical work

**Not recommended for:**
- Professional client work (until v1.0)
- Mission-critical projects
- Long-form documentaries

---

### How can I support the project?

**Non-financial:**
- ⭐ Star on GitHub
- 📢 Share with friends
- 🐛 Report bugs
- 💡 Suggest features
- 📝 Improve documentation
- 💻 Contribute code

**Financial (if donations enabled):**
- Sponsor on GitHub
- One-time donation
- Contribute to hosting costs

**Most valuable:** Your feedback and bug reports help make ClipForge better!

---

## Didn't find your answer?

**Ask your question:**
- [GitHub Discussions](https://github.com/clipforge/clipforge/discussions)
- Create new issue: [GitHub Issues](https://github.com/clipforge/clipforge/issues)
- Email: support@clipforge.dev

We'll add popular questions to this FAQ!

---

**Last Updated:** October 28, 2025
**Version:** 0.1.0 MVP
**Suggest an FAQ entry:** [Open an issue](https://github.com/clipforge/clipforge/issues)
