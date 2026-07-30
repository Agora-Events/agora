//! # Notification Service
//!
//! Trait-based abstraction for sending notifications via different channels
//! (email, SMS, push, etc.). New providers are added by implementing
//! [`NotificationProvider`] and registering them in [`NotificationService`].

pub mod email;
pub mod sms;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A self-contained notification message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Recipient address — email address, phone number, device token, etc.
    pub recipient: String,
    /// Short subject line or title.
    pub subject: String,
    /// Full message body (plain text or HTML depending on provider).
    pub body: String,
}

/// The error type returned by notification providers.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    #[error("Provider configuration error: {0}")]
    ConfigError(String),
}

/// Implemented by every notification channel (email, SMS, push, …).
///
/// Providers are intentionally stateless from the caller's perspective —
/// all channel-specific config lives inside the concrete struct.
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Human-readable name used in logs, e.g. `"smtp-email"`.
    fn name(&self) -> &'static str;

    /// Send a notification. Returns `Ok(())` on successful delivery.
    async fn send(&self, notification: &Notification) -> Result<(), NotificationError>;
}

/// Orchestrates one or more [`NotificationProvider`]s.
///
/// Providers are tried in registration order. If a provider fails the error
/// is logged and the next provider is attempted (fan-out / fallback pattern).
pub struct NotificationService {
    providers: Vec<Box<dyn NotificationProvider>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider. Call this during application startup.
    pub fn register<P: NotificationProvider + 'static>(&mut self, provider: P) {
        self.providers.push(Box::new(provider));
    }

    /// Send `notification` through all registered providers.
    ///
    /// Returns the first error encountered, or `Ok(())` if all providers
    /// succeeded (or no providers are registered).
    pub async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        for provider in &self.providers {
            if let Err(e) = provider.send(notification).await {
                tracing::error!(
                    provider = provider.name(),
                    error = %e,
                    recipient = %notification.recipient,
                    "Notification delivery failed"
                );
                return Err(e);
            }
            tracing::info!(
                provider = provider.name(),
                recipient = %notification.recipient,
                subject = %notification.subject,
                "Notification sent"
            );
        }
        Ok(())
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
