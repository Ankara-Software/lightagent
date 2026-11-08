use clap::{Parser, Subcommand};
use colored::*;

mod git;
mod state;
mod llm;
mod executor;

use git::Git;
use state::StateManager;
use llm::LlmClient;
use executor::Executor;

#[derive(Parser)]
#[command(name = "lightagent")]
#[command(about = "Zero-Dependency, State-Preserving Agent CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start a new engineering task")]
    Start {
        #[arg(help = "The objective or task description")]
        task: String,

        #[arg(long, help = "Enable YOLO mode (run autonomously without approvals)")]
        yolo: bool,
    },
    #[command(about = "Resume the active task from the local state")]
    Resume {
        #[arg(long, help = "Enable YOLO mode (run autonomously without approvals)")]
        yolo: bool,
    },
    #[command(about = "Show the status of the current task and git alignment")]
    Status,
    #[command(about = "Clear the current active task state")]
    Clear,
}

const SYSTEM_PROMPT: &str = r#"You are LightAgent, a zero-dependency autonomous engineering assistant.
Your goal is to solve the developer's request. You have full access to compile code, run tests, and manage files.

You must respond with a JSON object in this exact format:
{
  "thought": "Brief explanation of your current reasoning",
  "action": {
    "type": "run_command" | "write_file" | "read_file" | "finish",
    "command": "The shell command to run (required for run_command)",
    "path": "File path (required for write_file / read_file)",
    "content": "File contents to write (required for write_file)",
    "message": "Final completion message (required for finish)"
  }
}
"#;

