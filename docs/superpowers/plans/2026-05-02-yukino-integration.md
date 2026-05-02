# Yukino Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Yukino as a first-class CC Switch app with Codex-compatible provider/proxy support plus sessions, usage, Skills, MCP, plugins, memories, and settings integration.

**Architecture:** Add `AppType::Yukino` and a shared Codex-compatible home abstraction so Codex uses `~/.codex` and Yukino uses `~/.yukino`. Keep provider/OAuth state owned by CC Switch and Yukino application login/runtime state owned by Yukino; CC Switch writes only the explicit Yukino config, auth, skills, MCP, plugin enablement, memory, and session files described in the design.

**Tech Stack:** Rust/Tauri 2, React 18, TypeScript, Vite, Vitest, SQLite via rusqlite, TOML via toml_edit, PowerShell on Windows.

---

## File Structure

Backend app identity and settings:

- Modify `src-tauri/src/app_config.rs`: add `Yukino` to `AppType`, `McpApps`, `SkillApps`, `PromptRoot`, `CommonConfigSnippets`, `MultiAppConfig`, and tests.
- Modify `src-tauri/src/settings.rs`: add `visibleApps.yukino`, `yukino_config_dir`, and `current_provider_yukino`.
- Modify `src-tauri/src/commands/config.rs`: return Yukino config status, directory, and common TOML snippet validation.
- Modify `src-tauri/src/commands/misc.rs`: include Yukino config directory override lookup.

Backend database and providers:

- Modify `src-tauri/src/database/mod.rs`: bump `SCHEMA_VERSION` from `10` to `11`.
- Modify `src-tauri/src/database/schema.rs`: add `enabled_yukino` columns, proxy row support, and `migrate_v10_to_v11`.
- Modify `src-tauri/src/database/dao/skills.rs`: read/write `enabled_yukino`.
- Modify `src-tauri/src/database/dao/mcp.rs`: read/write `enabled_yukino`.
- Modify `src-tauri/src/database/tests.rs`: migration and DAO coverage.
- Modify `src-tauri/src/provider.rs`: add Yukino to universal provider app/model mapping where Codex-compatible configs are generated.
- Modify `src-tauri/src/services/provider/mod.rs`: make Codex validation/live-write paths accept Yukino.
- Modify `src-tauri/src/services/proxy.rs` and proxy routing files that switch on app type: include Yukino as Codex-compatible.

Codex-compatible helpers:

- Modify `src-tauri/src/codex_config.rs`: introduce `CodexLikeApp` or `CodexLikeHome`, keep existing Codex wrappers, add Yukino wrappers.
- Create `src-tauri/src/yukino_config.rs`: small facade for Yukino config dir, auth path, config path, memory path, and plugin helpers.
- Modify `src-tauri/src/lib.rs`: register new module and Tauri commands.

Sessions and usage:

- Modify `src-tauri/src/session_manager/providers/codex.rs`: parameterize root/provider/resume behavior.
- Create `src-tauri/src/session_manager/providers/yukino.rs`: Yukino provider wrapper over Codex-like parser.
- Modify `src-tauri/src/session_manager/mod.rs`: include Yukino scanning, loading, and deletion root validation.
- Modify `src-tauri/src/services/session_usage_codex.rs`: parameterize into Codex-like importer and expose `sync_yukino_usage`.
- Modify `src-tauri/src/commands/usage.rs` and `src-tauri/src/lib.rs`: trigger Yukino usage sync.

Skills, MCP, plugins, memories:

- Modify `src-tauri/src/services/skill.rs`: add Yukino skills dir.
- Modify `src-tauri/src/mcp/codex.rs`: parameterize Codex TOML import/sync for Yukino.
- Create `src-tauri/src/commands/yukino.rs`: memory and plugin marketplace commands.
- Create `src-tauri/src/yukino_plugins.rs`: marketplace discovery and `[plugins.*]` enablement in `~/.yukino/config.toml`.

Frontend types and UI:

- Modify `src/lib/api/types.ts`, `src/types.ts`, `src/lib/api/skills.ts`: add `yukino`.
- Modify `src/config/appConfig.tsx`, `src/components/AppSwitcher.tsx`, `src/components/BrandIcons.tsx`, `src/components/ProviderIcon.tsx`: show Yukino.
- Modify `src/App.tsx`: include Yukino in app lists and view routing.
- Modify `src/hooks/useSettingsForm.ts`, `src/hooks/useSettings.ts`, `src/hooks/useDirectorySettings.ts`, `src/components/settings/DirectorySettings.tsx`, `src/components/settings/AppVisibilitySettings.tsx`, `src/lib/schemas/settings.ts`: Yukino settings.
- Modify `src/components/sessions/SessionManagerPage.tsx`, `src/components/sessions/utils.ts`, `src/hooks/useSessionSearch.ts`: Yukino sessions.
- Modify Skills and MCP components that enumerate app IDs: include Yukino.
- Create `src/lib/api/yukino.ts`, `src/hooks/useYukino.ts`, `src/components/yukino/YukinoMemoryPanel.tsx`, `src/components/yukino/YukinoPluginsPanel.tsx`.
- Modify `src/i18n/locales/en.json`, `src/i18n/locales/zh.json`, `src/i18n/locales/ja.json`: labels and messages.

---

### Task 1: Add Yukino App Identity And Settings Types

**Files:**
- Modify: `src-tauri/src/app_config.rs`
- Modify: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/commands/misc.rs`
- Modify: `src/lib/api/types.ts`
- Modify: `src/types.ts`
- Modify: `src/lib/schemas/settings.ts`
- Test: `src-tauri/src/app_config.rs`
- Test: `src-tauri/src/settings.rs`
- Test: `tests/hooks/useDirectorySettings.test.tsx`

- [ ] **Step 1: Write failing Rust tests for AppType and settings**

Add tests in `src-tauri/src/app_config.rs` under the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn app_type_accepts_yukino() {
    let parsed: AppType = "yukino".parse().expect("parse yukino");
    assert_eq!(parsed, AppType::Yukino);
    assert_eq!(parsed.as_str(), "yukino");
    assert!(AppType::all().any(|app| app == AppType::Yukino));
}

#[test]
fn yukino_is_switch_mode() {
    assert!(!AppType::Yukino.is_additive_mode());
}

#[test]
fn skill_and_mcp_apps_support_yukino() {
    let mut skills = SkillApps::default();
    skills.set_enabled_for(&AppType::Yukino, true);
    assert!(skills.is_enabled_for(&AppType::Yukino));
    assert!(skills.enabled_apps().contains(&AppType::Yukino));

    let mut mcp = McpApps::default();
    mcp.set_enabled_for(&AppType::Yukino, true);
    assert!(mcp.is_enabled_for(&AppType::Yukino));
    assert!(mcp.enabled_apps().contains(&AppType::Yukino));
}
```

