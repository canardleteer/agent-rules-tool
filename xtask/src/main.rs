mod spec_update;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "agent-rules-tool workspace tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Refresh vendored agent-rules-spec artifacts under spec/
    SpecUpdate {
        #[command(subcommand)]
        command: SpecUpdateCommand,
    },
}

#[derive(Subcommand)]
enum SpecUpdateCommand {
    /// Re-discover upstream files and refresh spec/
    Fresh(SpecUpdateArgs),
    /// Re-pull files listed in spec/index.yaml
    Update(SpecUpdateArgs),
    /// Check whether vendored spec/ is current with upstream
    Check(SpecUpdateArgs),
}

#[derive(clap::Args)]
struct SpecUpdateArgs {
    /// Upstream git reference (branch, tag, or commit). Default: repository default branch.
    #[arg(long = "ref")]
    git_ref: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::SpecUpdate { command } => match command {
            SpecUpdateCommand::Fresh(args) => spec_update::run_fresh(args.git_ref.as_deref()),
            SpecUpdateCommand::Update(args) => spec_update::run_update(args.git_ref.as_deref()),
            SpecUpdateCommand::Check(args) => spec_update::run_check(args.git_ref.as_deref()),
        },
    }
}
