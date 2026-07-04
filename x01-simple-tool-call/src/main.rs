mod tools;
use tools::{read_file_tool_definition, run_tool};

mod messages;
use messages::Message;

mod utils;
use utils::send;

mod prompts;
use prompts::get_system_prompt;

use std::env;
use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
    
    dotenvy::dotenv().ok();
    let max_turns: u8 = env::var("MAX_TURNS")
        .context("MAX_TURNS not set.")?
        .parse()
        .context("MAX_TURNS not a valid number.")?;
    
    // Process get the user query from the cli arguments like `cargo run "What is rust programming?"`
    let user_question = std::env::args()
        . nth(1)                         // take the first argument
        .unwrap_or_else(|| "".to_string());

    if &user_question == "" {
        println!("Empty query. Stopped");
        // print the info and stop immediately. 
        return Ok(());
    }

    // tell agent that it can use these tools
    let tools = vec![
        read_file_tool_definition()
    ];

    // messages - conversation history between 
    // user <> agent <> tool
    let mut messages: Vec<Message> = vec![
        Message::system(get_system_prompt()),
        Message::user(user_question),
    ];

    for _ in 0..max_turns {
        
        let response = send(
            &tools,
            &messages
        ).await?;

        let choice = response
            .choices
            .into_iter()
            .next()
            .context("no choices returned")?;

        let msg = choice.message;

        match msg.tool_calls {
            Some(tool_calls) if !tool_calls.is_empty() => {
                // record the assistant's tool-call request in history first,
                // so the follow-up "tool" messages have something to reply to.
                messages.push(Message::assistant(msg.content.clone(), Some(tool_calls.clone())));

                for call in tool_calls {
                    let result = run_tool(&call.function.name, &call.function.arguments)
                        .unwrap_or_else(|e| format!("Error: {e}"));
                    messages.push(Message::tool_result(call.id, result));
                }
            }
            _ => {
                if let Some(text) = msg.content {
                    println!("{text}");
                }
                break;
            }
        }
    }

    Ok(())
}