Add tests in `src-tauri/src/settings.rs`:

```rust
#[test]
fn visible_apps_defaults_show_yukino() {
    let visible = VisibleApps::default();
    assert!(visible.is_visible(&AppType::Yukino));
}

#[test]
fn yukino_override_dir_resolves_tilde() {
    let mut settings = AppSettings::default();
    settings.yukino_config_dir = Some("~/.yukino-alt".to_string());
    settings.normalize_paths();
    assert_eq!(settings.yukino_config_dir.as_deref(), Some("~/.yukino-alt"));
}
```

- [ ] **Step 2: Run failing backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml app_type_accepts_yukino yukino_is_switch_mode skill_and_mcp_apps_support_yukino visible_apps_defaults_show_yukino yukino_override_dir_resolves_tilde
```

Expected: fail because `AppType::Yukino`, `VisibleApps.yukino`, and `AppSettings.yukino_config_dir` do not exist.

- [ ] **Step 3: Implement backend app identity**

In `src-tauri/src/app_config.rs`, update structs and matches:

```rust
pub struct McpApps {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub opencode: bool,
    pub hermes: bool,
    #[serde(default)]
    pub yukino: bool,
}
```

```rust
pub struct SkillApps {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub opencode: bool,
    pub hermes: bool,
    #[serde(default)]
    pub yukino: bool,
}
```

Add `AppType::Yukino` to the enum, `as_str`, `all`, `FromStr`, `is_enabled_for`, `set_enabled_for`, `enabled_apps`, and `is_empty`. Keep OpenClaw excluded from Skills/MCP; Yukino is supported.

Extend these structures with `yukino`:

```rust
pub struct PromptRoot {
    pub claude: PromptConfig,
    pub codex: PromptConfig,
    pub gemini: PromptConfig,
    pub opencode: PromptConfig,
    pub openclaw: PromptConfig,
    pub hermes: PromptConfig,
    pub yukino: PromptConfig,
}

pub struct CommonConfigSnippets {
    pub claude: Option<String>,
    pub codex: Option<String>,
    pub gemini: Option<String>,
    pub opencode: Option<String>,
    pub openclaw: Option<String>,
    pub hermes: Option<String>,
    pub yukino: Option<String>,
}
```

- [ ] **Step 4: Implement backend settings fields**

In `src-tauri/src/settings.rs`, extend `VisibleApps` and `AppSettings`:

```rust
pub struct VisibleApps {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub opencode: bool,
    pub openclaw: bool,
    pub hermes: bool,
    #[serde(default = "default_true")]
    pub yukino: bool,
}
```

```rust
pub struct AppSettings {
    pub yukino_config_dir: Option<String>,
    pub current_provider_yukino: Option<String>,
}
```

Add normalization, `get_yukino_override_dir`, and current-provider match arms:

```rust
pub fn get_yukino_override_dir() -> Option<PathBuf> {
    let settings = settings_store().read().ok()?;
    settings
        .yukino_config_dir
        .as_ref()
        .map(|p| resolve_override_path(p))
}
```

- [ ] **Step 5: Implement config commands**

In `src-tauri/src/commands/config.rs`, treat Yukino like Codex for TOML validation:

```rust
match app_type {
    "codex" | "yukino" => {
        snippet
            .parse::<toml_edit::DocumentMut>()
            .map_err(invalid_toml_format_error)?;
    }
    _ => {}
}
```

Add `AppType::Yukino` match arms for status, config dir, and open config folder. They should call Yukino config helpers created in Task 3; until Task 3 lands, call `crate::config::get_home_dir().join(".yukino")` behind a temporary private helper in this file and replace it in Task 3.

- [ ] **Step 6: Implement frontend app types**

Add `"yukino"` to:

```ts
export type AppId =
  | "claude"
  | "codex"
  | "gemini"
  | "opencode"
  | "openclaw"
  | "hermes"
  | "yukino";
```

Update `VisibleApps`, `Settings`, settings schema, and `skillsApi.AppType`/`SkillApps`.

- [ ] **Step 7: Run type and identity tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml app_type_accepts_yukino yukino_is_switch_mode skill_and_mcp_apps_support_yukino visible_apps_defaults_show_yukino yukino_override_dir_resolves_tilde
pnpm typecheck
```

Expected: Rust tests pass; TypeScript errors remain only where UI app maps still lack Yukino.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/app_config.rs src-tauri/src/settings.rs src-tauri/src/commands/config.rs src-tauri/src/commands/misc.rs src/lib/api/types.ts src/types.ts src/lib/schemas/settings.ts
git commit -m "feat: add Yukino app identity"
```

---

### Task 2: Add Database Schema And DAO Support

**Files:**
- Modify: `src-tauri/src/database/mod.rs`
- Modify: `src-tauri/src/database/schema.rs`
- Modify: `src-tauri/src/database/dao/skills.rs`
- Modify: `src-tauri/src/database/dao/mcp.rs`
- Modify: `src-tauri/src/database/tests.rs`

- [ ] **Step 1: Write failing migration and DAO tests**

Add tests in `src-tauri/src/database/tests.rs`:

```rust
#[test]
fn migration_v11_adds_yukino_columns_and_proxy_config() {
    let conn = Connection::open_in_memory().expect("open memory db");
    Database::create_tables_on_conn(&conn).expect("create tables");
    Database::set_user_version(&conn, 10).expect("set version");

    Database::apply_schema_migrations_on_conn(&conn).expect("apply migration");

    assert!(Database::has_column(&conn, "skills", "enabled_yukino").unwrap());
    assert!(Database::has_column(&conn, "mcp_servers", "enabled_yukino").unwrap());
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM proxy_config WHERE app_type = 'yukino'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
}

