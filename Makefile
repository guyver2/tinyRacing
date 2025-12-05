# Makefile for managing the Rust backend, Vue.js frontend, and PostgreSQL Docker setup

# Variables
VUE_DIR=tiny-racing-vue
CARGO_DIR=server
DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing

.PHONY: help db-up db-down db-shell run-sim run-vue seed-db db-wipe

help:
	@echo "Available make targets:"
	@echo "  db-up         Start the PostgreSQL Docker container"
	@echo "  db-down       Stop and remove PostgreSQL Docker container"
	@echo "  db-shell      psql shell into the running database"
	@echo "  db-seed       Seed the database using the seed script"
	@echo "  db-seed-randomize Seed the database using the seed script with randomization"
	@echo "  db-wipe       Delete all database content (tables, schema, everything)"
	@echo "  run-sim       Start the Rust backend"
	@echo "  run-vue       Start the Vue.js frontend"
	@echo "  format        Format the code using prettier and cargo fmt"
	@echo "  clean         Clean the Vue.js frontend and Rust backend"
	@echo "  clean-all     Clean the Vue.js frontend and Rust backend and remove the PostgreSQL Docker container"

db-up:
	@if [ "$$(docker ps -q -f name=tiny_racing_db)" = "" ]; then \
		docker compose up -d; \
	else \
		echo "PostgreSQL container already running."; \
	fi

db-down:
	docker compose down

db-migrate:
	cd $(CARGO_DIR) && cargo run --example run_migrations

db-migrate-down:
	cd $(CARGO_DIR) && cargo run --example run_migrations down

db-shell:
	docker exec -it tiny_racing_db psql -U $(DB_USER) -d $(DB_NAME)

db-seed:
	cd $(CARGO_DIR) && cargo run --example seed_db

db-seed-randomize:
	cd $(CARGO_DIR) && cargo run --example seed_db -- randomize

db-wipe:
	@echo "⚠️  WARNING: This will delete ALL database content (tables, schemas, everything)!"
	@echo -n "Are you sure? [y/N] " && read ans && [ $${ans:-N} = y ]
	docker exec -i tiny_racing_db psql -U tiny_racing -d postgres -c "DROP DATABASE IF EXISTS tiny_racing;"
	docker exec -i tiny_racing_db psql -U tiny_racing -d postgres -c "CREATE DATABASE tiny_racing OWNER tiny_racing;"
	@echo "✅ Database wiped successfully. Run 'make db-migrate' to recreate the schema."

db-export-sqlx:
	cd $(CARGO_DIR) && cargo sqlx prepare --database-url $(DATABASE_URL)

run-sim:
	cd $(CARGO_DIR) && cargo run -- ../assets/race.json

run-vue:
	cd $(VUE_DIR) && npm install && npm run dev

format:
	cd $(VUE_DIR) && npm run format
	cd $(CARGO_DIR) && cargo fmt

clean:
	cd $(VUE_DIR) && npm run clean
	cd $(CARGO_DIR) && cargo clean

clean-all: clean
	docker compose down