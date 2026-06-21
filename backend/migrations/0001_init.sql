-- Schema for the Grand Casino Rehmann backend, split into entities.
--
--   users         – identity row, one per Kanidm `sub`. Parent of the others.
--   bank_accounts – the player's money (1:1 with users).
--   stats         – aggregated play statistics (1:1 with users).
--   loans         – individual loans (drives the per-window loan limit).
--   spins         – audit trail of every spin.

CREATE TABLE IF NOT EXISTS users (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- OIDC subject claim ("sub"); the stable identifier we key users on.
    subject     TEXT        NOT NULL UNIQUE,
    -- Account name (initially the Kanidm name; validated: <=20 chars,
    -- A-Z a-z 0-9 . _ -). Admin-editable local copy.
    username    TEXT        NOT NULL,
    email       TEXT,
    -- Cached mirror of the OIDC admin role for the admin list. Authorization
    -- always uses the live token claim, never this.
    is_admin    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS bank_accounts (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id    BIGINT      NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    amount     BIGINT      NOT NULL DEFAULT 1000,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS stats (
    id                 BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id            BIGINT      NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    total_spent        BIGINT      NOT NULL DEFAULT 0,
    -- Net profit; may be negative.
    total_profit       BIGINT      NOT NULL DEFAULT 0,
    highest_win_streak INTEGER     NOT NULL DEFAULT 0,
    -- Running streak used to compute the highest.
    current_win_streak INTEGER     NOT NULL DEFAULT 0,
    -- Lifetime number of loans taken and their total value (open/owed).
    loans_taken        BIGINT      NOT NULL DEFAULT 0,
    loans_value        BIGINT      NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Each loan row. Used to enforce "max N loans per rolling window" and to show a
-- countdown until the next loan slot frees up.
CREATE TABLE IF NOT EXISTS loans (
    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id   BIGINT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    amount    BIGINT      NOT NULL CHECK (amount > 0),
    taken_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_loans_user_taken ON loans (user_id, taken_at);

CREATE TABLE IF NOT EXISTS spins (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id       BIGINT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    stake         BIGINT      NOT NULL,
    amount_earned BIGINT      NOT NULL,
    reels         INTEGER[]   NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_spins_user_created ON spins (user_id, created_at);
