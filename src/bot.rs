use std::collections::HashMap;
use std::sync::Arc;

use serenity::all::{ActivityData, ChannelId, GuildId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};
use async_openai::Chat;
use async_openai::types::{CreateChatCompletionRequest, ChatCompletionRequestMessage, Role};
use crate::openai;
use async_openai::Client as OpenAIClient;

struct ChatSession {
    channel_id: ChannelId,
    messages: Vec<ChatCompletionRequestMessage>,
}
pub struct Bot {
    chat_sessions: Arc<Mutex<HashMap<ChannelId, ChatSession>>>,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions_me(&ctx.http).await.unwrap() || msg.content.contains("@Clyde"){
            let channel_id = msg.channel_id;
            let mut chat_sessions = self.chat_sessions.lock().await;
            let openai_client = OpenAIClient::new();

            let chat_session = chat_sessions.entry(channel_id).or_insert(ChatSession {
                channel_id,
                messages: Vec::new(),            });

            let user_message = ChatCompletionRequestMessage {
                role: Role::User,
                content: msg.content.clone(),
            };

            chat_session.messages.push(user_message);

            let request = CreateChatCompletionRequest::Default {
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: Some(512_u16),
                stream: Some(false),
                user: Some(msg.author.name.clone()),
                messages: chat_session.messages.clone(),
            };

            let response = openai_client.chat().create(request).await;

            match response {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        let assistant_message = ChatCompletionRequestMessage {
                            role: Role::Assistant,
                            content: choice.message.content.clone(),
                        };
                        chat_session.messages.push(assistant_message);

                        if let Err(why) = msg.channel_id.say(&ctx.http, &choice.message.content).await {
                            error!("Error sending message: {why:?}");
                        }
                    } else {
                        info!("No choices found in the response");
                    }
                }
                Err(e) => {
                    error!("Failed to generate response: {}", e);
                    msg.reply(&ctx.http, format!("Failed to generate response: {}", e)).await.unwrap();
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_activity(Some(ActivityData::competing("the worst AI assistant competition")));
    }
}