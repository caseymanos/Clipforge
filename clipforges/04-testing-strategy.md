# Testing Strategy

## Testing Pyramid

### Unit Tests (40%)
- Rust backend functions
- Data structure operations
- FFmpeg command builders
- Timeline logic

### Integration Tests (40%)
- Full import-to-export pipeline
- Cross-module interactions
- Database operations
- FFmpeg execution

### Manual Tests (20%)
- UI/UX flows
- Platform-specific features
- Performance benchmarks
- Real-world scenarios

## Rust Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timeline_operations() {
        // Test logic
    }
    
    #[tokio::test]
    async fn test_async_operations() {
        // Test async logic
    }
}
```

Run tests:
```bash
cargo test
cargo test --package clipforge --lib
```

## Manual Testing Checklist

### Import
- [ ] MP4, MOV, WebM, AVI files
- [ ] Various codecs
- [ ] Large files (1GB+)
- [ ] Corrupted files

### Timeline
- [ ] 20+ clips without lag
- [ ] Drag and drop
- [ ] Trim/split operations
- [ ] Undo/redo

### Export
- [ ] 1080p export completes
- [ ] Progress accurate
- [ ] Cancel works
- [ ] Output plays correctly

### Recording
- [ ] Screen capture
- [ ] Webcam capture
- [ ] Audio capture
- [ ] Permissions

## Performance Benchmarks

```bash
# Timeline rendering
- Target: 30 FPS with 20 clips
- Measure: Chrome DevTools FPS counter

# Export speed
- Target: 1x real-time for 1080p
- Measure: FFmpeg progress output

# Memory usage
- Target: <300MB during editing
- Measure: Activity Monitor / Task Manager
```

## CI/CD Pipeline

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test
      - run: cargo clippy
```
