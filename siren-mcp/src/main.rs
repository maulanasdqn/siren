mod rpc;
mod server;
mod tools;

use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Some(request) = rpc::parse(&line) else {
            continue;
        };
        if let Some(response) = server::handle(request) {
            rpc::write_message(&response);
        }
    }
}
