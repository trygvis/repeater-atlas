CREATE TABLE ham_club
(
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT,
    description TEXT
);

CREATE TABLE repeater
(
    id        BIGSERIAL PRIMARY KEY,
    ham_club  BIGINT REFERENCES ham_club,
    call_sign TEXT
);

CREATE TABLE repeater_change_log
(
    id         BIGSERIAL PRIMARY KEY,
    repeater   BIGINT REFERENCES repeater,
    created_at TIMESTAMP,
    body       TEXT
);

CREATE TABLE ham_operator
(
    id   BIGSERIAL PRIMARY KEY,
    name TEXT
);
