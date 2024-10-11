run:
	RUST_LOG=debug cargo run
db-migrate:
	DATABASE_URL=postgres://postgres:postgrespassword@localhost:5432 sea-orm-cli migrate up
build:
	cargo build
