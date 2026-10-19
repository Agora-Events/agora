//! Stub SMS provider.
//!
//! Replace the `send` body with a real Twilio/SNS client when ready.

use async_trait::async_trait;

use super::{Notification, NotificationError, NotificationProvider};

/// Sends notifications via SMS.
pub struct SmsProvider {
    /// Originating phone number or short code.
    pub from_number: String,
}

#[async_trait]
impl NotificationProvider for SmsProvider {
    fn name(&self) -> &'static str {
        "sms"
    }

    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        // TODO: integrate twilio-rs or aws-sdk-sns here.
        tracing::debug!(
            from = %self.from_number,
            to = %notification.recipient,
            "[stub] Would send SMS"
        );
        Ok(())
    }
}
