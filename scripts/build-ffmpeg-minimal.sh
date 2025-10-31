#!/bin/bash
# Minimal FFmpeg build script for ClipForge (macOS Apple Silicon)
# This builds FFmpeg with only the features ClipForge needs to keep binary size small

set -e

echo "Building minimal FFmpeg for ClipForge (Apple Silicon)"
echo "======================================================"

# Configuration
FFMPEG_VERSION="n7.1"
BUILD_DIR="/tmp/ffmpeg-clipforge-build"
INSTALL_DIR="/tmp/ffmpeg-clipforge-install"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARIES_DIR="$PROJECT_ROOT/src-tauri/binaries"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Step 1: Checking dependencies...${NC}"
if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed"
    exit 1
fi

if ! command -v make &> /dev/null; then
    echo "Error: make is required. Install Xcode Command Line Tools"
    exit 1
fi

if ! command -v pkg-config &> /dev/null; then
    echo "Warning: pkg-config not found. Installing via Homebrew..."
    brew install pkg-config
fi

# Check for x264 library
echo -e "${BLUE}Step 2: Checking for libx264...${NC}"
if ! pkg-config --exists x264; then
    echo "libx264 not found. Installing via Homebrew..."
    brew install x264
fi

echo -e "${GREEN}✓ Dependencies OK${NC}"

# Clean and create build directory
echo -e "${BLUE}Step 3: Setting up build directory...${NC}"
rm -rf "$BUILD_DIR"
rm -rf "$INSTALL_DIR"
mkdir -p "$BUILD_DIR"
mkdir -p "$INSTALL_DIR"

# Clone FFmpeg if needed
echo -e "${BLUE}Step 4: Getting FFmpeg source...${NC}"
if [ ! -d "$BUILD_DIR/ffmpeg" ]; then
    echo "Cloning FFmpeg $FFMPEG_VERSION..."
    git clone --depth 1 --branch "$FFMPEG_VERSION" https://git.ffmpeg.org/ffmpeg.git "$BUILD_DIR/ffmpeg"
fi

cd "$BUILD_DIR/ffmpeg"

echo -e "${BLUE}Step 5: Configuring FFmpeg...${NC}"
./configure \
  --prefix="$INSTALL_DIR" \
  --arch=arm64 \
  --target-os=darwin \
  --cc=clang \
  \
  `# Disable everything by default` \
  --disable-all \
  --disable-autodetect \
  --disable-doc \
  --disable-htmlpages \
  --disable-manpages \
  --disable-podpages \
  --disable-txtpages \
  --disable-debug \
  --disable-ffplay \
  --disable-network \
  --disable-postproc \
  \
  `# Enable core components` \
  --enable-ffmpeg \
  --enable-ffprobe \
  --enable-avcodec \
  --enable-avformat \
  --enable-avfilter \
  --enable-swscale \
  --enable-swresample \
  \
  `# Video codecs - encoders` \
  --enable-libx264 \
  --enable-encoder=libx264 \
  --enable-encoder=mjpeg \
  \
  `# Video codecs - decoders` \
  --enable-decoder=h264 \
  --enable-decoder=hevc \
  --enable-decoder=mpeg4 \
  --enable-decoder=mjpeg \
  --enable-decoder=png \
  --enable-decoder=rawvideo \
  \
  `# Audio codecs - encoders` \
  --enable-encoder=aac \
  --enable-encoder=pcm_s16le \
  \
  `# Audio codecs - decoders` \
  --enable-decoder=aac \
  --enable-decoder=mp3 \
  --enable-decoder=pcm_s16le \
  --enable-decoder=pcm_f32le \
  \
  `# Demuxers (input formats)` \
  --enable-demuxer=mov \
  --enable-demuxer=mp4 \
  --enable-demuxer=m4v \
  --enable-demuxer=concat \
  --enable-demuxer=image2 \
  --enable-demuxer=mjpeg \
  --enable-demuxer=wav \
  --enable-demuxer=mp3 \
  --enable-demuxer=aac \
  \
  `# Muxers (output formats)` \
  --enable-muxer=mp4 \
  --enable-muxer=mov \
  --enable-muxer=image2 \
  --enable-muxer=mjpeg \
  --enable-muxer=wav \
  \
  `# Filters - Video` \
  --enable-filter=scale \
  --enable-filter=crop \
  --enable-filter=trim \
  --enable-filter=concat \
  --enable-filter=fade \
  --enable-filter=eq \
  --enable-filter=geq \
  --enable-filter=boxblur \
  --enable-filter=unsharp \
  --enable-filter=split \
  --enable-filter=setpts \
  --enable-filter=format \
  --enable-filter=null \
  --enable-filter=color \
  --enable-filter=fps \
  --enable-filter=overlay \
  \
  `# Filters - Audio` \
  --enable-filter=volume \
  --enable-filter=aformat \
  --enable-filter=anull \
  --enable-filter=atrim \
  --enable-filter=asetpts \
  --enable-filter=asplit \
  --enable-filter=loudnorm \
  --enable-filter=aresample \
  \
  `# Parsers` \
  --enable-parser=h264 \
  --enable-parser=hevc \
  --enable-parser=aac \
  --enable-parser=mpeg4video \
  --enable-parser=mjpeg \
  \
  `# Protocols` \
  --enable-protocol=file \
  --enable-protocol=pipe \
  \
  `# macOS specific - CRITICAL: Must enable indev for screen capture` \
  --enable-avfoundation \
  --enable-indev=avfoundation \
  --enable-videotoolbox \
  --enable-audiotoolbox \
  \
  `# Build options` \
  --enable-static \
  --disable-shared \
  --enable-pthreads \
  --enable-zlib \
  --enable-gpl \
  --pkg-config-flags="--static"

