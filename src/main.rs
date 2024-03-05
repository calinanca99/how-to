use std::env;

use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::Value;

fn main() {
    let api_key = env::var("OPENAI_API_KEY").unwrap();

    let args = std::env::args().collect::<Vec<_>>();
    let prompt = if let Some(p) = args.get(1) {
        p
    } else {
        eprintln!("Usage: how-to \"PROMPT\"");
        return;
    };

    let mut headers = HeaderMap::new();
    let token = format!("Bearer {api_key}");
    headers.insert(AUTHORIZATION, token.parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    let body = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
          {
            "role": "system",
            "content": "
            You are a personal assistant that is a master of the Linux terminal and CLI tools.
            
            For each question you answer with exactly one command. The output must be plain-text that can be copy and pasted. 
            Markdown is not allowed.
            "
          },
          {
            "role": "user",
            "content": prompt
          }
        ]
    });
    let body = body.to_string();
    let response = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .body(body)
        .send()
        .unwrap()
        .text()
        .unwrap();

    let v: Value = serde_json::from_str(&response).unwrap();
    let response = &v["choices"][0]["message"]["content"].as_str().unwrap();

    println!("{response}");
}
