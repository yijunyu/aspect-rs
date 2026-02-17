//! Circuit breaker aspect for fault tolerance.

use aspect_core::{Aspect, AspectError, ProceedingJoinPoint};
use parking_lot::Mutex;
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Circuit breaker states following the classic pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through normally.
    Closed,
    /// Circuit is open, requests fail fast without calling the function.
    Open { until: Instant },
    /// Circuit is half-open, testing if the service has recovered.
    HalfOpen,
}

/// Circuit breaker aspect for preventing cascading failures.
///
/// Implements the circuit breaker pattern to protect services from
/// repeated failures and allow time for recovery.
///
/// # States
/// - **Closed**: Normal operation, failures are tracked
/// - **Open**: Fast-fail mode after threshold is exceeded
/// - **Half-Open**: Testing recovery with limited requests
///
/// # Example
///
/// ```rust,ignore
/// use aspect_std::CircuitBreakerAspect;
/// use aspect_macros::aspect;
/// use std::time::Duration;
///
/// // Open after 5 failures, timeout after 30 seconds
/// let breaker = CircuitBreakerAspect::new(5, Duration::from_secs(30));
///
/// #[aspect(breaker.clone())]
/// fn call_external_service() -> Result<String, String> {
///     // This call is protected by the circuit breaker
///     Ok("success".to_string())
/// }
/// ```
#[derive(Clone)]
pub struct CircuitBreakerAspect {
    state: Arc<Mutex<CircuitBreakerState>>,
}

struct CircuitBreakerState {
    circuit_state: CircuitState,
    failure_count: usize,
    success_count: usize,
    failure_threshold: usize,
    timeout: Duration,
    half_open_max_requests: usize,
}

impl CircuitBreakerAspect {
    /// Create a new circuit breaker.
    ///
    /// # Arguments
    /// * `failure_threshold` - Number of failures before opening circuit
    /// * `timeout` - How long to wait before attempting recovery
    ///
    /// # Example
    /// ```rust
    /// use aspect_std::CircuitBreakerAspect;
    /// use std::time::Duration;
    ///
    /// let breaker = CircuitBreakerAspect::new(5, Duration::from_secs(30));
    /// ```
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState {
                circuit_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                failure_threshold,
                timeout,
                half_open_max_requests: 1,
            })),
        }
    }

    /// Set the maximum number of requests to allow in half-open state.
    pub fn with_half_open_requests(self, max_requests: usize) -> Self {
        self.state.lock().half_open_max_requests = max_requests;
        self
    }

    /// Get the current circuit state.
    pub fn state(&self) -> CircuitState {
        self.state.lock().circuit_state.clone()
    }

    /// Manually reset the circuit breaker to closed state.
    pub fn reset(&self) {
        let mut state = self.state.lock();
        state.circuit_state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
    }

    /// Record a successful call.
    fn record_success(&self) {
        let mut state = self.state.lock();

        match state.circuit_state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                // Transition back to closed after successful test
                if state.success_count >= state.half_open_max_requests {
                    state.circuit_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::Open { .. } => {
                // Shouldn't happen, but reset counts
                state.failure_count = 0;
                state.success_count = 0;
            }
        }
    }

    /// Record a failed call.
    fn record_failure(&self) {
        let mut state = self.state.lock();

        match state.circuit_state {
            CircuitState::HalfOpen => {
                // Failure in half-open state immediately reopens circuit
                state.circuit_state = CircuitState::Open {
                    until: Instant::now() + state.timeout,
                };
                state.success_count = 0;
            }
            CircuitState::Closed => {
                state.failure_count += 1;
                if state.failure_count >= state.failure_threshold {
                    // Open the circuit
                    state.circuit_state = CircuitState::Open {
                        until: Instant::now() + state.timeout,
                    };
                }
            }
            CircuitState::Open { .. } => {
                // Already open, nothing to do
            }
        }
    }

    /// Check if a request should be allowed through.
    fn should_allow_request(&self) -> Result<(), AspectError> {
        let mut state = self.state.lock();

        match state.circuit_state {
            CircuitState::Closed => Ok(()),
            CircuitState::HalfOpen => Ok(()),
            CircuitState::Open { until } => {
                if Instant::now() >= until {
                    // Timeout expired, transition to half-open
                    state.circuit_state = CircuitState::HalfOpen;
                    state.success_count = 0;
                    Ok(())
                } else {
                    Err(AspectError::execution(
                        "Circuit breaker is OPEN - failing fast",
                    ))
                }
            }
        }
    }
}

impl Aspect for CircuitBreakerAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        // Check if request should be allowed
        self.should_allow_request()?;

        // Attempt the call
        match pjp.proceed() {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_initially() {
        let breaker = CircuitBreakerAspect::new(3, Duration::from_secs(1));
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_opens_after_threshold() {
        let breaker = CircuitBreakerAspect::new(3, Duration::from_secs(60));

        // Record failures
        for _ in 0..3 {
            breaker.record_failure();
        }

        // Circuit should be open
        match breaker.state() {
            CircuitState::Open { .. } => (),
            _ => panic!("Circuit should be open"),
        }
    }

    #[test]
    fn test_circuit_rejects_when_open() {
        let breaker = CircuitBreakerAspect::new(1, Duration::from_secs(60));

        breaker.record_failure();

        // Should reject requests
        assert!(breaker.should_allow_request().is_err());
    }

    #[test]
    fn test_circuit_transitions_to_half_open() {
        let breaker = CircuitBreakerAspect::new(1, Duration::from_millis(100));

        breaker.record_failure();
        assert!(matches!(breaker.state(), CircuitState::Open { .. }));

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should transition to half-open when checked
        assert!(breaker.should_allow_request().is_ok());
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_closes_after_success() {
        let breaker = CircuitBreakerAspect::new(1, Duration::from_millis(50));

        // Open the circuit
        breaker.record_failure();
        std::thread::sleep(Duration::from_millis(60));

        // Allow request (transitions to half-open)
        breaker.should_allow_request().unwrap();

        // Success should close the circuit
        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_reset() {
        let breaker = CircuitBreakerAspect::new(2, Duration::from_secs(60));

        breaker.record_failure();
        breaker.record_failure();

        assert!(matches!(breaker.state(), CircuitState::Open { .. }));

        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
}
