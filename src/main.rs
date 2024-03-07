use std::{env, fs::File, io::Read};

use anyhow::bail;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE},
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub api_key: String,
    pub prompt: String,
    pub model: String,
}

impl Configuration {
    /// Attempts to load the configuration file from $HOME/.how_to_config.json.
    ///
    /// The configuration file must be present for the CLI to run.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = match dirs::home_dir() {
            Some(mut home_path) => {
                home_path.push(".how_to_config.json");
                home_path
            }
            None => {
                bail!("Cannot find $HOME directory")
            }
        };

        let mut config_file = match File::open(config_path) {
            Ok(f) => f,
            Err(_) => {
                bail!("Cannot open the configuration file. Make sure to create one using `how-to generate`.")
            }
        };

        let mut s = String::new();
        config_file.read_to_string(&mut s)?;

        let config = match serde_json::from_str(&s) {
            Ok(c) => c,
            Err(_) => {
                bail!("Cannot parse the configuration file. Delete the configuration file and create a new one using `how-to generate`.")
            }
        };

        Ok(config)
    }
}

fn main() -> anyhow::Result<()> {
    let mut config = Configuration::load()?;

    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        config.api_key = api_key;
    };
    if let Ok(prompt) = env::var("GPT_PROMPT") {
        config.prompt = prompt;
    };
    if let Ok(model) = env::var("GPT_MODEL") {
        config.model = model;
    };

    let args = std::env::args().collect::<Vec<_>>();
    let prompt = if let Some(p) = args.get(1) {
        p
    } else {
        eprintln!("Usage: how-to \"PROMPT\"");
        return Ok(());
    };

    let mut headers = HeaderMap::new();
    let token = format!("Bearer {}", config.api_key);
    headers.insert(AUTHORIZATION, token.parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    let body = serde_json::json!({
        "model": config.model,
        "messages": [
          {
            "role": "system",
            "content": config.prompt
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

    Ok(())
}
