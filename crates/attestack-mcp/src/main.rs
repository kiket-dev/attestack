mod server;

use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use server::{handle_request, McpRequest};

fn main() -> io::Result<()> {
    let repo_root = env::current_dir()?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(err) => {
                let response = server::error_response(None, -32700, format!("parse error: {err}"));
                writeln!(stdout, "{}", serde_json::to_string(&response).unwrap())?;
                stdout.flush()?;
                continue;
            }
        };

        let response = handle_request(&repo_root, request);
        writeln!(stdout, "{}", serde_json::to_string(&response).unwrap())?;
        stdout.flush()?;
    }

    Ok(())
}

pub fn repo_root_from_env() -> PathBuf {
    env::var("ATTESTACK_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}
