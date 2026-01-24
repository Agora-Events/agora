use std::env;

pub mod cors;
pub mod security;

pub use cors::create_cors_layer;
pub use security::create_security_headers_layer;

pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/agora".to_string()),
        }
    }
}
