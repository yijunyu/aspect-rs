# Advice Types

Comparison of the four advice types.

| Advice | When | Use Cases |
|--------|------|-----------|
| `before` | Before function | Logging, validation, authz |
| `after` | After success | Logging, caching, metrics |
| `after_throwing` | On error | Error logging, rollback |
| `around` | Wraps execution | Timing, caching, transactions |

See [Core Concepts](README.md) for details.
