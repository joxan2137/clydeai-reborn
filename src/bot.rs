use std::collections::HashMap;
use std::sync::Arc;

use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequestArgs, Role,
};
use async_openai::Client as OpenAIClient;
use serenity::all::{ActivityData, ChannelId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

pub struct ChatSession {
    channel_id: ChannelId,
    messages: Vec<ChatCompletionRequestMessage>,
}
pub struct Bot {
    pub chat_sessions: Arc<Mutex<HashMap<ChannelId, ChatSession>>>,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg
            .mentions_me(&ctx.http)
            .await
            .expect("jakims cudem nie wiadomo czy ktos cie zpingowal xd")
            || msg.content.contains("@Clyde")
        {
            let channel_id = msg.channel_id;
            let mut chat_sessions = self.chat_sessions.lock().await;
            let openai_client = OpenAIClient::new();

            let chat_session = chat_sessions.entry(channel_id).or_insert(ChatSession {
                channel_id,
                messages: Vec::new(),
            });

            let user_message =
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::from(msg.content.clone()),
                    role: Role::User,
                    name: None,
                });
            chat_session.messages.push(user_message);

            let request = CreateChatCompletionRequestArgs::default()
                .model("gpt-3.5-turbo".to_string())
                .messages(chat_session.messages.clone())
                // Add other request parameters here
                .build()
                .unwrap();

            let response = openai_client.chat().create(request).await;

            match response {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        let assistant_message = ChatCompletionRequestMessage::Assistant(
                            ChatCompletionRequestAssistantMessage::default(),
                        );

                        chat_session.messages.push(assistant_message);

                        if let Err(why) = msg
                            .reply(
                                &ctx.http,
                                choice.message.content.clone().unwrap_or_default(),
                            )
                            .await
                        {
                            error!("Error sending message: {why:?}");
                        }
                    } else {
                        info!("No choices found in the response");
                    }
                }
                Err(e) => {
                    error!("Failed to generate response: {}", e);
                    msg.reply(&ctx.http, format!("Failed to generate response: {}", e))
                        .await
                        .unwrap();
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_activity(Some(ActivityData::competing(
            "the worst AI assistant competition",
        )));
    }
}