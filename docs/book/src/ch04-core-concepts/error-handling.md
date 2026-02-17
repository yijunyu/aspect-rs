# Error Handling

aspect-rs supports both `Result` and panic-based error handling.

## With Result

```rust
#[aspect(LoggingAspect::new())]
fn may_fail(x: i32) -> Result<i32, Error> {
    if x < 0 {
        Err(Error::NegativeValue)
    } else {
        Ok(x * 2)
    }
}
```

The `after_throwing` advice is called when `Err` is returned.

## With Panics

```rust
impl Aspect for PanicHandler {
    fn after_throwing(&self, ctx: &JoinPoint, error: &dyn Any) {
        if let Some(msg) = error.downcast_ref::<&str>() {
            eprintln!("Panic in {}: {}", ctx.function_name, msg);
        }
    }
}
```

See [The Aspect Trait](aspect-trait.md) for more on `after_throwing`.
