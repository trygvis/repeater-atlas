-- This file should undo anything in `up.sql`
ALTER TABLE ham_club
    ALTER COLUMN name DROP NOT NULL;

ALTER TABLE ham_club
    DROP CONSTRAINT ham_club_name_unique;
