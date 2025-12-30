# Janus

Service to link Discord accounts with [Dsek](https://dsek.se) accounts using [Linked Roles](https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Linked-Roles-Community-Members)

## Setup (with podman compose)

1. Copy example.env to .env and fill it in
2. Configure the bot in Authentik and on the discord developer portal
3. Run `podman run -it janus /janus -r` to register the bot (this
   configures the discord application. If this has already been done you can
   skip this step).
4. Run `podman run -it --network host  --env-file ./.env janus /janus -s` to
   set up the database.
5. Run `podman compose up` to get the database and the bot up and running.

It is recommended to use [ngrok](https://ngrok.com) to properly forward traffic
whilst testing.

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
- [x] SQLite -> Postgres
- [ ] Fix containerfile so dependencies can be cached and not rebuild every time.
