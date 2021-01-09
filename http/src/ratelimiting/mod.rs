pub mod headers;
pub mod in_memory;
pub mod ticket;

pub use self::in_memory::InMemoryRatelimiter;

use self::ticket::TicketReceiver;
use crate::routing::Path;
use std::{
    error::Error,
    fmt::Debug,
    future::Future,
    pin::Pin,
    time::{UNIX_EPOCH, Duration, Instant, SystemTime},
};

pub struct Bucket {
    limit: u64,
    remaining: u64,
    reset_after: u64,
    started_at: u64,
}

impl Bucket {
    /// Total number of tickets allotted in a cycle.
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Number of tickets remaining.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Number of milliseconds after the [`started_at`] time the bucket will
    /// refresh.
    pub fn reset_after(&self) -> u64 {
        self.reset_after
    }

    /// When the bucket's ratelimit refresh countdown started in milliseconds
    /// from the Unix epoch.
    pub fn started_at(&self) -> u64 {
        self.started_at
    }

    /// How long until the bucket will refresh.
    ///
    /// May return `None` when the system clock is before the Unix epoch or
    /// the bucket has already refreshed.
    pub fn time_remaining(&self) -> Option<Duration> {
        let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_millis();
        let reset_after = self.reset_after();
        let elapsed = started_at.elapsed();

        if elapsed > Duration::from_millis(reset_after) {
            return None;
        }

        Some(Duration::from_millis(reset_after) - elapsed)
    }
}

pub trait Ratelimiter: Debug + Send + Sync {
    /// Retrieve the basic information of the bucket for a given path.
    fn bucket(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<Option<Bucket>, Box<dyn Error + Send + Sync>>> + Send + 'static>>;

    /// Whether the ratelimiter is currently globally locked.
    fn globally_locked(&self) -> Pin<Box<dyn Future<Output = Result<bool, Box<dyn Error + Send + Sync>>> + Send + 'static>>;

    /// Determine if the ratelimiter has a bucket for the given path.
    fn has(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<bool, Box<dyn Error + Send + Sync>>> + Send + 'static>>;

    /// Retrieve a ticket to know when to send a request.
    ///
    /// The provided future will be ready when a ticket in the bucket is
    /// available. Tickets are ready in order of retrieval.
    fn ticket(&self, path: Path) -> Pin<Box<dyn Future<Output = Result<TicketReceiver, Box<dyn Error + Send + Sync>>> + Send + 'static>>;
}
