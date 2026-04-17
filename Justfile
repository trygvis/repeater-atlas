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

# Build /static/vendor, keeps packages under node_modules for tools to index
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
  mkdir -p static/vendor/lucide/icons
  mkdir -p static/vendor/lucide/shared/src/utils
  cp node_modules/lucide/dist/esm/createElement.js static/vendor/lucide/
  cp node_modules/lucide/dist/esm/defaultAttributes.js static/vendor/lucide/
  cp node_modules/lucide/dist/esm/replaceElement.js static/vendor/lucide/
  cp node_modules/lucide/dist/esm/iconsAndAliases.js static/vendor/lucide/
  cp node_modules/lucide/dist/esm/icons/chevron-left.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/chevron-right.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/crosshair.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/map.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/pencil.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/search.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/trash-2.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/user.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/icons/x.js static/vendor/lucide/icons/
  cp node_modules/lucide/dist/esm/shared/src/utils/hasA11yProp.js static/vendor/lucide/shared/src/utils/
  cp node_modules/lucide/dist/esm/shared/src/utils/mergeClasses.js static/vendor/lucide/shared/src/utils/
  cp node_modules/lucide/dist/esm/shared/src/utils/toCamelCase.js static/vendor/lucide/shared/src/utils/
  cp node_modules/lucide/dist/esm/shared/src/utils/toPascalCase.js static/vendor/lucide/shared/src/utils/

# Build assets, but remove anything we don't need
assets-ci: assets
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

# Runs the repeater-atlas application in the foreground.
run: assets
  cargo run --bin repeater-atlas

# Builds a Docker image of this repository into 'repeater-atlas:latest'
docker-image:
  docker build -t repeater-atlas:latest .
