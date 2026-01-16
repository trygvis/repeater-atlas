DROP DATABASE IF EXISTS relay_atlas WITH (FORCE);
DROP OWNED BY relay_atlas CASCADE;
DROP ROLE IF EXISTS relay_atlas;

CREATE ROLE relay_atlas LOGIN PASSWORD 'relay_atlas';
CREATE DATABASE relay_atlas OWNER relay_atlas;
