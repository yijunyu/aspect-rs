# Configuration

Configuring aspects for different environments and use cases.

## Environment-Based Configuration

aspect-rs supports multiple strategies for environment-specific configuration.

### Compile-Time Configuration

Use Rust's conditional compilation for zero-runtime-cost configuration:

```rust
use aspect_std::{LoggingAspect, MetricsAspect};

// Debug builds: verbose logging
#[cfg_attr(debug_assertions, aspect(LoggingAspect::verbose()))]
// Release builds: standard logging
#[cfg_attr(not(debug_assertions), aspect(LoggingAspect::new()))]
fn environment_aware_function() -> Result<(), Error> {
    perform_operation()
}

// Only apply metrics in production
#[cfg_attr(not(debug_assertions), aspect(MetricsAspect::new()))]
fn production_only_metrics() -> Result<(), Error> {
    business_logic()
}

// Development-only detailed tracing
#[cfg_attr(debug_assertions, aspect(TracingAspect::detailed()))]
fn development_tracing() -> Result<(), Error> {
    complex_operation()
}
```

**Benefits:**
- Zero runtime overhead (decided at compile time)
- No conditional checks in production
- Type-safe configuration
- Clear separation of environments

### Runtime Configuration

For dynamic behavior, use runtime configuration:

```rust
use std::sync::Arc;
use aspect_core::prelude::*;

struct ConfigurableAspect {
    config: Arc<AspectConfig>,
}

#[derive(Clone)]
struct AspectConfig {
    enabled: bool,
    log_level: LogLevel,
    threshold_ms: u64,
}

impl Aspect for ConfigurableAspect {
    fn before(&self, ctx: &JoinPoint) {
        if !self.config.enabled {
            return;
        }

        if self.config.log_level >= LogLevel::Debug {
            println!("[{}] Entering: {}", self.config.log_level, ctx.function_name);
        }
    }

    fn after(&self, ctx: &JoinPoint, _result: &dyn Any) {
        if self.config.enabled && self.config.log_level >= LogLevel::Info {
            println!("[{}] Exiting: {}", self.config.log_level, ctx.function_name);
        }
    }
}

// Configuration can be changed at runtime
#[aspect(ConfigurableAspect {
    config: ASPECT_CONFIG.clone(),
})]
fn dynamically_configured() -> Result<(), Error> {
    Ok(())
}
```

### Environment Variables

Load configuration from environment variables:

```rust
use std::env;
use std::time::Duration;

fn create_rate_limiter() -> RateLimitAspect {
    let limit: usize = env::var("RATE_LIMIT")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);

    let window_secs: u64 = env::var("RATE_LIMIT_WINDOW")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);

    RateLimitAspect::new(limit, Duration::from_secs(window_secs))
}

#[aspect(create_rate_limiter())]
fn env_configured_endpoint() -> Result<Response, Error> {
    handle_request()
}
```

### Configuration Files

Load from TOML/JSON configuration:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct AspectSettings {
    logging_enabled: bool,
    timing_threshold_ms: u64,
    metrics_enabled: bool,
    cache_ttl_secs: u64,
}

fn load_settings() -> AspectSettings {
    let config_str = std::fs::read_to_string("config.toml")
        .expect("Failed to read config");
    toml::from_str(&config_str).expect("Failed to parse config")
}

fn create_aspects(settings: &AspectSettings) -> Vec<Box<dyn Aspect>> {
    let mut aspects = Vec::new();

    if settings.logging_enabled {
        aspects.push(Box::new(LoggingAspect::new()));
    }

    aspects.push(Box::new(TimingAspect::with_threshold(
        Duration::from_millis(settings.timing_threshold_ms),
    )));

    if settings.metrics_enabled {
        aspects.push(Box::new(MetricsAspect::new()));
    }

    aspects.push(Box::new(CachingAspect::with_ttl(
        Duration::from_secs(settings.cache_ttl_secs),
    )));

    aspects
}
```

## Feature Flags

Use feature flags for gradual rollouts and A/B testing:

```rust
use std::sync::Arc;

struct FeatureFlags {
    flags: HashMap<String, bool>,
}

impl FeatureFlags {
    fn is_enabled(&self, flag: &str) -> bool {
        *self.flags.get(flag).unwrap_or(&false)
    }
}

