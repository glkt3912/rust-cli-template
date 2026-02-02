# Design Patterns

このテンプレートに組み込まれた設計パターンと、その出典となった実プロジェクトの対比。

---

## 1. 二層エラーハンドリング (thiserror + anyhow)

**テンプレートでの実装**: `error.rs` + `main.rs`

```rust
// ライブラリ層: 型安全なエラー (error.rs)
#[derive(Debug, Error)]
pub enum AppError {
    #[error("[{span}] parse error: {message}")]
    Parse { span: Span, message: String },
    // ...
}

// アプリ層: 文脈付きエラー伝搬 (main.rs)
fn run(args: &Args) -> anyhow::Result<()> {
    args.validate().context("argument validation failed")?;
    // AppError は自動で anyhow::Error に変換される
}
```

**出典比較**:

| プロジェクト | ライブラリ層 | アプリ層 |
|---|---|---|
| **テンプレート** | `thiserror` (AppError) | `anyhow` (.context()) |
| **rpg** | 手動 `Display` + `Error` 実装 (RpgError) | `process::exit(1)` |
| **gymeat** | `thiserror` (MealPlannerError) | `process::exit` + exit code マッピング |
| **youtube-audio-downloader** | なし (Tauri が吸収) | `anyhow` (.context()) |

**rpg のアプローチ**:

rpg は `thiserror` を使わず `fmt::Display` と `std::error::Error` を手動実装している。小規模プロジェクトではこれで十分だが、バリアント数が増えると `thiserror` の `#[error("...")]` マクロの方が保守しやすい。

```rust
// rpg の手動実装 (error.rs)
impl fmt::Display for RpgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpgError::InvalidLength(len) => write!(f, "Error: Invalid password length: {}", len),
            // 全バリアントを手書き...
        }
    }
}
impl std::error::Error for RpgError {}
```

**gymeat のアプローチ**:

gymeat は `thiserror` を使い、さらにエラーの exit code マッピングを行っている。

```rust
// gymeat: エラー種別ごとに exit code を分ける
match error_kind {
    MealPlannerError::InvalidWeight(_) => exit(2),
    MealPlannerError::FileWriteError { .. } => exit(3),
    // ...
}
```

テンプレートでは exit code は一律 1 としている。分けたい場合は gymeat パターンを参照。

---

## 2. OutputFormatter トレイト

**テンプレートでの実装**: `output.rs`

```rust
pub trait OutputFormatter {
    fn format_node(&self, node: &Node) -> Result<String, AppError>;
    fn format_name(&self) -> &'static str;
}
```

**出典: gymeat の OutputFormatter**:

```rust
// gymeat: ドメイン固有の3メソッド + 名前ゲッター
pub trait OutputFormatter {
    fn format_daily_plan(&self, plan: &DailyPlan, db: &MealDatabase, show_recipe: bool) -> Result<String>;
    fn format_weekly_plan(&self, plan: &WeeklyPlan, db: &MealDatabase, show_recipe: bool) -> Result<String>;
    fn format_monthly_plan(&self, plan: &MonthlyPlan, db: &MealDatabase, show_recipe: bool) -> Result<String>;
    fn format_name(&self) -> &'static str;
}
```

gymeat は 6 つの実装 (Terminal, JSON, CSV, Markdown, PDF) を持ち、`src/output/` ディレクトリに分割している。テンプレートは 2 実装 (Text, JSON) を 1 ファイルに収め、分割の指針をコメントで示している。

**ディスパッチの違い**:

```rust
// テンプレート: Box<dyn Trait> (実行時ディスパッチ)
let formatter: Box<dyn OutputFormatter> = match args.format.as_str() {
    "json" => Box::new(JsonFormatter),
    _ => Box::new(TextFormatter),
};

// gymeat: match 内で直接呼び出し (コンパイル時ディスパッチ)
let content = match format {
    "json" => JsonFormatter.format_daily_plan(&plan, &db, show_recipe)?,
    "csv" => CsvFormatter.format_daily_plan(&plan, &db, show_recipe)?,
    // ...
};
```

テンプレートはトレイトオブジェクト (`Box<dyn>`) を使い、パターンの汎用性を示している。パフォーマンスが重要なら gymeat のようにコンパイル時ディスパッチを使う。

---

## 3. バリデーション分離

**テンプレートでの実装**: `cli.rs` の `Args::validate()`

```rust
impl Args {
    pub fn validate(&self) -> Result<(), AppError> {
        match self.format.as_str() {
            "text" | "json" => {}
            other => return Err(AppError::Other(format!("unsupported format: '{other}'")))
        }
        // ...
    }
}
```

**出典: gymeat の PlanConfig::validate()**:

