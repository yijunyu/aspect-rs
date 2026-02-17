# Architecture

System design and component organization of aspect-rs.

## Crate Structure

```
aspect-rs/
├── aspect-core/      # Core traits (zero dependencies)
├── aspect-macros/    # Procedural macros  
├── aspect-runtime/   # Global aspect registry
├── aspect-std/       # 8 standard aspects
├── aspect-pointcut/  # Pointcut expression parsing
├── aspect-weaver/    # Code weaving logic
└── aspect-rustc-driver/ # Phase 3 automatic weaving
```

## Design Principles

1. **Zero Runtime Overhead** - Compile-time weaving
2. **Type Safety** - Full Rust type checking
3. **Thread Safety** - All aspects `Send + Sync`
4. **Composability** - Aspects can be combined
5. **Extensibility** - Easy to create custom aspects

See [Crate Organization](crates.md) for details.
