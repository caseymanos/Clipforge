# ClipForge Performance Testing Guide

**Date:** October 28, 2025
**Baseline established:** MacBook Pro M4 Pro (24GB RAM)
**Test videos location:** `/Users/caseymanos/GauntletAI/clipforge/test-videos/`

---

## Prerequisites

✅ ClipForge running in development mode (`npm run tauri:dev`)
✅ 20 test videos downloaded (see README.md in this directory)
✅ Chrome DevTools knowledge (for FPS measurement)
✅ Activity Monitor open (for memory tracking)

---

## Test 1: Baseline Memory Usage

**Status:** ✅ MEASURED

**Result:**
- ClipForge process (empty, no videos): **79.7 MB**
- Node/Vite processes: ~58 MB + 33 MB = 91 MB
- **Total baseline:** ~171 MB (development mode with hot reload)

**Note:** Production build will use less memory (no dev tools)

---

## Test 2: Timeline FPS Performance

**Objective:** Measure timeline rendering performance with 20+ clips

### Steps:

1. **Open Chrome DevTools:**
   - Right-click in ClipForge window
   - Select "Inspect Element"
   - DevTools panel opens

2. **Enable FPS Meter:**
   - Press `Cmd+Shift+P` (Command Palette)
   - Type "Show frames"
   - Select "Show frames per second (FPS) meter"
   - Green FPS counter appears in top-left corner

3. **Import All 20 Test Videos:**
   - Click "Import Media" button in ClipForge
   - Navigate to: `/Users/caseymanos/GauntletAI/clipforge/test-videos/`
   - Select all 20 MP4 files (Cmd+A)
   - Click "Open"
   - Wait for thumbnails to generate

4. **Add All Clips to Timeline:**
   - Double-click each video in Media Library
   - All 20 clips should appear on timeline
   - Timeline should now have 200 seconds of content (20 clips × 10s each)

5. **Measure FPS During Operations:**

   **a) Idle Timeline:**
   - Don't touch anything
   - Record FPS from meter
   - Expected: 60 FPS

   **b) Horizontal Scrolling:**
   - Hold Shift + scroll mouse wheel
   - Observe FPS while scrolling
   - Expected: ≥30 FPS
   - Record minimum FPS seen

   **c) Dragging a Clip:**
   - Click and drag a clip left/right
   - Observe FPS during drag
   - Expected: ≥30 FPS
   - Record minimum FPS

   **d) Zooming In/Out:**
   - Scroll mouse wheel (no Shift)
   - Zoom in and out rapidly
   - Expected: ≥30 FPS
   - Record minimum FPS

   **e) Scrubbing Playhead:**
   - Drag the red playhead circle
   - Move it back and forth quickly
   - Expected: ≥30 FPS
   - Record minimum FPS

6. **Take Screenshot:**
   - Capture FPS meter showing worst-case FPS
   - Save to: `test-videos/screenshots/timeline-fps.png`

7. **Record Results:**
   - Update `docs/performance-results.md` section 1
   - Fill in the measurements table

### Expected Results:

| Scenario | Min FPS | Target | Status |
|----------|---------|--------|--------|
| Idle timeline | 60 | 60 | ✅ |
| Scrolling | ≥30 | ≥30 | ? |
| Dragging | ≥30 | ≥30 | ? |
| Zooming | ≥30 | ≥30 | ? |
| Scrubbing | ≥30 | ≥30 | ? |

---

## Test 3: Memory Usage During Editing

**Objective:** Verify memory stays <300MB during editing

### Steps:

1. **Open Activity Monitor:**
   ```bash
   open -a "Activity Monitor"
   ```

2. **Find ClipForge Process:**
   - Search for "clipforge" in search box
   - Sort by Memory column
   - Watch the Memory column

3. **Record Memory at Each Step:**

   **a) Baseline (already measured):**
   - ClipForge just launched: **79.7 MB** ✅

   **b) After Importing 20 Videos:**
   - Import all videos (if not already done)
   - Wait for thumbnails to finish
   - Record memory usage
   - Expected: <150MB

   **c) After Adding All to Timeline:**
   - Double-click all 20 clips to add to timeline
   - Wait for timeline to render
   - Record memory usage
   - Expected: <250MB

   **d) During Playback/Scrubbing:**
   - Scrub through the timeline
   - Play/pause a few times
   - Record peak memory
   - Expected: <300MB

   **e) After Reload (Leak Test):**
   - Close and reopen ClipForge
   - Import and add clips again
   - Check if memory returns to baseline or keeps growing
   - Expected: Should return to ~150-250MB range

4. **Take Screenshots:**
   - Activity Monitor showing memory usage
   - Save to: `test-videos/screenshots/memory-usage.png`

5. **Record Results:**
   - Update `docs/performance-results.md` section 2

### Expected Results:

| Workflow Step | Memory (MB) | Target | Status |
|---------------|-------------|--------|--------|
| Baseline | 80 | <50 | ⚠️ (dev mode) |
| After importing 20 videos | ? | <150 | ? |
| Timeline with 20 clips | ? | <250 | ? |
| Peak during preview | ? | <300 | ? |
| After reload | ? | ~same | ? |

