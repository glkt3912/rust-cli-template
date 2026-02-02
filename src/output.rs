use std::path::PathBuf;

use crate::error::AppError;
use crate::models::Node;

/// 出力フォーマットの抽象化トレイト。
///
/// プロジェクトが成長したら `src/output/mod.rs` に分割し、
/// フォーマットごとに別ファイルにする（gymeat の src/output/ パターン）。
pub trait OutputFormatter {
    /// AST ノードをフォーマットした文字列を返す。
    fn format_node(&self, node: &Node) -> Result<String, AppError>;

    /// フォーマット名を返す（例: "text", "json"）。
    fn format_name(&self) -> &'static str;
}

/// テキスト形式: Debug 表現で出力。
pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format_node(&self, node: &Node) -> Result<String, AppError> {
        Ok(format!("{node:?}"))
    }

    fn format_name(&self) -> &'static str {
        "text"
    }
}

/// JSON 形式: serde_json で整形出力。
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_node(&self, node: &Node) -> Result<String, AppError> {
        serde_json::to_string_pretty(node)
            .map_err(|e| AppError::Other(format!("JSON serialization failed: {e}")))
    }

    fn format_name(&self) -> &'static str {
        "json"
    }
}

/// 出力先の指定。
pub enum OutputDestination {
    Stdout,
    File(PathBuf),
}

/// 指定された出力先にコンテンツを書き込む。
pub fn write_output(content: &str, destination: OutputDestination) -> Result<(), AppError> {
    match destination {
        OutputDestination::Stdout => {
            println!("{content}");
            Ok(())
        }
        OutputDestination::File(path) => {
            std::fs::write(&path, content)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;
    use crate::models::{Node, NodeKind};

    #[test]
    fn text_formatter_formats_node() {
        let node = Node::new(NodeKind::IntLiteral(42), Span::new(1, 1, 0));
        let formatter = TextFormatter;
        let result = formatter.format_node(&node).unwrap();
        assert!(result.contains("42"));
    }

    #[test]
    fn json_formatter_produces_valid_json() {
        let node = Node::new(NodeKind::IntLiteral(42), Span::new(1, 1, 0));
        let formatter = JsonFormatter;
        let result = formatter.format_node(&node).unwrap();
        let _: serde_json::Value = serde_json::from_str(&result).unwrap();
    }
}
