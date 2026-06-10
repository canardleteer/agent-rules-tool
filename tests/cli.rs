use agent_rules_tool::cli::Cli;
use clap::CommandFactory;

#[test]
fn test_cli_builds() {
    Cli::command().debug_assert();
}
