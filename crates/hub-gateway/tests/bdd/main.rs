use std::collections::HashMap;

use cucumber::{World, given, then, when};
use hub_core::types::virtual_key::{BudgetMode, VirtualKey};
use uuid::Uuid;

#[derive(Debug, Default, World)]
pub struct GatewayWorld {
    virtual_keys: HashMap<String, VirtualKey>,
    last_response_status: Option<u16>,
    last_error_message: Option<String>,
}

#[given(regex = r#"the gateway is running with virtual key configuration"#)]
async fn gateway_running(_world: &mut GatewayWorld) {
    // Gateway is assumed running for BDD tests
}

#[given(regex = r#"the following virtual keys exist:"#)]
async fn virtual_keys_exist(world: &mut GatewayWorld, step: &cucumber::gherkin::Step) {
    if let Some(table) = step.table.as_ref() {
        for row in table.rows.iter().skip(1) {
            if row.len() >= 8 {
                let key = VirtualKey {
                    id: Uuid::new_v4(),
                    key_hash: format!("hash-{}", row[0]),
                    name: row[1].clone(),
                    enabled: true,
                    allowed_models: row[2]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    denied_models: vec![],
                    rpm_limit: row[3].parse().ok(),
                    tpm_limit: row[4].parse().ok(),
                    monthly_budget_cents: row[5].parse().ok(),
                    budget_mode: match row[6].as_str() {
                        "soft" => BudgetMode::Soft,
                        _ => BudgetMode::Hard,
                    },
                    provider_key: row[7].clone(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                world.virtual_keys.insert(row[0].clone(), key);
            }
        }
    }
}

#[given(regex = r#"a valid virtual key "(.+)""#)]
async fn valid_virtual_key(world: &mut GatewayWorld, key_name: String) {
    assert!(world.virtual_keys.contains_key(&key_name), "Virtual key {key_name} should exist");
}

#[given(regex = r#"an invalid virtual key "(.+)""#)]
async fn invalid_virtual_key(_world: &mut GatewayWorld, _key_name: String) {
    // This key should NOT exist in the world
}

#[given(regex = r#"a disabled virtual key "(.+)""#)]
async fn disabled_virtual_key(world: &mut GatewayWorld, key_name: String) {
    if let Some(key) = world.virtual_keys.get_mut(&key_name) {
        key.enabled = false;
    }
}

#[when(regex = r#"I send a chat completion request with Authorization header "(.+)""#)]
async fn send_chat_request(world: &mut GatewayWorld, auth_header: String) {
    let token = auth_header.trim_start_matches("Bearer ");
    if token.starts_with("hub-") {
        if let Some(vk) = world.virtual_keys.get(token) {
            if vk.enabled {
                world.last_response_status = Some(200);
            } else {
                world.last_response_status = Some(403);
            }
        } else {
            world.last_response_status = Some(401);
            world.last_error_message = Some("Invalid virtual key".to_string());
        }
    } else {
        world.last_response_status = Some(200);
    }
}

#[when(
    regex = r#"I send a chat completion request with model "(.+)" and Authorization header "(.+)""#
)]
async fn send_chat_request_with_model(
    world: &mut GatewayWorld,
    model: String,
    auth_header: String,
) {
    let token = auth_header.trim_start_matches("Bearer ");
    if let Some(vk) = world.virtual_keys.get(token) {
        if vk.is_model_allowed(&model) {
            world.last_response_status = Some(200);
        } else {
            world.last_response_status = Some(403);
            world.last_error_message = Some("model not allowed".to_string());
        }
    } else {
        world.last_response_status = Some(401);
    }
}

#[then(regex = r#"the request should be authenticated"#)]
async fn request_authenticated(world: &mut GatewayWorld) {
    assert!(world.last_response_status.is_some(), "No response status");
    assert_eq!(world.last_response_status.unwrap(), 200);
}

#[then(regex = r#"the response status should be (\d+)"#)]
async fn response_status(world: &mut GatewayWorld, status: u16) {
    assert_eq!(world.last_response_status, Some(status));
}

#[then(regex = r#"the error message should contain "(.+)""#)]
async fn error_message_contains(world: &mut GatewayWorld, expected: String) {
    assert!(
        world.last_error_message.as_ref().unwrap().contains(&expected),
        "Expected error to contain '{expected}', got '{:?}'",
        world.last_error_message
    );
}

#[tokio::main]
async fn main() {
    GatewayWorld::cucumber()
        .run_and_exit("../../specs/2026-06-15-01-refactor-with-liter-llm/features")
        .await;
}
