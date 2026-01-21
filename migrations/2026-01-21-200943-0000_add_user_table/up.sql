-- Your SQL goes here
CREATE TABLE app_user
(
    id            BIGSERIAL PRIMARY KEY,
    call_sign     TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