echo -e "${GREEN}✓ Configuration complete${NC}"

# Build
echo -e "${BLUE}Step 6: Building FFmpeg (this may take 10-15 minutes)...${NC}"
NUM_CORES=$(sysctl -n hw.ncpu)
echo "Using $NUM_CORES cores for compilation"
make -j"$NUM_CORES"

echo -e "${GREEN}✓ Build complete${NC}"

# Install to temp location
echo -e "${BLUE}Step 7: Installing to temporary directory...${NC}"
make install

# Check binary sizes
echo -e "${BLUE}Step 8: Checking binary sizes...${NC}"
echo "ffmpeg:  $(du -h "$INSTALL_DIR/bin/ffmpeg" | cut -f1)"
echo "ffprobe: $(du -h "$INSTALL_DIR/bin/ffprobe" | cut -f1)"

# Copy binaries to project
echo -e "${BLUE}Step 9: Copying binaries to project...${NC}"
mkdir -p "$BINARIES_DIR"
cp "$INSTALL_DIR/bin/ffmpeg" "$BINARIES_DIR/ffmpeg-aarch64-apple-darwin"
cp "$INSTALL_DIR/bin/ffprobe" "$BINARIES_DIR/ffprobe-aarch64-apple-darwin"

# Make them executable
chmod +x "$BINARIES_DIR/ffmpeg-aarch64-apple-darwin"
chmod +x "$BINARIES_DIR/ffprobe-aarch64-apple-darwin"

echo -e "${GREEN}✓ Binaries copied to: $BINARIES_DIR${NC}"

# Test the binaries
echo -e "${BLUE}Step 10: Testing binaries...${NC}"
"$BINARIES_DIR/ffmpeg-aarch64-apple-darwin" -version | head -n 1
"$BINARIES_DIR/ffprobe-aarch64-apple-darwin" -version | head -n 1

# Cleanup
echo -e "${BLUE}Step 11: Cleaning up build directory...${NC}"
echo "Build artifacts are in: $BUILD_DIR"
echo "You can delete this directory to save space:"
echo "  rm -rf $BUILD_DIR"

echo ""
echo -e "${GREEN}======================================================"
echo "✓ FFmpeg build complete!"
echo "======================================================${NC}"
echo ""
echo "Binaries location:"
echo "  $BINARIES_DIR/ffmpeg-aarch64-apple-darwin"
echo "  $BINARIES_DIR/ffprobe-aarch64-apple-darwin"
echo ""
echo "Next steps:"
echo "  1. Configure Tauri to bundle these binaries"
echo "  2. Update ffmpeg_utils.rs to use bundled path"
echo "  3. Build ClipForge with: npm run tauri build"
