run:
	RUST_LOG=debug cargo run
db-migrate:
	DATABASE_URL=postgres://username:password@localhost:5432 sea-orm-cli migrate up
build:
	cargo build
