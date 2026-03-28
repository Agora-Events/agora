//! Database model structs for the Agora platform.
//!
//! Each module contains a struct that maps directly to a PostgreSQL table via
//! [`sqlx::FromRow`]. The entity relationships are:
//!
//! ```text
//! Organizer ──< Event ──< TicketTier ──< Ticket ──< Transaction
//!                                            └──────── User
//! ```
//!
//! All primary keys are UUID v4. `updated_at` fields are maintained automatically
//! by database triggers defined in the initial schema migration.

pub mod event;
pub mod organizer;
pub mod ticket;
pub mod transaction;
pub mod user;
