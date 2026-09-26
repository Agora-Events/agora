import re

with open("Cargo.toml", "r") as f:
    data = f.read()
data = data.replace('redis = { version = "0.25", features = ["tokio-comp"] }', 'redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }')
data = re.sub(r'# Redis for caching\nredis = \{ version = "0.24", features = \["tokio-comp", "connection-manager"\] \}\n', '', data)
with open("Cargo.toml", "w") as f:
    f.write(data)

with open("src/handlers/mod.rs", "r") as f:
    data = f.read()
if "pub mod profile;" not in data:
    data = data.replace("pub mod monitoring;", "pub mod monitoring;\npub mod profile;\npub mod soroban_listener;")
with open("src/handlers/mod.rs", "w") as f:
    f.write(data)

with open("src/handlers/events.rs", "r") as f:
    data = f.read()
data = data.replace("""    let wallet_address = match sqlx::query_scalar::<_, String>(
        "SELECT wallet_address FROM organizers WHERE id = $1",
    )
    .bind(organizer_id)""", """    let wallet_address = match sqlx::query_scalar::<_, String>(
        "SELECT wallet_address FROM organizers WHERE id = $1",
    )
    .bind(event_id)""")
with open("src/handlers/events.rs", "w") as f:
    f.write(data)

with open("src/routes/mod.rs", "r") as f:
    data = f.read()
data = data.replace(".with_state(pool)", ".with_state(rates_state)")
with open("src/routes/mod.rs", "w") as f:
    f.write(data)
