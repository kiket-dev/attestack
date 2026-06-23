mod agent;
mod ci;
mod commands;
mod run;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use attestack_store::BundleCreateOptions;
use clap::{Parser, Subcommand};
use commands::{
    bundle_create, doctor, git_status_message, init, note, pr_summary, report, snapshot, start,
    status, stop, verify_path,
};
use run::{run_command, RunOptions};

#[derive(Parser)]
#[command(name = "attestack", about = "Local-first proof layer for AI-assisted software work")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Attestack in the current directory
    Init {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        update_gitignore: bool,
        #[arg(long)]
        json: bool,
    },
    /// Start a new session
    Start {
        title: String,
        #[arg(long)]
        allow_parallel: bool,
        #[arg(long)]
        no_git: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show current session status
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Add a note to the active session
    Note {
        text: String,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Run a command and record it
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        capture_output: bool,
        #[arg(long)]
        no_capture_output: bool,
        #[arg(long)]
        shell: bool,
        #[arg(long)]
        json: bool,
    },
    /// Capture a Git snapshot
    Snapshot {
        #[arg(long)]
        include_diff: bool,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Close the active session
    Stop {
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        no_report: bool,
        #[arg(long)]
        json: bool,
    },
    /// Generate or print a session report
    Report {
        session_id: Option<String>,
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        include_command_output: bool,
        #[arg(long)]
        json: bool,
    },
    /// Print a PR-friendly evidence summary
    PrSummary {
        session_id: Option<String>,
        #[arg(long, help = "Bundle path to include verify instructions")]
        bundle: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Bundle commands
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },
    /// Verify a bundle or local session
    Verify {
        path: String,
        #[arg(long)]
        public_key: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        strict: bool,
    },
    /// Record AI agent activity
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// CI workflow helpers
    Ci {
        #[command(subcommand)]
        command: CiCommands,
    },
    /// Check local installation and repository readiness
    Doctor {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum BundleCommands {
    /// Create a portable evidence bundle
    Create {
        session_id: Option<String>,
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        include_diff: bool,
        #[arg(long)]
        include_command_output: bool,
        #[arg(long)]
        redact_paths: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Record an AI tool call (hash-only by default)
    ToolCall {
        #[arg(long)]
        tool: String,
        #[arg(long)]
        input_hash: Option<String>,
        #[arg(long)]
        output_hash: Option<String>,
        #[arg(long)]
        summary: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Record an agent or human decision
    Decision {
        #[arg(long)]
        summary: String,
        #[arg(long)]
        rationale: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Record approval or rejection of generated work
    Approval {
        #[arg(long)]
        subject: String,
        #[arg(long)]
        approved: bool,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Record a prompt content hash
    Prompt {
        #[arg(long)]
        content_hash: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Record a response content hash
    Response {
        #[arg(long)]
        content_hash: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum CiCommands {
    /// Initialize (if needed) and start a CI session
    Start {
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Run a command inside the active CI session
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        capture_output: bool,
        #[arg(long)]
        no_capture_output: bool,
        #[arg(long)]
        shell: bool,
        #[arg(long)]
        json: bool,
    },
    /// Stop the CI session and export a redacted bundle
    Finish {
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    let cwd = env::current_dir().map_err(|err| err.to_string())?;
    match cli.command {
        Commands::Init { force, update_gitignore, json } => {
            let store_root = init(&cwd, force, update_gitignore)?;
            if json {
                println!(
                    "{{\"store_root\":\"{}\"}}",
                    json_escape(&store_root.display().to_string())
                );
            } else {
                println!("Attestack initialized");
                println!("Store: {}", store_root.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Start { title, allow_parallel, no_git, json } => {
            ensure_initialized(&cwd)?;
            let session = start(&cwd, title, allow_parallel, !no_git)?;
            if json {
                println!(
                    "{{\"session_id\":\"{}\",\"title\":\"{}\"}}",
                    json_escape(&session.session_id),
                    json_escape(&session.title)
                );
            } else {
                println!("Session started");
                println!("ID: {}", session.session_id);
                println!("Store: .attestack/sessions/{}", session.session_id);
                if let Some(message) = git_status_message(&cwd) {
                    eprintln!("note: {message}");
                }
                println!();
                println!("Next:");
                println!("  attestack run -- pnpm test");
                println!("  attestack note \"Reviewed auth path\"");
                println!("  attestack stop");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Status { json } => {
            ensure_initialized(&cwd)?;
            let open = status(&cwd)?;
            match open {
                Some(session) => {
                    if json {
                        println!(
                            "{{\"session_id\":\"{}\",\"title\":\"{}\",\"status\":\"open\"}}",
                            json_escape(&session.session_id),
                            json_escape(&session.title)
                        );
                    } else {
                        println!("Open session: {}", session.session_id);
                        println!("Title: {}", session.title);
                    }
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    if json {
                        println!("{{\"status\":\"none\"}}");
                    } else {
                        println!("No open session");
                    }
                    Ok(ExitCode::from(2))
                }
            }
        }
        Commands::Note { text, session, json } => {
            ensure_initialized(&cwd)?;
            note(&cwd, text.clone(), session)?;
            if json {
                println!("{{\"note\":{}}}", json_string(&text));
            } else {
                println!("Note recorded");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Run { command, .. } if command.is_empty() => {
            Err("run requires a command after --".into())
        }
        Commands::Run { command, session, capture_output: _, no_capture_output, shell, json } => {
            ensure_initialized(&cwd)?;
            let store_output = !no_capture_output;
            let result = run_command(
                &cwd,
                command,
                RunOptions { session_id: session, capture_output: store_output, use_shell: shell },
            )?;
            if json {
                println!(
                    "{{\"exit_code\":{},\"command_id\":\"{}\",\"duration_ms\":{}}}",
                    result.exit_code,
                    json_escape(&result.command_id),
                    result.duration_ms
                );
            } else {
                println!(
                    "Command recorded (id: {}, exit: {}, duration: {} ms)",
                    result.command_id, result.exit_code, result.duration_ms
                );
            }
            Ok(ExitCode::from(result.exit_code as u8))
        }
        Commands::Snapshot { include_diff, session, json } => {
            ensure_initialized(&cwd)?;
            let payload = snapshot(&cwd, session, include_diff)?;
            if json {
                println!(
                    "{{\"head\":{},\"branch\":{},\"dirty\":{}}}",
                    optional_json_string(payload.head.as_deref()),
                    optional_json_string(payload.branch.as_deref()),
                    payload.dirty
                );
            } else {
                println!("Git snapshot recorded");
                if let Some(head) = payload.head {
                    println!("HEAD: {head}");
                }
                if let Some(branch) = payload.branch {
                    println!("Branch: {branch}");
                }
                println!("Dirty: {}", payload.dirty);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Stop { session, no_report, json } => {
            ensure_initialized(&cwd)?;
            let session = stop(&cwd, session, !no_report, true)?;
            if json {
                println!(
                    "{{\"session_id\":\"{}\",\"status\":\"closed\"}}",
                    json_escape(&session.session_id)
                );
            } else {
                println!("Session stopped");
                println!("ID: {}", session.session_id);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Report { session_id, output, include_command_output, json } => {
            ensure_initialized(&cwd)?;
            let write_to_file = output.is_some();
            let (session, content) =
                report(&cwd, session_id, output.map(PathBuf::from), include_command_output)?;
            if json {
                println!(
                    "{{\"session_id\":\"{}\",\"report\":{}}}",
                    json_escape(&session.session_id),
                    json_string(&content)
                );
            } else if write_to_file {
                println!("Report written to {content}");
            } else {
                println!("{content}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::PrSummary { session_id, bundle, json } => {
            ensure_initialized(&cwd)?;
            let bundle_path = bundle.map(PathBuf::from);
            let (session, content) = pr_summary(&cwd, session_id, bundle_path)?;
            if json {
                println!(
                    "{{\"session_id\":\"{}\",\"summary\":{}}}",
                    json_escape(&session.session_id),
                    json_string(&content)
                );
            } else {
                println!("{content}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Bundle { command } => match command {
            BundleCommands::Create {
                session_id,
                output,
                include_diff,
                include_command_output,
                redact_paths,
                json,
            } => {
                ensure_initialized(&cwd)?;
                let result = bundle_create(
                    &cwd,
                    session_id,
                    BundleCreateOptions {
                        output: output.map(PathBuf::from),
                        include_diff,
                        include_command_output,
                        redact_paths,
                    },
                )?;
                if json {
                    println!(
                        "{{\"bundle_id\":\"{}\",\"bundle_path\":\"{}\"}}",
                        json_escape(&result.bundle_id),
                        json_escape(&result.bundle_path.display().to_string())
                    );
                } else {
                    println!("Bundle created");
                    println!("ID: {}", result.bundle_id);
                    println!("Path: {}", result.bundle_path.display());
                    println!("Files: {}", result.manifest.files.len());
                    println!("Included: session metadata, events, report");
                    if include_command_output {
                        println!("Included: command output artifacts");
                    } else {
                        println!("Excluded: command output (pass --include-command-output)");
                    }
                    if include_diff {
                        println!("Included: git diff artifacts");
                    } else {
                        println!("Excluded: git diffs (pass --include-diff)");
                    }
                    if redact_paths {
                        println!("Paths redacted for export");
                    }
                }
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Verify { path, public_key, json, strict } => {
            let target = Path::new(&path);
            let key_path = public_key.as_deref().map(Path::new);
            let report = if target.is_absolute() {
                verify_path(&cwd, target, key_path)?
            } else {
                verify_path(&cwd, &cwd.join(target), key_path)?
            };
            if json {
                println!("{}", serde_json::to_string(&report).map_err(|err| err.to_string())?);
            } else if report.verified {
                println!("Verification passed");
                println!("Target: {}", report.target);
                if let Some(count) = report.event_count {
                    println!("Events: {count}");
                }
                if let Some(count) = report.file_count {
                    println!("Files: {count}");
                }
                if let Some(signed) = report.signature_verified {
                    println!("Signature: {}", if signed { "valid" } else { "invalid" });
                }
                for warning in &report.warnings {
                    println!("warning: {warning}");
                }
            } else {
                println!("Verification failed");
                println!("Target: {}", report.target);
                for error in &report.errors {
                    println!("- {error}");
                }
            }
            let failed = !report.verified || (strict && !report.warnings.is_empty());
            Ok(if failed { ExitCode::from(1) } else { ExitCode::SUCCESS })
        }
        Commands::Agent { command } => {
            ensure_initialized(&cwd)?;
            match command {
                AgentCommands::ToolCall {
                    tool,
                    input_hash,
                    output_hash,
                    summary,
                    session,
                    json,
                } => {
                    agent::agent_tool_call(
                        &cwd,
                        session,
                        tool.clone(),
                        input_hash,
                        output_hash,
                        summary,
                    )?;
                    if json {
                        println!(
                            "{{\"recorded\":\"ai.tool_call\",\"tool\":{}}}",
                            json_string(&tool)
                        );
                    } else {
                        println!("AI tool call recorded");
                    }
                    Ok(ExitCode::SUCCESS)
                }
                AgentCommands::Decision { summary, rationale, session, json } => {
                    agent::agent_decision(&cwd, session, summary.clone(), rationale)?;
                    if json {
                        println!(
                            "{{\"recorded\":\"ai.decision\",\"summary\":{}}}",
                            json_string(&summary)
                        );
                    } else {
                        println!("AI decision recorded");
                    }
                    Ok(ExitCode::SUCCESS)
                }
                AgentCommands::Approval { subject, approved, session, json } => {
                    agent::agent_approval(&cwd, session, subject.clone(), approved)?;
                    if json {
                        println!(
                            "{{\"recorded\":\"ai.approval\",\"subject\":{},\"approved\":{approved}}}",
                            json_string(&subject)
                        );
                    } else {
                        println!("AI approval recorded");
                    }
                    Ok(ExitCode::SUCCESS)
                }
                AgentCommands::Prompt { content_hash, model, session, json } => {
                    agent::agent_prompt(&cwd, session, content_hash.clone(), model.clone())?;
                    if json {
                        println!(
                            "{{\"recorded\":\"ai.prompt\",\"content_hash\":{}}}",
                            json_string(&content_hash)
                        );
                    } else {
                        println!("AI prompt hash recorded");
                    }
                    Ok(ExitCode::SUCCESS)
                }
                AgentCommands::Response { content_hash, model, session, json } => {
                    agent::agent_response(&cwd, session, content_hash.clone(), model)?;
                    if json {
                        println!(
                            "{{\"recorded\":\"ai.response\",\"content_hash\":{}}}",
                            json_string(&content_hash)
                        );
                    } else {
                        println!("AI response hash recorded");
                    }
                    Ok(ExitCode::SUCCESS)
                }
            }
        }
        Commands::Ci { command } => match command {
            CiCommands::Start { title, force, json } => {
                let result = ci::ci_start(&cwd, title, force)?;
                if json {
                    println!(
                        "{{\"session_id\":\"{}\",\"title\":\"{}\"}}",
                        json_escape(&result.session_id),
                        json_escape(&result.title)
                    );
                } else {
                    println!("CI session started");
                    println!("ID: {}", result.session_id);
                    println!("Title: {}", result.title);
                }
                Ok(ExitCode::SUCCESS)
            }
            CiCommands::Run { command, .. } if command.is_empty() => {
                Err("ci run requires a command after --".into())
            }
            CiCommands::Run {
                command,
                session,
                capture_output: _,
                no_capture_output,
                shell,
                json,
            } => {
                if !cwd.join(".attestack/config.toml").is_file() {
                    return Err(ci::map_ci_start_error(
                        "Attestack is not initialized; run `attestack ci start` first".into(),
                    ));
                }
                let store_output = !no_capture_output;
                let result = run_command(
                    &cwd,
                    command,
                    RunOptions {
                        session_id: session,
                        capture_output: store_output,
                        use_shell: shell,
                    },
                )?;
                if json {
                    println!(
                        "{{\"exit_code\":{},\"command_id\":\"{}\",\"duration_ms\":{}}}",
                        result.exit_code,
                        json_escape(&result.command_id),
                        result.duration_ms
                    );
                } else {
                    println!(
                        "CI command recorded (id: {}, exit: {}, duration: {} ms)",
                        result.command_id, result.exit_code, result.duration_ms
                    );
                }
                Ok(ExitCode::from(result.exit_code as u8))
            }
            CiCommands::Finish { session, output, json } => {
                let result = ci::ci_finish(&cwd, session, output.map(PathBuf::from))?;
                if json {
                    println!(
                        "{{\"session_id\":\"{}\",\"bundle_id\":\"{}\",\"bundle_path\":\"{}\"}}",
                        json_escape(&result.session_id),
                        json_escape(&result.bundle_id),
                        json_escape(&result.bundle_path.display().to_string())
                    );
                } else {
                    println!("CI session finished");
                    println!("Session: {}", result.session_id);
                    println!("Bundle: {}", result.bundle_path.display());
                }
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Doctor { json } => {
            let report = doctor(&cwd);
            if json {
                println!("{}", serde_json::to_string(&report).map_err(|err| err.to_string())?);
            } else {
                for check in &report.checks {
                    let status = match check.status {
                        attestack_store::CheckStatus::Pass => "pass",
                        attestack_store::CheckStatus::Warn => "warn",
                        attestack_store::CheckStatus::Fail => "fail",
                    };
                    println!("[{status}] {} — {}", check.name, check.message);
                }
            }
            Ok(if report.passed() { ExitCode::SUCCESS } else { ExitCode::from(1) })
        }
    }
}

fn ensure_initialized(cwd: &Path) -> Result<(), String> {
    if !cwd.join(".attestack/config.toml").is_file() {
        return Err("Attestack is not initialized; run `attestack init` first".into());
    }
    Ok(())
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn optional_json_string(value: Option<&str>) -> String {
    match value {
        Some(value) => json_string(value),
        None => "null".into(),
    }
}
