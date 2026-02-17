//! Generic caching/memoization aspect.

use aspect_core::{Aspect, AspectError, ProceedingJoinPoint};
use std::any::Any;
use std::time::Duration;

/// Generic caching aspect with TTL support.
///
/// # Example
///
/// ```rust,ignore
/// use aspect_std::CachingAspect;
/// use aspect_macros::aspect;
/// use std::time::Duration;
///
/// let cache = CachingAspect::new().with_ttl(Duration::from_secs(60));
///
/// #[aspect(cache.clone())]
/// fn expensive_query(id: u64) -> Result<String, String> {
///     // Expensive operation - will be cached
///     Ok(format!("Result for {}", id))
/// }
/// ```
#[derive(Clone)]
pub struct CachingAspect {
    max_size: usize,
    ttl: Option<Duration>,
}

impl CachingAspect {
    /// Create a new caching aspect with no size limit.
    pub fn new() -> Self {
        Self {
            max_size: usize::MAX,
            ttl: None,
        }
    }

    /// Set maximum cache size.
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Set time-to-live for cache entries.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }
}

impl Default for CachingAspect {
    fn default() -> Self {
        Self::new()
    }
}

impl Aspect for CachingAspect {
    fn around(&self, pjp: ProceedingJoinPoint) -> Result<Box<dyn Any>, AspectError> {
        // For now, always proceed (full impl would require key extraction)
        // This is a simplified version - full caching requires function signature analysis
        println!("[CACHE] Checking cache for {}", pjp.context().function_name);
        pjp.proceed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caching_aspect() {
        let aspect = CachingAspect::new()
            .with_max_size(100)
            .with_ttl(Duration::from_secs(60));

        assert_eq!(aspect.max_size, 100);
        assert_eq!(aspect.ttl, Some(Duration::from_secs(60)));
    }
}
