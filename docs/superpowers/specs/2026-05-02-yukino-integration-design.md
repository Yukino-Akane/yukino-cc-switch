# Yukino Integration Design

Date: 2026-05-02

## Summary

Add Yukino as a first-class app in CC Switch. Yukino should appear next to Claude, Codex, Gemini, OpenCode, OpenClaw, and Hermes, with its own provider list, settings visibility, session management, usage import, Skills, MCP, and memory management.

Yukino is Codex-compatible at the provider and runtime-config layer. Its home directory is `~/.yukino`, which contains Codex-style `config.toml`, `auth.json`, `sessions/`, `skills/`, and plugin/runtime state. The integration should reuse Codex-compatible parsing and writing behavior while keeping Yukino application state isolated from OpenAI/Codex OAuth state.

## Goals

- Add `yukino` as a supported `AppType` and frontend `AppId`.
- Manage Yukino providers independently from Codex providers.
- Write Yukino provider changes to `~/.yukino/config.toml` and `~/.yukino/auth.json`.
- Reuse Codex-compatible config semantics: `model_provider`, `[model_providers.*]`, `wire_api`, `model`, reasoning effort, and auth JSON.
- Allow Yukino providers to use existing `codex_oauth` provider/proxy support without merging OAuth state into Yukino app login state.
- Scan, display, search, resume, delete, and import usage from `~/.yukino/sessions/**/*.jsonl`.
- Support Yukino Skills, plugin marketplace management, and MCP through CC Switch UX.
- Add a Yukino memory panel for `~/.yukino/memories/raw_memories.md`.
- Preserve current Codex behavior and avoid accidental writes to `~/.codex` when the active app is Yukino.

## Non-Goals

- Do not implement a separate Yukino OAuth account system in CC Switch.
- Do not read or mutate Yukino desktop app login/runtime state beyond the explicit files managed by the feature.
- Do not change Codex provider behavior except through shared refactors that preserve existing outputs.
- Do not import or display private prompt history from Yukino application state files.
- Do not read or write plugin OAuth credentials in `~/.yukino/.credentials.json`; those remain owned by Yukino.

## Architecture

Introduce a Codex-compatible app layer rather than duplicating the current Codex implementation.

Core concept:

- `AppType::Codex` maps to the Codex-compatible home `~/.codex`.
- `AppType::Yukino` maps to the Codex-compatible home `~/.yukino`.
- Shared helpers accept an app/home parameter and operate on `auth.json`, `config.toml`, `sessions/`, and `skills/`.

Suggested backend shape:

- Add `AppType::Yukino`.
- Add `settings.yukino_config_dir` with default `~/.yukino`.
- Refactor `codex_config.rs` path helpers into app-aware helpers:
  - current Codex public functions remain as wrappers for compatibility.
  - new helpers accept a `CodexLikeApp` or `AppType`.
- Refactor Codex session scanning and usage import to accept:
  - provider id: `codex` or `yukino`
  - config/home dir
  - log label for diagnostics
- Extend provider switching paths so `AppType::Yukino` uses Codex-compatible provider config validation and live writes, pointed at the Yukino home.

## Provider And Proxy Flow

Yukino provider records live under `providers.app_type = 'yukino'`. They use the same settings shape as Codex providers:

- `auth`: object written to `~/.yukino/auth.json`
- `config`: TOML written to `~/.yukino/config.toml`

Switch flow:

1. User selects a Yukino provider in the Yukino tab.
2. CC Switch validates it using Codex-compatible validation.
3. CC Switch writes `auth.json` and `config.toml` atomically in `~/.yukino`.
4. Existing Yukino-specific config sections in `config.toml` must be preserved when safe, including features, projects, marketplaces, plugins, skills config, memory settings, and Windows sandbox settings.
5. Provider-specific model-provider sections are updated using the same stable-provider strategy used for Codex.

Proxy behavior:

- Yukino can use the existing Codex-compatible proxy providers and wire APIs.
- `codex_oauth` may be selected for Yukino providers.
- `codex_oauth` tokens remain owned by CC Switch's provider/OAuth storage.
- Yukino app login/runtime state remains owned by Yukino and is not written by CC Switch.

## Sessions And Usage

Yukino sessions are stored under `~/.yukino/sessions/YYYY/MM/DD/*.jsonl` and use a Codex-like rollout JSONL shape.

Session manager:

- Add `yukino` to provider filters.
- Add a Yukino session provider that reuses the Codex JSONL parser with a different root and provider id.
- Resume command support should be disabled for Yukino until CC Switch has a verified Yukino CLI/deeplink resume command. The session list and message viewer still work without resume.
- Deletion must validate paths stay inside `~/.yukino/sessions`.

Usage import:

- Refactor `session_usage_codex` into a Codex-like importer.
- Import Yukino usage into `proxy_request_logs` with `app_type = 'yukino'` and `data_source = 'session'`.
- Track sync state using the full file path, so Codex and Yukino files do not collide.

## Skills And Plugins

Yukino Skills should participate in the unified Skills system.

Database:

- Add `enabled_yukino` to `skills`.
- Add `enabled_yukino` to any app-availability structs and API DTOs.

Paths:

- Yukino live skills directory defaults to `~/.yukino/skills`.
- Existing unified/SSOT storage still controls installed skill content when configured.
- Sync to Yukino should mirror the Codex skill sync pattern but target the Yukino directory.

Yukino plugin marketplace support should manage plugin discovery and enablement without touching plugin credentials.

Marketplace sources:

- Read marketplace manifests from `~/.yukino/marketplaces/*/.agents/plugins/marketplace.json`.
- Read bundled/local marketplace manifests referenced by `[marketplaces.*]` entries in `~/.yukino/config.toml` when the source path exists.
- Do not scan transient plugin build/cache directories unless they are referenced by a marketplace entry.

Plugin enablement:

- Read enabled plugins from `[plugins."<plugin>@<marketplace>"] enabled = true` in `~/.yukino/config.toml`.
- Enable or disable plugins by updating only the relevant `[plugins.*]` entries in `~/.yukino/config.toml`.
- Preserve all unrelated TOML sections and comments where possible.
- Do not read or write `~/.yukino/.credentials.json`.
- Do not mutate `.codex-global-state.json`; CC Switch should treat it as Yukino application UI/runtime state.

## MCP

Yukino MCP should use Codex-style TOML MCP config in `~/.yukino/config.toml`.

Database:

- Add `enabled_yukino` to `mcp_servers`.
- Extend `McpApps` and frontend app maps with Yukino.

Sync behavior:

- Import `[mcp_servers.*]` from Yukino's config.
- Sync enabled MCP servers into Yukino's config using the Codex TOML format.
- Preserve unrelated Yukino config sections.

## Memories

Add a Yukino memory panel for the simple current memory file:

- Path: `~/.yukino/memories/raw_memories.md`
- Read returns an empty string when the file is missing.
- Write creates parent directories and atomically replaces the file.
- The panel should not inspect or mutate the `.git` repository internals in `~/.yukino/memories`.

The initial UI can mirror the Hermes memory surface where practical, but it should use Yukino labels and a single memory blob.

## Frontend

Add Yukino to:

- `AppId` type and app constants.
- `AppSwitcher`.
- `APP_ICON_MAP`.
- visible app settings.
- provider list and provider form app selectors.
- session manager filter.
- Skills and MCP app badges.
- settings directory picker.
- translations in English, Chinese, and Japanese.

Use a simple Yukino text/icon fallback if no dedicated icon asset exists in the repository. Avoid changing existing app colors except where Yukino needs a distinct accent.

## Database Migration

Add a new schema migration version that:

- Adds `enabled_yukino BOOLEAN NOT NULL DEFAULT 0` to `skills`.
- Adds `enabled_yukino BOOLEAN NOT NULL DEFAULT 0` to `mcp_servers`.
- Does not seed a generic official Yukino provider. On first startup with `~/.yukino/config.toml`, import the live Yukino config as a normal user provider if no Yukino providers exist.
- Adds `proxy_config` support for `app_type = 'yukino'` if proxy takeover and request logging are enabled for Yukino.

The migration must preserve all existing rows and must be safe to run on older databases missing intermediate optional columns.

## Error Handling And Safety

- All path writes must resolve under the effective Yukino config dir.
- Atomic writes should be used for `auth.json`, `config.toml`, and memory files.
- When preserving config sections fails due to invalid TOML, return a clear error and do not partially write live config.
- OAuth token errors should mention the provider/OAuth layer, not Yukino app login.
- Session deletion must reject foreign paths.
- Logging must avoid printing API keys, OAuth tokens, and credentials.

## Testing

Backend tests:

- `AppType::Yukino` parse/format/all-list behavior.
- Yukino config dir defaults and override handling.
- Codex-compatible live write writes to `.yukino`, not `.codex`.
- Yukino live write preserves unrelated `config.toml` sections.
- Yukino provider validation accepts Codex-compatible settings.
- Yukino sessions scan JSONL fixtures.
- Yukino session deletion rejects foreign paths.
- Yukino usage import records `app_type = 'yukino'`.
- Skills and MCP DAO read/write include `enabled_yukino`.
- Yukino plugin discovery reads marketplace manifests and enablement from `config.toml`.
- Yukino plugin enable/disable preserves unrelated config and never writes `.credentials.json`.
- Yukino memory read/write round trip.

Frontend tests:

- app switcher renders Yukino when visible.
- settings form round-trips Yukino config dir.
- session filter includes Yukino.
- Skills/MCP badges include Yukino.
- Yukino plugin marketplace page renders discovered plugins and current enablement.
- provider flows can target Yukino.

Manual verification:

- Switch a Yukino provider and confirm only `~/.yukino/auth.json` and `~/.yukino/config.toml` change.
- Confirm `~/.codex` remains unchanged when switching Yukino.
- Confirm existing Codex provider switching still works.
- Confirm Yukino sessions appear in the session manager.
- Confirm `codex_oauth` can be selected for a Yukino provider without changing Yukino app state files.

## Rollout

1. Land the shared Codex-compatible abstraction with Codex behavior unchanged.
2. Add Yukino app type, settings, and provider switching.
3. Add sessions and usage import.
4. Add Skills and MCP support.
5. Add memory panel.
6. Complete frontend labels/tests and manual verification.
