CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS clients (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name              TEXT         NOT NULL,
    api_key_hash      TEXT         NOT NULL UNIQUE,
    vapid_private_key TEXT         NOT NULL,
    vapid_public_key  TEXT         NOT NULL,
    vapid_subject     TEXT         NOT NULL DEFAULT 'mailto:admin@example.com',
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_clients_api_key_hash ON clients(api_key_hash);
