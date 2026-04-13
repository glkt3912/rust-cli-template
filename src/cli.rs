use clap::{Parser, Subcommand};

use crate::error::AppError;

/// A professional Rust CLI template with modular architecture.
#[derive(Parser, Debug)]
#[command(name = "rust-cli-template", version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<SubCommand>,

    /// Input file path to process
    #[arg(short, long)]
    pub input: Option<String>,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Output format (json / text)  [parse サブコマンドで使用]
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

/// サブコマンド一覧。
/// 省略時は `parse` と同等の動作をする（後方互換性の維持）。
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// 入力をパースし AST を表示する（デフォルト動作）
    Parse,
    /// 入力を検証するのみ。出力なし、終了コードで結果を返す
    Check,
}

impl Args {
    /// clap の宣言的バリデーションでは表現できない制約を検証する。
    pub fn validate(&self) -> Result<(), AppError> {
        // `check` サブコマンドでは format は使用しないためスキップ
        let is_check = matches!(self.command, Some(SubCommand::Check));
        if !is_check {
            match self.format.as_str() {
                "text" | "json" => {}
                other => {
                    return Err(AppError::Other(format!(
                        "unsupported format: '{other}' (expected 'text' or 'json')"
                    )));
                }
            }
        }

        if let Some(ref path) = self.input
            && !std::path::Path::new(path).exists()
        {
            return Err(AppError::Other(format!("input file not found: '{path}'")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_known_formats() {
        let args = Args {
            command: None,
            input: None,
            verbose: false,
            format: "json".to_string(),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn validate_rejects_unknown_format() {
        let args = Args {
            command: None,
            input: None,
            verbose: false,
            format: "xml".to_string(),
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn validate_skips_format_check_for_check_subcommand() {
        let args = Args {
            command: Some(SubCommand::Check),
            input: None,
            verbose: false,
            format: "xml".to_string(), // check では無視される
        };
        assert!(args.validate().is_ok());
    }
}
