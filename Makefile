.PHONY: all test

# This builds all targets
all:
	cargo build --all-targets

# This runs all unit and integration tests
test:
	cargo test

# This drops and initializes the database
db-init:
	cat init.sql | bin/psql -U admin postgres
	diesel migration run
