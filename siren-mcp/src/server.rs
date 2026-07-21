use crate::rpc::{self, Request};
use crate::tools;
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2025-06-18";

pub fn handle(request: Request) -> Option<Value> {
    let id = request.id.clone();
    match request.method.as_str() {
        "initialize" => id.map(|id| rpc::success(id, initialize(&request.params))),
        "notifications/initialized" => None,
        "ping" => id.map(|id| rpc::success(id, json!({}))),
        "tools/list" => id.map(|id| rpc::success(id, json!({ "tools": tools::list() }))),
        "tools/call" => id.map(|id| tools_call(id, &request.params)),
        _ => id.map(|id| rpc::error(id, -32601, "method not found")),
    }
}

fn initialize(params: &Value) -> Value {
    let version = params["protocolVersion"].as_str().unwrap_or(PROTOCOL_VERSION);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "siren", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn tools_call(id: Value, params: &Value) -> Value {
    let name = params["name"].as_str().unwrap_or_default();
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
    match tools::call(name, &arguments) {
        Ok(text) => rpc::success(
            id,
            json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        ),
        Err(e) => rpc::success(
            id,
            json!({ "content": [{ "type": "text", "text": format!("error: {e}") }], "isError": true }),
        ),
    }
}
