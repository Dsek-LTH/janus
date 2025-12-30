FROM rust:1.92 as builder

LABEL se.dsek.env.required="DSEK_CLIENT_ID, DSEK_CLIENT_SECRET, DSEK_REDIRECT_URI, DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET, DISCORD_REDIRECT_URI, DISCORD_BOT_TOKEN, COOKIE_SECRET, DATABASE_URL"


LABEL se.dsek.env.dsek_client_id.description="dsek (authentik) provider client id."
LABEL se.dsek.env.dsek_client_secret.description="dsek (authentik) provider client secret."
LABEL se.dsek.env.dsek_redirect_uri.description="Our full redirect uri (with https://). Make sure this is also entered into the authentik provider so it knows where to send users."

LABEL se.dsek.env.discord_client_id.description="Discord client id (also called application id), from the OAuth2 page of the discord application."
LABEL se.dsek.env.discord_client_secret.description="Discord client secret, right next to the client id."
LABEL se.dsek.env.discord_redirect_uri.description="Our full redirect uri (with https://). Make sure this is entered into the discord redirects under the OAuth2 page so discord knows where to send users."
LABEL se.dsek.env.discord_bot_token.description="Discord bot token from the Bot page."

LABEL se.dsek.env.cookie_secret.description="Secret for generating cookies (can sort of be whatever)"
LABEL se.dsek.env.database_url.description="url for postgres database"


RUN mkdir --parents /app/.sqlx && mkdir /app/src 
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src/

RUN cargo build --release

# ---------- runtime stage ----------
FROM gcr.io/distroless/cc-debian12@sha256:0c8eac8ea42a167255d03c3ba6dfad2989c15427ed93d16c53ef9706ea4691df
COPY --from=builder /app/target/release/janus /janus

USER nonroot:nonroot

EXPOSE 3000

CMD ["/janus"]
