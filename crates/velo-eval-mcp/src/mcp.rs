//! Minimal MCP stdio transport: newline-delimited JSON-RPC 2.0.
//!
//! Implements the subset of the Model Context Protocol needed to expose the
//! evaluation tool registry: `initialize`, `tools/list`, `tools/call`, `ping`.

use std::io::{BufRead, Write};

use serde_json::{json, Value};

use crate::tools::{self, ToolOutput};

const SERVER_NAME: &str = "velo-eval";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PROTOCOL: &str = "2025-06-18";

/// Run the MCP server loop over the given reader/writer (stdin/stdout in prod).
pub fn serve(reader: impl BufRead, mut writer: impl Write) -> std::io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = error_response(Value::Null, -32700, &format!("parse error: {e}"));
                write_msg(&mut writer, &resp)?;
                continue;
            }
        };

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        // Notifications (no id) get no response.
        let Some(id) = id else {
            continue;
        };

        let resp = match method {
            "initialize" => {
                let requested = params
                    .get("protocolVersion")
                    .and_then(Value::as_str)
                    .unwrap_or(DEFAULT_PROTOCOL);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": requested,
                        "capabilities": { "tools": { "listChanged": false } },
                        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
                        "instructions": "Evaluation tools for the VeloSim cycling simulator: run deterministic ride scenarios, render real wgpu scene+HUD screenshots for multimodal UI evaluation, validate workouts and FIT exports."
                    }
                })
            }
            "ping" => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
            "tools/list" => {
                let tools: Vec<Value> = tools::registry()
                    .iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema,
                        })
                    })
                    .collect();
                json!({ "jsonrpc": "2.0", "id": id, "result": { "tools": tools } })
            }
            "tools/call" => handle_tool_call(id, &params),
            _ => error_response(id, -32601, &format!("method not found: {method}")),
        };
        write_msg(&mut writer, &resp)?;
    }
    Ok(())
}

fn handle_tool_call(id: Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let Some(tool) = tools::find(name) else {
        return error_response(id, -32602, &format!("unknown tool: {name}"));
    };

    match (tool.run)(&args) {
        Ok(output) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "content": content_blocks(&output), "isError": false }
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": e }],
                "isError": true
            }
        }),
    }
}

fn content_blocks(output: &ToolOutput) -> Vec<Value> {
    let mut blocks = Vec::new();
    if !output.text.is_empty() {
        blocks.push(json!({ "type": "text", "text": output.text }));
    }
    for image in &output.images {
        blocks.push(json!({
            "type": "image",
            "data": base64_encode(&image.png),
            "mimeType": "image/png",
        }));
    }
    blocks
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn write_msg(writer: &mut impl Write, msg: &Value) -> std::io::Result<()> {
    let mut line = serde_json::to_string(msg)?;
    line.push('\n');
    writer.write_all(line.as_bytes())?;
    writer.flush()
}

/// Dependency-free base64 (standard alphabet, padded).
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(requests: &str) -> Vec<Value> {
        let mut out = Vec::new();
        serve(requests.as_bytes(), &mut out).unwrap();
        String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    #[test]
    fn base64_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn initialize_and_list_tools() {
        let responses = roundtrip(concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            "\n",
        ));
        assert_eq!(responses.len(), 2);
        assert_eq!(
            responses[0]["result"]["serverInfo"]["name"],
            json!("velo-eval")
        );
        let tools = responses[1]["result"]["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "render_frame"));
    }

    #[test]
    fn tool_call_returns_text_content() {
        let responses = roundtrip(concat!(
            r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"feature_inventory","arguments":{}}}"#,
            "\n",
        ));
        let content = responses[0]["result"]["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], json!("text"));
        assert!(content[0]["text"].as_str().unwrap().contains("velo-core"));
        assert_eq!(responses[0]["result"]["isError"], json!(false));
    }

    #[test]
    fn unknown_tool_is_tool_error() {
        let responses = roundtrip(concat!(
            r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"nope"}}"#,
            "\n",
        ));
        assert!(responses[0].get("error").is_some());
    }

    #[test]
    fn unknown_method_is_rpc_error() {
        let responses = roundtrip(concat!(
            r#"{"jsonrpc":"2.0","id":9,"method":"resources/list"}"#,
            "\n",
        ));
        assert_eq!(responses[0]["error"]["code"], json!(-32601));
    }
}
