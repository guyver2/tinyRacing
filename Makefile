# Makefile for managing the Rust backend, Vue.js frontend, and PostgreSQL Docker setup

# Variables
VUE_DIR=tiny-racing-vue
CARGO_DIR=server

.PHONY: help db-up db-down db-shell run-sim run-vue seed-db

help:
	@echo "Available make targets:"
	@echo "  db-up         Start the PostgreSQL Docker container"
	@echo "  db-down       Stop and remove PostgreSQL Docker container"
	@echo "  db-shell      psql shell into the running database"
	@echo "  db-seed       Seed the database using the seed script"
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

db-shell:
	docker exec -it tiny_racing_db psql -U $(DB_USER) -d $(DB_NAME)

db-seed:
	cd $(CARGO_DIR) && cargo run --example seed_db

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