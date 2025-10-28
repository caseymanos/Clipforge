# ClipForge

**A fast, modern desktop video editor built with Rust and Svelte**

ClipForge is a cross-platform video editing application that combines the performance of native Rust with the flexibility of a modern web UI. Edit videos, record your screen, and export professional content—all from a single, lightweight desktop app.

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/clipforge/clipforge)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0--mvp-orange)](CHANGELOG.md)

---

## Features

✨ **Non-Destructive Editing**
- Your original files are never modified
- All edits stored in project files
- Unlimited undo/redo (planned)

🎬 **Timeline Editor**
- Canvas-based timeline with smooth 60 FPS rendering
- Drag-and-drop clips between tracks
- Trim, split, and arrange clips visually
- Multi-track support (video, audio, overlay)

📹 **Screen Recording**
- Built-in screen capture (macOS supported, Windows/Linux coming soon)
- Auto-import recorded videos to timeline
- Permission management

📁 **Media Library**
- Import multiple video formats (MP4, MOV, WebM, AVI, MKV)
- Automatic thumbnail generation
- Search and filter your media
- Duplicate detection via file hashing

🎨 **Video Effects** *(In Progress)*
- Brightness, contrast, saturation
- Blur and sharpen
- Fade in/out transitions
- More effects coming soon

💾 **Project Management**
- Save and load complete projects (.cfp format)
- JSON-based project files
- Cross-platform compatibility

🚀 **Fast Export**
- Powered by FFmpeg
- Export presets (YouTube 1080p, Instagram, Twitter)
- Real-time progress tracking
- Target: 1x real-time export speed for 1080p

---

## Installation

### macOS

**Requirements:**
- macOS 11.0 (Big Sur) or later
- FFmpeg (install via Homebrew)

**Install FFmpeg:**
```bash
brew install ffmpeg
```

