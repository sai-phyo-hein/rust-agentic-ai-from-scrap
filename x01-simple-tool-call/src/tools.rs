use serde::{Serialize, Deserialize};
use crate::utils::sandbox_root;
use std::env;
use anyhow::{Result, anyhow, bail};
use std::path::Path;

// --- Start: Tool Definitions ---
#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: FunctionDef
}

#[derive(Debug, Serialize, Clone)]
pub struct FunctionDef {
    name: &'static str,
    description: &'static str,
    parameters: serde_json::Value
}


impl Tool {
    pub fn new(name: &'static str, description: &'static str, parameters: serde_json::Value) -> Self {
        Self {
            kind: "function",
            function: FunctionDef { name, description, parameters }
         }
    }
}

pub fn read_file_tool_definition() -> Tool {
    Tool::new (
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

// --- End: Tool Definition ---

// --- Start: Tools ---

#[derive(Debug, Deserialize)]
pub struct ReadFileArgs {
    path: String,
}

fn read_file_tool(args: ReadFileArgs) -> Result<String> {
    dotenvy::dotenv().ok();
    let max_file_chars: usize = env::var("MAX_FILE_CHARS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .unwrap_or(10_000);
    let root = sandbox_root()?;
    let resolved = root.join(Path::new(&args.path));
    let canonical = resolved
        .canonicalize()
        .map_err(|e| anyhow!("cannot resolve {}: {e}", args.path))?;
    if !canonical.starts_with(&root) {
        bail!("path {} escapes the sandbox", args.path);
    }
    let bytes = std::fs::read(&canonical)?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(text.chars().take(max_file_chars).collect())
}

// --- Start: tool call ---

pub fn run_tool(name: &str, arguments_json: &str) -> Result<String> {
    match name {
        "read_file" => {
            let args: ReadFileArgs = serde_json::from_str(arguments_json)
                .map_err(|e| anyhow!("invalid arguments for read_file: {e}"))?;
            read_file_tool(args)
        }
        other => Err(anyhow!("unknow tool: {other}")),
    }
}

// --- End: tool call ---