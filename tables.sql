-- PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS connected_accounts;
DROP TABLE IF EXISTS discord_tokens;
DROP TABLE IF EXISTS authorized_discord_users;
DROP TABLE IF EXISTS authorized_dsek_users;

-- PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS authorized_discord_users (
    user_id         TEXT NOT NULL,
    username        TEXT NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE TABLE IF NOT EXISTS authorized_dsek_users (
    stil_id         TEXT NOT NULL,
    name            TEXT NOT NULL,
    PRIMARY KEY (stil_id)
);

CREATE TABLE IF NOT EXISTS discord_tokens (
    user_id         TEXT NOT NULL,
    access_token    TEXT NOT NULL,
    refresh_token   TEXT NOT NULL,
    expires_at      BIGINT NOT NULL,
    PRIMARY KEY (user_id),
    FOREIGN KEY (user_id) REFERENCES authorized_discord_users(user_id)
);

CREATE TABLE IF NOT EXISTS connected_accounts (
    user_id         TEXT NOT NULL,
    stil_id         TEXT NOT NULL,
    PRIMARY KEY (stil_id),
    FOREIGN KEY (user_id) REFERENCES authorized_discord_users(user_id),
    FOREIGN KEY (stil_id) REFERENCES authorized_dsek_users(stil_id)
);
