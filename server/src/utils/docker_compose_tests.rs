#[cfg(test)]
mod tests {
    use std::fs;

    fn load_compose() -> serde_yaml::Value {
        let content = fs::read_to_string("docker-compose.yml")
            .expect("docker-compose.yml should exist in the server directory");
        serde_yaml::from_str(&content).expect("docker-compose.yml should be valid YAML")
    }

    #[tokio::test]
    async fn test_compose_file_exists_and_is_valid_yaml() {
        let compose = load_compose();
        assert!(
            compose.is_mapping(),
            "docker-compose.yml root should be a mapping"
        );
    }

    #[tokio::test]
    async fn test_compose_has_postgres_service() {
        let compose = load_compose();
        assert!(
            compose["services"]["postgres"].is_mapping(),
            "services should contain a 'postgres' entry"
        );
    }

    #[tokio::test]
    async fn test_postgres_uses_latest_image() {
        let compose = load_compose();
        let image = compose["services"]["postgres"]["image"]
            .as_str()
            .expect("postgres image should be a string");
        assert_eq!(image, "postgres:latest");
    }

    #[tokio::test]
    async fn test_postgres_env_vars_are_set() {
        let compose = load_compose();
        let required_keys = ["POSTGRES_USER", "POSTGRES_PASSWORD", "POSTGRES_DB"];
        for key in &required_keys {
            assert!(
                compose["services"]["postgres"]["environment"][key].is_string(),
                "environment should contain {key}"
            );
        }
    }

    #[tokio::test]
    async fn test_postgres_db_name_is_agora() {
        let compose = load_compose();
        let db = compose["services"]["postgres"]["environment"]["POSTGRES_DB"]
            .as_str()
            .expect("POSTGRES_DB should be a string");
        assert_eq!(db, "agora");
    }

    #[tokio::test]
    async fn test_postgres_port_mapping_exists() {
        let compose = load_compose();
        let ports = compose["services"]["postgres"]["ports"]
            .as_sequence()
            .expect("ports should be a sequence");
        assert!(!ports.is_empty(), "ports should have at least one mapping");
        let port_str = ports[0].as_str().expect("port entry should be a string");
        assert_eq!(port_str, "5432:5432");
    }

    #[tokio::test]
    async fn test_volumes_section_exists() {
        let compose = load_compose();
        assert!(
            compose["volumes"].is_mapping(),
            "top-level volumes section should exist"
        );
    }

    #[tokio::test]
    async fn test_postgres_volume_mount_exists() {
        let compose = load_compose();
        let volumes = compose["services"]["postgres"]["volumes"]
            .as_sequence()
            .expect("postgres volumes should be a sequence");
        assert!(
            !volumes.is_empty(),
            "postgres should have at least one volume mount"
        );
        let vol_str = volumes[0]
            .as_str()
            .expect("volume entry should be a string");
        assert!(
            vol_str.contains("/var/lib/postgresql/data"),
            "volume should mount postgres data directory"
        );
    }

    #[tokio::test]
    async fn test_database_url_matches_compose_defaults() {
        let compose = load_compose();
        let env = &compose["services"]["postgres"]["environment"];
        let user = env["POSTGRES_USER"].as_str().unwrap();
        let password = env["POSTGRES_PASSWORD"].as_str().unwrap();
        let db = env["POSTGRES_DB"].as_str().unwrap();

        let expected_url = format!("postgres://{}:{}@localhost:5432/{}", user, password, db);
        let env_example = fs::read_to_string(".env.example").expect(".env.example should exist");

        assert!(
            env_example.contains(&expected_url),
            ".env.example DATABASE_URL should match docker-compose credentials: {expected_url}"
        );
    }
}