static FEATURE_FLAGS: Lazy<Arc<FeatureFlags>> = Lazy::new(|| {
    Arc::new(FeatureFlags {
        flags: load_feature_flags(),
    })
});

struct FeatureGatedAspect {
    feature_name: String,
    inner_aspect: Box<dyn Aspect>,
}

impl Aspect for FeatureGatedAspect {
    fn before(&self, ctx: &JoinPoint) {
        if FEATURE_FLAGS.is_enabled(&self.feature_name) {
            self.inner_aspect.before(ctx);
        }
    }

    fn after(&self, ctx: &JoinPoint, result: &dyn Any) {
        if FEATURE_FLAGS.is_enabled(&self.feature_name) {
            self.inner_aspect.after(ctx, result);
        }
    }
}

#[aspect(FeatureGatedAspect {
    feature_name: "new_caching_system".to_string(),
    inner_aspect: Box::new(CachingAspect::new()),
})]
fn gradually_rolled_out() -> Result<Data, Error> {
    // New caching only applies if feature flag enabled
    fetch_data()
}
```

## Multi-Environment Setup

### Development Configuration

```rust
#[cfg(debug_assertions)]
mod dev_config {
    use super::*;

    pub fn logging() -> LoggingAspect {
        LoggingAspect::verbose()
            .with_timestamps()
            .with_source_location()
            .with_thread_info()
    }

    pub fn timing() -> TimingAspect {
        TimingAspect::new() // Log all timings
    }

    pub fn caching() -> CachingAspect {
        CachingAspect::with_ttl(Duration::from_secs(10)) // Short TTL for dev
    }
}

#[cfg(debug_assertions)]
use dev_config as config;
```

### Production Configuration

```rust
#[cfg(not(debug_assertions))]
mod prod_config {
    use super::*;

    pub fn logging() -> LoggingAspect {
        LoggingAspect::new()
            .with_level(Level::Info) // Less verbose
            .with_structured_output() // JSON for log aggregation
    }

    pub fn timing() -> TimingAspect {
        TimingAspect::with_threshold(Duration::from_millis(100)) // Only warn on slow ops
    }

    pub fn caching() -> CachingAspect {
        CachingAspect::with_ttl(Duration::from_secs(3600)) // 1 hour TTL
            .with_max_size(10000) // Limit memory usage
    }
}

#[cfg(not(debug_assertions))]
use prod_config as config;
```

### Usage

```rust
#[aspect(config::logging())]
#[aspect(config::timing())]
#[aspect(config::caching())]
fn multi_env_function() -> Result<Data, Error> {
    // Automatically uses correct configuration for environment
    fetch_data()
}
```

## Aspect Configuration Patterns

### Builder Pattern

```rust
struct ConfigurableLoggingAspect {
    level: LogLevel,
    include_timestamps: bool,
    include_thread_info: bool,
    output: OutputFormat,
}

impl ConfigurableLoggingAspect {
    fn builder() -> LoggingAspectBuilder {
        LoggingAspectBuilder::default()
    }
}

struct LoggingAspectBuilder {
    level: LogLevel,
    include_timestamps: bool,
    include_thread_info: bool,
    output: OutputFormat,
}

impl LoggingAspectBuilder {
    fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    fn with_timestamps(mut self) -> Self {
        self.include_timestamps = true;
        self
    }

    fn with_thread_info(mut self) -> Self {
        self.include_thread_info = true;
        self
    }

    fn json_output(mut self) -> Self {
        self.output = OutputFormat::Json;
        self
    }

    fn build(self) -> ConfigurableLoggingAspect {
        ConfigurableLoggingAspect {
            level: self.level,
            include_timestamps: self.include_timestamps,
            include_thread_info: self.include_thread_info,
            output: self.output,
        }
    }
}

// Usage
#[aspect(ConfigurableLoggingAspect::builder()
    .level(LogLevel::Debug)
    .with_timestamps()
    .json_output()
    .build()
)]
fn custom_configured_function() -> Result<(), Error> {
    Ok(())
}
```

### Configuration Profiles

```rust
enum Profile {
    Development,
    Staging,
    Production,
}

struct ProfiledAspects {
    profile: Profile,
}

impl ProfiledAspects {
    fn logging(&self) -> LoggingAspect {
        match self.profile {
            Profile::Development => LoggingAspect::verbose(),
            Profile::Staging => LoggingAspect::new(),
            Profile::Production => LoggingAspect::structured(),
        }
    }

