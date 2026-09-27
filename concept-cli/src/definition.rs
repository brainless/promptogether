use crate::posts::Match;
use llm_sdk::xiaomi::XiaomiClient;

const SYSTEM_PROMPT: &str = include_str!("../prompts/definition.txt");

pub async fn generate(
    term: &str,
    matches: &[Match],
    api_key: &str,
    model: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let context = matches
        .iter()
        .take(8)
        .map(|item| format!("Post {}: {}", item.slug, item.snippet))
        .collect::<Vec<_>>()
        .join("\n\n");
    let request = format!("Term: {term}\n\nPost excerpts:\n{context}");
    let client = XiaomiClient::new(api_key)?;
    let response = client
        .message_builder()
        .model(model)
        .system_message(SYSTEM_PROMPT)
        .user_message(&request)
        .max_completion_tokens(300)
        .send()
        .await?;
    let choice = response
        .choices
        .first()
        .ok_or("model returned no choices")?;
    if choice.finish_reason.as_deref() != Some("stop") {
        return Err(format!(
            "model did not finish (finish_reason: {})",
            choice.finish_reason.as_deref().unwrap_or("missing")
        )
        .into());
    }
    let definition = choice.message.content.as_deref().unwrap_or("").trim();
    if definition.is_empty() || definition.len() > 600 || definition.contains('\n') {
        return Err(
            "model returned an empty, long, or multiline definition; nothing was saved".into(),
        );
    }
    Ok(definition.to_string())
}
