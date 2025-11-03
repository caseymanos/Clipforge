# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

The ClipForge team takes security bugs seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please report security vulnerabilities using one of these methods:

1. **GitHub Security Advisories** (Preferred)
   - Navigate to the [Security tab](https://github.com/caseymanos/Clipforge/security)
   - Click "Report a vulnerability"
   - Fill out the advisory form with details

2. **Private Issue**
   - Email the maintainers directly via GitHub
   - Include "SECURITY" in the subject line

### What to Include

Please provide as much information as possible:

- Type of vulnerability (e.g., command injection, XSS, path traversal)
- Full paths of source files related to the vulnerability
- Location of the affected source code (tag/branch/commit)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability
- Suggested fix (if you have one)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: 1-7 days
  - High: 7-30 days
  - Medium: 30-90 days
  - Low: Best effort

### What to Expect

1. We will acknowledge receipt of your vulnerability report
2. We will confirm the vulnerability and determine its severity
3. We will develop and test a fix
4. We will release a patched version
5. We will publicly disclose the vulnerability (with credit to you, if desired)

## Security Best Practices for Contributors

When contributing to ClipForge, please follow these security guidelines:

### 1. Input Validation

Always validate and sanitize user input:

```rust
// Good: Validate file paths
fn validate_path(path: &Path) -> Result<(), Error> {
    if !path.exists() {
        return Err(Error::FileNotFound);
    }
    // Additional validation...
    Ok(())
}
```

### 2. Command Injection Prevention

Never use shell string interpolation with user input:

```rust
// BAD: Command injection vulnerability
Command::new("sh")
    .arg("-c")
    .arg(format!("ffmpeg -i {}", user_input))
    .output()?;

// GOOD: Safe argument passing
Command::new("ffmpeg")
    .arg("-i")
    .arg(user_input)
    .output()?;
```

### 3. Path Traversal Prevention

Validate file paths to prevent directory traversal attacks:

```rust
// Ensure paths are within allowed directories
fn is_safe_path(path: &Path) -> bool {
    let allowed_dirs = vec![
        dirs::home_dir(),
        dirs::document_dir(),
        // ...
    ];

    path.canonicalize()
        .ok()
        .and_then(|p| {
            allowed_dirs.iter().any(|dir| {
                dir.as_ref()
                    .and_then(|d| Some(p.starts_with(d)))
                    .unwrap_or(false)
            }).then(|| ())
        })
        .is_some()
}
```

### 4. Dependency Security

- Keep dependencies up to date
- Review dependency changes in PRs
- Use `cargo audit` to check for known vulnerabilities
- Avoid dependencies with known security issues

### 5. Sensitive Data

- Never commit API keys, passwords, or secrets
- Use environment variables for sensitive configuration
- Add `.env` files to `.gitignore`
- Sanitize logs to prevent leaking sensitive data

### 6. Error Handling

- Don't expose internal details in error messages
- Log detailed errors server-side
- Return generic error messages to users

```rust
// BAD: Exposes internal paths
Err(format!("Failed to read {}", internal_path))

// GOOD: Generic user message, detailed logging
log::error!("Failed to read {}", internal_path);
Err("Failed to read file".to_string())
```

## Known Security Considerations

### FFmpeg

ClipForge uses FFmpeg for video processing. Be aware:

- Keep FFmpeg updated to the latest version
- FFmpeg can execute arbitrary code via certain input files
- Only process trusted video files
- We pass arguments safely (not via shell)

### File System Access

- ClipForge requires file system access by design
- Tauri's file system API is used with proper allowlisting
- Users must grant explicit permission for file access

### Screen Recording (macOS)

- Requires screen recording permission
- Permission is requested at runtime
- Users can revoke permission in System Preferences

## Security Tools

We use the following tools to maintain security:

- `cargo audit` - Check for known vulnerabilities in dependencies
- `cargo clippy` - Linting for common security issues
- GitHub Dependabot - Automated dependency updates
- GitHub CodeQL - Static analysis for security issues

## Bug Bounty Program

We currently do not have a bug bounty program. However, we deeply appreciate security researchers who responsibly disclose vulnerabilities. We will publicly acknowledge your contribution (if desired) when we release a fix.

## Questions?

If you have questions about this security policy, please open a discussion in the [GitHub Discussions](https://github.com/caseymanos/Clipforge/discussions) section.

Thank you for helping keep ClipForge and its users safe!
