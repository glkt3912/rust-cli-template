# rust-cli-template

Rust CLI / ライブラリプロジェクトのテンプレート。モジュール分離、二層エラーハンドリング、トレイトベースの出力抽象化など、実プロジェクトで実証済みのパターンを組み込んでいる。

## 特徴

- **二層エラーハンドリング**: ライブラリ層は `thiserror`、アプリ層は `anyhow` で分離
- **Span 付きエラー**: ソースコードの行・列情報を保持し、パーサーや言語実装に応用可能
- **OutputFormatter トレイト**: 出力形式の差し替えが容易（text / JSON、拡張可能）
- **バリデーション分離**: `Args::validate()` で clap では表現できない制約を検証
- **LazyLock 正規表現**: `std::sync::LazyLock` による一度きりの正規表現コンパイル
- **統合テスト**: `assert_cmd` + `predicates` による CLI 動作の自動検証
- **CI 対応**: GitHub Actions で 3OS (macOS, Ubuntu, Windows) × fmt / clippy / test
- **リリース最適化**: LTO, strip, codegen-units=1 による小サイズバイナリ

## プロジェクト構成

```
src/
├── main.rs        CLI エントリポイント (anyhow + colored エラー表示)
├── lib.rs         コアロジック + モジュール宣言 + ユニットテスト
├── cli.rs         clap v4 derive による引数定義 + validate()
├── error.rs       Span 構造体 + AppError enum (thiserror)
├── models.rs      AST ノード (Node / NodeKind, serde 対応)
├── output.rs      OutputFormatter トレイト + Text / JSON 実装
└── validator.rs   LazyLock<Regex> による識別子バリデーション
tests/
└── cli_integration.rs   assert_cmd 統合テスト (5件)
.github/workflows/
└── ci.yml               GitHub Actions CI パイプライン
```

## インストール

```bash
git clone <repository-url>
cd rust-cli-template
cargo build
```

## 使い方

```bash
# デフォルト実行 (サンプル入力 "42" をパース)
cargo run
# => [result] Node { kind: IntLiteral(42), span: Span { line: 1, column: 1, offset: 0 } }

# JSON 形式で出力
cargo run -- --format json
# => {
# =>   "kind": { "IntLiteral": 42 },
# =>   "span": { "line": 1, "column": 1, "offset": 0 }
# => }

# ファイルを入力として指定
cargo run -- --input sample.txt

# verbose モード
cargo run -- --verbose
```

### オプション一覧

| オプション | 短縮 | 説明 | デフォルト |
|---|---|---|---|
| `--input <FILE>` | `-i` | 入力ファイルパス | なし (サンプル入力) |
| `--format <FMT>` | `-f` | 出力形式 (`text` / `json`) | `text` |
| `--verbose` | `-v` | 詳細出力を有効化 | `false` |
| `--help` | `-h` | ヘルプ表示 | |
| `--version` | `-V` | バージョン表示 | |

## 開発

```bash
# テスト実行 (ユニット + 統合)
cargo test

# Clippy (警告をエラーとして扱う)
cargo clippy -- -D warnings

# フォーマットチェック
cargo fmt -- --check

# リリースビルド (LTO + strip 有効)
cargo build --release
```

## 設計パターンの出典

このテンプレートは以下の実プロジェクトから設計パターンを抽出している。

| パターン | 出典プロジェクト |
|---|---|
| リリースプロファイル最適化 | rpg (パスワード生成CLI) |
| `assert_cmd` 統合テスト | rpg |
| `OutputFormatter` トレイト | gymeat (食事プランCLI) |
| `validate()` バリデーション分離 | gymeat |
| マルチOS CI パイプライン | gymeat |
| `LazyLock<Regex>` パターン | youtube-audio-downloader |

## 拡張ガイド

- **出力形式の追加**: `OutputFormatter` トレイトを実装し `main.rs` の match に追加
- **エラー種別の追加**: `AppError` enum にバリアントを追加
- **モジュールの階層化**: 規模が大きくなったら `src/output/mod.rs` + 個別ファイルに分割
- **プラットフォーム分岐**: `#[cfg(target_os = "...")]` で OS 固有処理を追加

## 技術スタック

| クレート | 用途 |
|---|---|
| `clap` v4 | CLI 引数定義 (derive) |
| `anyhow` | アプリ層エラーハンドリング |
| `thiserror` | ライブラリ層エラー定義 |
| `serde` / `serde_json` | シリアライズ / JSON 出力 |
| `colored` | ターミナルカラー出力 |
| `regex` | 正規表現バリデーション |
| `assert_cmd` / `predicates` | CLI 統合テスト |
