use anyhow::{anyhow, Context, Result};
use siren_domain::{CodingAgent, DomainError};
use std::process::Command;

pub const DEFAULT_PERMISSION_MODE: &str = "acceptEdits";

pub struct ClaudeCodeAgent {
    permission_mode: String,
    system: Option<String>,
    session: Option<String>,
}

impl ClaudeCodeAgent {
    pub fn new(permission_mode: String, system: Option<String>) -> Self {
        Self { permission_mode, system, session: None }
    }

    pub fn session(&self) -> Option<&str> {
        self.session.as_deref()
    }
}

impl CodingAgent for ClaudeCodeAgent {
    fn send(&mut self, prompt: &str) -> Result<String, DomainError> {
        send_inner(self, prompt).map_err(DomainError::inference)
    }
}

fn send_inner(agent: &mut ClaudeCodeAgent, prompt: &str) -> Result<String> {
    let mut command = Command::new("claude");
    command
        .arg("--print")
        .arg("--output-format")
        .arg("json")
        .arg("--permission-mode")
        .arg(&agent.permission_mode);
    if let Some(session) = &agent.session {
        command.arg("--resume").arg(session);
    }
    if let Some(system) = &agent.system {
        command.arg("--append-system-prompt").arg(system);
    }
    command.arg("--").arg(prompt);

    let output = command.output().context("running the claude cli")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("claude exited with {}: {}", output.status, stderr.trim()));
    }

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("parsing claude json output")?;
    if let Some(session) = value["session_id"].as_str() {
        agent.session = Some(session.to_string());
    }
    let reply = value["result"].as_str().unwrap_or_default().trim().to_string();
    if reply.is_empty() {
        return Err(anyhow!("claude returned no text result"));
    }
    Ok(reply)
}
