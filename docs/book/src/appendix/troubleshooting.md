# Appendix D: Troubleshooting

## Common Issues

### "cannot find macro `aspect` in this scope"

Add `aspect-macros` to dependencies:
```toml
[dependencies]
aspect-macros = "0.1"
```

### "no method named `before` found"

Implement the `Aspect` trait:
```rust
impl Aspect for YourAspect {
    fn before(&self, ctx: &JoinPoint) {
        // Your code
    }
}
```

### Aspect not being called

1. Verify `#[aspect(...)]` attribute is present
2. Check aspect implements `Aspect` trait
3. Ensure function is actually being called

### Performance issues

1. Profile with `cargo bench`
2. Check for expensive operations in aspects
3. Consider caching in aspects
4. Use `#[inline]` for hot paths

See [Getting Started](../ch03-getting-started/installation.md) for installation issues.
