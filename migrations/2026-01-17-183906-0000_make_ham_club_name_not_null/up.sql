-- Your SQL goes here
ALTER TABLE ham_club
    ALTER COLUMN name SET NOT NULL;

ALTER TABLE ham_club
    ADD CONSTRAINT ham_club_name_unique UNIQUE (name);
