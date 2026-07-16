CREATE TABLE IF NOT EXISTS subscriptions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id   UUID         NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    endpoint    TEXT         NOT NULL,
    p256dh      TEXT         NOT NULL,
    auth        TEXT         NOT NULL,
    user_agent  TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (client_id, endpoint)
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_client_id ON subscriptions(client_id);
