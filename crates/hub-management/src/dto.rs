use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVirtualKeyRequest {
    pub name: String,
    pub allowed_models: Option<Vec<String>>,
    pub denied_models: Option<Vec<String>>,
    pub rpm_limit: Option<u32>,
    pub tpm_limit: Option<u32>,
    pub monthly_budget_cents: Option<i64>,
    pub budget_mode: Option<String>,
    pub provider_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateVirtualKeyRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub allowed_models: Option<Vec<String>>,
    pub denied_models: Option<Vec<String>>,
    pub rpm_limit: Option<u32>,
    pub tpm_limit: Option<u32>,
    pub monthly_budget_cents: Option<i64>,
    pub budget_mode: Option<String>,
    pub provider_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VirtualKeyResponse {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub enabled: bool,
    pub allowed_models: Vec<String>,
    pub denied_models: Vec<String>,
    pub rpm_limit: Option<u32>,
    pub tpm_limit: Option<u32>,
    pub monthly_budget_cents: Option<i64>,
    pub budget_mode: String,
    pub provider_key: String,
}
