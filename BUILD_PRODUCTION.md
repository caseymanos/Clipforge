# ClipForge Production Build Guide

This guide covers building and distributing production-ready DMG installers for macOS.

## Quick Start

For experienced developers with code signing already set up:

```bash
# Ensure FFmpeg binaries are in place
ls -lh src-tauri/binaries/ffmpeg src-tauri/binaries/ffprobe

# Run the build script
./scripts/build-dmg.sh
```

The DMG will be created at: `src-tauri/target/release/bundle/dmg/ClipForge_0.1.0_universal.dmg`

## Prerequisites

### Required Tools

1. **macOS 11.0+** - Required for building and code signing
2. **Node.js 18+** - For frontend build
3. **Rust** - For Tauri backend
4. **FFmpeg binaries** - Must be placed in `src-tauri/binaries/`

Check prerequisites:
```bash
node --version    # Should be 18+
npm --version
cargo --version
rustc --version
```

### FFmpeg Binaries

The app bundles FFmpeg as a sidecar binary. You must provide these files:

- `src-tauri/binaries/ffmpeg` (~5.9MB)
- `src-tauri/binaries/ffprobe` (~5.9MB)

These should be:
- Universal binaries (arm64 + x86_64)
- Executable (`chmod +x`)
- From a trusted source (e.g., official FFmpeg builds)

If missing, the build script will exit with an error.

### Code Signing (Optional but Recommended)

For distribution outside development, you need:

