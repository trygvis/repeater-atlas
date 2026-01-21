CREATE TABLE ham_club
(
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    web_url     TEXT,
    email       TEXT
);
