# Contributing to ClipForge

Note: The main branch currently targets macOS on Apple Silicon (M1/M2/M3) only. Windows and Linux builds are not supported for end users at this time.

Thank you for your interest in contributing to ClipForge! We welcome contributions from the community.

## Code of Conduct

This project adheres to the Contributor Covenant [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers via GitHub Issues.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When creating a bug report, include as many details as possible:

- Use a clear and descriptive title
- Describe the exact steps to reproduce the problem
- Provide specific examples to demonstrate the steps
- Describe the behavior you observed and what you expected
- Include screenshots if applicable
- Note your environment (OS, ClipForge version, FFmpeg version)

Use the bug report template when creating a new issue.

### Suggesting Features

Feature suggestions are tracked as GitHub issues. When creating a feature suggestion:

- Use a clear and descriptive title
- Provide a detailed description of the proposed feature
- Explain why this feature would be useful
- List any alternatives you've considered

Use the feature request template when creating a new issue.

### Pull Requests

1. **Fork the repository** and create your branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Set up your development environment:**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install Node.js dependencies
   npm install

   # Install FFmpeg (required)
   # macOS:
   brew install ffmpeg
   # Note: Windows/Linux commands below are for historical reference; the main branch is macOS (Apple Silicon) only
   # Ubuntu/Debian (not supported target):
   sudo apt install ffmpeg
   # Windows (not supported target):
   # Download from https://ffmpeg.org/download.html
   ```

3. **Make your changes:**
   - Write clear, readable code
   - Follow the existing code style
   - Add tests if applicable
   - Update documentation if needed

4. **Test your changes:**
   ```bash
   # Run Rust tests
   cd src-tauri && cargo test

   # Run Rust linter
   cd src-tauri && cargo clippy

   # Format Rust code
   cd src-tauri && cargo fmt

   # Test the app in development mode
   npm run tauri dev

   # Test production build
   npm run tauri build
   ```

5. **Commit your changes:**
   - Use clear, descriptive commit messages
   - Follow conventional commit format: `type: description`
   - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

   Example:
   ```bash
   git commit -m "feat: add subtitle export feature"
   git commit -m "fix: resolve timeline rendering bug"
   git commit -m "docs: update installation instructions"
   ```

6. **Push to your fork and create a pull request:**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Fill out the pull request template** with all relevant information

## Development Workflow

### Project Structure

```
clipforge/
├── src/                  # Svelte frontend code
│   ├── lib/
│   │   ├── components/   # Svelte components
│   │   └── stores/       # State management
│   └── App.svelte        # Root component
├── src-tauri/            # Rust backend code
│   ├── src/
│   │   ├── commands/     # Tauri command handlers
│   │   ├── models/       # Data structures
│   │   └── main.rs       # Entry point
│   └── Cargo.toml
├── docs/                 # User documentation
└── README.md
```

### Code Style

**Rust:**
- Follow standard Rust formatting (`cargo fmt`)
- Run `cargo clippy` and address warnings
- Use meaningful variable names
- Add doc comments for public APIs
- Prefer explicit error handling over unwrap/expect

**TypeScript/Svelte:**
- Use TypeScript for type safety
- Follow existing component patterns
- Use reactive statements ($:) appropriately
- Keep components small and focused

**Commit Messages:**
- Use present tense ("add feature" not "added feature")
- Use imperative mood ("move cursor to..." not "moves cursor to...")
- Limit first line to 72 characters
- Reference issues and PRs in the body

### Running Tests

```bash
# Rust unit tests
cd src-tauri && cargo test

# Rust integration tests
cd src-tauri && cargo test --test integration

# Run with verbose output
cd src-tauri && cargo test -- --nocapture
```

### Building for Release

```bash
# Create optimized production build
npm run tauri build

# Output location:
# macOS: src-tauri/target/release/bundle/dmg/
# Windows/Linux paths listed for reference only; these platforms are not supported in the main branch
```

## Architecture Guidelines

### Backend (Rust/Tauri)

- Use async/await for I/O operations
- Pass file paths to FFmpeg (don't load entire videos into memory)
- Never use shell string interpolation with user input (command injection risk)
- Return Result types, never panic in production code
- Use structured logging (log crate)

Example of safe FFmpeg usage:
```rust
// GOOD: Safe argument passing
Command::new("ffmpeg")
    .arg("-i")
    .arg(&input_path)
    .arg("-codec")
    .arg("libx264")
    .arg(&output_path)
    .output()?;

// BAD: Command injection vulnerability
Command::new("sh")
    .arg("-c")
    .arg(format!("ffmpeg -i {}", user_input))  // NEVER DO THIS
    .output()?;
```

### Frontend (Svelte)

- Use stores for shared state
- Keep IPC calls in separate service modules
- Update UI optimistically, rollback on error
- Emit events for long-running operations

### Timeline Engine

- Never modify source files (non-destructive editing)
- Store all edits in Edit Decision List (EDL)
- Serialize projects to JSON
- Support unlimited undo/redo (future feature)

## Additional Resources

- [Tauri Documentation](https://tauri.app/v2/)
- [Svelte Documentation](https://svelte.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)

## Getting Help

- Check existing [documentation](docs/)
- Search [existing issues](https://github.com/caseymanos/Clipforge/issues)
- Start a [discussion](https://github.com/caseymanos/Clipforge/discussions)
- Read the [technical architecture](docs/architecture.md)

## License

By contributing to ClipForge, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing to ClipForge!
