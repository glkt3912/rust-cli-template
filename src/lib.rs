pub mod cli;
pub mod error;
pub mod models;
pub mod output;
pub mod validator;

use error::{AppError, Span};
use models::{Node, NodeKind};

/// サンプル: 入力文字列をパースし、AST ノードとして返す。
/// 実際のプロジェクトではここにコアロジックを実装する。
pub fn parse_input(input: &str) -> Result<Node, AppError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Parse {
            span: Span::new(1, 1, 0),
            message: "input is empty".to_string(),
        });
    }

    // 整数リテラルとしてパースを試みる
    if let Ok(n) = trimmed.parse::<i64>() {
        return Ok(Node::new(NodeKind::IntLiteral(n), Span::new(1, 1, 0)));
    }

    // それ以外は識別子として扱う
    Ok(Node::new(
        NodeKind::Identifier(trimmed.to_string()),
        Span::new(1, 1, 0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer() {
        let node = parse_input("42").unwrap();
        assert!(matches!(node.kind, NodeKind::IntLiteral(42)));
    }

    #[test]
    fn parse_identifier() {
        let node = parse_input("hello").unwrap();
        assert!(matches!(node.kind, NodeKind::Identifier(ref s) if s == "hello"));
    }

    #[test]
    fn parse_empty_input_returns_error() {
        let result = parse_input("");
        assert!(result.is_err());
    }
}