**Install ClipForge:**
1. Download `ClipForge.dmg` from [Releases](https://github.com/clipforge/clipforge/releases)
2. Open the DMG file
3. Drag ClipForge to your Applications folder
4. Launch ClipForge from Applications
5. Grant screen recording permissions when prompted (for screen capture feature)

---

### Windows

**Requirements:**
- Windows 10 or later
- FFmpeg (bundled with installer - coming soon)

**Install ClipForge:**
1. Download `ClipForge-setup.exe` from [Releases](https://github.com/clipforge/clipforge/releases)
2. Run the installer
3. Follow the installation wizard
4. Launch ClipForge from Start Menu

---

### Linux

**Requirements:**
- Ubuntu 20.04+ / Fedora 34+ / Arch Linux
- FFmpeg (install via package manager)

**Install FFmpeg:**

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install ffmpeg
```

**Fedora:**
```bash
sudo dnf install ffmpeg
```

**Arch Linux:**
```bash
sudo pacman -S ffmpeg
```

**Install ClipForge:**
1. Download `ClipForge.AppImage` from [Releases](https://github.com/clipforge/clipforge/releases)
2. Make it executable:
   ```bash
   chmod +x ClipForge.AppImage
   ```
3. Run:
   ```bash
   ./ClipForge.AppImage
   ```

---

## Quick Start

### 1. Import Videos

Click **"Import Media"** in the Media Library section, or drag-and-drop video files directly into the app.

**Supported formats:**
- MP4, MOV, WebM, AVI, MKV

### 2. Add Clips to Timeline

Double-click any video in the Media Library to add it to the timeline. Clips appear on the first available track.

### 3. Edit Your Video

- **Drag clips** to reposition them
- **Click a clip** to select it
- **Drag the edge handles** to trim the clip
- **Use the playhead** to scrub through your video
- **Zoom** with mouse wheel
- **Scroll** with Shift + mouse wheel

### 4. Export

Click **"Export"** (planned in UI), choose a preset (YouTube 1080p, Instagram, Twitter), and export your video to MP4.

**First export?** See our [Quickstart Tutorial](docs/quickstart.md) for a detailed walkthrough.

---

## Documentation

📖 **User Guides:**
- [User Guide](docs/user-guide.md) - Complete guide to using ClipForge
- [Quickstart Tutorial](docs/quickstart.md) - 5-minute getting started guide
- [Troubleshooting](docs/troubleshooting.md) - Common issues and solutions
- [Keyboard Shortcuts](docs/keyboard-shortcuts.md) - Speed up your workflow

🛠 **Developer Docs:**
- [Technical Architecture](clipforges/02-technical-architecture.md) - System design
- [Module Specifications](clipforges/) - Detailed implementation docs
- [API Reference](docs/api-reference.md) - Tauri command reference
- [Contributing Guide](CONTRIBUTING.md) - How to contribute

📊 **Project Status:**
- [Progress Tracker](progress.md) - Current implementation status (80% complete)
- [Audit Report](AUDIT_REPORT.md) - Comprehensive code audit
- [Performance Results](docs/performance-results.md) - Benchmark data

---

## Technology Stack

**Backend:**
- [Rust](https://www.rust-lang.org/) - Fast, safe systems programming
- [Tauri v2](https://tauri.app/) - Lightweight desktop app framework
- [SQLite](https://www.sqlite.org/) - Local database for media library
- [FFmpeg](https://ffmpeg.org/) - Video processing engine
- [Tokio](https://tokio.rs/) - Async runtime

**Frontend:**
- [Svelte 4](https://svelte.dev/) - Reactive UI framework
- [TypeScript](https://www.typescriptlang.org/) - Type-safe JavaScript
- [Konva.js](https://konvajs.org/) - Canvas-based timeline rendering
- [Vite 5](https://vitejs.dev/) - Lightning-fast build tool

---

## System Requirements

### Minimum

- **CPU:** Dual-core processor (Intel Core i3 or equivalent)
- **RAM:** 8 GB
- **Storage:** 500 MB for app + space for projects and exports
- **OS:** macOS 11.0+ / Windows 10+ / Ubuntu 20.04+
- **FFmpeg:** Version 4.0 or later

### Recommended

- **CPU:** Quad-core processor (Intel Core i5 / AMD Ryzen 5 or better)
- **RAM:** 16 GB
- **Storage:** SSD with 50 GB+ free space
- **GPU:** Dedicated graphics card (for better preview performance)
- **OS:** macOS 12.0+ / Windows 11 / Ubuntu 22.04+
- **FFmpeg:** Latest version (6.0+)

---

## Project Status

**Current Version:** v0.1.0 (MVP - 80% complete)

**Completed Modules:**
- ✅ Module 1: Application Shell (100%)
- ✅ Module 2: File System & Media (100%)
- ✅ Module 3: FFmpeg Integration (100%)
- ✅ Module 4: Screen Recording (100% macOS, Windows/Linux stubs)
- ✅ Module 5: Timeline Engine (95%)
- ✅ Module 6: Export & Rendering (100%)
- ✅ Module 7: Timeline UI (95%)
- ✅ Module 8: Video Preview (95%)

**Remaining Work:**
- Performance profiling and optimization
- Keyboard shortcuts
- Cross-platform testing (Windows/Linux)
- FFmpeg bundling strategy
- User documentation (in progress)

See [progress.md](progress.md) for detailed status.

---

## Performance Targets

ClipForge is designed to be fast and responsive:

| Metric | Target | Status |
|--------|--------|--------|
| Timeline rendering | ≥30 FPS with 20+ clips | 🟡 Testing |
| Memory usage | <300MB during editing | 🟡 Testing |
| Export speed | ≥1.0x real-time (1080p) | 🟡 Testing |
| Launch time | <3 seconds | ✅ ~2s |
| Bundle size | <15MB (without FFmpeg) | 🟡 Testing |

See [docs/performance-results.md](docs/performance-results.md) for detailed benchmarks.

---

## Building from Source

### Prerequisites

- **Rust:** Install from [rustup.rs](https://rustup.rs/)
- **Node.js:** v18 or later
- **FFmpeg:** Installed and in PATH

### Build Steps

```bash
# Clone the repository
git clone https://github.com/clipforge/clipforge.git
cd clipforge

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

See [CLAUDE.md](CLAUDE.md) for detailed development instructions.

---

## Troubleshooting

### FFmpeg not found

**Error:** `FFmpeg not found in system PATH`

**Solution:**
- **macOS:** `brew install ffmpeg`
- **Windows:** Download from [ffmpeg.org](https://ffmpeg.org/download.html) and add to PATH
- **Linux:** `sudo apt install ffmpeg` (Ubuntu) or equivalent

### Screen recording permission denied (macOS)

**Error:** Screen recording permission denied

**Solution:**
1. Open System Settings > Privacy & Security > Screen Recording
2. Enable ClipForge
3. Restart ClipForge

### Video won't import

**Error:** Failed to import video file

**Solution:**
- Verify file format is supported (MP4, MOV, WebM, AVI, MKV)
- Check file isn't corrupted (try playing in another app)
- Ensure sufficient disk space

For more issues, see [docs/troubleshooting.md](docs/troubleshooting.md)

---

## Contributing

We welcome contributions! ClipForge is open for community involvement.

**Ways to contribute:**
- 🐛 [Report bugs](https://github.com/clipforge/clipforge/issues)
- 💡 [Suggest features](https://github.com/clipforge/clipforge/issues)
- 📝 Improve documentation
- 💻 Submit pull requests

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Roadmap

### v1.0 (Production Release) - Week 8
- ✅ Core editing features
- ✅ Export functionality
- 🟡 Performance optimization
- 🟡 Complete documentation
- ⏳ Cross-platform testing
- ⏳ FFmpeg bundling

### v1.1 (Polish)
- ⏳ Undo/redo system
- ⏳ Keyboard shortcuts
- ⏳ More video effects
- ⏳ Audio track support
- ⏳ Batch export

### v2.0 (Advanced Features)
- ⏳ GPU-accelerated rendering
- ⏳ 4K support
- ⏳ Color grading tools
- ⏳ Plugin system
- ⏳ Cloud project sync

---

## License

ClipForge is licensed under the [MIT License](LICENSE).

**Third-party software:**
- FFmpeg: [LGPL/GPL](https://www.ffmpeg.org/legal.html)
- Tauri: [MIT/Apache-2.0](https://github.com/tauri-apps/tauri/blob/dev/LICENSE)
- Svelte: [MIT](https://github.com/sveltejs/svelte/blob/master/LICENSE.md)

---

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Desktop app framework
- [FFmpeg](https://ffmpeg.org/) - Video processing
- [Svelte](https://svelte.dev/) - UI framework
- [Konva.js](https://konvajs.org/) - Canvas rendering

Inspired by professional video editors like Adobe Premiere, DaVinci Resolve, and Final Cut Pro.

---

## Support

- 📧 Email: support@clipforge.dev (if applicable)
- 💬 Discussions: [GitHub Discussions](https://github.com/clipforge/clipforge/discussions)
- 🐛 Bug Reports: [GitHub Issues](https://github.com/clipforge/clipforge/issues)
- 📚 Documentation: [docs/](docs/)

---

## Screenshots

*(Screenshots to be added)*

---

**Made with ❤️ by the ClipForge team**

[Website](https://clipforge.dev) | [GitHub](https://github.com/clipforge/clipforge) | [Documentation](docs/) | [Releases](https://github.com/clipforge/clipforge/releases)
