//! Rate limiting aspect using token bucket algorithm.

use aspect_core::{Aspect, AspectError, ProceedingJoinPoint};
use parking_lot::Mutex;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rate limiting aspect with token bucket algorithm.
///
/// Limits the rate at which functions can be called, useful for API throttling,
/// resource protection, and preventing abuse.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_std::RateLimitAspect;
/// use aspect_macros::aspect;
/// use std::time::Duration;
///
/// // Allow 10 calls per second
/// let limiter = RateLimitAspect::new(10, Duration::from_secs(1));
///
/// #[aspect(limiter.clone())]
/// fn api_call(data: String) -> Result<(), String> {
///     // This function is rate-limited
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct RateLimitAspect {
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
    per_function: bool,
    function_states: HashMap<String, FunctionRateLimit>,
}

struct FunctionRateLimit {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimitAspect {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed
    /// * `window` - Time window for the limit
    ///
    /// # Example
    /// ```rust,ignore
    /// // 100 requests per minute
    /// let limiter = RateLimitAspect::new(100, Duration::from_secs(60));
    /// ```
    pub fn new(max_requests: u64, window: Duration) -> Self {
        let refill_rate = max_requests as f64 / window.as_secs_f64();

        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                tokens: max_requests as f64,
                max_tokens: max_requests as f64,
                refill_rate,
                last_refill: Instant::now(),
                per_function: false,
                function_states: HashMap::new(),
            })),
        }
    }

    /// Enable per-function rate limiting.
    ///
    /// When enabled, each function gets its own token bucket.
    pub fn per_function(self) -> Self {
        self.state.lock().per_function = true;
        self
    }

    /// Check if a request is allowed (consumes a token if available).
    fn try_acquire(&self, function_name: Option<&str>) -> bool {
        let mut state = self.state.lock();
        let now = Instant::now();

        if state.per_function {
            if let Some(name) = function_name {
                // Per-function rate limiting
                // Capture values before borrowing function_states
                let max_tokens = state.max_tokens;
                let refill_rate = state.refill_rate;

                let func_state = state
                    .function_states
                    .entry(name.to_string())
                    .or_insert_with(|| FunctionRateLimit {
                        tokens: max_tokens,
                        last_refill: now,
                    });

                // Refill tokens
                let elapsed = now.duration_since(func_state.last_refill).as_secs_f64();
                func_state.tokens = (func_state.tokens + elapsed * refill_rate).min(max_tokens);
                func_state.last_refill = now;

                if func_state.tokens >= 1.0 {
                    func_state.tokens -= 1.0;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            // Global rate limiting
            let elapsed = now.duration_since(state.last_refill).as_secs_f64();
            state.tokens = (state.tokens + elapsed * state.refill_rate).min(state.max_tokens);
            state.last_refill = now;

            if state.tokens >= 1.0 {
                state.tokens -= 1.0;
                true
            } else {
                false
            }
        }
    }

    /// Get current token count.
    pub fn available_tokens(&self) -> f64 {
        let mut state = self.state.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill).as_secs_f64();
        state.tokens = (state.tokens + elapsed * state.refill_rate).min(state.max_tokens);
        state.last_refill = now;
        state.tokens
    }
}

impl Aspect for RateLimitAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        let function_name = pjp.context().function_name;

        if self.try_acquire(Some(function_name)) {
            pjp.proceed()
        } else {
            Err(AspectError::execution(format!(
                "Rate limit exceeded for {}",
                function_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_basic() {
        let limiter = RateLimitAspect::new(5, Duration::from_secs(1));

        // Should allow 5 calls
        for _ in 0..5 {
            assert!(limiter.try_acquire(Some("test")));
        }

        // 6th call should be denied
        assert!(!limiter.try_acquire(Some("test")));
    }

    #[test]
    fn test_rate_limit_refill() {
        let limiter = RateLimitAspect::new(2, Duration::from_millis(100));

        // Consume both tokens
        assert!(limiter.try_acquire(Some("test")));
        assert!(limiter.try_acquire(Some("test")));
        assert!(!limiter.try_acquire(Some("test")));

        // Wait for refill
        std::thread::sleep(Duration::from_millis(150));

        // Should have at least 1 token now
        assert!(limiter.try_acquire(Some("test")));
    }

    #[test]
    fn test_per_function_limiting() {
        let limiter = RateLimitAspect::new(2, Duration::from_secs(1)).per_function();

        // Function A consumes its quota
        assert!(limiter.try_acquire(Some("func_a")));
        assert!(limiter.try_acquire(Some("func_a")));
        assert!(!limiter.try_acquire(Some("func_a")));

        // Function B should still have its quota
        assert!(limiter.try_acquire(Some("func_b")));
        assert!(limiter.try_acquire(Some("func_b")));
        assert!(!limiter.try_acquire(Some("func_b")));
    }

    #[test]
    fn test_available_tokens() {
        let limiter = RateLimitAspect::new(10, Duration::from_secs(1));

        let initial = limiter.available_tokens();
        assert!((initial - 10.0).abs() < 0.01);

        limiter.try_acquire(Some("test"));

        let after = limiter.available_tokens();
        assert!((after - 9.0).abs() < 0.01);
    }
}
