# Beginner's Tutorial: Building an AI-Powered Rust Project with Tool Integration

Welcome, aspiring AI engineers! This tutorial will guide you step-by-step through a simple yet powerful Rust project that uses AI reasoning with real-world tool integration. You’ll learn how to:

- Set up a basic AI-powered assistant with conversation history
- Use real tools (like reading files) in your AI agent
- Handle JSON-based tool communication and responses
- Structure your project using modular Rust code

All code is located in the `src/` directory of the project.

---

## 1. Project Overview

This project simulates a smart assistant that:
- Receives a question from the user via command line (e.g., `cargo run "What is Rust?"`)
- Understands the request using an AI model
- Can call tools (like reading a file) to fetch data
- Returns a well-formatted response back to the user

It uses:
- Rust for performance and safety
- Async `tokio` for concurrent operations
- `serde` for JSON serialization
- `reqwest` to send API requests to a language model (e.g., OpenRouter)
- A sandboxed file reading tool for security

---

## 2. Step-by-Step Breakdown

### 📁 File Structure (src/)
```
src/
├── main.rs           – Entry point & main loop
├── messages.rs       – Conversation history format
├── tools.rs          – Tools definition & execution
├── utils.rs          – Helper functions (e.g., sandboxing)
└── prompts.rs        – System prompt (AI's instructions)
```

We’ll go through each file in order.

---

### ✅ 1. `main.rs` – The Heart of the App

This is where the magic happens.

```rust
mod tools;
use tools::{read_file_tool_definition, run_tool};

mod messages;
use messages::Message;

mod utils;
use utils::send;

mod prompts;
use prompts::get_system_prompt;
```

- `mod tools;` – imports the tool definitions.
- `use tools::{read_file_tool_definition, run_tool};` – allows us to define what tools the AI can access and how they’re executed.

#### 🔁 Main Loop
The `main` function:
1. Loads environment variables (like `MAX_TURNS`, `MODEL`, `MAX_FILE_CHARS`)
2. Gets the user's question from `args`
3. Sets up the `messages` vector — a conversation history:
   ```rust
   let mut messages = vec![
       Message::system(get_system_prompt()),
       Message::user(user_question),
   ];
   ```
   This tells the AI the rules and starts the conversation.

4. Runs a loop (up to `MAX_TURNS`) to let the AI think and act:
   - Use `send()` to ask the AI model for a response.
   - If the AI wants to use a tool (like `read_file`), it writes a `tool_call`.
   - The app executes the tool with `run_tool()` and adds the result to `messages`.
   - Then, it sends this new `tool_result` back to the AI to continue reasoning.

5. When the AI finishes without needing more tools, it returns plain text — which we print.

> 💡 **Key Insight**: This loop mimics how real AI agents work — they think, sometimes call tools, and then re-evaluate.

---

### 📝 2. `messages.rs` – Structuring Conversations

This file defines how messages flow between user, assistant, and tools.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
}
```

- **`role`**: `"user"`, `"assistant"`, or `"tool"`.
- **`content`**: The text of the message.
- **`tool_calls`**: If the AI wants to run a tool, this lists what it needs.
- **`tool_call_id`**: A unique ID to match tool results back.

#### Example Usage:
```rust
Message::user("What is the purpose of Rust?")
Message::assistant(Some("I can help with that. Let me check the docs.", Some(tool_calls)))
Message::tool_result("call_123", "Rust is a systems programming language...")
```

> 🔄 The AI sends a message with `tool_calls`, then the system runs the tool and adds a `tool_result`, which the AI sees to continue.

---

### 🛠 3. `tools.rs` – Tools for the AI to Use

This file defines a secure way for the AI to interact with the system.

#### ✅ Tool Definition (`read_file`)
```rust
pub fn read_file_tool_definition() -> Tool {
    Tool::new(
        "read_file",
        "Read a UTF-8 text file from the current project.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to the project root."
                }
            },
            "required": ["path"]
        }),
    )
}
```

- This tells the AI: "I can run `read_file` if you pass a `path`."
- The `path` must be relative, and it’s validated later.

#### 🔐 Secure File Reading (`run_tool`)
```rust
fn read_file_tool(args: ReadFileArgs) -> Result<String> {
    let root = sandbox_root()?;
    let resolved = root.join(Path::new(&args.path));
    let canonical = resolved.canonicalize()?;

    // Prevent path traversal attacks
    if !canonical.starts_with(&root) {
        bail!("path escapes the sandbox")
    }

    let bytes = std::fs::read(&canonical)?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(text.chars().take(max_file_chars).collect())
}
```

> 🔒 **Security is built-in!**
- All files must be inside the project root.
- No `../` or absolute paths allowed.
- Large files are limited to `MAX_FILE_CHARS` (default: 10,000 chars).

---

### 🧰 4. `utils.rs` – Helpers for the Project

This file has helper functions:
- `sandbox_root()` → Returns the project root path.
- `send()` → Sends a JSON request to the AI model (e.g., OpenRouter).
  - It builds a JSON body with `model`, `messages`, `tools`, and `system` prompt.
  - It sends it using `reqwest` with an API key.
  - Returns the AI's response.

> ✅ **Note**: This is where you'd plug in models like `openai/gpt-4-turbo`, `mistralai/mixtral-8x7b`, or others by setting `MODEL` and `MODEL_URL`.

---

### 🧠 5. `prompts.rs` (Not shown, but implied)

Even though we didn't see its code, we know:
- `get_system_prompt()` returns a string like:
  ```
  You are a helpful AI assistant. You can use tools to help answer questions. 
  When you need a file, call read_file with the correct path.
  ```
- This sets the tone and rules for the AI.

---

## 3. How to Run It

1. Make sure you have:
   - Rust installed (`rustup`)
   - `.env` file in project root with:
     ```env
     MODEL=ollama/llama3
     MODEL_URL=https://openrouter.ai/api/v1/chat/completions
     OPENROUTER_API_KEY=your-api-key
     MAX_TURNS=5
     MAX_FILE_CHARS=5000
     ```

2. Run:
   ```bash
   cargo run "What is file reading in Rust?"
   ```

3. The AI will:
   - Think about the question
   - Ask to read a file (e.g., `src/main.rs`)
   - Get the file content
   - Tell you what it found

> 🧩 It’s like a tiny AI engineer — it reads code, understands it, and explains.

---

## 4. Key AI Engineering Concepts You Learned

| Concept | How It’s Used Here |
|-------|------------------|
| **Tool Calling** | AI asks to read a file; we run the tool |
| **Sandboxing** | Prevents unsafe file access |
| **Conversation History** | `messages` vector tracks the chat |
| **API Integration** | `reqwest` talks to a real AI model |
| **Security** | Path validation stops attacks |

---

## 5. Bonus: Extend It!

Try adding:
- `write_file` tool
- `list_files` tool
- `run_command` tool (with permission control)
- A web UI using `actix-web`

---

## ✅ Final Thoughts

This project shows how to build real AI agents in Rust — safe, modular, and powerful. You’ve now seen the core of modern AI engineering: **AI reasoning + tool integration + security safeguards**.

Keep exploring. The future is in your hands.

🚀 **Next Step:** Try running the project or modifying `tools.rs` to add your own tool!