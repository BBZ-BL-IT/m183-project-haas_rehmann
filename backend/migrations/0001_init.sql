-- Initial schema for the Grand Casino Rehmann backend.
--
-- A `user` row mirrors an OIDC identity (one row per Kanidm `sub`). The OIDC
-- token stays the source of truth for *authorization* (the admin role); the row
-- here only stores the game state (balance, stats) plus a cached copy of the
-- last seen display name / admin flag for the admin user-list view.

CREATE TABLE IF NOT EXISTS users (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- OIDC subject claim ("sub"); the stable identifier we key users on.
    subject     TEXT        NOT NULL UNIQUE,
    appname     TEXT        NOT NULL,
    email       TEXT,
    balance     BIGINT      NOT NULL DEFAULT 1000,
    total_spent BIGINT      NOT NULL DEFAULT 0,
    total_win   BIGINT      NOT NULL DEFAULT 0,
    -- Cached mirror of the OIDC admin role for the admin user list. Never used
    -- for an authorization decision (the live token claim is).
    is_admin    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Each loan a user takes. Used to enforce the "max 3 loans per day" rule and to
-- compute the total borrowed / outstanding amounts shown in the UI.
CREATE TABLE IF NOT EXISTS loans (
    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id   BIGINT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    amount    BIGINT      NOT NULL CHECK (amount > 0),
    taken_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_loans_user_taken ON loans (user_id, taken_at);

-- Audit trail of every spin. Handy for a security module: it lets us prove the
-- server (not the client) decided every outcome.
CREATE TABLE IF NOT EXISTS spins (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id       BIGINT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    stake         BIGINT      NOT NULL,
    amount_earned BIGINT      NOT NULL,
    reels         INTEGER[]   NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_spins_user_created ON spins (user_id, created_at);
