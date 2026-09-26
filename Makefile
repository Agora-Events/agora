.PHONY: up down logs clean help

# Default target
help:
	@echo "Agora Development Stack - Available Commands"
	@echo "============================================="
	@echo ""
	@echo "  make up       - Start all services (postgres, redis, stellar-rpc, backend, frontend)"
	@echo "  make down     - Stop all services and remove containers"
	@echo "  make logs     - Stream logs from all services"
	@echo "  make clean    - Stop services and remove volumes (WARNING: destroys data)"
	@echo ""
	@echo "Environment Variables:"
	@echo "  POSTGRES_USER     - PostgreSQL username (default: user)"
	@echo "  POSTGRES_PASSWORD - PostgreSQL password (default: password)"
	@echo "  POSTGRES_DB       - PostgreSQL database name (default: agora)"
	@echo "  POSTGRES_PORT     - PostgreSQL port (default: 5432)"
	@echo "  REDIS_PORT        - Redis port (default: 6379)"
	@echo "  STELLAR_RPC_PORT  - Stellar RPC port (default: 8000)"
	@echo "  BACKEND_PORT      - Backend API port (default: 3001)"
	@echo "  FRONTEND_PORT     - Frontend port (default: 3000)"
	@echo "  RUST_LOG          - Rust logging level (default: info)"
	@echo "  NEXT_PUBLIC_API_URL - Frontend API URL (default: http://localhost:3001/api/v1)"

up:
	@echo "Starting Agora development stack..."
	docker compose up --build

down:
	@echo "Stopping Agora development stack..."
	docker compose down

logs:
	docker compose logs -f

clean:
	@echo "WARNING: This will remove all containers and volumes, destroying all data!"
	@echo "Press Ctrl+C to cancel, or wait 5 seconds to continue..."
	@sleep 5
	docker compose down -v
	@echo "Clean complete."