use std::fs;
use std::io::Write;
use reqwest;
use serde_json::{Value};
use rpassword::read_password;
use serde::Serialize;
use clap::Parser;

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    messages: Vec<Message>,
    temperature: f32,
    top_p: f32,
    max_tokens: i32,
}

#[derive(Parser)]
#[command(name = "ai")]
#[command(about = "A command line tool for interacting with Azure OpenAI services")]
#[command(after_help = "\
CREDENTIALS:
    The tool securely stores the following credentials:
    - Azure OpenAI API Key
    - Azure OpenAI Endpoint
    - Azure OpenAI Deployment Name

    On first launch, you will be prompted to enter these credentials.
    They will be stored securely in the system keyring for future use.
    Use --delete-keys to remove stored credentials.")]
struct Cli {
    /// Input to process - either a file path or direct text
    #[arg(index = 1)]
    input: Option<String>,

    /// System prompt from file or direct text
    #[arg(long, value_name = "PROMPT")]
    prompt: Option<String>,

    /// Delete all stored credentials
    #[arg(long)]
    delete_keys: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.delete_keys {
        delete_credentials()?;
        println!("All credentials deleted from secure storage.");
        return Ok(());
    }

    let input = match cli.input {
        Some(input_arg) => fs::read_to_string(&input_arg).unwrap_or_else(|_| input_arg),
        None => {
            let _ = Cli::parse_from(&["ai", "--help"]);
            std::process::exit(1);
        }
    };

    let system_prompt = match cli.prompt {
        Some(prompt_arg) => fs::read_to_string(&prompt_arg).unwrap_or_else(|_| prompt_arg),
        None => String::from("You are an AI assistant that helps people find information.")
    };

    // Read from a secure credential store
    let api_key = get_credential("api_key")?;
    let endpoint = get_credential("endpoint")?;
    let deployment = get_credential("deployment")?;

    // Create the chat request
    let chat_request = ChatRequest {
        messages: vec![
            Message {
                role: "system".to_string(),
                content: vec![Content {
                    content_type: "text".to_string(),
                    text: system_prompt,
                }],
            },
            Message {
                role: "user".to_string(),
                content: vec![Content {
                    content_type: "text".to_string(),
                    text: input.clone(),  // Clone here so we can use input later
                }],
            },
        ],
        temperature: 0.7,
        top_p: 0.95,
        max_tokens: 16384,
    };

    // Create HTTP client
    let client = reqwest::Client::new();

    // Prepare API request
    let url = format!("{}/openai/deployments/{}/chat/completions?api-version=2024-02-15-preview", endpoint, deployment);
    let response = client
        .post(&url)
        .header("api-key", api_key)
        .json(&chat_request)
        .send()
        .await?;

    let response_json: Value = response.json().await?;
    if let Some(choices) = response_json["choices"].as_array() {
        if let Some(message) = choices[0]["message"]["content"].as_str() {
            println!("{}", message);
        } else {
            print_error_response(&response_json, &input)?;
        }
    } else {
        print_error_response(&response_json, &input)?;
    }

    Ok(())
}

fn get_credential(cred_type: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (keyring_id, prompt_message) = match cred_type {
        "api_key" => (
            "azure_openai",
            "Please enter your API key (input will be hidden): "
        ),
        "endpoint" => (
            "azure_openai_endpoint",
            "Please enter your endpoint (e.g., https://your-resource.openai.azure.com): "
        ),
        "deployment" => (
            "azure_openai_deployment",
            "Please enter your deployment name: "
        ),
        _ => return Err("Invalid credential type".into()),
    };

    let keyring_entry = keyring::Entry::new("actionitems", keyring_id)?;
    
    // Try to get from keyring first
    match keyring_entry.get_password() {
        Ok(value) => Ok(value.trim().to_string()),
        Err(_) => {
            // Prompt for value if not found
            println!("{} not found in secure storage.", cred_type);
            print!("{}", prompt_message);
            std::io::stdout().flush()?;
            
            let value = if cred_type == "api_key" {
                read_password()?.trim().to_string()
            } else {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            };
            
            // Store in keyring for future use
            keyring_entry.set_password(&value)?;
            println!("{} securely stored for future use.", cred_type);
            
            Ok(value)
        }
    }
}

fn delete_credentials() -> Result<(), Box<dyn std::error::Error>> {
    let cred_types = [
        ("API key", "azure_openai"),
        ("Endpoint", "azure_openai_endpoint"),
        ("Deployment", "azure_openai_deployment"),
    ];

    for (cred_name, keyring_id) in cred_types {
        let keyring_entry = keyring::Entry::new("actionitems", keyring_id)?;
        match keyring_entry.delete_password() {
            Ok(_) => println!("{} deleted from secure storage.", cred_name),
            Err(e) => {
                if e.to_string().contains("No such key") {
                    println!("No {} was stored.", cred_name);
                } else {
                    return Err(e.into());
                }
            }
        }
    }
    Ok(())
}

fn print_error_response(response_json: &Value, input: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check for 401 error
    if let Some(error) = response_json.get("error") {
        if let Some("401") = error.get("code").and_then(|c| c.as_str()) {
            // Delete the API key
            let keyring_entry = keyring::Entry::new("actionitems", "azure_openai")?;
            keyring_entry.delete_password()?;
            println!("Authentication failed. API key has been cleared.");
            println!("Please run the tool again to enter a new API key.");
            std::process::exit(1);
        }
    }

    // Print out how many tokens we sent
    // Rough estimate: 1 token ≈ 4 chars in English
    println!("Sent approximately {} tokens", input.len() / 4);
    println!("\nRaw API Response:\n");
    println!("{}", serde_json::to_string_pretty(response_json)?);
    Ok(())
}