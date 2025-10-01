use crate::env;
use serde_json::json;

#[tokio::main]
pub async fn start() {
    let client_id = env::var("DISCORD_CLIENT_ID");
    let token = env::var("DISCORD_BOT_TOKEN");

    let metadata = json!({});

    let endpoint =
        format!("https://discord.com/api/v10/applications/{client_id}/role-connections/metadata");

    let client = reqwest::Client::new();
    let res = client
        .put(&endpoint)
        .json(&metadata)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {token}"))
        .send()
        .await;

    if let Ok(res) = res {
        if res.status().is_success() {
            println!("SUCCESS: {}", res.text().await.unwrap());
        } else {
            println!("ERROR: {}", res.text().await.unwrap());
        }
    }
}
