use agent_rules_tool::cli::{Cli, Commands, SeverityArg};
use agent_rules_tool::format::RuleFormat;
use agent_rules_tool::lint::{exceeds_threshold, lint_directory, lint_string};
use agent_rules_tool::migrate::{
    MigrateOptions, MigrateWarning, build_inputs_from_dirs, migrate_paths, migrate_string,
};
use agent_rules_tool::report::{report_to_yaml_aggregate, report_to_yaml_single};
use agent_rules_tool::spec::DEFAULT_LINT_DIR;
use agent_rules_tool::{FileLintResult, LintOptions, Severity};
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match run(cli).await {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
            error!(error = %err, "command failed");
            ExitCode::from(1)
        }
    }
}

async fn run(cli: Cli) -> Result<u8, agent_rules_tool::Error> {
    match cli.command {
        Commands::Lint {
            directory,
            input_file,
            severity,
            report,
        } => run_lint(directory, input_file, severity, report).await,
        Commands::Migrate {
            from,
            directory,
            input_file,
            to,
            output,
            force,
        } => run_migrate(from, directory, input_file, to, output, force).await,
    }
}

async fn run_lint(
    directory: Option<PathBuf>,
    input_file: Option<PathBuf>,
    severity: SeverityArg,
    report: Option<PathBuf>,
) -> Result<u8, agent_rules_tool::Error> {
    let threshold: Severity = severity.into();
    let options = LintOptions {
        severity_threshold: threshold,
        filename_hint: None,
    };

    let (results, single): (Vec<FileLintResult>, Option<FileLintResult>) =
        if let Some(file) = input_file {
            let content = tokio::fs::read_to_string(&file).await?;
            let filename_hint = file
                .file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string);
            let mut opts = options.clone();
            opts.filename_hint = filename_hint;
            let report_result = lint_string(&content, &opts)?;
            emit_violations(&file.display().to_string(), &report_result.violations);
            let file_result = FileLintResult {
                path: file.clone(),
                report: report_result,
            };
            (Vec::new(), Some(file_result))
        } else {
            let dir = directory.unwrap_or_else(|| PathBuf::from(DEFAULT_LINT_DIR));
            if !dir.exists() {
                warn!(dir = %dir.display(), "directory does not exist; nothing to lint");
                return Ok(0);
            }
            let results = lint_directory(&dir, &options)?;
            if results.is_empty() {
                info!(dir = %dir.display(), "no markdown files found");
            }
            for r in &results {
                emit_violations(&r.path.display().to_string(), &r.report.violations);
            }
            (results, None)
        };

    let failed = if let Some(ref file_result) = single {
        exceeds_threshold(&file_result.report.violations, threshold)
    } else if results.is_empty() {
        false
    } else {
        results
            .iter()
            .any(|r| exceeds_threshold(&r.report.violations, threshold))
    };

    if let Some(path) = report {
        let yaml = if let Some(ref file_result) = single {
            report_to_yaml_single(&file_result.report)?
        } else {
            report_to_yaml_aggregate(&results)?
        };
        tokio::fs::write(&path, yaml).await?;
        info!(path = %path.display(), "wrote lint report");
    }

    Ok(if failed { 1 } else { 0 })
}

async fn run_migrate(
    from: agent_rules_tool::format::RuleFormatArg,
    directory: Option<PathBuf>,
    input_file: Option<PathBuf>,
    to: agent_rules_tool::format::RuleFormatArg,
    output: Option<PathBuf>,
    force: bool,
) -> Result<u8, agent_rules_tool::Error> {
    let from_format = RuleFormat::from(from);
    let to_format = RuleFormat::from(to);
    let options = MigrateOptions {
        from: from_format,
        to: to_format,
        force,
        filename_hint: None,
    };

    if let Some(file) = input_file {
        let content = tokio::fs::read_to_string(&file).await?;
        let filename_hint = file
            .file_stem()
            .and_then(|s| s.to_str())
            .map(str::to_string);
        let mut opts = options.clone();
        opts.filename_hint = filename_hint;
        let migrated = migrate_string(&content, &opts)?;
        emit_migration_warnings(&migrated.warnings);

        if let Some(out) = output {
            let written =
                agent_rules_tool::io::write_file_atomic(&out, &migrated.content, force).await?;
            if written == agent_rules_tool::io::WriteOutcome::Skipped {
                warn!(path = %out.display(), "output exists; skipping (use --force to overwrite)");
            } else {
                info!(path = %out.display(), "wrote migrated file");
            }
        } else {
            print!("{}", migrated.content);
        }
        return Ok(0);
    }

    let project_root = env::current_dir()?;
    let output_root = output
        .clone()
        .unwrap_or_else(|| PathBuf::from(to_format.default_output_dir()));

    let inputs = build_inputs_from_dirs(&project_root, directory.as_deref(), from_format)?;
    if inputs.is_empty() {
        warn!("no rule files found to migrate");
        return Ok(0);
    }

    let summary = migrate_paths(&inputs, &output_root, &options).await?;
    emit_migration_warnings(&summary.warnings);
    info!(
        written = summary.written.len(),
        skipped = summary.skipped.len(),
        output = %output_root.display(),
        "migration complete"
    );
    Ok(0)
}

fn emit_violations(path: &str, violations: &[agent_rules_tool::Violation]) {
    for v in violations {
        match v.severity {
            Severity::Error => error!(
                path = path,
                field = %v.field,
                spec = %v.spec_ref,
                "{}",
                v.message
            ),
            Severity::Warn => warn!(
                path = path,
                field = %v.field,
                spec = %v.spec_ref,
                "{}",
                v.message
            ),
        }
    }
}

fn emit_migration_warnings(warnings: &[MigrateWarning]) {
    for w in warnings {
        if let Some(ref field) = w.field {
            warn!(field = %field, "{}", w.message);
        } else {
            warn!("{}", w.message);
        }
    }
}
