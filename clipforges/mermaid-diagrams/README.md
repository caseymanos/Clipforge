# ClipForge Architecture Diagrams

This directory contains Mermaid diagrams visualizing ClipForge's architecture, data flows, and module relationships.

## 📊 Available Diagrams

### 1. [High-Level Architecture](./01-high-level-architecture.mermaid)
**Overview of the entire system**
- Frontend layer (Svelte components)
- IPC layer (Tauri commands/events)
- Backend layer (Rust services)
- Data layer (SQLite, File System, Cache)

**Use this when:** Understanding overall system structure

---

### 2. [Module Dependencies](./02-module-dependencies.mermaid)
**Module dependency graph with phases**
- Shows which modules depend on others
- Color-coded by implementation phase
- Critical path visualization

**Use this when:** Planning module implementation order

**Legend:**
- 🟢 **Green:** Phase 1 (Weeks 1-2) - Foundation
- 🔵 **Blue:** Phase 2 (Weeks 3-4) - Core Editing  
- 🟠 **Orange:** Phase 3 (Weeks 5-6) - Recording & Export

---

### 3. [Video Import Flow](./03-video-import-flow.mermaid)
**Sequence diagram for importing videos**
- User action → Frontend → Backend → Database
- Shows deduplication logic
- Thumbnail generation process

**Use this when:** Implementing Module 2 (File System)

---

### 4. [Export Flow](./04-export-flow.mermaid)
**Sequence diagram for exporting videos**
- Timeline validation
- FFmpeg command building
- Progress tracking with events
- File system output

**Use this when:** Implementing Module 6 (Export)

---

### 5. [Storage Architecture](./05-storage-architecture.mermaid)
**SQLite + optional cache pattern**
- FileService structure
- Cache-through strategy
- Database indexing

**Use this when:** Understanding storage decisions (HashMap vs SQLite)

**Key Points:**
- Dashed box = Optional cache (Phase 7-8)
- Solid boxes = Required components
- Shows cache-through pattern

---

### 6. [Timeline Structure](./06-timeline-structure.mermaid)
**Timeline data model**
- Timeline → Tracks → Clips hierarchy
- Clip properties (trim, position, effects)
- Non-destructive editing concept

**Use this when:** Implementing Module 5 (Timeline Engine) or Module 7 (Timeline UI)

---

### 7. [Recording State Machine](./07-recording-state-machine.mermaid)
**Screen recording state transitions**
- Permission flow
- Platform-specific implementations
- Error handling paths

**Use this when:** Implementing Module 4 (Screen Recording)

---

## 🎨 Viewing Diagrams

### In GitHub/GitLab
These platforms render Mermaid automatically. Just view the `.mermaid` files.

### In VS Code
Install the **Mermaid Preview** extension:
```bash
code --install-extension bierner.markdown-mermaid
```

### Online Viewer
Copy diagram code to: https://mermaid.live/

### In Documentation Site
If using MkDocs, Docusaurus, or similar:
```markdown
```mermaid
[paste diagram code]
```
```

---

## 🔄 Updating Diagrams

When architecture changes:
1. Update relevant `.mermaid` file
2. Verify renders correctly
3. Update corresponding module documentation
4. Note changes in CHANGELOG.md

---

## 📚 Related Documentation

- [Technical Architecture](../architecture/02-technical-architecture.md) - Written description
- [Data Structures](../architecture/data-structures.md) - Type definitions
- [Storage Decision](../architecture/storage-decision.md) - SQLite vs HashMap

---

## 🎓 Mermaid Syntax Reference

**Graph Types:**
- `graph TB` - Top to bottom
- `graph LR` - Left to right
- `sequenceDiagram` - Sequence interactions
- `stateDiagram-v2` - State machines

**Common Shapes:**
- `[Rectangle]` - Process/Component
- `[(Database)]` - Database
- `{Diamond}` - Decision
- `((Circle))` - Start/End

**Arrows:**
- `-->` - Normal arrow
- `-.->` - Dotted arrow
- `==>` - Thick arrow

**Styling:**
```mermaid
style NodeName fill:#667eea,color:#fff
```

---

**Total Diagrams:** 7  
**Format:** Mermaid (text-based)  
**Last Updated:** October 27, 2025
