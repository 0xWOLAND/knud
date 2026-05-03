use std::{error::Error, io::Write};

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

pub async fn realtime_demo() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let mut req = "wss://api.openai.com/v1/realtime?model=gpt-realtime".into_client_request()?;
    req.headers_mut()
        .insert("Authorization", format!("Bearer {api_key}").parse()?);

    let (ws, _) = connect_async(req).await?;
    let (mut write, mut read) = ws.split();

    write
        .send(Message::text(
            json!({
                "type": "session.update",
                "session": {
                    "type": "realtime",
                    "instructions": "Use the tool when needed, then answer briefly.",
                    "tools": [{
                        "type": "function",
                        "name": "get_weather",
                        "description": "Get the weather for a city.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "city": { "type": "string" }
                            },
                            "required": ["city"]
                        }
                    }],
                }
            })
            .to_string(),
        ))
        .await?;

    write
        .send(Message::text(
            json!({
                "type": "conversation.item.create",
                "item": {
                    "type": "message",
                    "role": "user",
                    "content": [
                        { "type": "input_text", "text": "What is the weather in San Francisco?" }
                    ]
                }
            })
            .to_string(),
        ))
        .await?;

    write
        .send(Message::text(
            json!({
                "type": "response.create",
                "response": {
                    "output_modalities": ["text"]
                }
            })
            .to_string(),
        ))
        .await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let event: serde_json::Value = serde_json::from_str(&text)?;
            match event["type"].as_str() {
                Some("response.output_text.delta") => {
                    print!("{}", event["delta"].as_str().unwrap_or(""));
                    std::io::stdout().flush()?;
                }
                Some("response.done") => {
                    if let Some(call) = event["response"]["output"]
                        .as_array()
                        .and_then(|items| items.iter().find(|item| item["type"] == "function_call"))
                    {
                        let call_id = call["call_id"].as_str().unwrap_or("");
                        let arguments = serde_json::from_str::<serde_json::Value>(
                            call["arguments"].as_str().unwrap_or("{}"),
                        )?;
                        let city = arguments["city"].as_str().unwrap_or("unknown");

                        println!("[tool] get_weather city={city}");

                        write
                            .send(Message::text(
                                json!({
                                    "type": "conversation.item.create",
                                    "item": {
                                        "type": "function_call_output",
                                        "call_id": call_id,
                                        "output": json!({
                                            "city": city,
                                            "forecast": "sunny",
                                            "temperature_f": 68
                                        })
                                        .to_string()
                                    }
                                })
                                .to_string(),
                            ))
                            .await?;

                        write
                            .send(Message::text(
                                json!({
                                    "type": "response.create",
                                    "response": {
                                        "output_modalities": ["text"],
                                        "instructions": "Answer the user using the tool result."
                                    }
                                })
                                .to_string(),
                            ))
                            .await?;
                    } else {
                        println!();
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
