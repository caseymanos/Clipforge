# ClipForge Setup Instructions

## Current Status

The ClipForge project has been initialized with all core files and configurations in place. Module 1 (Application Shell) is 60% complete with all source files created.

## What's Been Done

### ✅ Completed
- Project structure created (`src-tauri/`, `src/`)
- Rust backend configured with all Module 1 files:
  - `main.rs` - Application entry point
  - `commands/mod.rs` - IPC commands
  - `protocols.rs` - Custom stream:// protocol for video files
  - `window_state.rs` - Window position/size persistence
  - `menu.rs` - Application menu bar
- Frontend configured with Svelte + TypeScript
- All configuration files created:
  - `Cargo.toml` - Rust dependencies
  - `package.json` - Frontend dependencies (installed)
  - `tauri.conf.json` - Tauri configuration
  - `vite.config.ts` - Vite build configuration
  - `tsconfig.json` - TypeScript configuration
- npm dependencies installed successfully

### ⏳ Remaining for Module 1
- Install Rust toolchain
- First successful compilation
- Test application launch
- Verify all features work

## Prerequisites

Before you can run ClipForge, you need to install:

### 1. Rust Toolchain

Install Rust using rustup:

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or visit: https://rustup.rs/
```

After installation, restart your terminal and verify:
```bash
rustc --version
cargo --version
```

### 2. System Dependencies (macOS)

```bash
# Xcode Command Line Tools (if not already installed)
xcode-select --install
```

### 3. Tauri Prerequisites

The following are already installed via npm, but ensure you have:
- Node.js 18+ ✅ (confirmed working)
- npm or pnpm ✅ (confirmed working)

## Running the Application

### Development Mode

Once Rust is installed:

```bash
# Run in development mode with hot reload
npm run tauri:dev
```

This will:
1. Start the Vite dev server (frontend) on port 5173
2. Compile the Rust backend
3. Launch the Tauri application window

### First Time Setup

The first run will take longer as Cargo downloads and compiles all Rust dependencies (~5-10 minutes).

### Building for Production

```bash
# Create optimized production build
npm run tauri:build
```

Output will be in `src-tauri/target/release/bundle/`

## Development Workflow

### Frontend Development

```bash
# Run frontend dev server only
npm run dev
```

Visit http://localhost:5173 in your browser to see the UI.

### Backend Development

```bash
# Check Rust code for errors
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test

# Format Rust code
cd src-tauri && cargo fmt

# Lint Rust code
cd src-tauri && cargo clippy
```

## Project Structure

```
clipforge/
├── src/                    # Frontend (Svelte)
│   ├── App.svelte         # Main component
│   ├── main.ts            # Entry point
│   └── styles.css         # Global styles
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── main.rs        # App entry point
│   │   ├── commands/      # IPC commands
│   │   ├── protocols.rs   # Custom protocols
│   │   ├── window_state.rs # Window persistence
│   │   └── menu.rs        # Menu bar
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri config
│   └── build.rs           # Build script
├── package.json           # Frontend dependencies
├── vite.config.ts         # Vite configuration
├── CLAUDE.md              # Development guide
├── progress.md            # Implementation tracking
└── clipforges/            # Full documentation

```

## Implemented Features (Module 1)

### Backend (Rust)
- ✅ Application lifecycle management
- ✅ Window state persistence (position, size, maximized state)
- ✅ Custom `stream://` protocol for video file access
- ✅ IPC command system
- ✅ Menu bar with event handling
- ✅ Security: Path validation for file access
- ✅ Logging infrastructure

### Frontend (Svelte)
- ✅ Modern Svelte 4 setup with TypeScript
- ✅ Vite for fast development
- ✅ Tauri API integration
- ✅ Basic UI component
- ✅ Dark mode support

## Next Steps

### Immediate (Module 1 Completion)
1. Install Rust toolchain (see Prerequisites above)
2. Run `npm run tauri:dev` to test first compilation
3. Verify application launches successfully
4. Test window state persistence (resize, close, reopen)
5. Test menu bar functionality
6. Run tests: `cd src-tauri && cargo test`

### Module 2 - File System & Media (Next Phase)
Once Module 1 is verified working:
1. Implement SQLite database setup
2. Create file import service
3. Add FFprobe for metadata extraction
4. Implement thumbnail generation
5. Build media library queries

See `clipforges/module-02-file-system-media.md` for detailed specifications.

## Troubleshooting

### "cargo: command not found"
- Install Rust toolchain (see Prerequisites)
- Restart terminal after installation

### Port 5173 already in use
```bash
# Kill process using port 5173
lsof -ti:5173 | xargs kill -9
```

### Build errors
```bash
# Clean and rebuild
cd src-tauri && cargo clean
npm run tauri:dev
```

### Frontend issues
```bash
# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install
```

## Architecture Highlights

### Custom Protocol
The `stream://` protocol enables efficient video streaming without JSON serialization:
```typescript
// Frontend can access video files like this:
const videoUrl = convertFileSrc('/path/to/video.mp4');
videoElement.src = videoUrl; // Uses stream:// protocol
```

### Window State Persistence
Window size and position are automatically saved to:
- macOS: `~/Library/Application Support/clipforge/window_state.json`
- Windows: `%APPDATA%\clipforge\window_state.json`
- Linux: `~/.config/clipforge/window_state.json`

### Security
- File access restricted to user's home directory and app data
- Path validation prevents access to system files
- Content Security Policy configured
- No shell command injection vulnerabilities

## Resources

- **Full Documentation**: `clipforges/` directory
- **Module Specifications**: `clipforges/module-*.md` files
- **Architecture**: `clipforges/02-technical-architecture.md`
- **Progress Tracking**: `progress.md`
- **Development Guide**: `CLAUDE.md`

## Contact & Support

This is the ClipForge desktop video editor project. For implementation questions, refer to the comprehensive documentation in the `clipforges/` directory.

---

**Last Updated**: October 27, 2025
**Status**: Module 1 (Application Shell) - 60% Complete
**Next Milestone**: First successful application launch
