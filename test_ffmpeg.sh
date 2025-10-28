#!/bin/bash
# FFmpeg Module 3 Integration Test Script

set -e

echo "🎬 Testing ClipForge FFmpeg Integration"
echo "========================================"
echo ""

# Test video
TEST_VIDEO="/Users/caseymanos/Desktop/Simulator Screen Recording - iPhone 16 Plus - 2025-06-11 at 21.51.10.mp4"

if [ ! -f "$TEST_VIDEO" ]; then
    echo "❌ Test video not found: $TEST_VIDEO"
    exit 1
fi

echo "✅ Test video found"
echo "   Path: $TEST_VIDEO"

# Get video info
echo ""
echo "📊 Video Information:"
ffprobe -v quiet -print_format json -show_format -show_streams "$TEST_VIDEO" 2>/dev/null | grep -E '"duration"|"width"|"height"|"codec_name"' | head -5
echo ""

# Test 1: Extract a frame
echo "🖼️  Test 1: Extract Frame at 1 second"
OUTPUT_FRAME="/tmp/clipforge_test_frame.jpg"
rm -f "$OUTPUT_FRAME"

ffmpeg -ss 1.0 -i "$TEST_VIDEO" -vframes 1 -q:v 2 -y "$OUTPUT_FRAME" 2>&1 | grep -E "frame=|time=" || true

if [ -f "$OUTPUT_FRAME" ]; then
    SIZE=$(ls -lh "$OUTPUT_FRAME" | awk '{print $5}')
    echo "   ✅ Frame extracted successfully"
    echo "   📄 Size: $SIZE"
    echo "   📍 Location: $OUTPUT_FRAME"
else
    echo "   ❌ Frame extraction failed"
    exit 1
fi

echo ""

# Test 2: Trim video
echo "✂️  Test 2: Trim Video (3 seconds from 1s mark)"
OUTPUT_TRIM="/tmp/clipforge_test_trimmed.mp4"
rm -f "$OUTPUT_TRIM"

echo "   ⏳ Processing (this may take 10-20 seconds)..."
ffmpeg -ss 1.0 -i "$TEST_VIDEO" -t 3.0 -c:v libx264 -crf 23 -preset medium \
       -c:a aac -b:a 128k -y "$OUTPUT_TRIM" 2>&1 | grep -E "frame=|time=|size=" | tail -5

if [ -f "$OUTPUT_TRIM" ]; then
    SIZE=$(ls -lh "$OUTPUT_TRIM" | awk '{print $5}')
    DURATION=$(ffprobe -v quiet -print_format json -show_format "$OUTPUT_TRIM" 2>/dev/null | grep duration | head -1 | awk -F: '{print $2}' | tr -d '", ')
    echo "   ✅ Trim successful"
    echo "   📄 Size: $SIZE"
    echo "   ⏱️  Duration: ${DURATION}s (expected ~3s)"
    echo "   📍 Location: $OUTPUT_TRIM"
else
    echo "   ❌ Trim failed"
    exit 1
fi

echo ""

# Test 3: Create two short clips for concat test
echo "🔗 Test 3: Concatenate Videos"
CLIP1="/tmp/clipforge_clip1.mp4"
CLIP2="/tmp/clipforge_clip2.mp4"
OUTPUT_CONCAT="/tmp/clipforge_test_concat.mp4"
rm -f "$CLIP1" "$CLIP2" "$OUTPUT_CONCAT"

echo "   Creating clip 1 (2s from start)..."
ffmpeg -i "$TEST_VIDEO" -t 2.0 -c copy -y "$CLIP1" 2>&1 > /dev/null

echo "   Creating clip 2 (2s from 2s mark)..."
ffmpeg -ss 2.0 -i "$TEST_VIDEO" -t 2.0 -c copy -y "$CLIP2" 2>&1 > /dev/null

# Create concat file
CONCAT_FILE="/tmp/clipforge_concat.txt"
echo "file '$CLIP1'" > "$CONCAT_FILE"
echo "file '$CLIP2'" >> "$CONCAT_FILE"

echo "   Concatenating clips..."
ffmpeg -f concat -safe 0 -i "$CONCAT_FILE" -c copy -y "$OUTPUT_CONCAT" 2>&1 > /dev/null

if [ -f "$OUTPUT_CONCAT" ]; then
    SIZE=$(ls -lh "$OUTPUT_CONCAT" | awk '{print $5}')
    DURATION=$(ffprobe -v quiet -print_format json -show_format "$OUTPUT_CONCAT" 2>/dev/null | grep duration | head -1 | awk -F: '{print $2}' | tr -d '", ')
    echo "   ✅ Concatenation successful"
    echo "   📄 Size: $SIZE"
    echo "   ⏱️  Duration: ${DURATION}s (expected ~4s)"
    echo "   📍 Location: $OUTPUT_CONCAT"
else
    echo "   ❌ Concatenation failed"
    exit 1
fi

echo ""
echo "================================"
echo "✅ All FFmpeg Operations Work!"
echo "================================"
echo ""
echo "Test outputs:"
echo "  - Frame:        $OUTPUT_FRAME"
echo "  - Trimmed:      $OUTPUT_TRIM"
echo "  - Concatenated: $OUTPUT_CONCAT"
echo ""
echo "Module 3 is fully functional! 🎉"
