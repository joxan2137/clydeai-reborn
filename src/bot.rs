
use async_openai::Client as OpenAIClient;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};
use crate::openai;

pub struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_me(&ctx.http).await.unwrap() || msg.content.contains("@Clyde") {
            info!("Request sent. From: {}. Content: {}", msg.author.name, msg.content);
            let prompt = msg.content.replace("<@1233458195960172605>", "");

            match openai::generate_response(&prompt).await {
                Ok(response) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
                        error!("Error sending message: {why:?}");
                    }
                }
                Err(e) => {
                    error!("Failed to generate response: {}", e);
                    if let Err(why) = msg.channel_id.say(&ctx.http, &e).await {
                        error!("Error sending message: {why:?}");
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}