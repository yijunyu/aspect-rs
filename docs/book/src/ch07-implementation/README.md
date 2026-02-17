# Implementation Details

Technical deep-dive into aspect-rs internals for contributors.

## Topics Covered

1. **Macro Code Generation** - How `#[aspect(...)]` works
2. **Pointcut Matching** - Pattern matching algorithm (Phase 2-3)
3. **MIR Extraction** - Extracting MIR for automatic weaving (Phase 3)
4. **Code Weaving** - Inserting aspect code at compile time
5. **Performance Optimizations** - Achieving <10ns overhead

This chapter is for contributors and those curious about implementation details.

See [Macro Code Generation](macros.md).
