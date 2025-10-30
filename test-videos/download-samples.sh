#!/bin/bash

# Download 20 sample videos for ClipForge performance testing
# Using Coverr.co free stock videos (no authentication required)

set -e

DOWNLOAD_DIR="/Users/caseymanos/GauntletAI/clipforge/test-videos"
cd "$DOWNLOAD_DIR"

echo "Downloading 20 sample videos for ClipForge performance testing..."
echo "Source: Coverr.co (Creative Commons)"
echo ""

# Coverr provides direct download links for their videos
# These are curated sample videos of various lengths and resolutions

# Note: Coverr download URLs change, so we'll use a more reliable approach
# We'll download from sample video providers that offer stable URLs

# Array of sample video URLs (using stable sources)
declare -a VIDEO_URLS=(
    # Using sample videos from various sources with stable URLs
    "https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_1mb.mp4"
    "https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_2mb.mp4"
    "https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_5mb.mp4"
    "https://sample-videos.com/video321/mp4/720/big_buck_bunny_720p_10mb.mp4"
    "https://sample-videos.com/video321/mp4/240/big_buck_bunny_240p_1mb.mp4"
    "https://sample-videos.com/video321/mp4/240/big_buck_bunny_240p_2mb.mp4"
    "https://sample-videos.com/video321/mp4/480/big_buck_bunny_480p_1mb.mp4"
    "https://sample-videos.com/video321/mp4/480/big_buck_bunny_480p_2mb.mp4"
    "https://sample-videos.com/video321/mp4/480/big_buck_bunny_480p_5mb.mp4"
    "https://sample-videos.com/video321/mp4/480/big_buck_bunny_480p_10mb.mp4"
)

# Additional test videos from test video repository
declare -a MORE_URLS=(
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/1080/Big_Buck_Bunny_1080_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/1080/Big_Buck_Bunny_1080_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/sintel/mp4/h264/360/Sintel_360_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/sintel/mp4/h264/720/Sintel_720_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/sintel/mp4/h264/1080/Sintel_1080_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/jellyfish/mp4/h264/720/Jellyfish_720_10s_1MB.mp4"
)

# Combine arrays
ALL_URLS=("${VIDEO_URLS[@]}" "${MORE_URLS[@]}")

# Download first 20 videos
count=1
for url in "${ALL_URLS[@]}"; do
    if [ $count -le 20 ]; then
        filename=$(basename "$url")
        echo "[$count/20] Downloading: $filename"

        if curl -L -f -o "$filename" "$url" --connect-timeout 30 --max-time 120; then
            echo "  ✓ Downloaded successfully"
        else
            echo "  ✗ Download failed, skipping..."
        fi

        count=$((count + 1))
    else
        break
    fi
done

echo ""
echo "Download complete!"
echo "Videos saved to: $DOWNLOAD_DIR"
echo ""
ls -lh *.mp4 2>/dev/null | wc -l | xargs echo "Total videos downloaded:"
