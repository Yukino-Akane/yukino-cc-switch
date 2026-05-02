use serde_json::json;

use cc_switch_lib::{
    get_default_cost_multiplier_test_hook, get_pricing_model_source_test_hook,
    set_default_cost_multiplier_test_hook, set_pricing_model_source_test_hook, AppError, AppType,
    Provider, ProviderService,
};

#[path = "support.rs"]
mod support;
use support::{create_test_state, ensure_test_home, reset_test_fs, test_mutex};

// 测试使用 Mutex 进行串行化，跨 await 持锁是预期行为
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn default_cost_multiplier_commands_round_trip() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let state = create_test_state().expect("create test state");

    let default = get_default_cost_multiplier_test_hook(&state, "claude")
        .await
        .expect("read default multiplier");
    assert_eq!(default, "1");

    set_default_cost_multiplier_test_hook(&state, "claude", "1.5")
        .await
        .expect("set multiplier");
    let updated = get_default_cost_multiplier_test_hook(&state, "claude")
        .await
        .expect("read updated multiplier");
    assert_eq!(updated, "1.5");

    let err = set_default_cost_multiplier_test_hook(&state, "claude", "not-a-number")
        .await
        .expect_err("invalid multiplier should error");
    // 错误已改为 Localized 类型（支持 i18n）
    match err {
        AppError::Localized { key, .. } => {
            assert_eq!(key, "error.invalidMultiplier");
        }
        other => panic!("expected localized error, got {other:?}"),
    }
}

// 测试使用 Mutex 进行串行化，跨 await 持锁是预期行为
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn pricing_model_source_commands_round_trip() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let _home = ensure_test_home();

    let state = create_test_state().expect("create test state");

    let default = get_pricing_model_source_test_hook(&state, "claude")
        .await
        .expect("read default pricing model source");
    assert_eq!(default, "response");

    set_pricing_model_source_test_hook(&state, "claude", "request")
        .await
        .expect("set pricing model source");
    let updated = get_pricing_model_source_test_hook(&state, "claude")
        .await
        .expect("read updated pricing model source");
    assert_eq!(updated, "request");

    let err = set_pricing_model_source_test_hook(&state, "claude", "invalid")
        .await
        .expect_err("invalid pricing model source should error");
    // 错误已改为 Localized 类型（支持 i18n）
    match err {
        AppError::Localized { key, .. } => {
            assert_eq!(key, "error.invalidPricingMode");
        }
        other => panic!("expected localized error, got {other:?}"),
    }
}

// 测试使用 Mutex 进行串行化，跨 await 持锁是预期行为
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn yukino_proxy_takeover_writes_yukino_live_not_codex_live() {
    let _guard = test_mutex().lock().expect("acquire test mutex");
    reset_test_fs();
    let home = ensure_test_home();

    let state = create_test_state().expect("create test state");
    let provider = Provider::with_id(
        "yukino-proxy".to_string(),
        "Yukino Proxy".to_string(),
        json!({
            "auth": {"OPENAI_API_KEY": "yukino-key"},
            "config": r#"model_provider = "custom"
model = "gpt-5.5"

[model_providers.custom]
name = "custom"
base_url = "https://api.yukino.example/v1"
wire_api = "responses"
requires_openai_auth = true
"#
        }),
        None,
    );

    ProviderService::add(&state, AppType::Yukino, provider, false).expect("add Yukino provider");
    ProviderService::switch(&state, AppType::Yukino, "yukino-proxy")
        .expect("switch Yukino provider");

    state
        .proxy_service
        .set_takeover_for_app("yukino", true)
        .await
        .expect("enable Yukino takeover");

    let yukino_auth = home.join(".yukino").join("auth.json");
    let yukino_config = home.join(".yukino").join("config.toml");
    let auth_value: serde_json::Value =
        cc_switch_lib::read_json_file(&yukino_auth).expect("read Yukino auth after takeover");
    assert_eq!(
        auth_value
            .get("OPENAI_API_KEY")
            .and_then(|value| value.as_str()),
        Some("PROXY_MANAGED"),
        "Yukino takeover should replace the Yukino live token with the proxy placeholder"
    );

    let config_text = std::fs::read_to_string(&yukino_config).expect("read Yukino config");
    assert!(
        config_text.contains("http://127.0.0.1:"),
        "Yukino takeover should point .yukino/config.toml at the local proxy"
    );

    assert!(
        !home.join(".codex").join("auth.json").exists(),
        "Yukino proxy takeover must not write Codex auth.json"
    );
    assert!(
        !home.join(".codex").join("config.toml").exists(),
        "Yukino proxy takeover must not write Codex config.toml"
    );

    let status = state
        .proxy_service
        .get_takeover_status()
        .await
        .expect("read takeover status");
    assert!(status.yukino, "Yukino takeover status should be true");
}
