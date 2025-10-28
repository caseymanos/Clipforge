# Module 7: Timeline UI (Svelte)

**Owner:** TBD  
**Dependencies:** Module 5 (Timeline Engine), Module 1 (Application Shell)  
**Phase:** 2 (Weeks 3-4)  
**Estimated Effort:** 7-8 days

## Overview

Visual timeline editor with drag-drop, scrubbing, Canvas rendering using Konva.js.

## Components

```
src/lib/components/
├── Timeline.svelte          # Main container
├── Track.svelte            # Individual track
├── Clip.svelte             # Draggable clip
├── Playhead.svelte         # Time indicator
├── TimeRuler.svelte        # Time labels
└── Waveform.svelte         # Audio visualization
```

## Key Features

### Drag & Drop
- Drag clips from media library to timeline
- Reposition clips within track
- Move clips between tracks

### Trimming
- Resize handles on clip edges
- Click-drag to adjust in/out points
- Snap to adjacent clips

### Zoom & Scroll
- Mouse wheel to zoom
- Horizontal scroll for long timelines
- Maintain playhead visibility

### Canvas Rendering
Use Konva.js for performance:
```svelte
<script>
import Konva from 'konva';

let stage = new Konva.Stage({
  container: 'timeline',
  width: 1200,
  height: 400
});

let layer = new Konva.Layer();
stage.add(layer);
</script>
```

## State Management

```typescript
// Svelte stores
export const timelineStore = writable<Timeline>({...});
export const playheadTime = writable<number>(0);
export const selectedClipId = writable<string | null>(null);
```

## Acceptance Criteria

- [ ] Display timeline with multiple tracks
- [ ] Drag clips to reposition
- [ ] Resize clips from edges (trim)
- [ ] Zoom in/out with mouse wheel
- [ ] Scroll horizontally
- [ ] Display clip thumbnails
- [ ] 30 FPS with 20+ clips

---

**Status:** Not Started  
**Target Completion:** Week 4, Mid
