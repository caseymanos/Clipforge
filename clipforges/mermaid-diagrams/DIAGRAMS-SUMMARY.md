# Architecture Diagrams Summary

## Quick Visual Reference

### 🎯 Which Diagram Should I Use?

| Your Question | Diagram to View |
|---------------|-----------------|
| "How does the whole system fit together?" | [01-high-level-architecture](./01-high-level-architecture.mermaid) |
| "Which modules should I build first?" | [02-module-dependencies](./02-module-dependencies.mermaid) |
| "How does video import work?" | [03-video-import-flow](./03-video-import-flow.mermaid) |
| "How does export work?" | [04-export-flow](./04-export-flow.mermaid) |
| "Why SQLite instead of HashMap?" | [05-storage-architecture](./05-storage-architecture.mermaid) |
| "How is timeline data structured?" | [06-timeline-structure](./06-timeline-structure.mermaid) |
| "How does screen recording work?" | [07-recording-state-machine](./07-recording-state-machine.mermaid) |

---

## Diagram Gallery

### 1️⃣ High-Level Architecture
```
Frontend (Svelte) 
    ↓ 
IPC Layer (Tauri)
    ↓
Backend (Rust)
    ↓
Data Layer (SQLite + File System)
```
**Shows:** Complete system layers and data flow

---

### 2️⃣ Module Dependencies
```
Module 1 (Shell) → Everything else
Module 2 (Files) → Module 3, 5
Module 5 (Timeline) → Module 6, 7, 8
```
**Shows:** Build order and critical dependencies

---

### 3️⃣ Video Import Flow
```
User drops file
  → Calculate hash
  → Check if duplicate
  → Extract metadata (FFprobe)
  → Generate thumbnail
  → Save to SQLite
  → Update UI
```
**Shows:** Step-by-step import process

---

### 4️⃣ Export Flow
```
User clicks export
  → Validate timeline
  → Build FFmpeg command
  → Spawn FFmpeg process
  → Parse progress (stderr)
  → Emit progress events
  → Write output file
  → Show notification
```
**Shows:** Real-time export with progress tracking

---

### 5️⃣ Storage Architecture
```
FileService
  → Check HashMap cache (optional)
  → Query SQLite (persistent)
  → Load from file system
```
**Shows:** Hybrid storage strategy

---

### 6️⃣ Timeline Structure
```
Timeline
  ├── Track 1 (Video)
  │   ├── Clip A (0-10s)
  │   ├── Clip B (10-25s)
  │   └── Clip C (25-40s)
  ├── Track 2 (Audio)
  └── Track 3 (Overlay)
```
**Shows:** Non-destructive editing structure

---

### 7️⃣ Recording State Machine
```
Idle → List Sources → Request Permission 
  → Recording → Stop → Finalize → Import
```
**Shows:** Screen recording lifecycle

---

## Color Coding

### Module Dependencies (Diagram 2)
- 🟢 **Green:** Phase 1 (Foundation) - Build first
- 🔵 **Blue:** Phase 2 (Core Editing) - MVP features
- 🟠 **Orange:** Phase 3 (Recording & Export) - Advanced features

### System Components
- 🔵 **Blue:** Frontend (Svelte)
- 🟢 **Green:** Backend (Rust)
- 🟡 **Yellow:** Data Layer (SQLite/Cache)
- ⚫ **Gray:** External (FFmpeg)

---

## Using These Diagrams

### For Project Planning
1. **Start:** Review high-level architecture
2. **Next:** Study module dependencies for build order
3. **Then:** Deep dive into specific flows as needed

### For Implementation
1. **Before coding:** Review relevant diagram
2. **During coding:** Reference data flows
3. **After coding:** Verify implementation matches design

### For Onboarding
Show new team members in this order:
1. High-level architecture (big picture)
2. Module dependencies (what to build when)
3. Import/export flows (main user workflows)

---

## Diagram Statistics

| Diagram | Type | Nodes | Complexity |
|---------|------|-------|------------|
| 01-high-level | Graph | 18 | Medium |
| 02-dependencies | Graph | 8 | Simple |
| 03-import-flow | Sequence | 7 | Medium |
| 04-export-flow | Sequence | 6 | Complex |
| 05-storage | Graph | 9 | Simple |
| 06-timeline | Graph | 14 | Medium |
| 07-recording | State | 9 | Medium |

**Total:** 7 diagrams visualizing all major architecture components

---

## Keeping Diagrams Updated

When making architectural changes:

1. ✅ Update the relevant `.mermaid` file
2. ✅ Test rendering (use https://mermaid.live/)
3. ✅ Update related documentation
4. ✅ Note in CHANGELOG.md

**Diagram ownership:**
- Tech lead reviews all diagram changes
- Module owners update their flow diagrams
- Weekly architecture review validates diagrams

---

## Tools & Resources

**Editing:**
- Online: https://mermaid.live/
- VS Code: Mermaid Preview extension
- IntelliJ: Mermaid plugin

**Learning Mermaid:**
- Official docs: https://mermaid.js.org/
- Cheat sheet: https://jojozhuang.github.io/tutorial/mermaid-cheat-sheet/

**Exporting:**
```bash
# To PNG (requires mermaid-cli)
mmdc -i diagram.mermaid -o diagram.png

# To SVG
mmdc -i diagram.mermaid -o diagram.svg -b transparent
```

---

**Created:** October 27, 2025  
**Last Updated:** October 27, 2025  
**Version:** 1.0  
**Format:** Mermaid (text-based diagrams)
