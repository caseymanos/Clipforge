#!/bin/bash

# Download 10 more sample videos to reach 20 total

set -e

DOWNLOAD_DIR="/Users/caseymanos/GauntletAI/clipforge/test-videos"
cd "$DOWNLOAD_DIR"

echo "Downloading 10 additional sample videos..."
echo ""

# Additional test videos with different content
declare -a MORE_URLS=(
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_5MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_5MB.mp4"
    "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/1080/Big_Buck_Bunny_1080_10s_5MB.mp4"
    "https://test-videos.co.uk/vids/sintel/mp4/h264/360/Sintel_360_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/sintel/mp4/h264/720/Sintel_720_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/sintel/mp4/h264/1080/Sintel_1080_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/jellyfish/mp4/h264/360/Jellyfish_360_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/jellyfish/mp4/h264/1080/Jellyfish_1080_10s_1MB.mp4"
    "https://test-videos.co.uk/vids/jellyfish/mp4/h264/720/Jellyfish_720_10s_2MB.mp4"
    "https://test-videos.co.uk/vids/jellyfish/mp4/h264/1080/Jellyfish_1080_10s_2MB.mp4"
)

# Download videos
count=11
for url in "${MORE_URLS[@]}"; do
    filename=$(basename "$url")
    echo "[$count/20] Downloading: $filename"

    if curl -L -f -o "$filename" "$url" --connect-timeout 30 --max-time 120; then
        echo "  ✓ Downloaded successfully"
    else
        echo "  ✗ Download failed, skipping..."
    fi

    count=$((count + 1))
done

echo ""
echo "Download complete!"
echo "Videos saved to: $DOWNLOAD_DIR"
echo ""
ls -lh *.mp4 2>/dev/null | wc -l | xargs echo "Total videos downloaded:"