#[test]
fn skill_and_mcp_daos_round_trip_yukino_enabled() {
    let db = Database::memory().expect("create memory db");

    let mut skill = InstalledSkill {
        id: "local:test".to_string(),
        name: "test".to_string(),
        description: None,
        directory: "test".to_string(),
        repo_owner: None,
        repo_name: None,
        repo_branch: None,
        readme_url: None,
        apps: SkillApps::default(),
        installed_at: 1,
        content_hash: None,
        updated_at: 0,
    };
    skill.apps.yukino = true;
    db.save_skill(&skill).expect("save skill");
    let loaded = db.get_installed_skill("local:test").unwrap().unwrap();
    assert!(loaded.apps.yukino);

    let server = McpServer {
        id: "echo".to_string(),
        name: "echo".to_string(),
        server: serde_json::json!({"type":"stdio","command":"echo"}),
        apps: McpApps {
            yukino: true,
            ..Default::default()
        },
        description: None,
        homepage: None,
        docs: None,
        tags: vec![],
    };
    db.save_mcp_server(&server).expect("save server");
    let loaded = db.get_all_mcp_servers().unwrap().remove("echo").unwrap();
    assert!(loaded.apps.yukino);
}
```

- [ ] **Step 2: Run failing database tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migration_v11_adds_yukino_columns_and_proxy_config skill_and_mcp_daos_round_trip_yukino_enabled
```

Expected: fail because schema version and columns are not present.

- [ ] **Step 3: Implement schema version 11**

In `src-tauri/src/database/mod.rs`:

```rust
pub(crate) const SCHEMA_VERSION: i32 = 11;
```

In `create_tables_on_conn`, add `enabled_yukino BOOLEAN NOT NULL DEFAULT 0` to `mcp_servers` and `skills`. Add a Yukino seed row to `proxy_config`:

```rust
conn.execute(
    "INSERT OR IGNORE INTO proxy_config (app_type, max_retries,
    streaming_first_byte_timeout, streaming_idle_timeout, non_streaming_timeout,
    circuit_failure_threshold, circuit_success_threshold, circuit_timeout_seconds,
    circuit_error_rate_threshold, circuit_min_requests)
    VALUES ('yukino', 3, 60, 120, 600, 4, 2, 60, 0.6, 10)",
    [],
)?;
```

If the current `CHECK (app_type IN (...))` blocks Yukino, update the proxy migration that rebuilds `proxy_config` so the check includes `'yukino'`.

Add to `apply_schema_migrations_on_conn`:

```rust
10 => {
    log::info!("迁移数据库从 v10 到 v11（添加 Yukino 支持）");
    Self::migrate_v10_to_v11(conn)?;
    Self::set_user_version(conn, 11)?;
}
```

Add:

```rust
fn migrate_v10_to_v11(conn: &Connection) -> Result<(), AppError> {
    Self::add_column_if_missing(
        conn,
        "mcp_servers",
        "enabled_yukino",
        "BOOLEAN NOT NULL DEFAULT 0",
    )?;
    Self::add_column_if_missing(
        conn,
        "skills",
        "enabled_yukino",
        "BOOLEAN NOT NULL DEFAULT 0",
    )?;
    Self::ensure_proxy_config_allows_yukino(conn)?;
    conn.execute(
        "INSERT OR IGNORE INTO proxy_config (app_type, max_retries,
        streaming_first_byte_timeout, streaming_idle_timeout, non_streaming_timeout,
        circuit_failure_threshold, circuit_success_threshold, circuit_timeout_seconds,
        circuit_error_rate_threshold, circuit_min_requests)
        VALUES ('yukino', 3, 60, 120, 600, 4, 2, 60, 0.6, 10)",
        [],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}
```

Implement `ensure_proxy_config_allows_yukino` by following the existing proxy table rebuild style in the earlier proxy migration and copying all existing columns.

- [ ] **Step 4: Update DAO SQL**

In `src-tauri/src/database/dao/skills.rs`, include `enabled_yukino` in all SELECT, INSERT, and UPDATE statements. Row mapping should set:

```rust
apps: SkillApps {
    claude: row.get(8)?,
    codex: row.get(9)?,
    gemini: row.get(10)?,
    opencode: row.get(11)?,
    hermes: row.get(12)?,
    yukino: row.get(13)?,
},
installed_at: row.get(14)?,
content_hash: row.get(15)?,
updated_at: row.get::<_, i64>(16).unwrap_or(0),
```

In `src-tauri/src/database/dao/mcp.rs`, include `enabled_yukino` in SELECT and INSERT. Row mapping should set `apps.yukino`.

- [ ] **Step 5: Run database tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml database
```

Expected: all database tests pass.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/database/mod.rs src-tauri/src/database/schema.rs src-tauri/src/database/dao/skills.rs src-tauri/src/database/dao/mcp.rs src-tauri/src/database/tests.rs
git commit -m "feat: add Yukino database support"
```

---

### Task 3: Create Codex-Compatible Home Abstraction

**Files:**
- Modify: `src-tauri/src/codex_config.rs`
- Create: `src-tauri/src/yukino_config.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/prompt_files.rs`
- Test: `src-tauri/src/codex_config.rs`
- Test: `src-tauri/src/yukino_config.rs`

- [ ] **Step 1: Write failing tests for separate homes**

Add in `src-tauri/src/codex_config.rs` tests:

```rust
#[test]
fn codex_like_app_defaults_to_separate_home_dirs() {
    let home = crate::config::get_home_dir();
    assert_eq!(
        get_codex_like_config_dir(CodexLikeApp::Codex),
        home.join(".codex")
    );
    assert_eq!(
        get_codex_like_config_dir(CodexLikeApp::Yukino),
        home.join(".yukino")
    );
}

#[test]
fn codex_like_paths_use_requested_home() {
    let yukino = CodexLikeApp::Yukino;
    assert!(get_codex_like_auth_path(yukino).ends_with(".yukino/auth.json"));
    assert!(get_codex_like_config_path(yukino).ends_with(".yukino/config.toml"));
}
```

