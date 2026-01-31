CREATE TYPE CALL_SIGN_KIND AS ENUM (
  'repeater',
  'contact'
  );

CREATE TYPE CONTACT_KIND AS ENUM (
  'organization',
  'individual'
  );

-- Global index of call signs. Each call sign is either a repeater or a contact.
CREATE TABLE call_sign
(
  value     TEXT PRIMARY KEY,
  kind      CALL_SIGN_KIND NOT NULL,

  CHECK (value = UPPER(value))
);

-- A contact can be either an organization or an individual, with optional contact fields.
CREATE TABLE contact
(
  id           BIGSERIAL PRIMARY KEY,
  call_sign    TEXT         UNIQUE REFERENCES call_sign (value) ON DELETE CASCADE,
  kind         CONTACT_KIND NOT NULL,
  display_name TEXT         NOT NULL,
  description  TEXT,
  web_url      TEXT,
  email        TEXT,
  phone        TEXT,
  address      TEXT
);
