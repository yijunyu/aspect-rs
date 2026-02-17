# Publishing aspect-rs to crates.io

This guide explains how to publish the aspect-rs crates to crates.io.

## Publishable Crates

The following crates can be published to crates.io:

1. **aspect-core** - Core traits and types (no dependencies)
2. **aspect-macros** - Procedural macros (depends on aspect-core)
3. **aspect-runtime** - Runtime support (depends on aspect-core)
4. **aspect-std** - Standard aspects library (depends on all above)

## Non-Publishable Crates

The following crates are marked with `publish = false` and will NOT be published:

- **aspect-examples** - Example code (not a library)
- **aspect-driver** - Requires nightly Rust + rustc-dev components
- **aspect-rustc-driver** - Requires nightly Rust + rustc-dev components
- **cargo-aspect** - Not yet ready for publication

## Publishing Order

**IMPORTANT**: You must publish the crates in this exact order due to dependencies:

### 1. Publish aspect-core first

```bash
cd aspect-core
cargo publish
cd ..
```

Wait for the crate to be available on crates.io (usually takes 1-2 minutes).

### 2. Publish aspect-macros and aspect-runtime

These can be published in parallel after aspect-core is available:

```bash
# Terminal 1
cd aspect-macros
cargo publish
cd ..

# Terminal 2 (or after macros completes)
cd aspect-runtime
cargo publish
cd ..
```

Wait for both to be available on crates.io.

### 3. Publish aspect-std

After all dependencies are available:

```bash
cd aspect-std
cargo publish
cd ..
```

## Verification Before Publishing

Before publishing, verify each crate:

```bash
# Test that the crate can be packaged and built
cargo publish --dry-run -p aspect-core
cargo publish --dry-run -p aspect-macros
cargo publish --dry-run -p aspect-runtime
cargo publish --dry-run -p aspect-std
```

All should complete successfully with "aborting upload due to dry run".

## Checklist

Before publishing, ensure:

- [ ] All tests pass: `cargo test --workspace`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Examples compile: `cargo build --workspace --examples`
- [ ] Version numbers updated in Cargo.toml (if not first release)
- [ ] CHANGELOG.md updated
- [ ] Git committed with clean working directory
- [ ] README.md has correct crates.io badges
- [ ] All publishable crates have proper metadata (description, keywords, categories)

## Post-Publishing

After publishing:

1. Verify crates appear on crates.io:
   - https://crates.io/crates/aspect-core
   - https://crates.io/crates/aspect-macros
   - https://crates.io/crates/aspect-runtime
   - https://crates.io/crates/aspect-std

2. Test installation from crates.io:
   ```bash
   cargo new test-aspect-rs
   cd test-aspect-rs
   cargo add aspect-core aspect-macros aspect-std
   cargo build
   ```

3. Update documentation links in README.md if needed

4. Tag the release in git:
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

5. Create GitHub release with changelog

## Troubleshooting

### "no matching package found" Error

If you get errors like "no matching package named `aspect-core` found":

- Make sure you published aspect-core first
- Wait 1-2 minutes for crates.io index to update
- Run `cargo update` to refresh the index

### "package.publish must be set to true" Error

This is expected for aspect-examples, aspect-driver, aspect-rustc-driver, and cargo-aspect.
These crates are intentionally not publishable.

### Version Conflicts

If you need to publish a new version:

1. Update version in workspace `Cargo.toml`
2. Commit the change
3. Follow the publishing order again

## Contact

For issues with publishing, contact the maintainers or open an issue at:
https://github.com/yijunyu/aspect-rs/issues