**Note:** Dev mode uses more memory than production build

---

## Test 4: Export Speed Benchmark

**Objective:** Measure export speed for 1080p video

### Test Cases:

#### Test 4a: Simple Export (3 clips, 30 seconds)

1. **Create Simple Timeline:**
   - Clear timeline (remove all clips)
   - Add 3 clips: Big_Buck_Bunny_1080_10s_1MB.mp4 (3 times)
   - Total duration: 30 seconds

2. **Export to MP4:**
   - Open DevTools Console (Cmd+Option+I)
   - Run this export command:

```javascript
// Simple export test
const startTime = Date.now();

const result = await invoke('export_timeline', {
  timeline: $timelineStore,
  settings: {
    video_codec: "libx264",
    audio_codec: "aac",
    video_bitrate: 8000,
    audio_bitrate: 192,
    framerate: 30.0,
    resolution: { width: 1920, height: 1080 },
    format: "mp4"
  },
  output_path: '/Users/caseymanos/Desktop/test-simple-export.mp4',
  media_files_map: $mediaLibraryStore.reduce((acc, f) => { acc[f.id] = f; return acc; }, {})
});

const duration = (Date.now() - startTime) / 1000;
console.log(`Export completed in ${duration.toFixed(2)} seconds`);
console.log(`Speed: ${(30 / duration).toFixed(2)}x real-time`);
```

3. **Record Results:**
   - Export duration: ___ seconds
   - Export speed: ___ x real-time
   - Target: ≥1.0x
   - Status: ___

#### Test 4b: Medium Export (10 clips, 100 seconds)

1. **Create Medium Timeline:**
   - Add 10 different clips
   - Total duration: 100 seconds

2. **Export and measure** (same command, adjust duration in calculation)

3. **Record Results:**
   - Export duration: ___ seconds
   - Export speed: ___ x real-time
   - Target: ≥0.8x
   - Status: ___

#### Test 4c: Complex Export (20 clips, 200 seconds)

1. **Create Complex Timeline:**
   - Add all 20 clips
   - Total duration: 200 seconds

2. **Export and measure**

3. **Record Results:**
   - Export duration: ___ seconds
   - Export speed: ___ x real-time
   - Target: ≥0.5x
   - Status: ___

---

##Test 5: Bundle Size (Already Complete)

**Status:** ✅ MEASURED

**Result:**
- Binary size: 14.9 MB
- Target: <15 MB
- **Status: PASS** ✅

See commit 3c502bc for details.

---

## Recording Results

After completing each test, update `/Users/caseymanos/GauntletAI/clipforge/docs/performance-results.md` with actual measurements.

### How to Update:

1. Open `docs/performance-results.md`
2. Find the relevant test section (1, 2, 3, or 4)
3. Replace "TBD" with actual measurements
4. Update status from 🟡 to ✅ or ❌
5. Add any notes about bottlenecks or issues

---

## Troubleshooting

### FPS meter not showing
- Make sure you typed "Show frames" in Command Palette
- Try refreshing DevTools (Cmd+R with DevTools focused)

### Export command fails
- Check that timeline has clips
- Verify output path is writable
- Check console for error messages

### Memory keeps growing
- This indicates a memory leak
- Document the growth rate
- Check if reloading returns to baseline

### Timeline is laggy
- This is important data - record the FPS
- Note which operations are slowest
- Check CPU usage in Activity Monitor

---

## Quick Command Reference

**Kill all dev servers:**
```bash
pkill -f "tauri dev" && pkill -f "vite"
```

**Start fresh dev server:**
```bash
cd /Users/caseymanos/GauntletAI/clipforge && npm run tauri:dev
```

**Check memory usage:**
```bash
ps aux | grep "[c]lipforge" | awk '{print $4 "% (" $6/1024 " MB)"}'
```

**List all test videos:**
```bash
ls -lh /Users/caseymanos/GauntletAI/clipforge/test-videos/*.mp4
```

---

## Screenshots Directory

Create this directory to store test screenshots:

```bash
mkdir -p /Users/caseymanos/GauntletAI/clipforge/test-videos/screenshots
```

**Screenshots to capture:**
1. `timeline-fps.png` - FPS meter during worst-case scenario
2. `memory-usage.png` - Activity Monitor showing peak memory
3. `devtools-performance.png` - Chrome DevTools Performance tab recording (optional)

---

## Next Steps

After completing all tests:

1. ✅ Update `docs/performance-results.md` with all measurements
2. ✅ Take screenshots for documentation
3. ✅ Commit results: `git add docs/performance-results.md test-videos/screenshots/`
4. ✅ Update `progress.md` to mark performance profiling complete
5. ✅ If any tests fail (FPS <30, Memory >300MB), create optimization plan

---

**Happy Testing!** 🎬

Remember: These tests are critical for verifying ClipForge meets its performance targets before production release.
