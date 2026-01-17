.PHONY: all test assets

# This builds all targets
all:
	cargo build --all-targets

# This runs all unit and integration tests
test:
	cargo test

# This copies HTMX into static assets
assets:
	npm ci
	rm -rf static/vendor
	mkdir -p static/vendor
	cp node_modules/htmx.org/dist/htmx.min.js static/vendor/htmx.min.js
	rm -rf node_modules

# This drops and initializes the database
db-init:
	cat init.sql | bin/psql -U admin postgres
	diesel migration run
