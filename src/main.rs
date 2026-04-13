use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use rust_cli_template::cli::{Args, SubCommand};
use rust_cli_template::output::{JsonFormatter, OutputFormatter, TextFormatter};
use rust_cli_template::parse_input;

fn load_input(input: &Option<String>) -> Result<String> {
    match input {
        Some(path) => {
            std::fs::read_to_string(path).with_context(|| format!("failed to read file: {}", path))
        }
        None => Ok("42".to_string()),
    }
}

fn run_parse(input: &str, format: &str, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("{} parsing input...", "[info]".blue());
    }

    let node = parse_input(input)?;

    let formatter: Box<dyn OutputFormatter> = match format {
        "json" => Box::new(JsonFormatter),
        _ => Box::new(TextFormatter),
    };

    let output = formatter
        .format_node(&node)
        .context("output formatting failed")?;

    match formatter.format_name() {
        "text" => println!("{} {output}", "[result]".green()),
        _ => println!("{output}"),
    };

    Ok(())
}

fn run_check(input: &str, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("{} checking input...", "[info]".blue());
    }

    parse_input(input)?;

    if verbose {
        eprintln!("{} input is valid", "[info]".blue());
    }

    Ok(())
}

fn run(args: &Args) -> Result<()> {
    args.validate().context("argument validation failed")?;

    let input = load_input(&args.input)?;

    match args.command.as_ref().unwrap_or(&SubCommand::Parse) {
        SubCommand::Parse => run_parse(&input, &args.format, args.verbose)?,
        SubCommand::Check => run_check(&input, args.verbose)?,
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(err) = run(&args) {
        eprintln!("{} {err:#}", "[error]".red().bold());
        std::process::exit(1);
    }
}
