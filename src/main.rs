mod bot;
mod openai;

use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use shuttle_serenity::{SerenityService, ShuttleSerenity};

struct Bot;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> ShuttleSerenity {
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow::anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    if let Some(api_key) = secret_store.get("OPENAI_API_KEY") {
        std::env::set_var("OPENAI_API_KEY", api_key);
    } else {
        return Err(anyhow::anyhow!("'OPENAI_API_KEY' was not found").into());
    }

    // Set gateway intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(bot::Bot)
        .await
        .expect("Error creating client");

    Ok(SerenityService::from(client))
}