# ClipForge Documentation - Quick Navigation

## 🚀 Getting Started

**New to the project?** Start here:
1. Read [Product Overview](./planning/01-product-overview.md)
2. Review [Technical Architecture](./architecture/02-technical-architecture.md)
3. Check [Implementation Timeline](./planning/03-implementation-timeline.md)

## 📦 For Implementers

**Assigned a module?** Follow this workflow:

1. **Read your module spec** (see list below)
2. **Check dependencies** - What modules must be done first?
3. **Review data structures** - [Data Structures Reference](./architecture/data-structures.md)
4. **Check the API** - [API Reference](./architecture/api-reference.md) (if exists)
5. **Implement & test**
6. **Update README** with your progress

## 📋 Module Assignments

### Backend Modules (Rust)

| Module | File | Phase | Priority |
|--------|------|-------|----------|
| 1. Application Shell | [module-01](./modules/module-01-application-shell.md) | 1 | Critical |
| 2. File System & Media | [module-02](./modules/module-02-file-system-media.md) | 1 | Critical |
| 3. FFmpeg Integration | [module-03](./modules/module-03-ffmpeg-integration.md) | 1 | Critical |
| 4. Screen Recording | [module-04](./modules/module-04-screen-recording.md) | 3 | High |
| 5. Timeline Engine | [module-05](./modules/module-05-timeline-engine.md) | 1 | Critical |
| 6. Export & Rendering | [module-06](./modules/module-06-export-rendering.md) | 3 | High |

### Frontend Modules (Svelte)

| Module | File | Phase | Priority |
|--------|------|-------|----------|
| 7. Timeline UI | [module-07](./modules/module-07-timeline-ui.md) | 2 | Critical |
| 8. Video Preview | [module-08](./modules/module-08-video-preview.md) | 2 | Critical |

## 🏗️ Architecture Documents

- [Technical Architecture](./architecture/02-technical-architecture.md) - System design
- [Data Structures](./architecture/data-structures.md) - Core types
- [Dependencies](./architecture/dependencies.md) - Required packages

## 📅 Planning Documents

- [Product Overview](./planning/01-product-overview.md) - Vision & goals
- [Implementation Timeline](./planning/03-implementation-timeline.md) - 8-week schedule
- [Testing Strategy](./planning/04-testing-strategy.md) - Testing approach

## 🎯 Phase Breakdown

### Phase 1: Foundation (Weeks 1-2)
**Goal:** Core infrastructure
- Module 1: Application Shell
- Module 2: File System & Media
- Module 3: FFmpeg Integration
- Module 5: Timeline Engine

### Phase 2: Core Editing (Weeks 3-4)
**Goal:** Visual editor
- Module 7: Timeline UI
- Module 8: Video Preview
- **MVP CHECKPOINT**

### Phase 3: Recording & Export (Weeks 5-6)
**Goal:** Complete workflow
- Module 4: Screen Recording
- Module 6: Export & Rendering

### Phase 4: Polish (Weeks 7-8)
**Goal:** Production ready
- Cross-platform testing
- Performance optimization
- Bug fixes
- Documentation

## ✅ Definition of Done

Each module is complete when:
- [ ] All acceptance criteria met
- [ ] Unit tests written and passing
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] Demo-able functionality

## 🔗 Quick Links

- **Main README:** [README.md](./README.md)
- **Product Vision:** [planning/01-product-overview.md](./planning/01-product-overview.md)
- **Technical Arch:** [architecture/02-technical-architecture.md](./architecture/02-technical-architecture.md)
- **Timeline:** [planning/03-implementation-timeline.md](./planning/03-implementation-timeline.md)

## 💡 Tips for Success

1. **Read before coding** - Each module has detailed implementation notes
2. **Test incrementally** - Don't wait until the end
3. **Ask questions early** - Blockers compound quickly
4. **Communicate progress** - Daily updates keep team aligned
5. **Follow the timeline** - Built-in buffer for contingencies

## 📞 Getting Help

- Stuck on a module? Check the **Acceptance Criteria** section
- Need clarification? Review the **Implementation Details**
- Platform-specific issues? See **Platform-Specific Considerations**
- Architecture questions? Read **Technical Architecture** doc

---

**Project Duration:** 8 weeks  
**Team Size:** 3-5 engineers  
**Target:** Production-ready desktop video editor

Good luck! 🎬
