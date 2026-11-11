//! A process-local byte budget for auxiliary sampled-pack files on the spool.
//!
//! Reservations are acquired before a temporary file is created and live
//! exactly as long as the file owner. This bounds sampled auxiliary files
//! across concurrent index loads within one process; it is not a filesystem
//! quota and does not account for push-ingest or mirror temporary files.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// A shared byte budget.
#[derive(Debug)]
pub struct DiskBudget {
    limit: u64,
    used: AtomicU64,
}

impl DiskBudget {
    /// Creates a budget with a maximum number of reserved bytes.
    pub fn new(limit: u64) -> Self {
        Self { limit, used: AtomicU64::new(0) }
    }

    /// Reserves bytes, returning None if the limit would be exceeded.
    ///
    /// A zero-byte reservation is valid. The compare-and-swap loop makes
    /// admission atomic across concurrent sampled-index and chain-map builds.
    pub fn try_reserve(self: &Arc<Self>, bytes: u64) -> Option<DiskReservation> {
        let mut current = self.used.load(Ordering::Acquire);
        loop {
            let next = current.checked_add(bytes)?;
            if next > self.limit {
                return None;
            }
            match self.used.compare_exchange_weak(
                current, next, Ordering::AcqRel, Ordering::Acquire,
            ) {
                Ok(_) => return Some(DiskReservation { budget: Arc::clone(self), bytes }),
                Err(observed) => current = observed,
            }
        }
    }

    /// Currently reserved bytes.
    pub fn used_bytes(&self) -> u64 {
        self.used.load(Ordering::Acquire)
    }

    /// Maximum reservable bytes.
    pub fn limit_bytes(&self) -> u64 {
        self.limit
    }
}

/// A reservation released when its owning temporary file is dropped.
#[derive(Debug)]
pub struct DiskReservation {
    budget: Arc<DiskBudget>,
    bytes: u64,
}

impl Drop for DiskReservation {
    fn drop(&mut self) {
        let previous = self.budget.used.fetch_sub(self.bytes, Ordering::AcqRel);
        debug_assert!(previous >= self.bytes, "disk reservation underflow");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::thread;

    #[test]
    fn reservations_never_exceed_the_limit_under_concurrency() {
        let budget = Arc::new(DiskBudget::new(100));
        let barrier = Arc::new(Barrier::new(21));
        let workers: Vec<_> = (0..20).map(|_| {
            let budget = Arc::clone(&budget);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                budget.try_reserve(10)
            })
        }).collect();
        barrier.wait();
        let reservations: Vec<_> = workers.into_iter()
            .filter_map(|worker| worker.join().expect("worker did not panic"))
            .collect();
        assert_eq!(reservations.len(), 10);
        assert_eq!(budget.used_bytes(), 100);
        drop(reservations);
        assert_eq!(budget.used_bytes(), 0);
    }

    #[test]
    fn failed_and_overflowing_reservations_do_not_change_usage() {
        let budget = Arc::new(DiskBudget::new(u64::MAX));
        let reservation = budget.try_reserve(u64::MAX).unwrap();
        assert!(budget.try_reserve(1).is_none());
        assert_eq!(budget.used_bytes(), u64::MAX);
        drop(reservation);
        assert_eq!(budget.used_bytes(), 0);
    }

    #[test]
    fn reservation_is_released_on_drop() {
        let budget = Arc::new(DiskBudget::new(32));
        let reservation = budget.try_reserve(24).unwrap();
        assert_eq!(budget.used_bytes(), 24);
        drop(reservation);
        assert_eq!(budget.used_bytes(), 0);
    }
}
