use std::env;
use serde::{Deserialize, Serialize};
use serde_json;
use reqwest;

#[derive(Serialize, Deserialize, Debug)]
struct LlmResponse {
    candidates: Option<Vec<Candidate>>,
    usage_metadata: Option<UsageMetadata>,
    model_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Candidate {
    content: Option<Content>,
    finish_reason: Option<String>,
    avg_logprobs: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    parts: Option<Vec<Part>>,
    role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Part {
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UsageMetadata {
    prompt_token_count: Option<i32>,
    candidates_token_count: Option<i32>,
    total_token_count: Option<i32>,
    prompt_tokens_details: Option<Vec<TokenDetails>>,
    candidates_tokens_details: Option<Vec<TokenDetails>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenDetails {
    modality: Option<String>,
    token_count: Option<i32>,
}

pub async fn call_llm_api(prompt: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key);

    let client = reqwest::Client::new();
    let json_body = format!(r#"{{"contents": [{{"parts":[{{"text": "{}"}}]}}]}}"#, prompt);

    let response = client.post(url)
        .header("Content-Type", "application/json")
        .body(json_body)
        .send()
        .await?;

    let response_body = response.text().await?;
    let llm_response: Result<LlmResponse, serde_json::Error> = serde_json::from_str(&response_body);

    match llm_response {
        Ok(llm_response) => {
            let response_text = llm_response.candidates
                .and_then(|candidates| candidates.into_iter().next())
                .and_then(|candidate| candidate.content)
                .and_then(|content| content.parts)
                .and_then(|parts| parts.into_iter().next())
                .and_then(|part| part.text)
                .unwrap_or_else(|| "No response from LLM".to_string());
            Ok(response_text)
        }
        Err(e) => {
            eprintln!("Failed to parse LLM response: {}", e);
            Ok("Failed to parse LLM response".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_call_llm_api() {
        // This test requires the GEMINI_API_KEY environment variable to be set.
        // It also depends on the availability of the Gemini API.
        // Consider mocking the API call for more reliable tests.

        if env::var("GEMINI_API_KEY").is_ok() {
            let prompt = "What is the capital of France?";
            let result = call_llm_api(prompt).await;
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(!response.is_empty());
            println!("{}", response);
        } else {
            println!("Skipping test_call_llm_api because GEMINI_API_KEY is not set");
        }
    }
}
