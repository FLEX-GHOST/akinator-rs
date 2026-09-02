use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::client::Akinator;
use crate::error::{Error, Result};

struct SessionEntry {
    session: Akinator,
    last_accessed: Instant,
}

/// A high-performance, bounded session manager for long-running bots and services.
/// Automatically evicts inactive sessions based on a TTL cleanup task to maintain low memory usage.
pub struct SessionManager<K: Eq + Hash + Clone + Send + Sync + 'static> {
    sessions: Arc<RwLock<HashMap<K, SessionEntry>>>,
    ttl: Duration,
    max_capacity: usize,
    _cleanup_handle: Option<JoinHandle<()>>,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static> SessionManager<K> {
    /// Creates a new [`SessionManager`] with the given TTL and maximum capacity limit.
    /// Spawns a background task to automatically purge expired sessions.
    pub fn new(ttl: Duration, max_capacity: usize) -> Arc<Self> {
        let sessions: Arc<RwLock<HashMap<K, SessionEntry>>> = Arc::new(RwLock::new(HashMap::new()));
        let cleanup_sessions = Arc::clone(&sessions);
        let cleanup_interval = ttl.div_f32(2.0).max(Duration::from_secs(5));

        let cleanup_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                let now = Instant::now();
                let mut lock = cleanup_sessions.write().await;
                lock.retain(|_, entry| now.duration_since(entry.last_accessed) < ttl);
            }
        });

        Arc::new(Self {
            sessions,
            ttl,
            max_capacity,
            _cleanup_handle: Some(cleanup_handle),
        })
    }

    /// Retrieves a cloned [`Akinator`] session for the given key and updates its access time.
    pub async fn get(&self, key: &K) -> Option<Akinator> {
        let mut lock = self.sessions.write().await;
        if let Some(entry) = lock.get_mut(key)
            && entry.last_accessed.elapsed() < self.ttl
        {
            entry.last_accessed = Instant::now();
            return Some(entry.session.clone());
        }
        None
    }

    /// Stores or updates an [`Akinator`] session for the given key.
    ///
    /// # Errors
    /// Returns [`Error::AkinatorError`] if the manager has reached its maximum capacity.
    pub async fn insert(&self, key: K, session: Akinator) -> Result<()> {
        let mut lock = self.sessions.write().await;
        if lock.len() >= self.max_capacity && !lock.contains_key(&key) {
            let now = Instant::now();
            let ttl = self.ttl;
            lock.retain(|_, entry| now.duration_since(entry.last_accessed) < ttl);

            if lock.len() >= self.max_capacity {
                return Err(Error::AkinatorError(
                    "SessionManager reached maximum capacity limit. Cannot insert new session.".to_string(),
                ));
            }
        }

        lock.insert(
            key,
            SessionEntry {
                session,
                last_accessed: Instant::now(),
            },
        );
        Ok(())
    }

    /// Removes an [`Akinator`] session from the manager.
    pub async fn remove(&self, key: &K) -> Option<Akinator> {
        let mut lock = self.sessions.write().await;
        lock.remove(key).map(|entry| entry.session)
    }

    /// Returns the current number of active sessions in memory.
    pub async fn len(&self) -> usize {
        let lock = self.sessions.read().await;
        lock.len()
    }

    /// Checks if there are no active sessions.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}
