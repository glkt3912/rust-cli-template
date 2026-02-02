use clap::Parser;

use crate::error::AppError;

/// A professional Rust CLI template with modular architecture.
#[derive(Parser, Debug)]
#[command(name = "rust-cli-template", version, about)]
pub struct Args {
    /// Input file path to process
    #[arg(short, long)]
    pub input: Option<String>,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Output format (json / text)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

impl Args {
    /// clap の宣言的バリデーションでは表現できない制約を検証する。
    ///
    /// より複雑なバリデーションが必要な場合は、別途 `Config` 構造体を
    /// 作成し `validate()` メソッドを持たせる（gymeat の PlanConfig を参照）。
    pub fn validate(&self) -> Result<(), AppError> {
        match self.format.as_str() {
            "text" | "json" => {}
            other => {
                return Err(AppError::Other(format!(
                    "unsupported format: '{other}' (expected 'text' or 'json')"
                )));
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
            input: None,
            verbose: false,
            format: "json".to_string(),
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn validate_rejects_unknown_format() {
        let args = Args {
            input: None,
            verbose: false,
            format: "xml".to_string(),
        };
        assert!(args.validate().is_err());
    }
}
