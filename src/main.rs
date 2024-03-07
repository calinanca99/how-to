use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Parser)]
struct Cli {
    /// Your question. Cannot be used together with `generate`
    prompt: Option<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a configuration file at $HOME/.how_to_config.json.
    /// Overwrites existent config. Cannot be used together with
    /// <PROMPT>.
    Generate { api_key: String },
}

#[derive(Debug, Deserialize, Serialize)]
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
                bail!("Cannot find $HOME directory.")
            }
        };

        let mut config_file = match File::open(config_path) {
            Ok(f) => f,
            Err(_) => {
                bail!("Cannot open the configuration file. Make sure to create one using `how-to generate`.")
            }
        };

        let mut s = String::new();
        config_file
            .read_to_string(&mut s)
            .context("Failed to read from the configuration file")?;

        let config = match serde_json::from_str(&s) {
            Ok(c) => c,
            Err(_) => {
                bail!("Cannot parse the configuration file. Delete the configuration file and create a new one using `how-to generate`.")
            }
        };

        Ok(config)
    }

    /// Create a new configuration file at $HOME/.how_to_config.json.
    ///
    /// It overwrites existent configuration.
    pub fn create(api_key: String) -> anyhow::Result<()> {
        let config_path = match dirs::home_dir() {
            Some(mut home_path) => {
                home_path.push(".how_to_config.json");
                home_path
            }
            None => {
                bail!("Cannot find $HOME directory")
            }
        };

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(config_path)
            .context("Failed to open the configuration file")?;

        let default_config = Configuration {
            api_key,
            prompt: "You are a personal assistant meant to help with using the CLI. You can only answer with the command that is the closest to what the user requires. The response mus be in plain text. You're not allowed to use any form of Markdown (e.g.: ```bash```). Also, if you don't know the command your answer must be: 'sorry, I can't help'.".to_string(),
            model: "gpt-3.5-turbo".to_string(),
        };
        let json = serde_json::to_string_pretty(&default_config)
            .context("Failed to serialize default configuration to JSON")?;
        file.write_all(json.as_bytes())
            .context("Failed to write to the configuration file")?;

        Ok(())
    }
}

// TODO: Refactor
fn handle_prompt(config: &Configuration, prompt: String) -> anyhow::Result<()> {
    let token = format!("Bearer {}", config.api_key);

    let mut headers = HeaderMap::new();
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
    })
    .to_string();
    let response = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .body(body)
        .send()?
        .text()?;

    let v: Value = serde_json::from_str(&response)?;
    let response = &v["choices"][0]["message"]["content"].as_str().unwrap();

    println!("{response}");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli {
            prompt: Some(prompt),
            command: None,
        } => {
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

            handle_prompt(&config, prompt)?;
        }
        Cli {
            prompt: None,
            command: Some(Command::Generate { api_key }),
        } => {
            Configuration::create(api_key)?;
        }
        _ => {
            bail!("Invalid usage. Use either `how-to <PROMPT>` or `how-to generate <API_KEY>`.")
        }
    }

    Ok(())
}
