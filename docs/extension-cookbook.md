# Extension Cookbook

テンプレートを拡張する際の具体的な手順集。

---

## 1. 出力形式を追加する (例: CSV)

### 変更ファイル

- `src/output.rs` (実装追加)
- `src/cli.rs` (validate に追加)
- `src/main.rs` (match にアーム追加)

### 手順

**output.rs**: フォーマッターを追加

```rust
pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
    fn format_node(&self, node: &Node) -> Result<String, AppError> {
        // Node の各フィールドをカンマ区切りで出力
        let kind = match &node.kind {
            NodeKind::IntLiteral(n) => format!("int,{n}"),
            NodeKind::StringLiteral(s) => format!("string,{s}"),
            NodeKind::Identifier(s) => format!("identifier,{s}"),
            NodeKind::BinaryOp { op, .. } => format!("binop,{op}"),
        };
        Ok(format!("type,value\n{kind}"))
    }

    fn format_name(&self) -> &'static str {
        "csv"
    }
}
```

**cli.rs**: validate に "csv" を追加

```rust
match self.format.as_str() {
    "text" | "json" | "csv" => {}
    // ...
}
```

**main.rs**: match にアーム追加

```rust
let formatter: Box<dyn OutputFormatter> = match args.format.as_str() {
    "json" => Box::new(JsonFormatter),
    "csv" => Box::new(CsvFormatter),
    _ => Box::new(TextFormatter),
};
```

---

## 2. エラー種別を追加する

### 変更ファイル

- `src/error.rs` (バリアント追加)

### 手順

**error.rs**: AppError に新バリアントを追加

```rust
#[derive(Debug, Error)]
pub enum AppError {
    // 既存...

    #[error("[{span}] type error: expected {expected}, found {found}")]
    TypeError {
        span: Span,
        expected: String,
        found: String,
    },

    #[error("config error: {0}")]
    Config(String),
}
```

`#[from]` アトリビュートで外部エラー型からの自動変換も可能:

```rust
#[error("serialization error: {0}")]
Serialization(#[from] serde_json::Error),
```

---

## 3. サブコマンドを追加する (gymeat パターン)

### 変更ファイル

- `src/cli.rs` (サブコマンド定義)
- `src/main.rs` (ディスパッチ)

### 手順

**cli.rs**: clap のサブコマンド

```rust
#[derive(Parser, Debug)]
#[command(name = "rust-cli-template", version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<SubCommand>,

    // 既存のフラグはグローバルオプションにする
    #[arg(short, long, default_value_t = false, global = true)]
    pub verbose: bool,
}

#[derive(clap::Subcommand, Debug)]
pub enum SubCommand {
    /// Parse input and display AST
    Parse {
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Validate input without producing output
    Check {
        #[arg(short, long)]
        input: String,
    },
}
```

**main.rs**: サブコマンドで分岐

```rust
fn run(args: &Args) -> Result<()> {
    match &args.command {
        Some(SubCommand::Parse { input, format }) => run_parse(input, format, args.verbose),
        Some(SubCommand::Check { input }) => run_check(input),
        None => run_parse(&None, &"text".to_string(), args.verbose),
    }
}
```

gymeat では `history list`, `history show <id>`, `history delete <id>` のようなネストしたサブコマンドも実装している。

---

## 4. ファイル永続化を追加する (gymeat パターン)

### 新規ファイル

- `src/history/mod.rs`
- `src/history/models.rs`
- `src/history/storage.rs`

### 手順

gymeat の履歴管理パターンに従い、`~/.rust-cli-template/` に JSON ファイルを保存する。

**history/models.rs**:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,          // UUID v4
    pub timestamp: String,   // RFC 3339
    pub input: String,
    pub result: String,
}
```

**history/storage.rs**:

```rust
use std::path::PathBuf;

pub struct HistoryStorage {
    dir: PathBuf,
}

impl HistoryStorage {
    pub fn new() -> Self {
        let dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".rust-cli-template")
            .join("history");
        std::fs::create_dir_all(&dir).ok();
        Self { dir }
    }

    pub fn save(&self, entry: &HistoryEntry) -> Result<(), std::io::Error> {
        let path = self.dir.join(format!("{}.json", entry.id));
        let json = serde_json::to_string_pretty(entry)?;
        std::fs::write(path, json)
    }

    pub fn list(&self) -> Result<Vec<HistoryEntry>, std::io::Error> {
        // ディレクトリ内の .json ファイルを読み込み
        // ...
    }
}
```

必要なクレート追加: `uuid = { version = "1", features = ["v4"] }`, `chrono`, `dirs`

---

## 5. OutputFormatter をディレクトリモジュールに分割する

### 変更

- `src/output.rs` → `src/output/mod.rs` + 個別ファイル

### 手順

```bash
mkdir src/output
mv src/output.rs src/output/mod.rs
```

**src/output/mod.rs**:

```rust
pub mod formatter;
pub mod json;
pub mod text;

pub use formatter::{write_output, OutputDestination, OutputFormatter};
pub use json::JsonFormatter;
pub use text::TextFormatter;
```

**src/output/formatter.rs**: トレイト定義と OutputDestination

**src/output/text.rs**: TextFormatter

**src/output/json.rs**: JsonFormatter

`lib.rs` の `pub mod output;` はそのまま動作する（Rust は `output.rs` と `output/mod.rs` を同等に扱う）。

---

## 6. プラットフォーム固有処理を追加する

### 手順

youtube-audio-downloader では `#[cfg(target_os)]` でコンパイル時分岐を行っている。

```rust
pub fn default_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().unwrap().join("Library/Application Support/myapp")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().unwrap().join("myapp")
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().unwrap().join("myapp")
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from(".myapp")
    }
}
```

---

## 7. 外部プロセス連携を追加する (youtube-audio-downloader パターン)

### 手順

youtube-audio-downloader は `yt-dlp` を子プロセスとして起動し、stdout をリアルタイムにパースしている。

```rust
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub fn run_external_tool(args: &[&str]) -> Result<String, AppError> {
    let mut child = Command::new("external-tool")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Other(format!("failed to spawn process: {e}")))?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let mut output = String::new();
    for line in reader.lines() {
        let line = line?;
        // リアルタイム処理 (プログレスバー、パース等)
        output.push_str(&line);
        output.push('\n');
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(AppError::Other("external tool failed".to_string()));
    }

    Ok(output)
}
```

言語処理系では、外部のコンパイラやリンカを呼び出す場合にこのパターンが使える。
