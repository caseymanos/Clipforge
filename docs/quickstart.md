# ClipForge Quickstart Tutorial

**Get started with ClipForge in 5 minutes**

This tutorial walks you through creating your first video project, from importing clips to exporting the final video.

**What you'll learn:**
- Importing video files
- Arranging clips on the timeline
- Trimming clips
- Exporting to MP4

**Time required:** 5-10 minutes

---

## Prerequisites

**Before you start:**
- ✅ ClipForge installed
- ✅ FFmpeg installed (see [User Guide](user-guide.md#installing-ffmpeg))
- ✅ 2-3 sample video clips (MP4, MOV, or WebM)

**Don't have sample videos?**
- Record your screen with QuickTime (macOS) or Xbox Game Bar (Windows)
- Download free stock videos from [Pexels](https://www.pexels.com/videos/) or [Pixabay](https://pixabay.com/videos/)

---

## Step 1: Launch ClipForge

### macOS
1. Open **Applications** folder
2. Double-click **ClipForge**
3. App opens showing three sections:
   - Media Library (top)
   - Video Preview (middle)
   - Timeline Editor (bottom)

### Windows
1. Click **Start menu**
2. Search for "ClipForge"
3. Click to launch

### Linux
```bash
./ClipForge.AppImage
```

**First launch:** App may take a few seconds to initialize.

---

## Step 2: Import Your First Video

### Method 1: Import Button (Current)

1. Look at the **Media Library** section (top of window)
2. Click the **"Import Media"** button
3. File picker dialog opens
4. Navigate to your video files
5. Select 2-3 videos (hold Cmd/Ctrl to select multiple)
6. Click **"Open"**

**What happens next:**
- Files appear in Media Library with thumbnail previews
- Metadata extracted (duration, resolution, codec)
- Thumbnails generated (may take a few seconds)

### Method 2: Drag-and-Drop *(Planned)*

1. Open Finder (macOS) or File Explorer (Windows)
2. Select video files
3. Drag files into Media Library section
4. Release to drop

---

## Step 3: Add Clips to Timeline

Now let's add your imported videos to the timeline.

### Adding Your First Clip

1. Look at the Media Library (top section)
2. Find the first video you want to edit
3. **Double-click** the video thumbnail

**Result:**
- Clip appears on the timeline (bottom section)
- Positioned at the start (time 0:00)
- Placed on first video track

### Adding More Clips

1. **Double-click** your second video in Media Library
2. Clip appears after the first clip
3. Repeat for third video

**What you should see:**
- Timeline now has 3 clips in sequence
- Each clip shows:
  - Thumbnail preview
  - Duration
  - Name (filename)

---

## Step 4: Preview Your Timeline

Let's watch what we've created so far.

### Using the Video Preview

1. Look at the **Video Preview** section (middle of window)
2. The **playhead** (red circle on timeline) shows current position
3. Click the **Play button** (▶) in preview controls
4. Video plays showing all clips in sequence
5. Click **Pause** (⏸) to stop

### Scrubbing the Timeline

1. **Drag the playhead** (red circle) left/right on timeline
2. Preview updates in real-time
3. Shows frame at that exact position

**Try this:**
- Drag playhead to start of second clip
- Preview shows first frame of that clip
- Click Play to watch from there

---

## Step 5: Trim a Clip

Let's remove unwanted parts from the first clip.

### How to Trim

1. **Click the first clip** on timeline to select it
2. Clip gets a white border (selected state)
3. **White handles** appear on left and right edges
4. **Drag the LEFT handle** to the right
   - This trims the start of the clip
   - Removes first few seconds
5. **Drag the RIGHT handle** to the left
   - This trims the end of the clip
   - Removes last few seconds

**Visual feedback:**
- Clip width changes as you drag
- Duration updates in real-time
- Preview shows trimmed portion

**Example:**
- Original clip: 10 seconds
- Trim start: 2 seconds
- Trim end: 1 second
- Result: 7 seconds (from 2s to 9s of original)

**Important:** Original file is NOT modified. Trimming is non-destructive!

---

## Step 6: Rearrange Clips

Let's change the order of clips.

### Dragging Clips

1. **Click and hold** the second clip
2. **Drag** to the left, before the first clip
3. Clip moves to new position
4. **Release mouse** to drop

**What happens:**
- Clips automatically reposition
- No gaps between clips (if dropped adjacent)
- Timeline duration updates

**Try this:**
- Move clips into different order
- Watch preview to see result
- Drag back if you don't like it

---

## Step 7: Zoom and Scroll

For precise editing, you'll want to zoom in.

### Zooming the Timeline

**Zoom In:**
- **Mouse wheel UP** (scroll up)
- Timeline expands horizontally
- Shows more detail

**Zoom Out:**
- **Mouse wheel DOWN** (scroll down)
- Timeline contracts
- Shows more clips at once

**Tip:** Zoom centers on your mouse cursor position.

### Scrolling Horizontally

**When zoomed in:**
- Hold **Shift key**
- **Mouse wheel** to scroll left/right

**Or use scrollbar:**
- Drag scrollbar at bottom of timeline *(if visible)*

---

## Step 8: Adjust the Playhead

The playhead determines what you see in preview and where new clips are added.

### Moving the Playhead

**Method 1: Drag**
- Click and drag the red circle
- Moves along timeline
- Preview updates in real-time

**Method 2: Click Timeline**
- Click anywhere on the time ruler (top of timeline)
- Playhead jumps to that position

**Method 3: Frame Stepping**
- Click **⏮** button to go back one frame
- Click **⏭** button to go forward one frame
- Useful for finding exact cut points

---

## Step 9: Export Your Video

Time to create the final video file!

### Export Process (Using Backend Command)

**Note:** UI export dialog is planned. For now, use this method for testing:

1. Open DevTools (Right-click > Inspect Element)
2. Go to Console tab
3. Run export command:

```javascript
// Get export presets
const presets = await invoke('get_export_presets');
console.log('Available presets:', presets);

// Choose preset (YouTube 1080p)
const settings = presets.find(p => p[0] === 'YouTube 1080p')[1];

// Get current timeline
const timeline = $timelineStore;

// Get media files
const mediaFiles = $mediaLibraryStore.reduce((acc, file) => {
  acc[file.id] = file;
  return acc;
}, {});

// Export
const result = await invoke('export_timeline', {
  timeline,
  settings,
  output_path: '/Users/yourname/Desktop/my-video.mp4',  // Change this!
  media_files_map: mediaFiles
});

console.log('Export complete:', result);
```

**Change output_path to your desired location!**

### What Happens During Export

1. **Validation:**
   - Checks timeline has clips
   - Verifies all media files exist

2. **Processing:**
   - FFmpeg renders each clip
   - Applies any effects *(if added)*
   - Concatenates clips together

3. **Progress:**
   - Check console for progress updates
   - Shows percentage complete
   - Encoding speed (e.g., "1.2x real-time")

4. **Completion:**
   - File saved to output path
   - Ready to share!

**Expected time:**
- Simple timeline (3 clips, 60 seconds): ~30-60 seconds
- Depends on computer speed and video resolution

---

## Step 10: Watch Your Final Video

1. Open Finder (macOS) or File Explorer (Windows)
2. Navigate to output path (e.g., Desktop)
3. Find `my-video.mp4`
4. Double-click to play

**Congratulations!** You've created your first video with ClipForge!

---

## What You've Learned

✅ **Importing videos** into Media Library
✅ **Adding clips** to timeline (double-click)
✅ **Previewing** your work (play/pause)
✅ **Trimming clips** (drag edge handles)
✅ **Rearranging clips** (drag to reorder)
✅ **Zooming and scrolling** (mouse wheel + Shift)
✅ **Moving the playhead** (drag or click)
✅ **Exporting** to MP4

---

## Next Steps

### Learn More Features

📖 **Read the full [User Guide](user-guide.md) to learn:**
- Splitting clips
- Using multiple tracks
- Adding effects *(when supported)*
- Screen recording
- Saving/loading projects
- Keyboard shortcuts

### Try Advanced Techniques

**Multi-track editing:**
- Add clips to different tracks
- Create picture-in-picture effects
- Layer video over video

**Effects:** *(Coming soon)*
- Brightness/contrast adjustments
- Blur and sharpen
- Fade transitions

**Save your project:**
- Click "Save Project" button
- Choose filename (e.g., `my-project.cfp`)
- Reload anytime

---

## Common Issues

### Video won't import
→ See [Troubleshooting: Import Failures](troubleshooting.md#import-failures)

### Timeline is laggy
→ See [Troubleshooting: Timeline Performance](troubleshooting.md#timeline-performance)

### Export fails
→ See [Troubleshooting: Export Errors](troubleshooting.md#export-errors)

### FFmpeg not found
→ See [Troubleshooting: FFmpeg Problems](troubleshooting.md#ffmpeg-problems)

---

## Tips for Better Results

**1. Organize your media:**
- Use descriptive filenames
- Keep source files in one folder
- Don't move files after importing

**2. Edit efficiently:**
- Zoom in for precise trims
- Use playhead to find cut points
- Preview before exporting

**3. Export settings:**
- YouTube: Use "YouTube 1080p" preset
- Instagram: Use "Instagram Post" preset (square)
- Twitter: Use "Twitter Video" preset (720p)

**4. Performance:**
- Close other apps while editing
- Export to SSD for faster speed
- Reduce timeline complexity if lagging

---

## Practice Project Ideas

**1. Simple montage:**
- Import 5 vacation clips
- Trim each to 5-10 seconds
- Arrange in chronological order
- Export as memories video

**2. Tutorial video:**
- Record screen capture (QuickTime on macOS)
- Import recording
- Trim mistakes/pauses
- Export for YouTube

**3. Social media highlight:**
- Import long gameplay/vlog footage
- Find best moments
- Trim to 60 seconds total
- Export for Instagram

---

## Getting Help

**Stuck? Resources available:**

📚 **Documentation:**
- [User Guide](user-guide.md) - Complete reference
- [Troubleshooting](troubleshooting.md) - Common issues
- [API Reference](api-reference.md) - For developers

💬 **Community:**
- [GitHub Discussions](https://github.com/clipforge/clipforge/discussions)
- [GitHub Issues](https://github.com/clipforge/clipforge/issues) - Bug reports
- Email: support@clipforge.dev *(if applicable)*

🛠 **Developer Docs:**
- [Technical Architecture](../clipforges/02-technical-architecture.md)
- [CLAUDE.md](../CLAUDE.md)
- [Module Specifications](../clipforges/)

---

## Feedback Welcome!

Help improve this tutorial:
- Found something confusing? [Open an issue](https://github.com/clipforge/clipforge/issues)
- Have a suggestion? [Start a discussion](https://github.com/clipforge/clipforge/discussions)
- Want to contribute? See [CONTRIBUTING.md](../CONTRIBUTING.md)

---

**Congratulations on completing the quickstart!**

You now have the basics to start creating videos with ClipForge. Explore the full [User Guide](user-guide.md) to unlock more features and become a power user.

Happy editing! 🎬

---

**Last Updated:** October 28, 2025
**Version:** 0.1.0 MVP
**Next Update:** After UI export dialog is added