- [ ] **Step 2: Run failing tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml codex_like_app_defaults_to_separate_home_dirs codex_like_paths_use_requested_home
```

Expected: fail because `CodexLikeApp` and helpers do not exist.

- [ ] **Step 3: Add the abstraction**

In `src-tauri/src/codex_config.rs` add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexLikeApp {
    Codex,
    Yukino,
}

impl CodexLikeApp {
    pub fn app_type(self) -> crate::app_config::AppType {
        match self {
            Self::Codex => crate::app_config::AppType::Codex,
            Self::Yukino => crate::app_config::AppType::Yukino,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Yukino => "Yukino",
        }
    }
}

pub fn get_codex_like_config_dir(app: CodexLikeApp) -> PathBuf {
    match app {
        CodexLikeApp::Codex => {
            if let Some(custom) = crate::settings::get_codex_override_dir() {
                return custom;
            }
            get_home_dir().join(".codex")
        }
        CodexLikeApp::Yukino => {
            if let Some(custom) = crate::settings::get_yukino_override_dir() {
                return custom;
            }
            get_home_dir().join(".yukino")
        }
    }
}

pub fn get_codex_like_auth_path(app: CodexLikeApp) -> PathBuf {
    get_codex_like_config_dir(app).join("auth.json")
}

pub fn get_codex_like_config_path(app: CodexLikeApp) -> PathBuf {
    get_codex_like_config_dir(app).join("config.toml")
}
```

Keep current wrappers:

```rust
pub fn get_codex_config_dir() -> PathBuf {
    get_codex_like_config_dir(CodexLikeApp::Codex)
}

pub fn get_codex_auth_path() -> PathBuf {
    get_codex_like_auth_path(CodexLikeApp::Codex)
}

pub fn get_codex_config_path() -> PathBuf {
    get_codex_like_config_path(CodexLikeApp::Codex)
}
```

Add app-aware versions of read/write functions, then make existing Codex functions call the Codex variant:

```rust
pub fn read_codex_like_config_text(app: CodexLikeApp) -> Result<String, AppError> {
    let path = get_codex_like_config_path(app);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(AppError::io(&path, e)),
    }
}
```

```rust
pub fn write_codex_like_live_atomic(
    app: CodexLikeApp,
    auth: &serde_json::Value,
    config_text: Option<&str>,
) -> Result<(), AppError> {
    let auth_path = get_codex_like_auth_path(app);
    let config_path = get_codex_like_config_path(app);
    write_codex_live_atomic_to_paths(&auth_path, &config_path, auth, config_text)
}
```

Extract the current body of `write_codex_live_atomic` into private `write_codex_live_atomic_to_paths`.

- [ ] **Step 4: Create Yukino facade**

Create `src-tauri/src/yukino_config.rs`:

```rust
use std::path::PathBuf;

use crate::codex_config::{self, CodexLikeApp};

pub fn get_yukino_dir() -> PathBuf {
    codex_config::get_codex_like_config_dir(CodexLikeApp::Yukino)
}

pub fn get_yukino_auth_path() -> PathBuf {
    codex_config::get_codex_like_auth_path(CodexLikeApp::Yukino)
}

pub fn get_yukino_config_path() -> PathBuf {
    codex_config::get_codex_like_config_path(CodexLikeApp::Yukino)
}

pub fn get_yukino_memory_path() -> PathBuf {
    get_yukino_dir().join("memories").join("raw_memories.md")
}
```

Register `mod yukino_config;` in `src-tauri/src/lib.rs`.

- [ ] **Step 5: Replace temporary home helpers**

Replace Task 1 temporary `~/.yukino` helpers in config commands and prompt files with `crate::yukino_config`.

- [ ] **Step 6: Run abstraction tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml codex_config yukino_config
```

Expected: tests pass and Codex wrappers preserve existing behavior.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/src/codex_config.rs src-tauri/src/yukino_config.rs src-tauri/src/lib.rs src-tauri/src/prompt_files.rs src-tauri/src/commands/config.rs
git commit -m "refactor: add Codex-compatible app homes"
```

---

### Task 4: Wire Yukino Provider Switching And Proxy Support

**Files:**
- Modify: `src-tauri/src/services/provider/mod.rs`
- Modify: `src-tauri/src/services/proxy.rs`
- Modify: `src-tauri/src/proxy/**`
- Modify: `src-tauri/src/provider.rs`
- Modify: `src/config/codexProviderPresets.ts`
- Modify: `src/config/universalProviderPresets.ts`
- Test: `src-tauri/src/services/provider/mod.rs`
- Test: `tests/utils/providerConfigUtils.codex.test.ts`

- [ ] **Step 1: Write failing provider live-write test**

Add a provider service test that uses temp HOME/settings override and switches a Yukino provider:

```rust
#[test]
fn switch_yukino_provider_writes_yukino_home_not_codex_home() {
    let state = test_state_with_temp_home();
    let home = test_home();
    let yukino_dir = home.join(".yukino");
    let codex_dir = home.join(".codex");

    let provider = Provider::with_id(
        "yukino-custom".to_string(),
        "Yukino Custom".to_string(),
        serde_json::json!({
            "auth": {"OPENAI_API_KEY": "yukino-key"},
            "config": "model_provider = \"custom\"\nmodel = \"gpt-5.5\"\n[model_providers.custom]\nname = \"custom\"\nbase_url = \"https://api.yukino.vip\"\nwire_api = \"responses\"\nrequires_openai_auth = true\n"
        }),
        None,
    );

    ProviderService::add(&state, AppType::Yukino, provider, false).expect("add");
    ProviderService::switch(&state, AppType::Yukino, "yukino-custom").expect("switch");

    assert!(yukino_dir.join("auth.json").exists());
    assert!(yukino_dir.join("config.toml").exists());
    assert!(!codex_dir.join("auth.json").exists());
}
```

Use the local provider test helpers already present in `services/provider/mod.rs`; if helper names differ, create local helpers in the test module that set the same temp HOME pattern used by existing provider tests.

