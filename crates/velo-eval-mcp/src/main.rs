//! velo-eval-mcp — MCP server + CLI for evaluating VeloSim headlessly.
//!
//! Modes:
//! - `serve` (default): MCP stdio server (newline-delimited JSON-RPC 2.0).
//! - `call <tool> [--args JSON] [--save-dir DIR]`: one-shot tool invocation;
//!   prints the text result and writes any PNG frames to files.
//! - `list-tools`: print the tool registry.

mod frames;
mod mcp;
mod scenario;
mod tools;

use std::io::{stdin, stdout};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "velo-eval-mcp", about = "VeloSim evaluation MCP server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the MCP stdio server (default).
    Serve,
    /// Invoke one tool directly and save any images to disk.
    Call {
        /// Tool name (see `list-tools`).
        tool: String,
        /// JSON arguments object.
        #[arg(long, default_value = "{}")]
        args: String,
        /// Directory to write PNG frames into.
        #[arg(long, default_value = ".")]
        save_dir: PathBuf,
    },
    /// Print available tools and their schemas.
    ListTools,
}

fn main() -> std::process::ExitCode {
    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Serve => {
            if let Err(e) = mcp::serve(stdin().lock(), stdout().lock()) {
                eprintln!("mcp server error: {e}");
                return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
        }
        Command::ListTools => {
            for tool in tools::registry() {
                println!("{}\n  {}\n", tool.name, tool.description);
            }
            std::process::ExitCode::SUCCESS
        }
        Command::Call {
            tool,
            args,
            save_dir,
        } => run_call(&tool, &args, &save_dir),
    }
}

fn run_call(tool: &str, args: &str, save_dir: &PathBuf) -> std::process::ExitCode {
    let args: serde_json::Value = match serde_json::from_str(args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid --args JSON: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let Some(def) = tools::find(tool) else {
        eprintln!("unknown tool: {tool} (try list-tools)");
        return std::process::ExitCode::FAILURE;
    };
    match (def.run)(&args) {
        Ok(output) => {
            println!("{}", output.text);
            for image in &output.images {
                let path = save_dir.join(format!("{}-{}.png", tool, image.name));
                if let Err(e) = std::fs::write(&path, &image.png) {
                    eprintln!("failed to write {}: {e}", path.display());
                    return std::process::ExitCode::FAILURE;
                }
                eprintln!("wrote {}", path.display());
            }
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tool error: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
