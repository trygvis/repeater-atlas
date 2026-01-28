CREATE TYPE ENTITY_KIND AS ENUM (
  'repeater',
  'contact'
  );

CREATE TYPE CONTACT_KIND AS ENUM (
  'organization',
  'individual'
  );

-- Global index of call signs. Each entity is either a repeater or a contact.
CREATE TABLE entity
(
  id        BIGSERIAL PRIMARY KEY,
  kind      ENTITY_KIND NOT NULL,
  call_sign TEXT UNIQUE,

  CHECK (call_sign = UPPER(call_sign)),
  CHECK (kind IS DISTINCT FROM 'repeater' OR call_sign IS NOT NULL)
);

-- A contact can be either an organization or an individual, with optional contact fields.
CREATE TABLE contact
(
  id           BIGSERIAL PRIMARY KEY,
  entity       BIGINT       NOT NULL UNIQUE REFERENCES entity (id) ON DELETE CASCADE,
  kind         CONTACT_KIND NOT NULL,
  display_name TEXT         NOT NULL,
  description  TEXT,
  web_url      TEXT,
  email        TEXT,
  phone        TEXT,
  address      TEXT
);