- [ ] **Step 2: Run failing provider test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml switch_yukino_provider_writes_yukino_home_not_codex_home
```

Expected: fail because `ProviderService` has no Yukino branch.

- [ ] **Step 3: Implement provider switch path**

In `src-tauri/src/services/provider/mod.rs`, locate Codex validation and live-write match arms. Add Yukino by sharing Codex-compatible behavior:

```rust
match app {
    AppType::Codex | AppType::Yukino => {
        let codex_like = match app {
            AppType::Codex => CodexLikeApp::Codex,
            AppType::Yukino => CodexLikeApp::Yukino,
            _ => unreachable!(),
        };
        crate::codex_config::write_codex_like_live_atomic_with_stable_provider(
            codex_like,
            &provider.settings_config,
        )?;
    }
    _ => {}
}
```

If the current function accepts separate `auth` and `config`, pass `CodexLikeApp` into the extracted writer from Task 3. Preserve the stable model-provider normalization for both apps.

- [ ] **Step 4: Preserve Yukino config sections**

Extend the Codex config merge logic to preserve these top-level sections from the existing target document when the new provider config does not intentionally replace them:

```rust
const CODEX_LIKE_PRESERVED_TABLES: &[&str] = &[
    "features",
    "memories",
    "projects",
    "marketplaces",
    "plugins",
    "skills",
    "windows",
    "mcp_servers",
];
```

Use `toml_edit::DocumentMut` to copy missing tables from the existing live config into the outgoing provider config. Add a test that seeds `.yukino/config.toml` with `[features] apps = true`, switches provider, and asserts `[features]` remains.

- [ ] **Step 5: Add proxy support**

Find proxy app-type checks that currently allow `claude`, `codex`, and `gemini`. Add `yukino` and route it through Codex-compatible providers. Request logs should store `app_type = "yukino"` when the active proxy is Yukino.

For command and DTO match arms, prefer:

```rust
AppType::Codex | AppType::Yukino => {
    // Codex-compatible proxy behavior
}
```

- [ ] **Step 6: Extend universal provider conversion**

In `src-tauri/src/provider.rs`, add `yukino` to `UniversalProviderApps` and `UniversalProviderModels`. Add a `to_yukino_provider` method that calls the same implementation as `to_codex_provider` but uses ids prefixed with `universal-yukino-`.

In frontend universal provider forms, add Yukino to app toggles and reuse Codex model fields for Yukino.

- [ ] **Step 7: Run provider and proxy tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml provider proxy_commands
pnpm test:unit -- tests/utils/providerConfigUtils.codex.test.ts
pnpm typecheck
```

Expected: tests pass. If proxy tests show hard-coded app count assumptions, update assertions to include Yukino.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/services/provider/mod.rs src-tauri/src/services/proxy.rs src-tauri/src/proxy src-tauri/src/provider.rs src/config/codexProviderPresets.ts src/config/universalProviderPresets.ts src-tauri/src/codex_config.rs
git commit -m "feat: support Yukino provider switching"
```

---

### Task 5: Add Yukino Sessions And Usage Import

**Files:**
- Modify: `src-tauri/src/session_manager/providers/codex.rs`
- Create: `src-tauri/src/session_manager/providers/yukino.rs`
- Modify: `src-tauri/src/session_manager/providers/mod.rs`
- Modify: `src-tauri/src/session_manager/mod.rs`
- Modify: `src-tauri/src/services/session_usage_codex.rs`
- Modify: `src-tauri/src/commands/usage.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/components/sessions/SessionManagerPage.tsx`
- Modify: `src/components/sessions/utils.ts`
- Modify: `src/hooks/useSessionSearch.ts`
- Test: `src-tauri/src/session_manager/providers/yukino.rs`
- Test: `src-tauri/src/services/session_usage_codex.rs`

- [ ] **Step 1: Write failing Yukino session tests**

Create tests in `src-tauri/src/session_manager/providers/yukino.rs`:

```rust
#[test]
fn scan_yukino_session_jsonl_reads_title_and_provider_id() {
    let temp = tempfile::tempdir().expect("tempdir");
    let sessions = temp.path().join("sessions").join("2026").join("05").join("02");
    std::fs::create_dir_all(&sessions).expect("mkdir");
    let path = sessions.join("rollout-2026-05-02T10-19-39-019de67b-fed1-7ed3-927c-49687535605b.jsonl");
    std::fs::write(
        &path,
        concat!(
            "{\"timestamp\":\"2026-05-02T10:19:39Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"019de67b-fed1-7ed3-927c-49687535605b\",\"cwd\":\"C:/work\"}}\n",
            "{\"timestamp\":\"2026-05-02T10:19:40Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":\"Hello Yukino\"}}\n"
        ),
    )
    .expect("write");

    let sessions = scan_sessions_from_root(temp.path());
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].provider_id, "yukino");
    assert_eq!(sessions[0].title.as_deref(), Some("Hello Yukino"));
    assert!(sessions[0].resume_command.is_none());
}
```

- [ ] **Step 2: Run failing session test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml scan_yukino_session_jsonl_reads_title_and_provider_id
```

Expected: fail because Yukino session provider does not exist.

- [ ] **Step 3: Parameterize Codex session parser**

In `src-tauri/src/session_manager/providers/codex.rs`, expose a reusable function:

```rust
pub fn scan_sessions_from_root_with_provider(
    root: &Path,
    provider_id: &str,
    resume_command: impl Fn(&str) -> Option<String>,
) -> Vec<SessionMeta> {
    let sessions_root = root.join("sessions");
    let mut files = Vec::new();
    collect_jsonl_files(&sessions_root, &mut files);
    files
        .into_iter()
        .filter_map(|path| parse_session_with_provider(&path, provider_id, &resume_command))
        .collect()
}
```

Make existing `scan_sessions` call it with provider `codex` and resume command `Some(format!("codex resume {session_id}"))`.

- [ ] **Step 4: Add Yukino session provider**

Create `src-tauri/src/session_manager/providers/yukino.rs`:

```rust
use std::path::Path;

use crate::session_manager::{SessionMessage, SessionMeta};

pub fn scan_sessions() -> Vec<SessionMeta> {
    scan_sessions_from_root(&crate::yukino_config::get_yukino_dir())
}

pub fn scan_sessions_from_root(root: &Path) -> Vec<SessionMeta> {
    super::codex::scan_sessions_from_root_with_provider(root, "yukino", |_| None)
}

pub fn load_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    super::codex::load_messages(path)
}

pub fn delete_session(root: &Path, path: &Path, session_id: &str) -> Result<bool, String> {
    super::codex::delete_session(root, path, session_id)
}
```

