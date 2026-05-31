use std::time::Duration;

/// Capped exponential backoff for background loops that otherwise retry on a
/// fixed tick. Each consecutive failure doubles the extra delay (starting at
/// the loop's base interval) up to [`LoopBackoff::MAX_DELAY`]; a success
/// resets it so steady-state behavior is unchanged. Keeps a persistent
/// Docker/DB outage from turning a ticking loop into a tight log-spam loop.
pub(crate) struct LoopBackoff {
    base: Duration,
    consecutive_failures: u32,
}

impl LoopBackoff {
    const MAX_DELAY: Duration = Duration::from_secs(300);

    pub(crate) fn new(base: Duration) -> Self {
        Self {
            base,
            consecutive_failures: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.consecutive_failures = 0;
    }

    /// Returns the extra delay to wait before the next retry, escalating on
    /// each consecutive failure and capped at [`Self::MAX_DELAY`].
    pub(crate) fn next_delay(&mut self) -> Duration {
        let exponent = self.consecutive_failures.min(16);
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        let factor = 1_u32 << exponent;
        self.base.saturating_mul(factor).min(Self::MAX_DELAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escalates_then_caps_and_resets() {
        let base = Duration::from_secs(30);
        let mut backoff = LoopBackoff::new(base);
        assert_eq!(backoff.next_delay(), base);
        assert_eq!(backoff.next_delay(), base * 2);
        assert_eq!(backoff.next_delay(), base * 4);
        // Eventually saturates at the cap rather than overflowing.
        for _ in 0..32 {
            assert!(backoff.next_delay() <= LoopBackoff::MAX_DELAY);
        }
        backoff.reset();
        assert_eq!(backoff.next_delay(), base);
    }
}
