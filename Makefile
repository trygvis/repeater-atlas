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
	mkdir -p static/vendor/leaflet
	cp node_modules/htmx.org/dist/htmx.min.js static/vendor/htmx.min.js
	cp node_modules/@picocss/pico/css/pico.min.css static/vendor/pico.min.css
	cp node_modules/leaflet/dist/leaflet.js static/vendor/leaflet/leaflet.js
	cp node_modules/leaflet/dist/leaflet.css static/vendor/leaflet/leaflet.css
	cp -r node_modules/leaflet/dist/images static/vendor/leaflet/images
	rm -rf node_modules

# This drops and initializes the database
db-init:
	cat init.sql | PGPASSWORD=admin bin/psql -U admin postgres
	diesel migration run

# Exports the schema to schema.tmp.sql. This is a complete dump of the current database schema to make it easy to get
# a complete overview over the complete schema.
db-export-schema:
	docker compose exec -it postgres pg_dump -U admin -d repeater_atlas --schema-only > schema.tmp.sql