Register it in `providers/mod.rs` and `session_manager/mod.rs`.

- [ ] **Step 5: Add usage importer**

In `src-tauri/src/services/session_usage_codex.rs`, introduce:

```rust
pub fn sync_codex_like_usage(
    db: &Database,
    app: CodexLikeApp,
    app_type: &'static str,
    log_prefix: &'static str,
) -> Result<SessionSyncResult, AppError> {
    let dir = get_codex_like_config_dir(app);
    let files = collect_codex_session_files(&dir);
    sync_codex_like_files(db, &files, app_type, log_prefix)
}

pub fn sync_yukino_usage(db: &Database) -> Result<SessionSyncResult, AppError> {
    sync_codex_like_usage(db, CodexLikeApp::Yukino, "yukino", "YUKINO-SYNC")
}
```

Pass `app_type` into request-log insertion so Yukino imports write `app_type = 'yukino'`.

- [ ] **Step 6: Wire commands and background sync**

In `commands/usage.rs`, include Yukino in `sync_session_usage`. In `lib.rs`, include Yukino in startup/interval background usage sync next to Codex.

- [ ] **Step 7: Update frontend session filters**

Add `"yukino"` to `ProviderFilter`, session filter select, labels, icon utils, and search types.

- [ ] **Step 8: Run session and usage tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml session_manager session_usage_codex
pnpm typecheck
```

Expected: tests pass and TypeScript accepts Yukino session filters.

- [ ] **Step 9: Commit**

```powershell
git add src-tauri/src/session_manager src-tauri/src/services/session_usage_codex.rs src-tauri/src/commands/usage.rs src-tauri/src/lib.rs src/components/sessions src/hooks/useSessionSearch.ts
git commit -m "feat: add Yukino sessions and usage import"
```

---

### Task 6: Add Yukino Skills And MCP Support

**Files:**
- Modify: `src-tauri/src/services/skill.rs`
- Modify: `src-tauri/src/mcp/codex.rs`
- Modify: `src-tauri/src/services/mcp.rs`
- Modify: `src-tauri/src/commands/mcp.rs`
- Modify: `src/config/appConfig.tsx`
- Modify: `src/components/skills/**`
- Modify: `src/components/mcp/**`
- Test: `src-tauri/tests/skill_sync.rs`
- Test: `tests/hooks/useMcpValidation.test.tsx`
- Test: `tests/hooks/useImportSkillsFromApps.test.tsx`

- [ ] **Step 1: Write failing SkillService path test**

In `src-tauri/tests/skill_sync.rs`, add:

```rust
#[test]
fn yukino_skills_dir_defaults_to_yukino_home() {
    let state = test_state();
    let home = state.home.path();
    let dir = SkillService::get_app_skills_dir(&AppType::Yukino).expect("skills dir");
    assert_eq!(dir, home.join(".yukino").join("skills"));
}
```

Use the support helper already imported by the file; if the temp home helper exposes a different field, use that field's path.

- [ ] **Step 2: Run failing skills test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test skill_sync yukino_skills_dir_defaults_to_yukino_home
```

Expected: fail because SkillService has no Yukino branch.

- [ ] **Step 3: Implement skills path and sync**

In `SkillService::get_app_skills_dir` add:

```rust
AppType::Yukino => {
    if let Some(custom) = crate::settings::get_yukino_override_dir() {
        return Ok(custom.join("skills"));
    }
}
```

And default:

```rust
AppType::Yukino => home.join(".yukino").join("skills"),
```

Because Task 2 added DAO support and Task 1 added `SkillApps`, existing sync should work once all app lists include Yukino.

- [ ] **Step 4: Write failing MCP import/sync test**

In `src-tauri/src/mcp/codex.rs` tests, add:

```rust
#[test]
fn import_from_yukino_marks_yukino_app() {
    let mut config = MultiAppConfig::default();
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("config.toml");
    std::fs::write(
        &config_path,
        "[mcp_servers.echo]\ntype = \"stdio\"\ncommand = \"echo\"\nargs = [\"hi\"]\n",
    )
    .expect("write");

    let changed = import_from_codex_like_text(
        &mut config,
        AppType::Yukino,
        &std::fs::read_to_string(config_path).unwrap(),
    )
    .expect("import");

    assert_eq!(changed, 1);
    let server = config.mcp.servers.unwrap().remove("echo").unwrap();
    assert!(server.apps.yukino);
    assert!(!server.apps.codex);
}
```

- [ ] **Step 5: Parameterize MCP Codex sync**

Refactor `import_from_codex` into text/path-aware helpers:

```rust
pub fn import_from_codex_like_text(
    config: &mut MultiAppConfig,
    app: AppType,
    text: &str,
) -> Result<usize, AppError> {
    let root: toml::Table = toml::from_str(text)
        .map_err(|e| AppError::McpValidation(format!("解析 config.toml 失败: {e}")))?;
    import_mcp_tables(config, app, &root)
}
```

Make `import_from_codex` call the helper with `AppType::Codex` and add `import_from_yukino` with `AppType::Yukino`.

For sync, introduce:

```rust
pub fn sync_enabled_to_codex_like(
    config: &MultiAppConfig,
    app: AppType,
    config_path: &Path,
) -> Result<(), AppError>
```

Collect enabled servers with `server.apps.is_enabled_for(&app)` and write `[mcp_servers]` to the provided TOML path.

- [ ] **Step 6: Update frontend Skills/MCP app lists**

Add `yukino` to `APP_IDS`, `SKILLS_APP_IDS`, and `MCP_APP_IDS` in `src/config/appConfig.tsx`. Update app badge maps and any hard-coded app arrays in Skills/MCP components.

- [ ] **Step 7: Run skills and MCP tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test skill_sync
cargo test --manifest-path src-tauri/Cargo.toml mcp
pnpm test:unit -- tests/hooks/useMcpValidation.test.tsx tests/hooks/useImportSkillsFromApps.test.tsx
pnpm typecheck
```

Expected: tests pass.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/services/skill.rs src-tauri/src/mcp/codex.rs src-tauri/src/services/mcp.rs src-tauri/src/commands/mcp.rs src-tauri/tests/skill_sync.rs src/config/appConfig.tsx src/components/skills src/components/mcp tests/hooks
git commit -m "feat: add Yukino skills and MCP support"
```

---

### Task 7: Add Yukino Plugin Marketplace And Memory Commands

**Files:**
- Create: `src-tauri/src/yukino_plugins.rs`
- Create: `src-tauri/src/commands/yukino.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/api/yukino.ts`
- Create: `src/hooks/useYukino.ts`
- Create: `src/components/yukino/YukinoMemoryPanel.tsx`
- Create: `src/components/yukino/YukinoPluginsPanel.tsx`
- Modify: `src/App.tsx`
- Test: `src-tauri/src/yukino_plugins.rs`

- [ ] **Step 1: Write failing plugin parser tests**

Create tests in `src-tauri/src/yukino_plugins.rs`:

```rust
#[test]
fn discover_plugins_reads_marketplace_manifest_and_config_enablement() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let marketplace_dir = root.join("marketplaces").join("yukino-curated");
    let manifest_dir = marketplace_dir.join(".agents").join("plugins");
    std::fs::create_dir_all(&manifest_dir).expect("mkdir");
    std::fs::write(
        manifest_dir.join("marketplace.json"),
        r#"{"plugins":[{"id":"browser-use","name":"Browser Use","description":"Browser automation"}]}"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("config.toml"),
        "[marketplaces.yukino-curated]\nsource_type = \"local\"\nsource = \"marketplaces/yukino-curated\"\n\n[plugins.\"browser-use@yukino-curated\"]\nenabled = true\n",
    )
    .expect("write config");

    let plugins = discover_plugins_from_root(root).expect("discover");
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].id, "browser-use@yukino-curated");
    assert!(plugins[0].enabled);
}

#[test]
fn set_plugin_enabled_preserves_credentials_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::write(root.join("config.toml"), "").expect("write config");
    std::fs::write(root.join(".credentials.json"), "{\"secret\":true}").expect("write creds");

    set_plugin_enabled_in_root(root, "browser-use@yukino-curated", true).expect("enable");

    assert_eq!(
        std::fs::read_to_string(root.join(".credentials.json")).unwrap(),
        "{\"secret\":true}"
    );
    let config = std::fs::read_to_string(root.join("config.toml")).unwrap();
    assert!(config.contains("[plugins.\"browser-use@yukino-curated\"]"));
    assert!(config.contains("enabled = true"));
}
```

- [ ] **Step 2: Run failing plugin tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml yukino_plugins
```

Expected: fail because module does not exist.

- [ ] **Step 3: Implement plugin discovery and enablement**

Create `src-tauri/src/yukino_plugins.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YukinoPlugin {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub marketplace: String,
    pub enabled: bool,
}
```

Implement:

- `discover_plugins() -> Result<Vec<YukinoPlugin>, AppError>`
- `discover_plugins_from_root(root: &Path) -> Result<Vec<YukinoPlugin>, AppError>`
- `set_plugin_enabled(id: &str, enabled: bool) -> Result<(), AppError>`
- `set_plugin_enabled_in_root(root: &Path, id: &str, enabled: bool) -> Result<(), AppError>`

Read manifests from `marketplaces/*/.agents/plugins/marketplace.json` and marketplace entries in `config.toml`. Parse plugin id as `<plugin>@<marketplace>`. Write enablement using `toml_edit`:

```rust
let plugins = doc["plugins"].or_insert(toml_edit::table());
plugins[plugin_id]["enabled"] = toml_edit::value(enabled);
```

Never open `.credentials.json`.

- [ ] **Step 4: Implement memory commands**

Create `src-tauri/src/commands/yukino.rs`:

```rust
#[tauri::command]
pub async fn read_yukino_memory() -> Result<String, String> {
    let path = crate::yukino_config::get_yukino_memory_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("Failed to read Yukino memory: {e}")),
    }
}

