//! Stub SMTP email provider.
//!
//! Replace the `send` body with a real SMTP/SES client when ready.
//! The trait contract stays the same regardless of the underlying transport.

use async_trait::async_trait;

use super::{Notification, NotificationError, NotificationProvider};

/// Sends notifications via SMTP email.
pub struct SmtpEmailProvider {
    /// SMTP server host, e.g. `"smtp.example.com"`.
    pub host: String,
    /// Sender address shown in the `From:` header.
    pub from_address: String,
}

#[async_trait]
impl NotificationProvider for SmtpEmailProvider {
    fn name(&self) -> &'static str {
        "smtp-email"
    }

    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        // TODO: integrate lettre or aws-sdk-sesv2 here.
        tracing::debug!(
            host = %self.host,
            from = %self.from_address,
            to = %notification.recipient,
            subject = %notification.subject,
            "[stub] Would send email"
        );
        Ok(())
    }
}