1. **Apple Developer Account** ($99/year)
2. **Developer ID Application Certificate**
   - Log in to [Apple Developer](https://developer.apple.com)
   - Navigate to Certificates, Identifiers & Profiles
   - Create a "Developer ID Application" certificate
   - Download and install in Keychain

3. **Set environment variables**:
```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_ID="your-apple-id@example.com"
export APPLE_PASSWORD="app-specific-password"  # Create at appleid.apple.com
export APPLE_TEAM_ID="TEAM_ID"
```

**Without code signing**: The build will work but users will see "unidentified developer" warnings.

## Build Process

### 1. Clean Build

The build script automatically:
- Runs `cargo clean` to remove previous builds
- Deletes `dist/` directory
- Installs fresh npm dependencies

### 2. Frontend Build

Vite builds the Svelte frontend:
- Bundles TypeScript/JavaScript
- Processes CSS
- Optimizes assets
- Outputs to `dist/`

### 3. Rust Backend Build

Cargo builds the Tauri backend:
- Compiles Rust code in release mode
- Links dependencies
- Creates optimized binary

### 4. Bundle Creation

Tauri creates platform bundles:
- `.app` bundle in `src-tauri/target/release/bundle/macos/`
- `.dmg` installer in `src-tauri/target/release/bundle/dmg/`

### 5. Code Signing (if configured)

If `APPLE_SIGNING_IDENTITY` is set:
- Signs the `.app` bundle
- Signs the DMG
- Prepares for notarization

## Running the Build

### Using the Build Script (Recommended)

```bash
./scripts/build-dmg.sh
```

This script:
- ✅ Validates all prerequisites
- ✅ Checks FFmpeg binaries exist
- ✅ Displays code signing status
- ✅ Shows version being built
- ✅ Cleans previous builds
- ✅ Installs dependencies
- ✅ Runs the build
- ✅ Reports output locations and sizes
- ✅ Provides next steps

### Manual Build

If you prefer manual control:

```bash
# Clean
cd src-tauri && cargo clean && cd ..
rm -rf dist/

# Install dependencies
npm install

# Build
npm run tauri build
```

## Outputs

After a successful build:

### Application Bundle
```
src-tauri/target/release/bundle/macos/ClipForge.app
Size: ~15-20MB
```

This is the application bundle. You can:
- Run directly: `open src-tauri/target/release/bundle/macos/ClipForge.app`
- Copy to `/Applications` manually
- Use for testing

### DMG Installer
```
src-tauri/target/release/bundle/dmg/ClipForge_0.1.0_universal.dmg
Size: ~12-18MB
```

This is the distributable installer. Users can:
- Download and open the DMG
- Drag ClipForge.app to Applications
- Eject and delete the DMG

## Notarization (Required for Public Distribution)

Apple requires notarization for apps distributed outside the App Store.

### 1. Submit for Notarization

After building with code signing:

```bash
xcrun notarytool submit \
  "src-tauri/target/release/bundle/dmg/ClipForge_0.1.0_universal.dmg" \
  --apple-id "your-apple-id@example.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait
```

This uploads the DMG to Apple's servers for automated security scanning. Usually takes 5-15 minutes.

### 2. Check Status

```bash
# List recent submissions
xcrun notarytool history --apple-id "your-apple-id@example.com" --password "app-specific-password"

# Get detailed log for a submission
xcrun notarytool log <submission-id> --apple-id "your-apple-id@example.com" --password "app-specific-password"
```

### 3. Staple the Ticket

Once approved:

```bash
xcrun stapler staple "src-tauri/target/release/bundle/dmg/ClipForge_0.1.0_universal.dmg"
```

This attaches the notarization ticket to the DMG, allowing offline verification.

### 4. Verify

```bash
spctl -a -vv -t install "src-tauri/target/release/bundle/dmg/ClipForge_0.1.0_universal.dmg"
```

Should show: `source=Notarized Developer ID`

## Testing Checklist

Before distributing, test on a clean macOS system:

### Installation Testing
- [ ] DMG opens without warnings (if notarized)
- [ ] App drags to Applications folder
- [ ] App opens from Applications folder
- [ ] First launch doesn't show "damaged" error

### Permissions Testing
- [ ] Screen recording permission prompt appears
- [ ] Microphone permission prompt appears
- [ ] Camera permission prompt appears (if using webcam)
- [ ] File system access works for saving recordings

### Core Functionality Testing
- [ ] Screen recording works (video and audio)
- [ ] Microphone recording produces clean audio (no stuttering)
- [ ] Webcam overlay appears with correct colors
- [ ] Video trimming and editing works
- [ ] Subtitle generation functions
- [ ] Export produces valid video files
- [ ] FFmpeg binaries are accessible (check with recordings)

### Performance Testing
- [ ] App launches in < 3 seconds
- [ ] Timeline remains responsive with 10+ clips
- [ ] Export completes without crashes
- [ ] Memory usage stays reasonable (< 500MB during editing)

## Distribution

### Internal Testing
For beta testers, you can:
1. Share the DMG via download link
2. Provide installation instructions
3. Collect feedback

**Note**: Without notarization, testers must right-click > Open to bypass Gatekeeper.

### Public Release
For public distribution:
1. **Must** be notarized and stapled
2. Host the DMG on your website or CDN
3. Provide SHA-256 checksum for verification
4. Consider creating a landing page with system requirements

### App Store (Alternative)
To distribute via Mac App Store:
1. Different provisioning profile required
2. Sandboxing restrictions apply
3. Different entitlements needed
4. Review process required

See Apple's App Store guidelines for details.

## Troubleshooting

### "Developer cannot be verified" error
- **Cause**: App not code signed or not notarized
- **Solution**: Complete code signing and notarization steps

### FFmpeg binaries not found
- **Cause**: Missing binaries in `src-tauri/binaries/`
- **Solution**: Place ffmpeg and ffprobe binaries in correct location

### Build fails with "linking error"
- **Cause**: Missing Rust dependencies
- **Solution**: Run `rustup update` and `cargo clean`

### Screen recording doesn't work
- **Cause**: Missing entitlements or permissions
- **Solution**: Check `src-tauri/entitlements.plist` includes screen recording permission

### Audio has artifacts
- **Cause**: FFmpeg not found or incorrect version
- **Solution**: Verify FFmpeg binaries are universal and from official source

### DMG creation fails
- **Cause**: Insufficient disk space or permissions
- **Solution**: Free up space, check write permissions on `src-tauri/target/`

## Version Management

To update the version before building:

1. **Update `src-tauri/Cargo.toml`**:
```toml
[package]
version = "0.2.0"
```

2. **Update `src-tauri/tauri.conf.json`**:
```json
{
  "version": "0.2.0"
}
```

3. **Update `package.json`**:
```json
{
  "version": "0.2.0"
}
```

The build script automatically detects and displays the version being built.

## CI/CD Integration

For automated builds, you can integrate the build script into GitHub Actions:

```yaml
name: Build DMG
on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - uses: dtolnay/rust-toolchain@stable

      # Add FFmpeg binaries (from secure storage)
      - name: Setup FFmpeg
        run: |
          # Download or decrypt your FFmpeg binaries
          chmod +x src-tauri/binaries/ffmpeg
          chmod +x src-tauri/binaries/ffprobe

      # Configure code signing (from secrets)
      - name: Configure signing
        env:
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: ./scripts/build-dmg.sh

      # Upload artifact
      - uses: actions/upload-artifact@v3
        with:
          name: ClipForge-DMG
          path: src-tauri/target/release/bundle/dmg/*.dmg
```

## Security Best Practices

1. **Never commit code signing credentials** to version control
2. **Use app-specific passwords** for Apple ID (not your main password)
3. **Store credentials** in environment variables or secrets manager
4. **Verify FFmpeg binaries** are from trusted sources
5. **Test on clean VM** before public distribution
6. **Provide checksums** with downloads for verification
7. **Sign all releases** to prevent tampering

## Support

For build issues:
- Check the [Tauri documentation](https://tauri.app/v1/guides/building/)
- Review [Apple's code signing guide](https://developer.apple.com/support/code-signing/)
- Open an issue on the ClipForge GitHub repository

For notarization help:
- [Apple's notarization documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- Use `xcrun notarytool log <submission-id>` to see rejection reasons
