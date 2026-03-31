set shell := ["bash", "-cu"]
set quiet := true

# Show available recipes and agent notes.
default: help

help:
  @just --list --unsorted
  @echo ""
  @echo "Notes for agents:"
  @echo "  - Run via: just <recipe>"
  @echo "  - For change validation: just all, just test, just db-init, just db-setup"
  @echo "  - For commands that require non-sandbox access, make sure to request."

# Build all Rust targets. Requires non-sandbox access.
all:
  cargo build --all-targets

# Run all unit and integration tests. Requires non-sandbox access.
test:
  cargo test

# Copy assets from node packages into static/vendor.
assets:
  npm ci
  rm -rf static/vendor
  mkdir -p static/vendor
  mkdir -p static/vendor/leaflet
  mkdir -p static/vendor/leaflet.markercluster
  cp node_modules/htmx.org/dist/htmx.min.js static/vendor/htmx.min.js
  cp node_modules/@picocss/pico/css/pico.css static/vendor/pico.css
  cp node_modules/leaflet/dist/leaflet.js static/vendor/leaflet/leaflet.js
  cp node_modules/leaflet/dist/leaflet.css static/vendor/leaflet/leaflet.css
  cp -r node_modules/leaflet/dist/images static/vendor/leaflet/images
  cp node_modules/leaflet.markercluster/dist/leaflet.markercluster.js \
    static/vendor/leaflet.markercluster/leaflet.markercluster.js
  cp node_modules/leaflet.markercluster/dist/MarkerCluster.css \
    static/vendor/leaflet.markercluster/MarkerCluster.css
  cp node_modules/leaflet.markercluster/dist/MarkerCluster.Default.css \
    static/vendor/leaflet.markercluster/MarkerCluster.Default.css
  rm -rf node_modules

# Drop and initialize the database, then export the schema. Requires non-sandbox access.
db-init:
  cat init.sql | PGPASSWORD=admin bin/psql -U admin postgres
  diesel migration run
  just db-export-schema

# Export the schema to schema.tmp.sql for inspection. Requires non-sandbox access.
db-export-schema:
  docker compose exec -it postgres pg_dump -U admin -d repeater_atlas --schema-only > schema.tmp.sql

# Load fixture data into the database. Requires non-sandbox access.
db-load-data:
  cargo run --bin load-data

# Re-initialize the database and load data. Requires non-sandbox access.
db-setup:
  just db-init
  just db-load-data

# Builds a Docker image of this repository into 'repeater-atlas:latest'
docker-image:
  docker build -t repeater-atlas:latest .
