# ClipForge Installation Guide

Complete installation instructions for ClipForge across all supported platforms.

## Table of Contents

- [System Requirements](#system-requirements)
- [macOS Installation](#macos-installation)
- [Windows Installation](#windows-installation)
- [Linux Installation](#linux-installation)
- [FFmpeg Installation](#ffmpeg-installation)
- [Troubleshooting](#troubleshooting)
- [Uninstallation](#uninstallation)

---

## System Requirements

### Minimum Requirements

- **CPU:** Dual-core processor (Intel Core i3 or AMD equivalent)
- **RAM:** 8 GB
- **Storage:** 500 MB for application + space for projects
- **Display:** 1280x720 resolution or higher
- **FFmpeg:** Version 4.0 or later

### Recommended Requirements

- **CPU:** Quad-core processor (Intel Core i5 / AMD Ryzen 5 or better)
- **RAM:** 16 GB
- **Storage:** SSD with 50 GB+ free space
- **Display:** 1920x1080 resolution or higher
- **GPU:** Dedicated graphics card (for better preview performance)
- **FFmpeg:** Version 6.0 or later

---

## macOS Installation

### Supported Versions

- macOS 11.0 (Big Sur) or later
- Both Intel and Apple Silicon (M1/M2/M3) Macs

### Step 1: Install FFmpeg

ClipForge requires FFmpeg for video processing. The easiest way to install it is via Homebrew:

```bash
# Install Homebrew if you don't have it
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install FFmpeg
brew install ffmpeg

# Verify installation
ffmpeg -version
```

### Step 2: Download ClipForge

1. Visit the [Releases page](https://github.com/clipforge/clipforge/releases)
2. Download the latest `ClipForge_universal.dmg` file
3. The universal binary works on both Intel and Apple Silicon Macs

### Step 3: Install ClipForge

1. Open the downloaded DMG file
2. Drag the ClipForge icon to the Applications folder
3. Eject the DMG
4. Launch ClipForge from Applications

### Step 4: Handle Security Warnings

On first launch, macOS may display a security warning:

1. If you see "ClipForge cannot be opened because it is from an unidentified developer":
   - Go to **System Settings > Privacy & Security**
   - Scroll down to find the message about ClipForge
   - Click **"Open Anyway"**
   - Confirm by clicking **"Open"** in the dialog

2. Grant screen recording permission (required for screen capture):
   - Go to **System Settings > Privacy & Security > Screen Recording**
   - Enable the checkbox next to ClipForge
   - Restart ClipForge if prompted

### Verification

Open Terminal and verify FFmpeg is accessible:

```bash
which ffmpeg
# Should output: /opt/homebrew/bin/ffmpeg (Apple Silicon) or /usr/local/bin/ffmpeg (Intel)

ffmpeg -version
# Should show FFmpeg version information
```

---

## Windows Installation

### Supported Versions

- Windows 10 (64-bit) or later
- Windows 11 (64-bit)

### Step 1: Install FFmpeg

#### Option A: Using Chocolatey (Recommended)

```powershell
# Install Chocolatey (run PowerShell as Administrator)
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install FFmpeg
choco install ffmpeg

# Verify installation
ffmpeg -version
```

#### Option B: Manual Installation

1. Download FFmpeg from [ffmpeg.org](https://ffmpeg.org/download.html#build-windows)
   - Choose "Windows builds from gyan.dev"
   - Download `ffmpeg-release-essentials.zip`

2. Extract the ZIP file to `C:\ffmpeg`

3. Add FFmpeg to PATH:
   - Open Start Menu and search for "Environment Variables"
   - Click "Edit the system environment variables"
   - Click "Environment Variables" button
   - Under "System variables", find and select "Path"
   - Click "Edit"
   - Click "New"
   - Add `C:\ffmpeg\bin`
   - Click "OK" on all dialogs

4. Verify installation:
   - Open Command Prompt
   - Run: `ffmpeg -version`

### Step 2: Download ClipForge

1. Visit the [Releases page](https://github.com/clipforge/clipforge/releases)
2. Download one of:
   - `ClipForge_x64_setup.exe` (NSIS installer - recommended)
   - `ClipForge_x64.msi` (MSI installer - for enterprise deployment)

### Step 3: Install ClipForge

#### NSIS Installer (recommended)

1. Double-click `ClipForge_x64_setup.exe`
2. If Windows Defender SmartScreen appears, click "More info" then "Run anyway"
3. Follow the installation wizard:
   - Accept the license agreement
   - Choose installation location (default: `C:\Program Files\ClipForge`)
   - Select "Create desktop shortcut" if desired
   - Click "Install"
4. Launch ClipForge from the Start Menu or desktop shortcut

#### MSI Installer

1. Double-click `ClipForge_x64.msi`
2. Follow the installation wizard
3. Launch ClipForge from the Start Menu

### Verification

Open Command Prompt and verify:

```cmd
ffmpeg -version
```

---

## Linux Installation

### Supported Distributions

- Ubuntu 20.04+ / Linux Mint 20+
- Debian 11+
- Fedora 34+
- Arch Linux / Manjaro
- Other distributions via AppImage

### Step 1: Install FFmpeg

#### Ubuntu/Debian/Linux Mint

```bash
sudo apt update
sudo apt install ffmpeg

# Verify installation
ffmpeg -version
```

#### Fedora

```bash
# Enable RPM Fusion Free repository
sudo dnf install https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm

# Install FFmpeg
sudo dnf install ffmpeg

# Verify installation
ffmpeg -version
```

#### Arch Linux/Manjaro

```bash
sudo pacman -S ffmpeg

# Verify installation
ffmpeg -version
```

### Step 2: Download ClipForge

Visit the [Releases page](https://github.com/clipforge/clipforge/releases) and download:

- `ClipForge_x86_64.AppImage` (recommended - works on all distributions)
- `clipforge_1.0.0_amd64.deb` (for Debian/Ubuntu)

### Step 3: Install ClipForge

#### Option A: AppImage (Recommended)

AppImage is a universal format that works on all Linux distributions:

```bash
# Download the AppImage
wget https://github.com/clipforge/clipforge/releases/latest/download/ClipForge_x86_64.AppImage

# Make it executable
chmod +x ClipForge_x86_64.AppImage

# Run ClipForge
./ClipForge_x86_64.AppImage
```

Optional: Integrate with your desktop environment:

```bash
# Install AppImageLauncher (Ubuntu/Debian)
sudo add-apt-repository ppa:appimagelauncher-team/stable
sudo apt update
sudo apt install appimagelauncher

# Now when you run the AppImage, it will offer to integrate it
```

#### Option B: Debian Package (Ubuntu/Debian only)

```bash
# Download the .deb file
wget https://github.com/clipforge/clipforge/releases/latest/download/clipforge_1.0.0_amd64.deb

# Install the package
sudo dpkg -i clipforge_1.0.0_amd64.deb

# Install any missing dependencies
sudo apt-get install -f

# Launch from application menu or terminal
clipforge
```

### System Libraries

ClipForge requires WebKit2GTK. On most systems this is already installed. If you encounter issues:

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0

# Fedora
sudo dnf install webkit2gtk4.0 gtk3

# Arch
sudo pacman -S webkit2gtk gtk3
```

---

## FFmpeg Installation

### Verifying FFmpeg Installation

After installing FFmpeg, verify it's accessible from the command line:

**macOS/Linux:**
```bash
which ffmpeg
ffmpeg -version
```

**Windows:**
```cmd
where ffmpeg
ffmpeg -version
```

You should see version information. ClipForge requires FFmpeg 4.0 or later.

### FFmpeg Not Found?

If ClipForge cannot find FFmpeg:

1. **Verify installation:** Run the verification commands above
2. **Check PATH:** Ensure FFmpeg directory is in your system PATH
3. **Restart ClipForge:** After installing FFmpeg, restart the application
4. **Restart your computer:** Sometimes required for PATH changes to take effect

### Recommended FFmpeg Version

While ClipForge works with FFmpeg 4.0+, we recommend using the latest version (6.0+) for:
- Better performance
- Wider codec support
- Bug fixes and security updates

---

## Troubleshooting

### macOS Issues

#### "App is damaged and can't be opened"

This is a Gatekeeper security measure. To fix:

```bash
# Remove the quarantine attribute
xattr -cr /Applications/ClipForge.app
```

Then try launching again.

#### Screen recording permission not working

1. Go to **System Settings > Privacy & Security > Screen Recording**
2. Remove ClipForge from the list if present
3. Quit ClipForge completely (Cmd+Q)
4. Launch ClipForge again
5. Grant permission when prompted
6. Restart ClipForge

### Windows Issues

#### FFmpeg not found after installation

1. Verify FFmpeg is in PATH:
   ```cmd
   echo %PATH%
   ```
   Should include `C:\ffmpeg\bin` (or your installation path)

2. Restart Command Prompt and test again
3. Restart your computer if PATH changes don't take effect

#### Windows Defender SmartScreen blocks installation

This is normal for unsigned applications:
1. Click "More info"
2. Click "Run anyway"

Note: A code signing certificate will be added in future releases to eliminate this warning.

#### "VCRUNTIME140.dll is missing"

Install Microsoft Visual C++ Redistributable:
1. Visit [Microsoft's download page](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)
2. Download and install "vc_redist.x64.exe"
3. Restart your computer
4. Launch ClipForge again

### Linux Issues

#### AppImage won't run

Make sure you have FUSE installed:

```bash
# Ubuntu/Debian
sudo apt install fuse libfuse2

# Fedora
sudo dnf install fuse fuse-libs

# Arch
sudo pacman -S fuse2
```

#### Missing WebKit2GTK

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-0

# Fedora
sudo dnf install webkit2gtk4.0

# Arch
sudo pacman -S webkit2gtk
```

#### Permission denied when running AppImage

```bash
# Make it executable
chmod +x ClipForge_x86_64.AppImage
```

### General Issues

#### ClipForge won't launch

1. Check system requirements (see top of this guide)
2. Verify FFmpeg is installed: `ffmpeg -version`
3. Check application logs:
   - macOS: `~/Library/Logs/ClipForge/`
   - Windows: `%APPDATA%\ClipForge\logs\`
   - Linux: `~/.local/share/ClipForge/logs/`

#### Video import fails

1. Verify the video file isn't corrupted (try playing in another app)
2. Ensure the format is supported (MP4, MOV, WebM, AVI, MKV)
3. Check available disk space
4. Verify FFmpeg can process the file:
   ```bash
   ffprobe your-video-file.mp4
   ```

#### Export fails

1. Ensure FFmpeg is installed and accessible
2. Check available disk space (exports can be large)
3. Verify output directory has write permissions
4. Check FFmpeg can write to the output format:
   ```bash
   ffmpeg -i input.mp4 -c copy test-output.mp4
   ```

---

## Uninstallation

### macOS

1. Quit ClipForge
2. Drag ClipForge from Applications to Trash
3. Remove application data (optional):
   ```bash
   rm -rf ~/Library/Application\ Support/ClipForge
   rm -rf ~/Library/Caches/com.clipforge.app
   rm -rf ~/Library/Logs/ClipForge
   ```

### Windows

#### NSIS Installer

1. Open Settings > Apps > Installed apps
2. Find "ClipForge"
3. Click the three dots > Uninstall
4. Follow the uninstallation wizard

Or use the uninstaller directly:
```
C:\Program Files\ClipForge\uninstall.exe
```

Remove application data (optional):
- Delete `%APPDATA%\ClipForge`
- Delete `%LOCALAPPDATA%\ClipForge`

#### MSI Installer

1. Open Control Panel > Programs > Programs and Features
2. Find "ClipForge"
3. Click "Uninstall"

### Linux

#### AppImage

Simply delete the AppImage file:
```bash
rm ClipForge_x86_64.AppImage
```

Remove application data (optional):
```bash
rm -rf ~/.local/share/ClipForge
rm -rf ~/.cache/ClipForge
```

#### Debian Package

```bash
sudo apt remove clipforge

# Also remove configuration files
sudo apt purge clipforge
```

---

## Getting Help

If you continue to experience issues:

1. Check the [Troubleshooting Guide](docs/troubleshooting.md)
2. Search [GitHub Issues](https://github.com/clipforge/clipforge/issues)
3. Create a new issue with:
   - Your OS and version
   - ClipForge version
   - FFmpeg version (`ffmpeg -version`)
   - Steps to reproduce the problem
   - Any error messages or logs

---

**Next Steps:**
- Read the [User Guide](docs/user-guide.md) to learn how to use ClipForge
- Check out the [Quick Start](README.md#quick-start) section
- Review [Keyboard Shortcuts](docs/keyboard-shortcuts.md)