fn run_loop(yolo: bool) {
    let state_manager = StateManager::new();
    let llm = LlmClient::new();

    if !state_manager.is_state_present() {
        println!("{}", "Error: No active task to run. Use 'lightagent start \"task\"' first.".red());
        return;
    }

    println!("{}", "=== LightAgent Autonomous Loop ===".bold().blue());
    if yolo {
        println!("{}", "YOLO mode enabled! Running autonomously without confirmation.".yellow());
    } else {
        println!("{}", "Confirmation mode enabled. You will be prompted before each action.".cyan());
    }

    // Main execution loop
    let mut steps_count = 0;
    let max_steps = 15;

    while steps_count < max_steps {
        steps_count += 1;
        println!("\n{} Step {}/{}...", "▶".green(), steps_count, max_steps);

        // 1. Build chat history from state file
        let steps = match state_manager.load_steps() {
            Ok(s) => s,
            Err(e) => {
                println!("Error loading state: {}", e);
                break;
            }
        };

        let mut history = String::new();
        for step in &steps {
            history.push_str(&format!("[Timestamp: {} | Type: {}]\n{}\n\n", step.timestamp, step.step_type, step.payload));
        }

        // 2. Query LLM
        println!("Thinking...");
        let response_val = match llm.get_next_action(SYSTEM_PROMPT, &history) {
            Ok(val) => val,
            Err(err) => {
                println!("{} {}", "LLM request failed:".red(), err);
                break;
            }
        };

        let thought = response_val.get("thought").and_then(|t| t.as_str()).unwrap_or("");
        let action = match response_val.get("action") {
            Some(a) => a,
            None => {
                println!("{}", "Error: LLM did not return an 'action' block.".red());
                break;
            }
        };

        let action_type = action.get("type").and_then(|t| t.as_str()).unwrap_or("");

        println!("{} {}", "Thought:".yellow(), thought);

        // 3. Execute action
        match action_type {
            "run_command" => {
                let cmd = action.get("command").and_then(|c| c.as_str()).unwrap_or("");
                println!("{} {}", "Action: Run command ->".cyan(), cmd);
                
                if !yolo {
                    print!("Approve execution? (y/N): ");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() != "y" {
                        println!("Action aborted by user.");
                        let _ = state_manager.append_step("error", "Action aborted by user");
                        break;
                    }
                }

                match Executor::run_command(cmd) {
                    Ok(out) => {
                        println!("{}", "Command succeeded.".green());
                        let _ = state_manager.append_step("command_run", &format!("Command: {}\nResult:\n{}", cmd, out));
                    }
                    Err(err) => {
                        println!("{}", "Command failed.".red());
                        let _ = state_manager.append_step("command_run", &format!("Command: {}\nResult:\n{}", cmd, err));
                    }
                }
            }
            "write_file" => {
                let path = action.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let content = action.get("content").and_then(|c| c.as_str()).unwrap_or("");
                println!("{} {}", "Action: Write file ->".cyan(), path);

                if !yolo {
                    print!("Approve file edit? (y/N): ");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() != "y" {
                        println!("Action aborted by user.");
                        let _ = state_manager.append_step("error", "Action aborted by user");
                        break;
                    }
                }

                match Executor::write_file(path, content) {
                    Ok(out) => {
                        println!("{}", out.green());
                        let _ = state_manager.append_step("file_edit", &out);
                    }
                    Err(err) => {
                        println!("{}", err.red());
                        let _ = state_manager.append_step("file_edit", &err);
                    }
                }
            }
            "read_file" => {
                let path = action.get("path").and_then(|p| p.as_str()).unwrap_or("");
                println!("{} {}", "Action: Read file ->".cyan(), path);

                match Executor::read_file(path) {
                    Ok(content) => {
                        println!("Read completed successfully ({} bytes).", content.len());
                        let _ = state_manager.append_step("file_read", &format!("Read file: {}\nContent:\n{}", path, content));
                    }
                    Err(err) => {
                        println!("{}", err.red());
                        let _ = state_manager.append_step("file_read", &err);
                    }
                }
            }
            "finish" => {
                let msg = action.get("message").and_then(|m| m.as_str()).unwrap_or("");
                println!("\n{} Task Completed!", "✔".green());
                println!("{} {}", "Message:".green(), msg);
                let _ = state_manager.append_step("completed", msg);
                break;
            }
            _ => {
                println!("{} Unknown action type: {}", "Error:".red(), action_type);
                break;
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let state_manager = StateManager::new();

    match cli.command {
        Commands::Start { task, yolo } => {
            println!("Initializing new task: {}", task.green());
            if state_manager.is_state_present() {
                println!("{}", "Warning: Overwriting active task state.".yellow());
                let _ = state_manager.clear_state();
            }
            
            let commit = Git::get_commit_hash();
            let branch = Git::get_branch();
            let payload = format!("Task: {}\nGit Branch: {}\nGit Commit: {}", task, branch, commit);

            if let Err(e) = state_manager.append_step("task_start", &payload) {
                println!("Failed to write state: {}", e);
                return;
            }

            run_loop(yolo);
        }
        Commands::Resume { yolo } => {
            if !state_manager.is_state_present() {
                println!("{}", "No active task found. Run 'lightagent start \"task\"' first.".red());
                return;
            }
            println!("{}", "Resuming active task...".green());
            
            // Check Git drift
            let current_commit = Git::get_commit_hash();
            if let Ok(steps) = state_manager.load_steps() {
                if let Some(start_step) = steps.first() {
                    if !start_step.payload.contains(&current_commit) {
                        println!("{}", "Warning: Git commit drift detected! Current commit differs from task start commit.".yellow());
                    }
                }
            }

            run_loop(yolo);
        }
        Commands::Status => {
            if !state_manager.is_state_present() {
                println!("{}", "No active task found.".yellow());
                return;
            }

            println!("{}", "=== LightAgent Status ===".bold().blue());
            let branch = Git::get_branch();
            let commit = Git::get_commit_hash();
            println!("Current Branch: {}", branch.green());
            println!("Current Commit: {}", commit.green());

            if let Ok(steps) = state_manager.load_steps() {
                if let Some(start_step) = steps.first() {
                    println!("\nTask Description:");
                    println!("{}", start_step.payload.white());
                }
                println!("\nExecution History ({} steps):", steps.len());
                for (idx, step) in steps.iter().enumerate() {
                    println!("  [{}] {} -> {}", idx + 1, step.step_type.cyan(), step.timestamp);
                }
            }
        }
        Commands::Clear => {
            if state_manager.is_state_present() {
                if let Ok(_) = state_manager.clear_state() {
                    println!("{}", "Cleared active task state.".green());
                }
            } else {
                println!("No active task state to clear.");
            }
        }
    }
}
