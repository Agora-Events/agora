//! Expo push notification provider.
//!
//! The mobile app is an Expo project, so device tokens are Expo push tokens
//! (`ExponentPushToken[...]`) and delivery goes through Expo's push service
//! rather than APNs/FCM directly. Expo fans the message out to the right
//! platform transport for us.
//!
//! Delivery is best-effort by design: the caller has already committed the
//! important state (a completed sale) before notifying, so a push failure is
//! logged and swallowed rather than failing the request.

use async_trait::async_trait;
use serde::Serialize;

use super::{Notification, NotificationError, NotificationProvider};

/// Expo's public push endpoint. Unauthenticated for tokens issued to the
/// project; an access token is only required if the project enables
/// enhanced security.
const EXPO_PUSH_URL: &str = "https://exp.host/--/api/v2/push/send";

/// One message in an Expo push request.
#[derive(Debug, Serialize)]
struct ExpoPushMessage<'a> {
    to: &'a str,
    title: &'a str,
    body: &'a str,
    sound: &'static str,
}

/// Sends notifications to mobile devices via Expo's push service.
pub struct ExpoPushProvider {
    client: reqwest::Client,
    /// Optional `EXPO_ACCESS_TOKEN` for projects with enhanced push security.
    access_token: Option<String>,
}

impl ExpoPushProvider {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            access_token: std::env::var("EXPO_ACCESS_TOKEN")
                .ok()
                .filter(|t| !t.is_empty()),
        }
    }
}

#[async_trait]
impl NotificationProvider for ExpoPushProvider {
    fn name(&self) -> &'static str {
        "expo-push"
    }

    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        let message = ExpoPushMessage {
            to: &notification.recipient,
            title: &notification.subject,
            body: &notification.body,
            sound: "default",
        };

        let mut request = self
            .client
            .post(EXPO_PUSH_URL)
            .header("accept", "application/json")
            .json(&[message]);

        if let Some(token) = &self.access_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.map_err(|e| {
            NotificationError::DeliveryFailed(format!("Expo push request failed: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::DeliveryFailed(format!(
                "Expo push returned {status}: {body}"
            )));
        }

        Ok(())
    }
}
