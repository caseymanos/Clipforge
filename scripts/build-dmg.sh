#!/bin/bash
set -e

# ClipForge Production DMG Build Script
# This script builds a production-ready DMG installer for macOS

echo "🚀 ClipForge Production Build Script"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}❌ Error: This script must be run on macOS${NC}"
    exit 1
fi

# Check for required tools
echo "📋 Checking prerequisites..."

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm not found. Please install Node.js${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ cargo not found. Please install Rust${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Prerequisites OK${NC}"
echo ""

# Check for FFmpeg binaries
echo "🎬 Checking FFmpeg binaries..."
if [[ ! -f "src-tauri/binaries/ffmpeg" ]] || [[ ! -f "src-tauri/binaries/ffprobe" ]]; then
    echo -e "${YELLOW}⚠️  FFmpeg binaries not found in src-tauri/binaries/${NC}"
    echo "Please ensure FFmpeg binaries are present before building"
    exit 1
fi

echo -e "${GREEN}✓ FFmpeg binaries found${NC}"
echo "  - ffmpeg: $(ls -lh src-tauri/binaries/ffmpeg | awk '{print $5}')"
echo "  - ffprobe: $(ls -lh src-tauri/binaries/ffprobe | awk '{print $5}')"
echo ""

# Check for code signing environment variables (optional but recommended)
echo "🔐 Checking code signing configuration..."
if [[ -z "$APPLE_SIGNING_IDENTITY" ]]; then
    echo -e "${YELLOW}⚠️  APPLE_SIGNING_IDENTITY not set${NC}"
    echo "The build will proceed but won't be code signed."
    echo "For distribution, please set:"
    echo "  export APPLE_SIGNING_IDENTITY=\"Developer ID Application: Your Name (TEAM_ID)\""
    echo ""
else
    echo -e "${GREEN}✓ Code signing configured${NC}"
    echo "  Identity: $APPLE_SIGNING_IDENTITY"
    echo ""
fi

# Get current version
VERSION=$(grep '^version' src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)
echo "📦 Building version: $VERSION"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cd src-tauri
cargo clean
cd ..
rm -rf dist/
echo -e "${GREEN}✓ Clean complete${NC}"
echo ""

# Install dependencies
echo "📥 Installing dependencies..."
npm install
echo -e "${GREEN}✓ Dependencies installed${NC}"
echo ""

# Build the app
echo "🔨 Building ClipForge..."
echo "This may take several minutes..."
echo ""

if npm run tauri build; then
    echo ""
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""

    # Find the built DMG
    DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name "*.dmg" 2>/dev/null | head -1)
    APP_PATH=$(find src-tauri/target/release/bundle/macos -name "*.app" 2>/dev/null | head -1)

    if [[ -n "$DMG_PATH" ]]; then
        DMG_SIZE=$(ls -lh "$DMG_PATH" | awk '{print $5}')
        echo "📦 DMG Installer:"
        echo "   Path: $DMG_PATH"
        echo "   Size: $DMG_SIZE"
        echo ""
    fi

    if [[ -n "$APP_PATH" ]]; then
        APP_SIZE=$(du -sh "$APP_PATH" | awk '{print $1}')
        echo "🎯 Application Bundle:"
        echo "   Path: $APP_PATH"
        echo "   Size: $APP_SIZE"
        echo ""
    fi

    # Check if notarization is needed
    if [[ -n "$APPLE_SIGNING_IDENTITY" ]] && [[ -n "$APPLE_ID" ]] && [[ -n "$APPLE_PASSWORD" ]]; then
        echo "🔔 Notarization credentials detected"
        echo "To notarize the app, run:"
        echo "  xcrun notarytool submit \"$DMG_PATH\" \\"
        echo "    --apple-id \"$APPLE_ID\" \\"
        echo "    --password \"$APPLE_PASSWORD\" \\"
        echo "    --team-id \"$APPLE_TEAM_ID\" \\"
        echo "    --wait"
        echo ""
        echo "After approval, staple the ticket:"
        echo "  xcrun stapler staple \"$DMG_PATH\""
        echo ""
    else
        echo -e "${YELLOW}⚠️  For App Store/public distribution, you need to:${NC}"
        echo "1. Code sign the app"
        echo "2. Notarize with Apple"
        echo "3. Staple the notarization ticket"
        echo ""
        echo "See RELEASE.md for detailed instructions"
        echo ""
    fi

    echo "✨ Build complete! The DMG is ready for testing."
    echo ""
    echo "Next steps:"
    echo "1. Test the DMG on a clean macOS system"
    echo "2. Verify screen recording permissions work"
    echo "3. Test microphone audio capture"
    echo "4. Check FFmpeg bundling (try recording/editing)"
    echo ""

else
    echo ""
    echo -e "${RED}❌ Build failed${NC}"
    echo "Check the error messages above for details"
    exit 1
fi
