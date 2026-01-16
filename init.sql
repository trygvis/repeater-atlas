DROP DATABASE IF EXISTS repeater_atlas WITH (FORCE);
DROP OWNED BY repeater_atlas CASCADE;
DROP ROLE IF EXISTS repeater_atlas;

CREATE ROLE repeater_atlas LOGIN PASSWORD 'repeater_atlas';
CREATE DATABASE repeater_atlas OWNER repeater_atlas;
