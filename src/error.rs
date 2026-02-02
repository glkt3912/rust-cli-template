use std::fmt;
use thiserror::Error;

/// ソースコード上の位置情報（行・列）を保持する構造体。
/// 言語処理系やパーサーでのエラー報告に利用可能。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// ライブラリレベルのエラー型。
/// `thiserror` による厳密な定義で、上位の `anyhow` と組み合わせて使う。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("[{span}] parse error: {message}")]
    Parse { span: Span, message: String },

    #[error("[{span}] validation error: {message}")]
    Validation { span: Span, message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}
