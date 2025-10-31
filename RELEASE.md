# ClipForge Release Guide

Quick reference for building and releasing ClipForge production packages.

## Quick Start

### Build Locally

```bash
# Check dependencies first
node scripts/check-deps.js

# Build for your platform
npm run build:mac      # macOS universal binary
npm run build:win      # Windows (on Windows only)
npm run build:linux    # Linux AppImage + Deb

# Or run production build (includes checks)
npm run build:prod
```

### Output Locations

After building, find your installers in:

```
src-tauri/target/release/bundle/
├── dmg/          # macOS: ClipForge_universal.dmg
├── nsis/         # Windows: ClipForge_x64_setup.exe
├── msi/          # Windows: ClipForge_x64.msi
├── appimage/     # Linux: ClipForge_x86_64.AppImage
└── deb/          # Linux: clipforge_1.0.0_amd64.deb
```

## GitHub Release Process

### Automated Release (Recommended)

1. **Update Version Number**
   ```bash
   # Update version in package.json
   npm version 1.0.0 --no-git-tag-version

   # Update version in src-tauri/tauri.conf.json
   # Manually change "version": "1.0.0"

   # Update version in src-tauri/Cargo.toml
   # Manually change version = "1.0.0"
   ```

2. **Commit Changes**
   ```bash
   git add .
   git commit -m "Release v1.0.0"
   git push
   ```

3. **Create and Push Tag**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

4. **Monitor GitHub Actions**
   - Go to GitHub Actions tab
   - Watch the "Release Build" workflow
   - Wait for all platforms to complete (~10-20 minutes)

5. **Publish Release**
   - Go to GitHub Releases
   - Find the draft release created by Actions
   - Review the uploaded assets:
     - macOS: `ClipForge_universal.dmg`
     - Windows: `ClipForge_x64_setup.exe`, `ClipForge_x64.msi`
     - Linux: `ClipForge_x86_64.AppImage`, `clipforge_1.0.0_amd64.deb`
     - Checksums: SHA256 files for each asset
   - Edit release notes if needed
   - Click "Publish release"

### Manual Release

If you need to build and upload manually:

```bash
# Build all platforms (requires access to each OS)
npm run build:mac      # On macOS
npm run build:win      # On Windows
npm run build:linux    # On Linux

# Create release on GitHub
gh release create v1.0.0 \
  --title "ClipForge v1.0.0" \
  --notes-file RELEASE_NOTES.md \
  --draft

# Upload assets
gh release upload v1.0.0 \
  src-tauri/target/release/bundle/dmg/*.dmg \
  src-tauri/target/release/bundle/nsis/*.exe \
  src-tauri/target/release/bundle/appimage/*.AppImage \
  src-tauri/target/release/bundle/deb/*.deb

# Publish when ready
gh release edit v1.0.0 --draft=false
```

## Pre-Release Checklist

### Code Quality

- [ ] All tests passing (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Frontend builds without errors (`npm run build`)
- [ ] Dependencies checked (`node scripts/check-deps.js`)

### Documentation

- [ ] README.md updated with latest features
- [ ] CHANGELOG.md includes all changes
- [ ] Version numbers updated in:
  - [ ] package.json
  - [ ] src-tauri/tauri.conf.json
  - [ ] src-tauri/Cargo.toml

### Testing

- [ ] App launches successfully
- [ ] Core features work:
  - [ ] Video import
  - [ ] Timeline editing
  - [ ] Video playback
  - [ ] Screen recording (macOS)
  - [ ] Video export
- [ ] FFmpeg integration works
- [ ] No console errors

### Build Configuration

- [ ] Icons present in `src-tauri/icons/`
- [ ] Entitlements configured (macOS)
- [ ] Bundle metadata correct in tauri.conf.json
- [ ] Build scripts tested (`npm run build:prod`)

## Platform-Specific Notes

### macOS

**Universal Binary**
- Builds for both Intel and Apple Silicon in one package
- Uses target: `universal-apple-darwin`

**Requirements**
- Xcode Command Line Tools
- macOS 11.0+ to build
- Targets macOS 11.0+ runtime

**Security**
- Unsigned builds will show security warnings
- Users must right-click > Open or go to System Settings
- See [CODE_SIGNING.md](CODE_SIGNING.md) to add signing

**File Size**
- DMG: ~10-15 MB (without FFmpeg)
- Universal binary is ~2x larger than single-arch

### Windows

**Installer Options**
- **NSIS** (recommended): Modern installer with uninstaller
- **MSI**: For enterprise deployment with Group Policy

**Requirements**
- Visual Studio Build Tools
- Windows 10+ to build
- Targets Windows 10+ runtime

**Security**
- Unsigned builds trigger SmartScreen warnings
- Users must click "More info" > "Run anyway"
- See [CODE_SIGNING.md](CODE_SIGNING.md) to add signing

**WebView2**
- Installer downloads WebView2 if not present
- Uses `downloadBootstrapper` mode

### Linux

**Package Options**
- **AppImage** (recommended): Universal, no installation needed
- **Deb**: For Debian/Ubuntu systems

**Requirements**
- `libwebkit2gtk-4.1-dev` and other system libraries
- See INSTALLATION.md for full list

**Dependencies**
- Deb package declares FFmpeg as dependency
- AppImage expects FFmpeg in system PATH

## Versioning

ClipForge follows [Semantic Versioning](https://semver.org/):

- **Major** (x.0.0): Breaking changes, major new features
- **Minor** (1.x.0): New features, backwards compatible
- **Patch** (1.0.x): Bug fixes, minor improvements

Examples:
- `1.0.0` - Initial production release
- `1.1.0` - Added new effects or features
- `1.0.1` - Fixed export bug

## Release Schedule

Suggested cadence:

- **Major releases**: Every 6-12 months
- **Minor releases**: Every 1-3 months
- **Patch releases**: As needed for critical bugs

## Troubleshooting Builds

### "FFmpeg not found" warning in CI

This is expected - FFmpeg is not bundled with the app. Users must install it separately. The warning doesn't prevent the build from succeeding.

### macOS build fails: "No signing identity"

If you haven't set up code signing yet, this is expected. The build will succeed but produce unsigned binaries. See [CODE_SIGNING.md](CODE_SIGNING.md) to add signing.

### Windows build fails: Missing VCRUNTIME

Install Visual Studio Build Tools:
```
https://visualstudio.microsoft.com/downloads/
```

### Linux build fails: Missing WebKit

Install required system libraries:
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

### Bundle size too large

Current targets:
- macOS DMG: <15 MB
- Windows installer: <10 MB
- Linux AppImage: <12 MB

If larger:
- Check for debug symbols (should use `--release`)
- Remove unused dependencies
- Optimize assets (icons, images)

## Post-Release Tasks

After publishing a release:

1. **Announce** on relevant channels
2. **Monitor** GitHub Issues for bug reports
3. **Update documentation** if issues are found
4. **Plan next release** based on feedback

## Resources

- [Tauri Build Docs](https://tauri.app/v1/guides/building/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [Semantic Versioning](https://semver.org/)
- [CODE_SIGNING.md](CODE_SIGNING.md) - Adding code signing
- [INSTALLATION.md](INSTALLATION.md) - User installation guide

## Version History

- **v1.0.0** (Oct 2025) - Initial production release
  - Core editing features
  - Screen recording (macOS)
  - Export functionality
  - Multi-platform support

---

**Questions?** See [CONTRIBUTING.md](CONTRIBUTING.md) or open a GitHub issue.
