use std::sync::LazyLock;

use regex::Regex;

/// 識別子の正規表現（英字またはアンダースコアで始まり、英数字・アンダースコアが続く）。
///
/// `std::sync::LazyLock` で一度だけコンパイルされる。
/// Rust 1.80 未満では `once_cell::sync::Lazy` が同等の機能を提供する
/// （youtube-audio-downloader の utils/validator.rs を参照）。
static IDENTIFIER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());

/// 文字列が有効な識別子かどうかを判定する。
pub fn is_valid_identifier(s: &str) -> bool {
    IDENTIFIER_REGEX.is_match(s)
}

// プラットフォーム固有の処理が必要な場合は #[cfg(target_os = "...")] を使う。
// youtube-audio-downloader の backend/src/services/dependency_checker.rs を参照。

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifiers() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("hello_world"));
        assert!(is_valid_identifier("x1"));
        assert!(is_valid_identifier("_"));
    }

    #[test]
    fn invalid_identifiers() {
        assert!(!is_valid_identifier("123abc"));
        assert!(!is_valid_identifier("hello world"));
        assert!(!is_valid_identifier("foo-bar"));
        assert!(!is_valid_identifier(""));
    }
}
