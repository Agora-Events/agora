use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// A single localised title/description for an event (Issue #1344).
///
/// The default locale lives on the `events` row itself; rows in
/// `event_translations` override title/description only when the caller
/// requests a matching language.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EventTranslation {
    pub id: Uuid,
    pub event_id: Uuid,
    pub locale: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Translation entry accepted from the `create_event` request body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EventTranslationInput {
    /// BCP-47-style language tag, e.g. "es", "fr", "pt-BR".
    pub locale: String,
    pub title: String,
    pub description: Option<String>,
}

/// Maximum length for a translation title.
pub const MAX_TRANSLATION_TITLE_LEN: usize = 500;

/// Maximum length for a translation description.
pub const MAX_TRANSLATION_DESCRIPTION_LEN: usize = 10_000;

/// Maximum number of translations accepted per event.
pub const MAX_TRANSLATIONS_PER_EVENT: usize = 20;

/// Validates a single translation input.
pub fn validate_translation(
    t: &EventTranslationInput,
) -> Result<(), String> {
    let locale = t.locale.trim();
    if locale.is_empty() || locale.len() > 20 {
        return Err("each translation requires a locale".to_string());
    }
    // Restrict to a conservative set of BCP-47-ish tags (letters, digits, dashes).
    if !locale
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(format!("invalid locale '{}'", t.locale));
    }
    if t.title.trim().is_empty() {
        return Err(format!(
            "translation title is required for locale '{}'",
            t.locale
        ));
    }
    if t.title.chars().count() > MAX_TRANSLATION_TITLE_LEN {
        return Err(format!(
            "translation title must be at most {MAX_TRANSLATION_TITLE_LEN} characters"
        ));
    }
    if t.description.as_deref().is_some_and(|d| {
        d.chars().count() > MAX_TRANSLATION_DESCRIPTION_LEN
    }) {
        return Err(format!(
            "translation description must be at most {MAX_TRANSLATION_DESCRIPTION_LEN} characters"
        ));
    }
    Ok(())
}

/// Validates the full translations list for a create/update request.
pub fn validate_translations(
    translations: &[EventTranslationInput],
) -> Result<(), String> {
    if translations.len() > MAX_TRANSLATIONS_PER_EVENT {
        return Err(format!(
            "at most {MAX_TRANSLATIONS_PER_EVENT} translations are allowed per event"
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for t in translations {
        let locale = t.locale.trim().to_lowercase();
        if !seen.insert(locale) {
            return Err(format!(
                "duplicate translation locale '{}'",
                t.locale
            ));
        }
        validate_translation(t)?;
    }
    Ok(())
}