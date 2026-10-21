//! # Cache Module
//!
//! Provides a [`CacheLayer`] trait plus Redis and in-memory implementations
//! for caching high-traffic payloads such as the events list.

use async_trait::async_trait;
use dashmap::DashMap;
use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache key for the default events list payload.
pub const EVENTS_LIST_CACHE_KEY: &str = "events:list";

/// TTL for the events list cache (60 seconds).
pub const EVENTS_LIST_CACHE_TTL: Duration = Duration::from_secs(60);

/// Abstraction over cache backends used by handlers.
#[async_trait]
pub trait CacheLayer: Send + Sync {
    /// Fetch a cached string value by key.
    async fn get(&self, key: &str) -> Option<String>;

    /// Store a string value with a TTL.
    async fn set(&self, key: &str, value: &str, ttl: Duration);

    /// Remove a cached value.
    async fn delete(&self, key: &str);
}

/// Redis cache client wrapper
#[derive(Clone)]
pub struct RedisCache {
    client: ConnectionManager,
}

impl RedisCache {
    /// Create a new Redis cache client
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self { client: manager })
    }

    /// Get a cached value by key
    pub async fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, RedisError> {
        let value: Option<String> = self.client.get(key).await?;
        match value {
            Some(json) => {
                crate::metrics::CACHE_HITS_TOTAL
                    .with_label_values(&[key])
                    .inc();
                let parsed = serde_json::from_str(&json).map_err(|e| {
                    RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON deserialization failed",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(parsed))
            }
            None => {
                crate::metrics::CACHE_MISSES_TOTAL
                    .with_label_values(&[key])
                    .inc();
                Ok(None)
            }
        }
    }

    /// Set a cached value with TTL
    pub async fn set<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), RedisError> {
        let json = serde_json::to_string(value).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::TypeError,
                "JSON serialization failed",
                e.to_string(),
            ))
        })?;
        self.client.set_ex(key, json, ttl.as_secs()).await
    }

    /// Delete a cached value
    pub async fn delete(&mut self, key: &str) -> Result<(), RedisError> {
        self.client.del(key).await
    }

    /// Invalidate the events list cache key.
    pub async fn invalidate_events_list(&mut self) {
        if let Err(e) = self.delete(EVENTS_LIST_CACHE_KEY).await {
            tracing::warn!("Failed to invalidate {}: {:?}", EVENTS_LIST_CACHE_KEY, e);
        }
    }

    /// Check if Redis is healthy
    pub async fn ping(&mut self) -> Result<(), RedisError> {
        redis::cmd("PING").query_async(&mut self.client).await
    }

    /// Explicitly close the Redis connection manager (Issue #1261). The
    /// underlying connection is also closed on drop, but calling this during
    /// shutdown makes the intent explicit and gives the caller a point to log.
    pub async fn close(self) {
        tracing::info!("Closing Redis connection");
    }

    /// Clone the underlying multiplexed connection manager so services can
    /// issue raw Redis commands (e.g. the waiting-room queue engine, #1187).
    pub fn connection(&self) -> redis::aio::ConnectionManager {
        self.client.clone()
    }
}

#[async_trait]
impl CacheLayer for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut client = self.client.clone();
        match client.get::<_, Option<String>>(key).await {
            Ok(value) => {
                if value.is_some() {
                    crate::metrics::CACHE_HITS_TOTAL
                        .with_label_values(&[key])
                        .inc();
                } else {
                    crate::metrics::CACHE_MISSES_TOTAL
                        .with_label_values(&[key])
                        .inc();
                }
                value
            }
            Err(e) => {
                tracing::warn!("Redis CacheLayer get error for {}: {:?}", key, e);
                None
            }
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) {
        let mut client = self.client.clone();
        if let Err(e) = client.set_ex::<_, _, ()>(key, value, ttl.as_secs()).await {
            tracing::warn!("Redis CacheLayer set error for {}: {:?}", key, e);
        }
    }

    async fn delete(&self, key: &str) {
        let mut client = self.client.clone();
        if let Err(e) = client.del::<_, ()>(key).await {
            tracing::warn!("Redis CacheLayer delete error for {}: {:?}", key, e);
        }
    }
}

/// In-memory LRU-style cache backed by DashMap (used in tests and as a fallback).
#[derive(Clone, Default)]
pub struct MemoryCache {
    entries: Arc<DashMap<String, CacheEntry>>,
}

struct CacheEntry {
    value: String,
    expires_at: Instant,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CacheLayer for MemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
        let now = Instant::now();
        if let Some(entry) = self.entries.get(key) {
            if entry.expires_at > now {
                return Some(entry.value.clone());
            }
        }
        self.entries.remove(key);
        None
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) {
        self.entries.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                expires_at: Instant::now() + ttl,
            },
        );
    }

    async fn delete(&self, key: &str) {
        self.entries.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_returns_cached_value_on_second_get() {
        let cache = MemoryCache::new();
        let payload = r#"{"items":[{"id":"1"}],"pagination":{"page_size":1,"has_more":false,"next_cursor":null}}"#;

        assert!(cache.get(EVENTS_LIST_CACHE_KEY).await.is_none());

        cache
            .set(EVENTS_LIST_CACHE_KEY, payload, EVENTS_LIST_CACHE_TTL)
            .await;

        let first = cache.get(EVENTS_LIST_CACHE_KEY).await;
        let second = cache.get(EVENTS_LIST_CACHE_KEY).await;

        assert_eq!(first.as_deref(), Some(payload));
        assert_eq!(second.as_deref(), Some(payload));
        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn test_memory_cache_delete_purges_key() {
        let cache = MemoryCache::new();
        cache
            .set(EVENTS_LIST_CACHE_KEY, "payload", EVENTS_LIST_CACHE_TTL)
            .await;
        cache.delete(EVENTS_LIST_CACHE_KEY).await;
        assert!(cache.get(EVENTS_LIST_CACHE_KEY).await.is_none());
    }

    #[tokio::test]
    async fn test_memory_cache_ttl_expiry() {
        let cache = MemoryCache::new();
        cache
            .set(EVENTS_LIST_CACHE_KEY, "payload", Duration::from_millis(20))
            .await;
        assert!(cache.get(EVENTS_LIST_CACHE_KEY).await.is_some());
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(cache.get(EVENTS_LIST_CACHE_KEY).await.is_none());
    }
}
