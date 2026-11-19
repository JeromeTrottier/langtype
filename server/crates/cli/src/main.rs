use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "kit", version, about = "Work with Mnemo kits")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate one or more kit folders (schema + references).
    Validate {
        #[arg(required = true)]
        dirs: Vec<PathBuf>,
    },
    /// Scaffold a kit folder (milestone M7).
    Init { dir: PathBuf },
    /// Run kit fixtures (milestone M7).
    Test { dir: PathBuf },
    /// Build a kit archive (milestone M7).
    Pack { dir: PathBuf },
}

fn main() -> anyhow::Result<ExitCode> {
    match Cli::parse().command {
        Command::Validate { dirs } => {
            let mut failed = false;
            for dir in dirs {
                let problems = mnemo_kits::validate_dir(&dir)?;
                if problems.is_empty() {
                    println!("ok      {}", dir.display());
                } else {
                    failed = true;
                    println!("invalid {}", dir.display());
                    for p in problems {
                        println!("  {p}");
                    }
                }
            }
            Ok(if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS })
        }
        Command::Init { .. } | Command::Test { .. } | Command::Pack { .. } => {
            anyhow::bail!("not implemented yet (milestone M7)")
        }
    }
}
