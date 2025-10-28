# ClipForge Keyboard Shortcuts

**Speed up your editing workflow with keyboard shortcuts**

Version: 0.1.0 (MVP)
Last Updated: October 28, 2025

---

## Current Status

🟡 **Keyboard shortcuts are partially implemented.**

**Currently Working:**
- Timeline zoom (mouse wheel)
- Timeline scroll (Shift + mouse wheel)

**Planned for v1.0:**
- Playback controls (Space, J/K/L)
- Frame stepping (Arrow keys)
- Clip operations (Delete, Cmd+C/V)
- Project management (Cmd+S/O)
- Undo/redo (Cmd+Z)

---

## Quick Reference

### Currently Implemented

| Action | Shortcut | Notes |
|--------|----------|-------|
| **Zoom in timeline** | Mouse Wheel Up | Centers on cursor position |
| **Zoom out timeline** | Mouse Wheel Down | Shows more clips at once |
| **Scroll timeline** | Shift + Mouse Wheel | Horizontal scroll when zoomed |

---

## Planned Shortcuts (v1.0)

### Playback Control

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Play / Pause** | Space | Toggle video playback |
| **Play backward** | J | Plays in reverse |
| **Pause** | K | Stops playback |
| **Play forward** | L | Standard playback |
| **Frame step backward** | Left Arrow | Move back one frame |
| **Frame step forward** | Right Arrow | Move forward one frame |
| **Jump to start** | Home | Playhead to timeline start |
| **Jump to end** | End | Playhead to timeline end |

**Pro tip:** J/K/L keys mimic professional editors (Premiere Pro, Final Cut Pro)

---

### Timeline Editing

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Delete clip** | Delete or Backspace | Remove selected clip |
| **Split clip** | Cmd/Ctrl + K | Split at playhead position |
| **Select all clips** | Cmd/Ctrl + A | Select all clips on timeline |
| **Deselect all** | Cmd/Ctrl + Shift + A | Clear selection |
| **Duplicate clip** | Cmd/Ctrl + D | Copy and paste selected clip |
| **Undo** | Cmd/Ctrl + Z | Undo last action |
| **Redo** | Cmd/Ctrl + Shift + Z | Redo last undone action |

---

### Clip Operations

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Copy** | Cmd/Ctrl + C | Copy selected clip |
| **Cut** | Cmd/Ctrl + X | Cut selected clip |
| **Paste** | Cmd/Ctrl + V | Paste clip at playhead |
| **Trim in point** | I | Set trim start to playhead |
| **Trim out point** | O | Set trim end to playhead |
| **Clear in/out** | Cmd/Ctrl + Shift + X | Remove trim points |

---

### Timeline Navigation

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Zoom to fit** | Shift + Z | Show entire timeline |
| **Zoom in** | + or = | Zoom in on timeline |
| **Zoom out** | - | Zoom out on timeline |
| **Scroll left** | Shift + Mouse Wheel ↑ | Scroll timeline left |
| **Scroll right** | Shift + Mouse Wheel ↓ | Scroll timeline right |
| **Next clip** | Down Arrow | Select next clip |
| **Previous clip** | Up Arrow | Select previous clip |

---

### Project Management

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Save project** | Cmd/Ctrl + S | Save current project |
| **Save as** | Cmd/Ctrl + Shift + S | Save with new name |
| **Open project** | Cmd/Ctrl + O | Load existing project |
| **New project** | Cmd/Ctrl + N | Create new timeline |
| **Import media** | Cmd/Ctrl + I | Open import dialog |
| **Export** | Cmd/Ctrl + E | Open export dialog |

---

### Application

| Action | Shortcut | Description |
|--------|----------|-------------|
| **Preferences** | Cmd/Ctrl + , | Open settings (planned) |
| **Quit** | Cmd/Ctrl + Q | Exit ClipForge |
| **Fullscreen** | Cmd/Ctrl + F | Toggle fullscreen mode |
| **Developer Tools** | Cmd/Ctrl + Shift + I | Open DevTools console |

---

## Platform-Specific Keys

### macOS
- **Cmd** = Command key (⌘)
- **Option** = Alt/Option key (⌥)
- **Control** = Control key (^)

### Windows/Linux
- **Ctrl** = Control key
- **Alt** = Alt key
- **Win** = Windows key (Windows only)

---

## Customizing Shortcuts (Future Feature)

**Planned for v1.1:**
- User-configurable shortcuts
- Import/export shortcut presets
- Conflict detection
- Reset to defaults

**Location:** Preferences > Keyboard Shortcuts

---

## Tips for Faster Editing

### Use Both Hands

**Left Hand:**
- Space (play/pause)
- J/K/L (playback control)
- Arrow keys (frame stepping)
- Cmd/Ctrl (modifier key)

**Right Hand:**
- Mouse (dragging clips, trimming)
- I/O (set in/out points)
- Delete (remove clips)

### Learn the J/K/L Workflow

1. **J** - Play backward
   - Press multiple times for faster reverse (JJ = 2x, JJJ = 3x)

