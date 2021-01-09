mod bucket;

use self::bucket::{BucketQueueTask, Bucket};
use super::{ticket::{self, TicketNotifier, TicketReceiver}, Bucket as InfoBucket, Ratelimiter};
use crate::routing::Path;
use futures_util::{future, lock::Mutex as AsyncMutex};
use std::{
    collections::hash_map::{Entry, HashMap},
    error::Error,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex,
    },
    pin::Pin,
};

/// Global lock. We use a pair to avoid actually locking the mutex every check.
/// This allows futures to only wait on the global lock when a global ratelimit
/// is in place by, in turn, waiting for a guard, and then each immediately
/// dropping it.
#[derive(Debug, Default)]
struct GlobalLockPair(AsyncMutex<()>, AtomicBool);

impl GlobalLockPair {
    pub fn lock(&self) {
        self.1.store(true, Ordering::Release);
    }

    pub fn unlock(&self) {
        self.1.store(false, Ordering::Release);
    }

    pub fn is_locked(&self) -> bool {
        self.1.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryRatelimiter {
    buckets: Arc<Mutex<HashMap<Path, Arc<Bucket>>>>,
    global: Arc<GlobalLockPair>,
}

impl InMemoryRatelimiter {
    /// Create a new in-memory ratelimiter.
    ///
    /// This is used by the [`Client`] to queue requests in order to avoid
    /// hitting the API's ratelimits.
    ///
    /// [`Client`]: super::super::client::Client
    pub fn new() -> Self {
        Self::default()
    }

    fn entry(
        &self,
        path: Path,
        tx: TicketNotifier,
    ) -> (Arc<Bucket>, bool) {
        // nb: not realisically point of contention
        let mut buckets = self.buckets.lock().unwrap();

        match buckets.entry(path.clone()) {
            Entry::Occupied(bucket) => {
                tracing::debug!("got existing bucket: {:?}", path);

                let bucket = bucket.into_mut();
                bucket.queue.push(tx);
                tracing::debug!("added request into bucket queue: {:?}", path);

                (Arc::clone(&bucket), false)
            }
            Entry::Vacant(entry) => {
                tracing::debug!("making new bucket for path: {:?}", path);
                let bucket = Bucket::new(path.clone());
                bucket.queue.push(tx);

                let bucket = Arc::new(bucket);
                entry.insert(Arc::clone(&bucket));

                (bucket, true)
            }
        }
    }
}

impl Ratelimiter for InMemoryRatelimiter {
    fn bucket(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<Option<InfoBucket>, Box<dyn Error + Send + Sync + 'static>>> + Send + 'static>> {
        if let Some(bucket) = self.buckets.lock().unwrap().get(path) {
            let started_at = bucket.started_at.lock().unwrap();

            Box::pin(future::ok(Some(InfoBucket {
                limit: bucket.limit(),
                remaining: bucket.remaining(),
                reset_after: bucket.reset_after(),
                started_at: *started_at,
            })))
        } else {
            Box::pin(future::ok(None))
        }
    }

    fn globally_locked(&self) -> Pin<Box<dyn Future<Output = Result<bool, Box<dyn Error + Send + Sync + 'static>>> + Send + 'static>> {
        Box::pin(future::ok(self.global.is_locked()))
    }

    fn has(&self, path: &Path) -> Pin<Box<dyn Future<Output = Result<bool, Box<dyn Error + Send + Sync + 'static>>> + Send + 'static>> {
        let has = self.buckets.lock().unwrap().contains_key(path);

        Box::pin(future::ok(has))
    }

    fn ticket(&self, path: Path) -> Pin<Box<dyn Future<Output = Result<TicketReceiver, Box<dyn Error + Send + Sync + 'static>>> + Send + 'static>> {
        tracing::debug!("getting bucket for path: {:?}", path);

        let (tx, rx) = ticket::channel();
        let (bucket, fresh) = self.entry(path.clone(), tx);

        if fresh {
            tokio::spawn(
                BucketQueueTask::new(
                    bucket,
                    Arc::clone(&self.buckets),
                    Arc::clone(&self.global),
                    path,
                )
                .run(),
            );
        }

        Box::pin(future::ok(rx))
    }
}
