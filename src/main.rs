use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use rust_cli_template::cli::Args;
use rust_cli_template::output::{JsonFormatter, OutputFormatter, TextFormatter};
use rust_cli_template::parse_input;

fn run(args: &Args) -> Result<()> {
    args.validate().context("argument validation failed")?;

    let input = match &args.input {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("failed to read file: {}", path))?,
        None => {
            // デモ用: 引数なしの場合はサンプル入力を使用
            "42".to_string()
        }
    };

    if args.verbose {
        eprintln!("{} parsing input...", "[info]".blue());
    }

    let node = parse_input(&input)?;

    let formatter: Box<dyn OutputFormatter> = match args.format.as_str() {
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

fn main() {
    let args = Args::parse();

    if let Err(err) = run(&args) {
        eprintln!("{} {err:#}", "[error]".red().bold());
        std::process::exit(1);
    }
}
