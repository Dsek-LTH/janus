# Janus

Service to link Discord accounts with [Dsek](https://dsek.se) accounts using [Linked Roles](https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Linked-Roles-Community-Members)

## Setup

1. Use Rust nightly
2. Initialize sqlite database with `sqlite3 database.db < tables.sql`
3. Populate the example.env file with info from your discord app and rename it to .env

It is recommended to use [ngrok](https://ngrok.com) to properly forward traffic whilst testing.

## Sequence diagram

The app performs a double OAuth flow (first to Discord, then to
Dsek/Authentik). Shown below is the sequence diagram for the flow, including
storage of tokens and user info in the database.

```mermaid
sequenceDiagram
    participant User as User (Browser)
    participant Janus as Janus Server
    participant Discord as Discord OAuth API
    participant Dsek as Dsek Aukthetik OAuth API
    participant DB as Database

    User->>Janus: GET /linked-role
    Note over Janus: Generate UUID state
    Note over Janus: Store state in session
    Janus-->>User: Redirect to Discord OAuth URL

    User->>Discord: Authenticate & Authorize
    Discord-->>User: Redirect to /discord-oauth-callback with code

    User->>Janus: GET /discord-oauth-callback?code=xxx&state=yyy
    Note over Janus: Verify state matches session
    Janus->>Discord: POST /oauth2/token with code
    Discord-->>Janus: Return access_token, refresh_token

    Janus->>Discord: GET /oauth2/@me with access_token
    Discord-->>Janus: Return user data (user_id, username)

    Janus->>DB: Store Discord user data
    Janus->>DB: Store Discord tokens
    Note over Janus: Store Discord user_id in session
    Note over Janus: Generate new UUID state for Dsek OAuth
    Janus-->>User: Redirect to Dsek OAuth URL

    User->>Dsek: Authenticate & Authorize
    Dsek-->>User: Redirect to /dsek-oauth-callback with code

    User->>Janus: GET /dsek-oauth-callback?code=xxx&state=yyy
    Note over Janus: Verify state matches session
    Janus->>Dsek: POST /o/token/ with code
    Dsek-->>Janus: Return id_token (JWT)
    Note over Janus: Decode JWT to extract user data

    Janus->>DB: Store Dsek user data
    Janus->>DB: Connect Discord and Dsek accounts

    DB-->>Janus: Get Discord token
    Note over Janus: Check if token expired
    alt Token expired
        Janus->>Discord: Refresh token
        Discord-->>Janus: New tokens
        Janus->>DB: Update tokens
    end

    Janus->>Discord: PUT /users/@me/applications/{client_id}/role-connection
    Discord-->>Janus: Success response

    Janus-->>User: Display success message
```

## TODOs

- [ ] Better (read: existing) logging
- [ ] Better error messages (probably using [anyhow](https://docs.rs/anyhow/latest/anyhow/)?)
- [ ] SQLite -> Postgres