```rust
// gymeat: Option フィールドの複合制約
pub fn validate(&self) -> Result<()> {
    if let Some(w) = self.weight {
        if w <= 0.0 || w > MAX_WEIGHT_KG as f32 {
            return Err(MealPlannerError::InvalidWeight(w));
        }
    }
    // 「体重・身長・年齢のどれかが指定されたら全て必須」
    if self.custom_calories.is_none()
        && (self.weight.is_some() || self.height.is_some() || self.age.is_some())
    {
        if self.weight.is_none() || self.height.is_none() || self.age.is_none() || self.gender.is_none() {
            return Err(MealPlannerError::ConfigValidationError("...".to_string()));
        }
    }
    Ok(())
}
```

gymeat のパターンが示す重要な点:
- **範囲チェック**: `contains()` やぶ比較で数値範囲を検証
- **Option の複合制約**: 「どれか一つが Some なら全て Some」のようなフィールド間依存
- **ドメイン固有エラー型**: `InvalidWeight(f32)` のように値を保持

テンプレートではシンプルな enum マッチと存在チェックのみ実装し、上記の発展パターンをここに記録している。

---

## 4. LazyLock 正規表現キャッシュ

**テンプレートでの実装**: `validator.rs`

```rust
use std::sync::LazyLock;
use regex::Regex;

static IDENTIFIER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());
```

**出典: youtube-audio-downloader の once_cell パターン**:

```rust
use once_cell::sync::Lazy;
use regex::Regex;

static YOUTUBE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(https?://)?(www\.)?(youtube\.com|youtu\.be)/.+$").unwrap());
```

`std::sync::LazyLock` は Rust 1.80 で安定化され、`once_cell::sync::Lazy` と同等の機能を外部依存なしで提供する。edition 2024 / rustc 1.80+ のプロジェクトでは標準ライブラリ版を使う。

**使いどころ**: 正規表現のコンパイルはコストが高い。ループや関数呼び出しのたびにコンパイルするのではなく、`LazyLock` で初回のみコンパイルし再利用する。

---

## 5. 統合テスト (assert_cmd)

**テンプレートでの実装**: `tests/cli_integration.rs`

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

**出典: rpg の統合テスト**:

```rust
// rpg: 正規表現でパスワード形式を厳密に検証
#[test]
fn test_custom_length() {
    Command::cargo_bin("rpg")
        .unwrap()
        .args(&["-l", "20"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(
            r"^[A-Za-z0-9!@#$%^&*()_+\-=\[\]{}|;:,.<>?]{20}\n$"
        ).unwrap());
}
```

rpg のテストは出力を正規表現で厳密に検証している。テンプレートは `contains` による部分一致にとどめ、実装の変更に対して脆くならないようにしている。

**テスト戦略の指針**:

| テスト対象 | 手法 | 例 |
|---|---|---|
| 正常系の出力 | `contains` で部分一致 | キーワードが含まれるか |
| 出力形式の厳密な検証 | `is_match` で正規表現 | rpg のパスワード形式 |
| エラー終了 | `.failure()` + stderr 確認 | 存在しないファイル |
| ヘルプ / バージョン | `contains` で固定文字列 | `"Usage:"`, バージョン番号 |

---

## 6. リリースプロファイル最適化

**テンプレートでの実装**: `Cargo.toml`

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**出典: rpg の Cargo.toml** (同一設定)

| オプション | 効果 | トレードオフ |
|---|---|---|
| `opt-level = 3` | 最大最適化 | コンパイル時間増加 |
| `lto = true` | リンク時最適化 (クレート間のインライン化) | コンパイル時間大幅増加 |
| `codegen-units = 1` | 単一コード生成ユニット (より良い最適化) | 並列コンパイル不可 |
| `strip = true` | デバッグシンボル除去 | バイナリサイズ削減、デバッグ不可 |

CLIツールの配布バイナリでは全て有効にするのが定石。開発中は `cargo build` (dev プロファイル) を使うため影響しない。

---

## 7. GitHub Actions CI

**テンプレートでの実装**: `.github/workflows/ci.yml`

**出典: gymeat の CI** (3OS マトリクス + --locked)

```yaml
strategy:
  matrix:
    os: [macos-latest, ubuntu-latest, windows-latest]
```

**rpg との違い**:

| 項目 | rpg | gymeat / テンプレート |
|---|---|---|
| OS | macOS, Ubuntu | macOS, Ubuntu, **Windows** |
| `--locked` | なし | あり (再現可能ビルド) |
| タイムアウト | なし | 30分 |
| キャッシュ | 3ブロック分割 | 1ブロック統合 |
| `shell: bash` | なし | あり (Windows 対応) |

`--locked` は `Cargo.lock` をリポジトリにコミットしている前提で、CI と開発環境で同一バージョンの依存を使うことを保証する。
