import re

# 1. Fix src/routes/mod.rs
with open("src/routes/mod.rs", "r") as f:
    data = f.read()

# Replace pool with rates_state for rates route
data = data.replace(".route(\"/rates\", get(get_rates))\n        .with_state(pool)", ".route(\"/rates\", get(get_rates))\n        .with_state(rates_state)")

# Strip out broken monitoring, leaderboard, profile routes and imports
data = re.sub(r"leaderboard::\{.*\},\n", "", data)
data = re.sub(r"monitoring::\{.*\},\n", "", data)
data = re.sub(r"profile::\{[\s\S]*?\},", "", data)

data = re.sub(r"let leaderboard_state = LeaderboardState \{[\s\S]*?\}?;", "", data)
data = re.sub(r"\.merge\([\s\S]*?leaderboard[\s\S]*?leaderboard_state\)", "", data)
data = re.sub(r"\.merge\([\s\S]*?monitoring[\s\S]*?monitoring_state\)", "", data)
data = re.sub(r"let profile_state = ProfileState \{[\s\S]*?\}?;", "", data)
data = re.sub(r"\.merge\([\s\S]*?profile[\s\S]*?profile_state\)", "", data)

with open("src/routes/mod.rs", "w") as f:
    f.write(data)


# 2. Fix src/models/event.rs
with open("src/models/event.rs", "r") as f:
    data = f.read()

data = data.replace(
    "pub minted_tickets: i64,\n}",
    "pub minted_tickets: i64,\n    pub image_url: Option<String>,\n    #[sqlx(default)]\n    pub is_free: bool,\n    #[sqlx(default)]\n    pub is_free_populated: bool,\n}"
)

# Fix tests
data = re.sub(
    r"minted_tickets: 0,\n(\s*)}",
    r"minted_tickets: 0,\n            total_tickets: 0,\n            image_url: None,\n            is_free: false,\n            is_free_populated: false,\n\1}",
    data
)

data = re.sub(
    r"updated_at: Utc::now\(\),\n(\s*)}",
    r"updated_at: Utc::now(),\n            total_tickets: 0,\n            minted_tickets: 0,\n            image_url: None,\n            is_free: false,\n            is_free_populated: false,\n\1}",
    data
)

with open("src/models/event.rs", "w") as f:
    f.write(data)


# 3. Fix src/handlers/events.rs
with open("src/handlers/events.rs", "r") as f:
    data = f.read()

data = data.replace("pub struct TicketTierResponse", "#[derive(serde::Deserialize)]\npub struct TicketTierResponse")
data = data.replace("let event = match sqlx::query_scalar::<_, Uuid>", "let organizer_id = match sqlx::query_scalar::<_, Uuid>")

# Fix TicketTierResponse in tests
data = data.replace(
    "quantity: 50,\n        sold: 5,",
    "total_quantity: 50,\n        available_quantity: 45,\n        description: None,\n        created_at: chrono::Utc::now(),"
)

# Fix Event in tests
data = re.sub(
    r"minted_tickets: 0,\n(\s*)}",
    r"minted_tickets: 0,\n            total_tickets: 0,\n            image_url: None,\n            is_free: false,\n            is_free_populated: false,\n            is_flagged: false,\n            is_featured: false,\n\1}",
    data
)

data = re.sub(
    r"updated_at: chrono::Utc::now\(\),\n(\s*)}",
    r"updated_at: chrono::Utc::now(),\n            total_tickets: 0,\n            minted_tickets: 0,\n            image_url: None,\n            is_free: false,\n            is_free_populated: false,\n            is_flagged: false,\n            is_featured: false,\n\1}",
    data
)

with open("src/handlers/events.rs", "w") as f:
    f.write(data)

# 4. Fix Cargo.toml
with open("Cargo.toml", "r") as f:
    data = f.read()
data = data.replace('redis = { version = "0.25", features = ["tokio-comp"] }', 'redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }')
data = re.sub(r'# Redis for caching\nredis = \{ version = "0.24", features = \["tokio-comp", "connection-manager"\] \}\n', '', data)
with open("Cargo.toml", "w") as f:
    f.write(data)