2. **K** - Pause
   - Hold K + J for slow reverse
   - Hold K + L for slow forward

3. **L** - Play forward
   - Press multiple times for faster playback (LL = 2x, LLL = 3x)

**This is the industry-standard playback control used in professional editing software.**

### Common Workflows

**Quick trim workflow:**
1. Scrub to desired start point
2. Press **I** to set in point
3. Scrub to desired end point
4. Press **O** to set out point
5. Press **Delete** to remove trimmed section

**Copy-paste workflow:**
1. Select clip (click)
2. **Cmd+C** to copy
3. Move playhead to new position
4. **Cmd+V** to paste
5. Drag to fine-tune position

**Frame-perfect editing:**
1. Press **K** to pause at rough position
2. Use **Arrow keys** to step frame-by-frame
3. Press **I** or **O** to mark point
4. Press **Space** to resume playback

---

## Shortcuts by Category

### Essential (Learn These First)

Must-know shortcuts for basic editing:

✅ **Space** - Play/Pause
✅ **Arrow keys** - Frame stepping
✅ **Delete** - Remove clip
✅ **Cmd+S** - Save project
✅ **Cmd+Z** - Undo

### Intermediate (Speed Up Workflow)

For regular users:

✅ **J/K/L** - Playback control
✅ **I/O** - Set in/out points
✅ **Cmd+C/V** - Copy/paste
✅ **Shift+Z** - Zoom to fit

### Advanced (Power Users)

For professional editors:

✅ **Cmd+K** - Split clip
✅ **Cmd+D** - Duplicate
✅ **Cmd+Shift+X** - Clear in/out
✅ **Cmd+A** - Select all

---

## Implementation Status

### ✅ Implemented (v0.1.0 MVP)

- Mouse wheel zoom
- Shift + mouse wheel scroll

### 🟡 Partially Implemented

- DevTools shortcut (Cmd+Shift+I) - works via Tauri

### ⏳ Planned (v1.0)

All shortcuts listed above in "Planned Shortcuts" section

**Timeline for implementation:**
- Week 6-7: Core shortcuts (playback, editing)
- Week 7-8: Advanced shortcuts (project management)
- Post v1.0: Customizable shortcuts (v1.1)

---

## Known Issues

### macOS-Specific

**Issue:** Some shortcuts conflict with system shortcuts
- Cmd+Space (Spotlight) - conflicts with play/pause
- Cmd+Shift+A (App Store) - conflicts with deselect all

**Solution:** Will use alternative shortcuts where conflicts exist

### Windows-Specific

**Issue:** Alt key triggers menu bar
**Workaround:** Use Ctrl instead of Alt for shortcuts

### Linux-Specific

**Issue:** Window manager may capture some shortcuts
**Solution:** Configure window manager to ignore ClipForge shortcuts

---

## Printable Cheat Sheet

```
┌─────────────────────────────────────────────────────────┐
│             CLIPFORGE KEYBOARD SHORTCUTS                │
├─────────────────────────────────────────────────────────┤
│ PLAYBACK                                                │
│   Space          Play / Pause                           │
│   J              Play backward                          │
│   K              Pause                                  │
│   L              Play forward                           │
│   ← →           Frame step                             │
├─────────────────────────────────────────────────────────┤
│ EDITING                                                 │
│   Delete         Remove clip                            │
│   Cmd+K          Split clip at playhead                 │
│   Cmd+Z          Undo                                   │
│   Cmd+Shift+Z    Redo                                   │
│   Cmd+C/X/V      Copy / Cut / Paste                     │
├─────────────────────────────────────────────────────────┤
│ TIMELINE                                                │
│   Mouse Wheel    Zoom in/out                            │
│   Shift+Wheel    Scroll left/right                      │
│   Shift+Z        Zoom to fit                            │
│   +/-            Zoom in/out (alternative)              │
├─────────────────────────────────────────────────────────┤
│ PROJECT                                                 │
│   Cmd+S          Save project                           │
│   Cmd+O          Open project                           │
│   Cmd+I          Import media                           │
│   Cmd+E          Export                                 │
├─────────────────────────────────────────────────────────┤
│ TRIMMING                                                │
│   I              Set trim in point                      │
│   O              Set trim out point                     │
│   Cmd+Shift+X    Clear in/out points                    │
└─────────────────────────────────────────────────────────┘
```

---

## Feedback & Suggestions

**Have ideas for better shortcuts?**

We want to hear from you! Common workflows that should have shortcuts?

**Submit suggestions:**
- [GitHub Issues](https://github.com/clipforge/clipforge/issues)
- [GitHub Discussions](https://github.com/clipforge/clipforge/discussions)
- Email: support@clipforge.dev

---

## Related Documentation

- [User Guide](user-guide.md) - Complete editing workflow
- [Quickstart Tutorial](quickstart.md) - Get started in 5 minutes
- [Troubleshooting](troubleshooting.md) - Common issues

---

**Last Updated:** October 28, 2025
**Version:** 0.1.0 MVP
**Implementation Status:** Partially complete (mouse shortcuts only)
**Next Update:** After v1.0 keyboard shortcuts implementation
