# Crate Guide

各依存クレートの役割、選定理由、典型的な使い方。

---

## clap (v4, derive)

**役割**: コマンドライン引数のパースとヘルプ生成。

**なぜ clap か**: Rust CLI エコシステムのデファクト。derive マクロで構造体から引数定義を自動生成でき、ボイラープレートが最小限。

**テンプレートでの使用箇所**: `src/cli.rs`

```rust
#[derive(Parser, Debug)]
#[command(name = "rust-cli-template", version, about)]
pub struct Args {
    #[arg(short, long)]
    pub input: Option<String>,
}
```

**覚えておくべきポイント**:

- `#[command(version)]`: `Cargo.toml` の version を自動取得
- `#[arg(short, long)]`: `-i` と `--input` の両方を生成
- `#[arg(default_value = "text")]`: 省略時のデフォルト値
- `#[command(subcommand)]`: サブコマンドの定義 (gymeat パターン)
- `#[arg(conflicts_with = "other")]`: 引数間の排他制約 (rpg パターン)

---

## thiserror (v2)

**役割**: ライブラリ層のカスタムエラー型を簡潔に定義。

**なぜ thiserror か**: `std::fmt::Display` と `std::error::Error` の実装を derive マクロで自動生成。rpg のように手動実装する必要がなくなる。

**テンプレートでの使用箇所**: `src/error.rs`

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("[{span}] parse error: {message}")]
    Parse { span: Span, message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),  // #[from] で自動変換
}
```

**覚えておくべきポイント**:

- `#[error("...")]`: `Display` 実装のフォーマット文字列
- `#[from]`: 他のエラー型からの `From` 実装を自動生成 (? 演算子で変換)
- `#[source]`: エラーチェインの source() を指定
- thiserror v2 は Rust 1.56+ 対応、MSRV が改善されている

---

## anyhow (v1)

**役割**: アプリケーション層のエラーハンドリング。

**なぜ anyhow か**: `anyhow::Result<T>` = `Result<T, anyhow::Error>` で、あらゆるエラー型を受け入れる。`.context()` でエラーに文脈情報を付与できる。

**テンプレートでの使用箇所**: `src/main.rs`

```rust
fn run(args: &Args) -> anyhow::Result<()> {
    args.validate().context("argument validation failed")?;
    // AppError → anyhow::Error に自動変換される
    let node = parse_input(&input)?;
    // ...
}
```

**覚えておくべきポイント**:

- `context("msg")`: エラーに文脈を追加（失敗時のみ評価）
- `with_context(|| format!("..."))`: 動的な文脈メッセージ
- `{err:#}`: alternate 表示でエラーチェイン全体を表示
- `{err:?}`: デバッグ表示でバックトレースも含む
- `anyhow::bail!("msg")`: `return Err(anyhow!("msg"))` のショートカット

**thiserror との使い分け**:

| | thiserror | anyhow |
|---|---|---|
| 用途 | ライブラリ (呼び出し元が match する) | アプリ (最終的に表示する) |
| エラー型 | カスタム enum | `anyhow::Error` (型消去) |
| パターンマッチ | 可能 | `downcast_ref` が必要 |
| 文脈付与 | なし | `.context()` |

---

## serde / serde_json (v1)

**役割**: データ構造のシリアライズ / デシリアライズ。

**なぜ serde か**: Rust のシリアライズフレームワークのデファクト。derive マクロで自動導出。

**テンプレートでの使用箇所**: `src/error.rs` (Span), `src/models.rs` (Node, NodeKind), `src/output.rs` (JsonFormatter)

```rust
#[derive(Serialize, Deserialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

// JSON 出力
serde_json::to_string_pretty(&node)?;
```

**覚えておくべきポイント**:

- `#[serde(rename_all = "camelCase")]`: フィールド名の命名規則変換
- `#[serde(skip)]`: 特定フィールドをシリアライズから除外
- `#[serde(tag = "type")]`: enum のタグ形式指定
- gymeat では履歴の JSON 永続化にも使用

---

## colored (v3)

**役割**: ターミナル出力への色付け。

**テンプレートでの使用箇所**: `src/main.rs`

```rust
use colored::Colorize;

eprintln!("{} {err:#}", "[error]".red().bold());
eprintln!("{} parsing...", "[info]".blue());
println!("{} {output}", "[result]".green());
```

**覚えておくべきポイント**:

- `.red()`, `.green()`, `.blue()`: 前景色
- `.bold()`, `.italic()`, `.underline()`: 装飾
- `.on_red()`: 背景色
- rpg では文字種ごとの色分け (大文字=Blue, 数字=Yellow, 記号=Red) を実装
- `NO_COLOR` 環境変数でグローバルに色を無効化可能
- rpg/gymeat は `atty` クレートで TTY 判定をしているが、colored v3 は自動判定をサポート

---

## regex (v1)

**役割**: 正規表現によるパターンマッチング。

**テンプレートでの使用箇所**: `src/validator.rs`

```rust
use std::sync::LazyLock;
use regex::Regex;

static IDENTIFIER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());
```

**覚えておくべきポイント**:

- `Regex::new()` はコンパイルコストが高い → `LazyLock` でキャッシュ
- `is_match()`: マッチ判定のみ (キャプチャ不要な場合に最速)
- `captures()`: キャプチャグループの取得
- youtube-audio-downloader では `\[download\]\s+(\d+\.?\d*)%` でプログレスをパース
- rpg の統合テストでは `predicate::str::is_match()` で出力形式を正規表現検証

---

## assert_cmd / predicates (dev-dependencies)

**役割**: CLI バイナリの統合テスト。

**テンプレートでの使用箇所**: `tests/cli_integration.rs`

```rust
fn cmd() -> assert_cmd::Command {
    let bin = cargo_bin("rust-cli-template");
    assert_cmd::Command::from(std::process::Command::new(bin))
}

#[test]
fn test_json_format() {
    cmd()
        .args(&["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\""));
}
```

**覚えておくべきポイント**:

- `cargo_bin("name")`: Cargo パッケージ名からバイナリパスを解決
- `.assert().success()`: 終了コード 0 を検証
- `.assert().failure()`: 非ゼロ終了コードを検証
- `predicate::str::contains()`: 部分文字列マッチ
- `predicate::str::is_match()`: 正規表現マッチ (rpg パターン)
- `.stdout()` / `.stderr()`: 出力先ごとの検証
