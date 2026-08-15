CREATE TABLE IF NOT EXISTS users
(
    id            UUID PRIMARY KEY     DEFAULT gen_random_uuid(),
    username      TEXT        NOT NULL UNIQUE,
    email         TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_name_len CHECK (length(username) <= 255),
    CONSTRAINT chk_email_len CHECK (length(email) <= 255),
    CONSTRAINT chk_password_hash_len CHECK (length(password_hash) <= 255)
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);