#[tauri::command]
pub async fn write_yukino_memory(content: String) -> Result<bool, String> {
    let path = crate::yukino_config::get_yukino_memory_path();
    crate::config::write_text_file(&path, &content).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn list_yukino_plugins() -> Result<Vec<crate::yukino_plugins::YukinoPlugin>, String> {
    crate::yukino_plugins::discover_plugins().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_yukino_plugin_enabled(id: String, enabled: bool) -> Result<bool, String> {
    crate::yukino_plugins::set_plugin_enabled(&id, enabled).map_err(|e| e.to_string())?;
    Ok(true)
}
```

Register module and commands in `commands/mod.rs` and `lib.rs`.

- [ ] **Step 5: Add frontend API and components**

Create `src/lib/api/yukino.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

export interface YukinoPlugin {
  id: string;
  name: string;
  description?: string;
  marketplace: string;
  enabled: boolean;
}

export const yukinoApi = {
  readMemory: () => invoke<string>("read_yukino_memory"),
  writeMemory: (content: string) =>
    invoke<boolean>("write_yukino_memory", { content }),
  listPlugins: () => invoke<YukinoPlugin[]>("list_yukino_plugins"),
  setPluginEnabled: (id: string, enabled: boolean) =>
    invoke<boolean>("set_yukino_plugin_enabled", { id, enabled }),
};
```

Create `useYukino.ts` with React Query hooks for memory and plugins. Create memory and plugin panels using existing `Textarea`, `Switch`, `Button`, and `Card` patterns used by Hermes/settings panels.

- [ ] **Step 6: Add App views**

In `src/App.tsx`, add view ids `yukinoMemory` and `yukinoPlugins`. Show action buttons in the Yukino provider toolbar. Render the new panels when selected.

- [ ] **Step 7: Run tests and typecheck**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml yukino_plugins
pnpm typecheck
```

Expected: pass.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/yukino_plugins.rs src-tauri/src/commands/yukino.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src/lib/api/yukino.ts src/hooks/useYukino.ts src/components/yukino src/App.tsx
git commit -m "feat: add Yukino plugins and memories"
```

---

### Task 8: Complete Frontend App Surface And Translations

**Files:**
- Modify: `src/components/AppSwitcher.tsx`
- Modify: `src/config/appConfig.tsx`
- Modify: `src/components/BrandIcons.tsx`
- Modify: `src/components/ProviderIcon.tsx`
- Modify: `src/components/settings/DirectorySettings.tsx`
- Modify: `src/components/settings/AppVisibilitySettings.tsx`
- Modify: `src/hooks/useDirectorySettings.ts`
- Modify: `src/hooks/useSettingsForm.ts`
- Modify: `src/hooks/useSettings.ts`
- Modify: `src/i18n/locales/en.json`
- Modify: `src/i18n/locales/zh.json`
- Modify: `src/i18n/locales/ja.json`
- Test: `tests/integration/App.test.tsx`
- Test: `tests/hooks/useSettingsForm.test.tsx`
- Test: `tests/hooks/useDirectorySettings.test.tsx`

- [ ] **Step 1: Write failing frontend tests**

Add to `tests/integration/App.test.tsx`:

```tsx
it("renders Yukino app switcher entry when visible", async () => {
  render(<App />);
  expect(await screen.findByRole("button", { name: /Yukino/i })).toBeInTheDocument();
});
```

Add to `tests/hooks/useSettingsForm.test.tsx`:

```ts
it("normalizes yukino config directory", () => {
  mockSettingsQuery({
    yukinoConfigDir: " C:\\Users\\me\\.yukino ",
  });
  const { result } = renderHook(() => useSettingsForm(), { wrapper });
  expect(result.current.settings?.yukinoConfigDir).toBe("C:\\Users\\me\\.yukino");
});
```

- [ ] **Step 2: Run failing frontend tests**

Run:

```powershell
pnpm test:unit -- tests/integration/App.test.tsx tests/hooks/useSettingsForm.test.tsx tests/hooks/useDirectorySettings.test.tsx
```

Expected: fail because Yukino UI and settings fields are incomplete.

- [ ] **Step 3: Add app switcher and icon maps**

Add Yukino to `ALL_APPS`, `APP_IDS`, `APP_ICON_MAP`, and app display names. Use `ProviderIcon icon="yukino"` and add explicit fallback handling in `ProviderIcon` that renders a "Y" glyph for the `yukino` icon name.

- [ ] **Step 4: Add settings directory and visibility**

In `useDirectorySettings.ts`, add:

```ts
type AppDirectoryKey =
  | "claude"
  | "codex"
  | "gemini"
  | "opencode"
  | "openclaw"
  | "hermes"
  | "yukino";
```

Add default folder `.yukino`, resolved directory state, settings field mapping, and load calls to `settingsApi.getConfigDir("yukino")`.

In `DirectorySettings.tsx`, add a `DirectoryInput`:

```tsx
<DirectoryInput
  label={t("settings.yukinoConfigDir")}
  value={settings.yukinoConfigDir ?? ""}
  resolvedValue={resolvedDirs.yukino}
  placeholder={t("settings.browsePlaceholderYukino")}
  onChange={(val) => onDirectoryChange("yukino", val)}
  onBrowse={() => onBrowseDirectory("yukino")}
  onReset={() => onResetDirectory("yukino")}
/>
```

- [ ] **Step 5: Add translations**

Add these keys in all three locale files:

```json
{
  "settings.yukinoConfigDir": "Yukino config directory",
  "settings.browsePlaceholderYukino": "e.g., ~/.yukino",
  "yukino.memory.title": "Yukino Memory",
  "yukino.plugins.title": "Yukino Plugins",
  "sessionManager.providerFilterYukino": "Yukino"
}
```

Use Chinese and Japanese translations matching the surrounding locale style.

- [ ] **Step 6: Run frontend tests**

Run:

```powershell
pnpm test:unit -- tests/integration/App.test.tsx tests/hooks/useSettingsForm.test.tsx tests/hooks/useDirectorySettings.test.tsx
pnpm typecheck
```

Expected: tests and typecheck pass.

- [ ] **Step 7: Commit**

```powershell
git add src/components/AppSwitcher.tsx src/config/appConfig.tsx src/components/BrandIcons.tsx src/components/ProviderIcon.tsx src/components/settings src/hooks src/i18n/locales tests
git commit -m "feat: expose Yukino in the UI"
```

---

### Task 9: Final Verification And Manual Safety Checks

**Files:**
- Modify only files needed to fix verification failures found in this task.

- [ ] **Step 1: Run full backend test suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all tests pass.

- [ ] **Step 2: Run frontend tests and typecheck**

Run:

```powershell
pnpm test:unit
pnpm typecheck
```

Expected: all tests pass and TypeScript reports no errors.

- [ ] **Step 3: Build renderer**

Run:

```powershell
pnpm build:renderer
```

Expected: Vite build completes successfully.

- [ ] **Step 4: Manual config safety check with temp dirs**

Create a temp home scenario by running a focused Rust test or a small test hook that switches a Yukino provider. Verify:

```text
%TEMP%\cc-switch-yukino-test-home\.yukino\auth.json exists
%TEMP%\cc-switch-yukino-test-home\.yukino\config.toml exists
%TEMP%\cc-switch-yukino-test-home\.codex\auth.json does not exist after Yukino-only switch
%TEMP%\cc-switch-yukino-test-home\.codex\config.toml does not exist after Yukino-only switch
```

If this is checked through an automated test from Task 4, rerun that test and record the passing command in the final implementation notes.

- [ ] **Step 5: Manual local app smoke test**

Run:

```powershell
pnpm dev:renderer
```

Open the renderer URL shown by Vite and verify:

- Yukino appears in the app switcher.
- Yukino settings directory defaults to `~/.yukino`.
- Yukino sessions view opens without crashing.
- Yukino memory panel opens and does not show `.git` internals.
- Yukino plugin panel lists plugins when `~/.yukino/marketplaces` manifests exist.

Stop the dev server after the smoke test.

- [ ] **Step 6: Record final git state**

Run:

```powershell
git status --short
```

Expected: no uncommitted implementation changes remain. If this command shows modified implementation files, return to the task that introduced those files, fix the verification failure there, rerun that task's tests, and commit from that task before repeating this final check.
