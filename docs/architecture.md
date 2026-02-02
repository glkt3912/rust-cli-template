# Architecture

## モジュール依存関係

```
main.rs
  ├── cli::Args          (引数パース + validate)
  ├── output::*          (OutputFormatter トレイトディスパッチ)
  └── lib::parse_input   (コアロジック)
        ├── error::AppError, Span
        └── models::Node, NodeKind

lib.rs (モジュール宣言)
  ├── pub mod cli
  ├── pub mod error       ← 他モジュールから参照される基盤
  ├── pub mod models      ← error に依存
  ├── pub mod output      ← error, models に依存
  └── pub mod validator   ← 独立 (regex のみ)
```

## 二層エラーハンドリング

ライブラリ層とアプリケーション層でエラー戦略を分離している。

```
┌─────────────────────────────────────────────────────┐
│  main.rs  (アプリケーション層)                        │
│                                                     │
│  anyhow::Result<()>                                 │
│    ├── .context("...") でエラーに文脈を付与           │
│    └── main() で catch → colored で stderr 表示      │
├─────────────────────────────────────────────────────┤
│  lib.rs / error.rs  (ライブラリ層)                   │
│                                                     │
│  AppError (thiserror)                               │
│    ├── Parse { span, message }                      │
│    ├── Validation { span, message }                 │
│    ├── Io(#[from] std::io::Error)                   │
│    └── Other(String)                                │
└─────────────────────────────────────────────────────┘
```

### なぜ分けるのか

| 層 | クレート | 目的 |
|---|---|---|
| ライブラリ | `thiserror` | 呼び出し元が `match` でエラー種別を判定できる。型安全。 |
| アプリ | `anyhow` | エラーの連鎖 (`.context()`) と最終表示を担当。 |

`thiserror` の `AppError` は `std::error::Error` を実装するため、`anyhow::Error` に自動変換される (`?` 演算子)。ライブラリを外部クレートとして公開する場合でも、`AppError` 単体で利用可能。

### Span によるエラー位置追跡

```rust
#[error("[{span}] parse error: {message}")]
Parse { span: Span, message: String },
```

エラーメッセージに `[1:5] parse error: unexpected token` のように行・列が含まれる。パーサーやコンパイラの実装では、レキサーが各トークンに `Span` を付与し、それがエラー型まで伝搬する設計を想定している。

## エントリポイントの構造

```rust
fn main() {
    let args = Args::parse();           // clap がパース + --help/--version を処理
    if let Err(err) = run(&args) {
        eprintln!("{} {err:#}", "[error]".red().bold());
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {
    args.validate()?;                   // clap では表現できない制約
    let input = /* ファイル読み込み or デフォルト */;
    let node = parse_input(&input)?;    // ライブラリ層 → AppError → anyhow 変換
    let formatter: Box<dyn OutputFormatter> = /* 選択 */;
    let output = formatter.format_node(&node)?;
    println!("{output}");
    Ok(())
}
```

`main()` は `run()` のエラーをキャッチして表示するだけ。`run()` は `?` でエラーを伝搬する。この分離により `run()` のロジックがテスト可能になる。

## OutputFormatter トレイト

```
OutputFormatter (trait)
  ├── format_node(&self, node: &Node) -> Result<String, AppError>
  └── format_name(&self) -> &'static str
        │
        ├── TextFormatter   → Debug 表現
        └── JsonFormatter   → serde_json::to_string_pretty
```

### ディスパッチ方式

`main.rs` で `Box<dyn OutputFormatter>` を使い、実行時に切り替える。

```rust
let formatter: Box<dyn OutputFormatter> = match args.format.as_str() {
    "json" => Box::new(JsonFormatter),
    _ => Box::new(TextFormatter),
};
```

新しい形式 (CSV, YAML 等) を追加する場合:

1. `output.rs` に `struct CsvFormatter;` と `impl OutputFormatter` を追加
2. `main.rs` の `match` にアームを追加
3. `cli.rs` の `validate()` で受け入れる format 名を追加

### スケールアウト: ディレクトリモジュール化

ファイルが大きくなったら以下のように分割する (gymeat パターン):

```
src/output/
├── mod.rs          pub use で再エクスポート
├── formatter.rs    トレイト定義 + OutputDestination
├── text.rs         TextFormatter
├── json.rs         JsonFormatter
└── csv.rs          CsvFormatter (新規)
```

## バリデーション戦略

```
clap (宣言的)               Args::validate() (手続き的)
  ├── 型変換                  ├── format が既知値か
  ├── デフォルト値             ├── input ファイルの存在確認
  ├── 短縮フラグ               └── 将来: フィールド間の複合制約
  └── ヘルプ生成
```

clap の `#[arg]` アトリビュートでは表現できない検証 (ファイル存在、フィールド間の整合性) を `validate()` メソッドに集約する。gymeat では `PlanConfig::validate()` で「体重・身長・年齢のうちどれかが指定されたら全て必須」のような複合制約を実装している。

プロジェクトが大きくなったら `Args` → `Config` 変換レイヤーを追加し、`Config::validate()` に移す。

## データフロー

```
CLI入力 → Args::parse() → Args::validate()
                              │
                              ▼
            ファイル読み込み or デフォルト入力
                              │
                              ▼
                     parse_input(&str)
                              │
                              ▼
                     Node { kind, span }
                              │
                              ▼
               OutputFormatter::format_node()
                              │
                              ▼
                      stdout / stderr
```

各段階で `Result` を返し、エラーは `?` で `run()` → `main()` に伝搬する。
