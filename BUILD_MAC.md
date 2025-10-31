# Building ClipForge for macOS

Simple guide to build ClipForge as a macOS application (DMG installer).

## Prerequisites

Ensure you have:
- ✅ macOS 11.0 or later
- ✅ Xcode Command Line Tools (`xcode-select --install`)
- ✅ Node.js 18+ (`node --version`)
- ✅ Rust (`rustc --version`)
- ✅ FFmpeg (`ffmpeg -version`)

Run the dependency checker:
```bash
node scripts/check-deps.js
```

## Build for macOS

### Quick Build

```bash
npm run build:mac
```

This creates a **universal binary** that works on both Intel and Apple Silicon Macs.

### Output Location

After building (takes 3-5 minutes), find your installer at:

```
src-tauri/target/release/bundle/dmg/ClipForge_universal.dmg
```

**File size:** ~10-15 MB (without FFmpeg bundled)

## Testing the Build

1. **Mount the DMG:**
   ```bash
   open src-tauri/target/release/bundle/dmg/ClipForge_universal.dmg
   ```

2. **Drag to Applications** (or test directly from DMG)

3. **Launch ClipForge:**
   ```bash
   open /Applications/ClipForge.app
   ```

4. **If you see security warning:**
   - Right-click ClipForge.app → Open
   - Or: System Settings → Privacy & Security → "Open Anyway"

## What's Included

The DMG contains:
- ✅ Universal binary (Intel + Apple Silicon)
- ✅ All frontend assets
- ✅ SQLite database
- ✅ Icon and metadata
- ❌ FFmpeg (users install separately)

## Distribution

### For Personal Use

Just share the DMG file. Users will need to:
1. Install FFmpeg: `brew install ffmpeg`
2. Mount the DMG
3. Drag ClipForge to Applications
4. Handle the security prompt

### For Public Release

See [CODE_SIGNING.md](CODE_SIGNING.md) to add:
- Code signing certificate
- Notarization
- Removes security warnings

## Troubleshooting

### Build fails: "No signing identity"

This is expected if you haven't set up code signing. The build will still succeed and produce an **unsigned DMG**.

To fix the warning, either:
- Ignore it (unsigned builds work fine for personal use)
- Add signing later (see [CODE_SIGNING.md](CODE_SIGNING.md))

### Build fails: "Command not found"

Install missing tools:
```bash
# Check what's missing
node scripts/check-deps.js

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
brew install node

# Install FFmpeg
brew install ffmpeg
```

### Build is slow

First build takes 3-5 minutes to compile Rust code. Subsequent builds are faster (~30 seconds).

### "Disk full" error

The build needs ~2 GB free space:
- Frontend build: ~100 MB
- Rust compilation: ~1.5 GB (in `target/` directory)
- Final DMG: ~10-15 MB

Clean previous builds:
```bash
rm -rf src-tauri/target/release
npm run build  # Rebuild
```

## Advanced Options

### Build for specific architecture

**Intel only:**
```bash
npm run build && tauri build --target x86_64-apple-darwin
```

**Apple Silicon only:**
```bash
npm run build && tauri build --target aarch64-apple-darwin
```

**Universal (both):**
```bash
npm run build:mac
```

### Debug build

For faster builds during development (larger size, no optimizations):
```bash
npm run build && tauri build --debug
```

## File Sizes

| Build Type | Size |
|------------|------|
| Universal binary | ~10-15 MB |
| Intel only | ~6-8 MB |
| Apple Silicon only | ~6-8 MB |
| Debug build | ~50-100 MB |

## Next Steps

After building:

1. **Test the app** - Import videos, edit, export
2. **Share the DMG** - Send to others or upload to GitHub
3. **Add code signing** - See [CODE_SIGNING.md](CODE_SIGNING.md) to remove security warnings

## Quick Reference

```bash
# Check dependencies
node scripts/check-deps.js

# Build for macOS
npm run build:mac

# Find the DMG
ls -lh src-tauri/target/release/bundle/dmg/

# Test the build
open src-tauri/target/release/bundle/dmg/ClipForge_universal.dmg
```

---

**Questions?** See [RELEASE.md](RELEASE.md) for more detailed build documentation.
