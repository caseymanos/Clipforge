# ClipForge Documentation

**Desktop Video Editor built with Rust/Tauri + Svelte**

## Documentation Structure

This documentation is organized into several key areas:

### 📋 Planning Documents
- **[Product Overview](./planning/01-product-overview.md)** - Vision, goals, and success metrics
- **[Technical Architecture](./architecture/02-technical-architecture.md)** - High-level system design
- **[Implementation Timeline](./planning/03-implementation-timeline.md)** - Phases and milestones
- **[Testing Strategy](./planning/04-testing-strategy.md)** - Testing approach and checklist
- **[Deployment Guide](./planning/05-deployment-guide.md)** - Build and distribution

### 🔧 Module Specifications

#### Core Backend Modules (Rust)
1. **[Module 1: Application Shell](./modules/module-01-application-shell.md)** - Tauri app lifecycle and window management
2. **[Module 2: File System & Media](./modules/module-02-file-system-media.md)** - Import and media library
3. **[Module 3: FFmpeg Integration](./modules/module-03-ffmpeg-integration.md)** - Video processing layer
4. **[Module 4: Screen Recording](./modules/module-04-screen-recording.md)** - Platform-specific capture
5. **[Module 5: Timeline Engine](./modules/module-05-timeline-engine.md)** - Edit Decision List and state
6. **[Module 6: Export & Rendering](./modules/module-06-export-rendering.md)** - Final video output

#### Frontend Modules (Svelte)
7. **[Module 7: Timeline UI](./modules/module-07-timeline-ui.md)** - Visual timeline editor
8. **[Module 8: Video Preview](./modules/module-08-video-preview.md)** - Playback and preview

### 📚 Reference
- **[Data Structures](./architecture/data-structures.md)** - Core types and interfaces
- **[API Reference](./architecture/api-reference.md)** - Tauri commands
- **[Dependencies](./architecture/dependencies.md)** - Required packages

## Quick Start for Implementers

1. **Pick a module** from the list above
2. **Read the module specification** thoroughly
3. **Review dependencies** - Check what other modules you need
4. **Check the architecture docs** for shared types
5. **Implement according to spec**
6. **Write tests** as you go
7. **Submit PR** with clear description

## Project Timeline

- **Phase 1 (Weeks 1-2):** Foundation - Modules 1, 2, 3, 5
- **Phase 2 (Weeks 3-4):** Core Editing - Modules 7, 8
- **Phase 3 (Weeks 5-6):** Recording & Export - Modules 4, 6
- **Phase 4 (Weeks 7-8):** Polish & Cross-Platform

## Team Communication

- **Daily standups:** Share progress and blockers
- **Weekly demos:** Show working features
- **Code reviews:** All PRs require review
- **Pair programming:** For complex problems

## Success Metrics

### MVP (Week 4)
- ✅ Import video files
- ✅ Arrange clips on timeline
- ✅ Trim clips
- ✅ Export to MP4
- ✅ Save/load projects

### Production Ready (Week 8)
- ✅ Screen recording
- ✅ Cross-platform builds
- ✅ No critical bugs
- ✅ Performance targets met

## Resources

- **Tauri Docs:** https://tauri.app/
- **Rust Book:** https://doc.rust-lang.org/book/
- **FFmpeg Docs:** https://ffmpeg.org/documentation.html
- **Project Discord:** [Link to team channel]

## Contributing

See each module's "Acceptance Criteria" section for definition of done.

All code must:
- Pass `cargo test`
- Pass `cargo clippy`
- Include unit tests
- Update documentation

---

**Last Updated:** October 27, 2025  
**Version:** 1.0  
**Project Duration:** 8 weeks
