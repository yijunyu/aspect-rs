# Background

This chapter provides essential background on Aspect-Oriented Programming concepts and explains what aspect-rs can do for your Rust projects.

## What You'll Learn

By the end of this chapter, you'll understand:

- What crosscutting concerns are and why they're problematic
- Core AOP terminology (aspects, join points, pointcuts, advice, weaving)
- The capabilities and limitations of aspect-rs
- How aspect-rs fits into Rust's programming model

## Chapter Outline

1. **[Crosscutting Concerns Explained](crosscutting.md)** - Deep dive into the problem AOP solves
2. **[AOP Terminology](terminology.md)** - Learn the vocabulary of aspect-oriented programming
3. **[What aspect-rs Can Do](capabilities.md)** - Concrete capabilities and use cases

## Prerequisites

This chapter assumes you're familiar with:
- Basic Rust (functions, traits, ownership)
- Common software patterns (decorators, middleware)
- Why separation of concerns matters

If you're new to Rust, consider reading [The Rust Book](https://doc.rust-lang.org/book/) first.

## Quick Refresher: Separation of Concerns

**Good software separates different responsibilities**:

```rust
// ✅ Good: Focused on one thing
fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

fn send_email(to: &str, subject: &str, body: &str) -> Result<(), Error> {
    smtp::send(to, subject, body)
}
```

```rust
// ❌ Bad: Mixed responsibilities
fn send_validated_logged_metered_email(
    to: &str,
    subject: &str,
    body: &str
) -> Result<(), Error> {
    // Validation logic
    if !to.contains('@') { return Err(...) }

    // Logging logic
    log::info!("Sending email to {}", to);

    // Timing logic
    let start = Instant::now();

    // Business logic
    let result = smtp::send(to, subject, body);

    // Metrics logic
    metrics::record("email_sent", start.elapsed());

    result
}
```

But what about concerns that apply **everywhere**? That's where AOP comes in.

Let's explore this in [Crosscutting Concerns Explained](crosscutting.md).
