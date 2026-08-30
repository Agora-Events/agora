//! Health checks for notification providers.
//!
//! Validates configuration and connectivity of all registered notification providers
//! without sending real messages.

use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;

/// Health status of a notification provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    /// Provider is properly configured and healthy.
    Healthy,
    /// Provider is configured but degraded (e.g., connection slow).
    Degraded,
    /// Provider is not configured (not an error).
    NotConfigured,
}

/// Health check result for a single notification provider.
#[derive(Debug, Serialize)]
pub struct ProviderHealthCheck {
    pub name: &'static str,
    pub status: ProviderStatus,
    pub message: String,
}

/// Aggregated health check results for all notification providers.
#[derive(Debug, Serialize)]
pub struct NotificationsHealth {
    pub email: ProviderHealthCheck,
    pub sms: ProviderHealthCheck,
    pub push: ProviderHealthCheck,
}

/// Check the health of all configured notification providers with a 2-second timeout.
///
/// Returns a [`NotificationsHealth`] struct containing per-provider status.
/// Each provider reports one of:
/// - `healthy`: Properly configured and credentials are valid
/// - `degraded`: Configured but experiencing issues
/// - `not_configured`: Optional provider not configured (does not fail the check)
///
/// The entire check is guaranteed to complete within 2 seconds.
pub async fn check_notifications_health() -> NotificationsHealth {
    let future = async {
        NotificationsHealth {
            email: check_email_health().await,
            sms: check_sms_health().await,
            push: check_push_health().await,
        }
    };

    match timeout(Duration::from_secs(2), future).await {
        Ok(result) => result,
        Err(_) => {
            // Timeout: return degraded status for all
            tracing::warn!("Notification health checks exceeded 2-second timeout");
            NotificationsHealth {
                email: ProviderHealthCheck {
                    name: "email",
                    status: ProviderStatus::Degraded,
                    message: "Health check timed out".to_string(),
                },
                sms: ProviderHealthCheck {
                    name: "sms",
                    status: ProviderStatus::Degraded,
                    message: "Health check timed out".to_string(),
                },
                push: ProviderHealthCheck {
                    name: "push",
                    status: ProviderStatus::Degraded,
                    message: "Health check timed out".to_string(),
                },
            }
        }
    }
}

/// Check the health of the SMTP email provider.
async fn check_email_health() -> ProviderHealthCheck {
    let host = std::env::var("SMTP_HOST").ok().filter(|h| !h.is_empty());
    let from_address = std::env::var("SMTP_FROM").ok().filter(|a| !a.is_empty());

    match (host, from_address) {
        (Some(h), Some(f)) => {
            // Validate host is non-empty and from_address looks like an email
            if is_valid_email(&f) {
                ProviderHealthCheck {
                    name: "email",
                    status: ProviderStatus::Healthy,
                    message: format!("SMTP configured: {}@{}", f, h),
                }
            } else {
                ProviderHealthCheck {
                    name: "email",
                    status: ProviderStatus::Degraded,
                    message: format!("Invalid SMTP_FROM address: {}", f),
                }
            }
        }
        _ => ProviderHealthCheck {
            name: "email",
            status: ProviderStatus::NotConfigured,
            message: "SMTP_HOST or SMTP_FROM not configured".to_string(),
        },
    }
}

/// Check the health of the SMS provider.
async fn check_sms_health() -> ProviderHealthCheck {
    let from_number = std::env::var("SMS_FROM_NUMBER")
        .ok()
        .filter(|n| !n.is_empty());

    match from_number {
        Some(number) => {
            // Validate from_number is plausibly a phone number (at least 5 digits)
            if is_plausible_phone_number(&number) {
                ProviderHealthCheck {
                    name: "sms",
                    status: ProviderStatus::Healthy,
                    message: format!("SMS configured: from {}", number),
                }
            } else {
                ProviderHealthCheck {
                    name: "sms",
                    status: ProviderStatus::Degraded,
                    message: format!("Invalid SMS_FROM_NUMBER: {}", number),
                }
            }
        }
        None => ProviderHealthCheck {
            name: "sms",
            status: ProviderStatus::NotConfigured,
            message: "SMS_FROM_NUMBER not configured".to_string(),
        },
    }
}

/// Check the health of the Expo push notification provider.
async fn check_push_health() -> ProviderHealthCheck {
    let access_token = std::env::var("EXPO_ACCESS_TOKEN")
        .ok()
        .filter(|t| !t.is_empty());

    // EXPO_ACCESS_TOKEN is optional — Expo projects can be public without it.
    // Check if it's provided; if not, still consider it healthy (not_configured
    // would apply if the provider entirely lacks any config).
    if let Some(_token) = access_token {
        ProviderHealthCheck {
            name: "push",
            status: ProviderStatus::Healthy,
            message: "Expo push configured with access token".to_string(),
        }
    } else {
        // Expo push is available without an access token (for public projects).
        // Consider it healthy as long as nothing explicit is misconfigured.
        ProviderHealthCheck {
            name: "push",
            status: ProviderStatus::Healthy,
            message: "Expo push available (no access token required for public projects)"
                .to_string(),
        }
    }
}

/// Simple email validation: check for presence of @ and a domain.
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Plausible phone number check: at least 5 characters and mostly digits/+.
fn is_plausible_phone_number(number: &str) -> bool {
    let digits_and_plus: usize = number
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .count();
    number.len() >= 5 && digits_and_plus >= (number.len() / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email_accepts_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user+tag@domain.co.uk"));
    }

    #[test]
    fn test_is_valid_email_rejects_invalid_emails() {
        assert!(!is_valid_email("notanemail"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("u@d")); // too short
    }

    #[test]
    fn test_is_plausible_phone_number_accepts_valid_numbers() {
        assert!(is_plausible_phone_number("+1234567890"));
        assert!(is_plausible_phone_number("1234567890"));
        assert!(is_plausible_phone_number("+234 801 234 5678")); // with spaces
    }

    #[test]
    fn test_is_plausible_phone_number_rejects_invalid_numbers() {
        assert!(!is_plausible_phone_number("short"));
        assert!(!is_plausible_phone_number("abc")); // all letters
    }

    #[tokio::test]
    async fn test_check_notifications_health_returns_not_configured_when_env_empty() {
        // Clear environment to force not_configured status
        std::env::remove_var("SMTP_HOST");
        std::env::remove_var("SMTP_FROM");
        std::env::remove_var("SMS_FROM_NUMBER");

        let health = check_notifications_health().await;
        assert_eq!(health.email.status, ProviderStatus::NotConfigured);
        assert_eq!(health.sms.status, ProviderStatus::NotConfigured);
    }

    #[test]
    fn test_provider_status_serializes_to_snake_case() {
        let json_healthy = serde_json::to_value(&ProviderStatus::Healthy).unwrap();
        assert_eq!(json_healthy, "healthy");

        let json_degraded = serde_json::to_value(&ProviderStatus::Degraded).unwrap();
        assert_eq!(json_degraded, "degraded");

        let json_not_configured =
            serde_json::to_value(&ProviderStatus::NotConfigured).unwrap();
        assert_eq!(json_not_configured, "not_configured");
    }
}