    fn rate_limit(&self) -> RateLimitAspect {
        match self.profile {
            Profile::Development => RateLimitAspect::new(10000, Duration::from_secs(60)),
            Profile::Staging => RateLimitAspect::new(1000, Duration::from_secs(60)),
            Profile::Production => RateLimitAspect::new(100, Duration::from_secs(60)),
        }
    }
}

lazy_static! {
    static ref ASPECTS: ProfiledAspects = ProfiledAspects {
        profile: detect_profile(),
    };
}

#[aspect(ASPECTS.logging())]
#[aspect(ASPECTS.rate_limit())]
fn profile_aware_endpoint() -> Result<Response, Error> {
    handle_request()
}
```

## Best Practices

### 1. Centralize Configuration

Create a single configuration module:

```rust
// config/aspects.rs
pub mod aspects {
    use super::*;

    pub fn web_handler() -> Vec<Box<dyn Aspect>> {
        vec![
            Box::new(auth()),
            Box::new(rate_limit()),
            Box::new(logging()),
            Box::new(timing()),
        ]
    }

    pub fn background_job() -> Vec<Box<dyn Aspect>> {
        vec![
            Box::new(logging()),
            Box::new(timing()),
            Box::new(retry()),
        ]
    }

    fn auth() -> AuthorizationAspect {
        AuthorizationAspect::require_role("user", get_roles)
    }

    fn rate_limit() -> RateLimitAspect {
        let limit = env::var("RATE_LIMIT").unwrap_or("100".into()).parse().unwrap();
        RateLimitAspect::new(limit, Duration::from_secs(60))
    }

    fn logging() -> LoggingAspect {
        if cfg!(debug_assertions) {
            LoggingAspect::verbose()
        } else {
            LoggingAspect::structured()
        }
    }

    fn timing() -> TimingAspect {
        TimingAspect::with_threshold(Duration::from_millis(100))
    }

    fn retry() -> RetryAspect {
        RetryAspect::new(3, Duration::from_millis(100))
    }
}
```

### 2. Use Type-Safe Configuration

```rust
#[derive(Clone, Debug)]
struct TypeSafeConfig {
    rate_limit: RateLimitConfig,
    caching: CachingConfig,
    timing: TimingConfig,
}

#[derive(Clone, Debug)]
struct RateLimitConfig {
    requests_per_window: usize,
    window: Duration,
}

#[derive(Clone, Debug)]
struct CachingConfig {
    ttl: Duration,
    max_size: usize,
}

#[derive(Clone, Debug)]
struct TimingConfig {
    warn_threshold: Duration,
}
```

### 3. Validate Configuration

```rust
impl TypeSafeConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.rate_limit.requests_per_window == 0 {
            return Err(ConfigError::InvalidRateLimit);
        }

        if self.caching.max_size == 0 {
            return Err(ConfigError::InvalidCacheSize);
        }

        Ok(())
    }
}
```

### 4. Document Configuration Options

```rust
/// Configuration for rate limiting
///
/// # Fields
/// * `requests_per_window` - Maximum requests allowed (must be > 0)
/// * `window` - Time window duration (recommended: 60 seconds)
///
/// # Examples
/// ```
/// let config = RateLimitConfig {
///     requests_per_window: 100,
///     window: Duration::from_secs(60),
/// };
/// ```
#[derive(Clone, Debug)]
struct RateLimitConfig {
    /// Maximum number of requests allowed in the time window
    requests_per_window: usize,
    /// Duration of the rate limiting window
    window: Duration,
}
```

## Summary

Configuration strategies covered:

1. **Compile-Time**: Zero overhead with `cfg` attributes
2. **Runtime**: Dynamic configuration changes
3. **Environment Variables**: 12-factor app compliance
4. **Feature Flags**: Gradual rollouts
5. **Multi-Environment**: Dev/staging/prod profiles

**Key Takeaways:**
- Use compile-time configuration for performance-critical code
- Runtime configuration enables dynamic behavior
- Feature flags enable safe gradual rollouts
- Centralize configuration for maintainability
- Always validate configuration

**Next Steps:**
- See [Testing](testing.md) for testing configured aspects
- Review [Production Patterns](production.md) for real-world usage
- Check [Advanced Patterns](advanced.md) for composition
