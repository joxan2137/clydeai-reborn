use async_openai::Client as OpenAIClient;
use async_openai::config::OpenAIConfig;
use async_openai::types::CreateCompletionRequestArgs;
use tracing::{error, info};

pub async fn generate_response(prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    let config = OpenAIConfig::new().with_api_key(&api_key);
    let openai_client = OpenAIClient::with_config(config);

    let request = CreateCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .prompt(prompt)
        .max_tokens(512_u16)
        .build()
        .expect("Failed to build CreateCompletionRequestArgs");

    match openai_client.completions().create(request).await {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                Ok(choice.text.clone())
            } else {
                info!("No choices found in the response");
                Err("No choices found in the response".to_string())
            }
        }
        Err(e) => {
            error!("Failed to get response: {}", e);
            if e.to_string().contains("quota") {
                Err("The developer stopped paying for the API, so I can't respond anymore.".to_string())
            } else {
                Err(format!("Failed to get response: {}", e))
            }
        }
    }
}
