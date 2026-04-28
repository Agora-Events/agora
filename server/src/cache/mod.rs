//! # Redis Cache Module
//!
//! Provides Redis caching functionality for high-traffic data like popular events.
//! Implements a cache layer between Axum handlers and PostgreSQL database.

use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

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
                let parsed = serde_json::from_str(&json).map_err(|e| {
                    RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON deserialization failed",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(parsed))
            }
            None => Ok(None),
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

    /// Check if Redis is healthy
    pub async fn ping(&mut self) -> Result<(), RedisError> {
        redis::cmd("PING").query_async(&mut self.client).await
    }
}
