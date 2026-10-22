//! SMTP email provider using `lettre`.
//!
//! Delivers transactional notifications via SMTP with STARTTLS support,
//! optional authentication, and configurable host/port.

use std::env;
use async_trait::async_trait;
use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use super::{Notification, NotificationError, NotificationProvider};

/// Sends notifications via SMTP email.
#[derive(Debug, Clone)]
pub struct SmtpEmailProvider {
    /// SMTP server host, e.g. `"smtp.example.com"`.
    pub host: String,
    /// SMTP port (e.g. 587 for STARTTLS, 465 for TLS, or 25).
    pub port: u16,
    /// Sender address shown in the `From:` header.
    pub from_address: String,
    /// Optional username & password credentials for SMTP authentication.
    pub credentials: Option<(String, String)>,
    /// Whether to enable TLS/STARTTLS.
    pub use_tls: bool,
}

impl SmtpEmailProvider {
    /// Create a new `SmtpEmailProvider` with default port 587 and TLS enabled.
    pub fn new(host: impl Into<String>, from_address: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 587,
            from_address: from_address.into(),
            credentials: None,
            use_tls: true,
        }
    }

    /// Set a custom SMTP port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set credentials for authenticated SMTP.
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.credentials = Some((username.into(), password.into()));
        self
    }

    /// Enable or disable TLS.
    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    /// Initialize provider from standard environment variables:
    /// - `SMTP_HOST` (required, or defaults to `"stub"`)
    /// - `SMTP_PORT` (default `587`)
    /// - `SMTP_FROM` / `SMTP_FROM_ADDRESS` (default `"noreply@agora.events"`)
    /// - `SMTP_USER` / `SMTP_USERNAME` (optional)
    /// - `SMTP_PASS` / `SMTP_PASSWORD` (optional)
    /// - `SMTP_USE_TLS` (default `true`, set `"false"` or `"0"` to disable)
    pub fn from_env() -> Self {
        let host = env::var("SMTP_HOST").unwrap_or_else(|_| "stub".to_string());
        let port = env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(587);
        let from_address = env::var("SMTP_FROM")
            .or_else(|_| env::var("SMTP_FROM_ADDRESS"))
            .unwrap_or_else(|_| "noreply@agora.events".to_string());

        let user = env::var("SMTP_USER").or_else(|_| env::var("SMTP_USERNAME")).ok();
        let pass = env::var("SMTP_PASS").or_else(|_| env::var("SMTP_PASSWORD")).ok();
        let credentials = match (user, pass) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        let use_tls = env::var("SMTP_USE_TLS")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(true);

        Self {
            host,
            port,
            from_address,
            credentials,
            use_tls,
        }
    }
}

#[async_trait]
impl NotificationProvider for SmtpEmailProvider {
    fn name(&self) -> &'static str {
        "smtp-email"
    }

    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        // If host is configured as "stub" or empty, log and return early (useful for testing/dev)
        if self.host == "stub" || self.host.is_empty() {
            tracing::debug!(
                host = %self.host,
                from = %self.from_address,
                to = %notification.recipient,
                subject = %notification.subject,
                "[stub] Would send email"
            );
            return Ok(());
        }

        let from: Mailbox = self.from_address.parse().map_err(|e| {
            NotificationError::ConfigError(format!(
                "Invalid from_address '{}': {}",
                self.from_address, e
            ))
        })?;

        let to: Mailbox = notification.recipient.parse().map_err(|e| {
            NotificationError::DeliveryFailed(format!(
                "Invalid recipient address '{}': {}",
                notification.recipient, e
            ))
        })?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(&notification.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(notification.body.clone())
            .map_err(|e| {
                NotificationError::DeliveryFailed(format!("Failed to build email message: {}", e))
            })?;

        let transport = if self.use_tls {
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)
                .map_err(|e| {
                    NotificationError::ConfigError(format!(
                        "Invalid SMTP relay host '{}': {}",
                        self.host, e
                    ))
                })?
                .port(self.port);

            if let Some((user, pass)) = &self.credentials {
                builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
            }
            builder.build()
        } else {
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
                .port(self.port);

            if let Some((user, pass)) = &self.credentials {
                builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
            }
            builder.build()
        };

        transport.send(email).await.map_err(|e| {
            NotificationError::DeliveryFailed(format!("SMTP delivery error: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_provider_builder_defaults() {
        let provider = SmtpEmailProvider::new("smtp.example.com", "alerts@example.com");
        assert_eq!(provider.host, "smtp.example.com");
        assert_eq!(provider.port, 587);
        assert_eq!(provider.from_address, "alerts@example.com");
        assert!(provider.credentials.is_none());
        assert!(provider.use_tls);
        assert_eq!(provider.name(), "smtp-email");
    }

    #[test]
    fn test_smtp_provider_custom_options() {
        let provider = SmtpEmailProvider::new("mail.internal", "bot@agora.events")
            .with_port(2525)
            .with_tls(false)
            .with_credentials("user1", "pass123");

        assert_eq!(provider.port, 2525);
        assert!(!provider.use_tls);
        assert_eq!(
            provider.credentials,
            Some(("user1".to_string(), "pass123".to_string()))
        );
    }

    #[tokio::test]
    async fn test_smtp_provider_stub_behavior() {
        let provider = SmtpEmailProvider::new("stub", "no-reply@agora.events");
        let notification = Notification {
            recipient: "user@example.com".to_string(),
            subject: "Welcome".to_string(),
            body: "Hello from Agora!".to_string(),
        };

        let res = provider.send(&notification).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_smtp_provider_invalid_recipient() {
        let provider = SmtpEmailProvider::new("smtp.example.com", "no-reply@agora.events");
        let notification = Notification {
            recipient: "not an email @@invalid".to_string(),
            subject: "Test".to_string(),
            body: "Hello".to_string(),
        };

        let res = provider.send(&notification).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            NotificationError::DeliveryFailed(msg) => {
                assert!(msg.contains("Invalid recipient"));
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_smtp_provider_invalid_sender() {
        let provider = SmtpEmailProvider::new("smtp.example.com", "not a valid sender");
        let notification = Notification {
            recipient: "user@example.com".to_string(),
            subject: "Test".to_string(),
            body: "Hello".to_string(),
        };

        let res = provider.send(&notification).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            NotificationError::ConfigError(msg) => {
                assert!(msg.contains("Invalid from_address"));
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}

