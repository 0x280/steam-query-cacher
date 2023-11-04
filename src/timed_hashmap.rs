use std::{hash::Hash, sync::Arc, time};

use dashmap::DashMap;
use tokio::sync::RwLock;

pub const DEFAULT_CLEANUP_INTERVAL: time::Duration = time::Duration::from_secs(60);

#[derive(Debug)]
pub struct TimedHashMap<K: Eq + Hash + Clone + Send + Sync, V: Clone + Send + Sync> {
    inner: Arc<RwLock<DashMap<K, (V, time::Instant, time::Duration)>>>,
    refresh_interval: bool,
    cleanup_interval: time::Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static>
    TimedHashMap<K, V>
{
    pub async fn new(refresh_interval: bool, cleanup_interval: Option<time::Duration>) -> Self {
        let instance = Self {
            inner: Arc::new(RwLock::new(DashMap::new())),
            refresh_interval,
            cleanup_interval: cleanup_interval.unwrap_or(DEFAULT_CLEANUP_INTERVAL),
        };

        instance.start_cleanup_task().await;

        instance
    }

    pub async fn insert(&self, key: K, value: V, duration: time::Duration) {
        self.inner
            .write()
            .await
            .insert(key, (value, time::Instant::now() + duration, duration));
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        if let Some(temp) = self.inner.read().await.get(key) {
            let (value, expiration, duration) = temp.value();
            if time::Instant::now() < *expiration {
                if self.refresh_interval {
                    self.inner.write().await.insert(
                        key.clone(),
                        (value.clone(), time::Instant::now() + *duration, *duration),
                    );
                }
                return Some(value.clone());
            } else {
                self.inner.write().await.remove(key);
            }
        }
        None
    }

    async fn cleanup(map: &Arc<RwLock<DashMap<K, (V, time::Instant, time::Duration)>>>) {
        map.write()
            .await
            .retain(|_, (_, expiration, _)| *expiration > time::Instant::now());
    }

    async fn start_cleanup_task(&self) {
        let interval = self.cleanup_interval;
        let map = self.inner.clone();

        tokio::spawn(async move {
            loop {
                if Arc::strong_count(&map) == 1 {
                    break;
                }

                Self::cleanup(&map).await;

                tokio::time::sleep(interval).await;
            }
        });
    }
}
