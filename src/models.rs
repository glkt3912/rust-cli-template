use serde::{Deserialize, Serialize};

use crate::error::Span;

/// AST ノードの種別。言語実装のベースとして拡張可能。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    /// 整数リテラル
    IntLiteral(i64),
    /// 文字列リテラル
    StringLiteral(String),
    /// 識別子
    Identifier(String),
    /// 二項演算: (演算子, 左辺, 右辺)
    BinaryOp {
        op: String,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

/// AST の各ノード。ソースコード上の位置情報を保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub kind: NodeKind,
    pub span: Span,
}

impl Node {
    pub fn new(kind: NodeKind, span: Span) -> Self {
        Self { kind, span }
    }
}
