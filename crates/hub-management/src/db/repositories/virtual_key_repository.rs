use hub_core::types::virtual_key::{BudgetMode, VirtualKey};
use sqlx::PgPool;
use uuid::Uuid;

pub struct VirtualKeyRepository {
    pool: PgPool,
}

impl VirtualKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, key: &VirtualKey) -> Result<VirtualKey, sqlx::Error> {
        let budget_mode_str = match key.budget_mode {
            BudgetMode::Hard => "hard",
            BudgetMode::Soft => "soft",
        };

        let result = sqlx::query_as::<_, VirtualKeyRow>(
            r#"
            INSERT INTO virtual_keys (id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key, created_at, updated_at
            "#,
        )
        .bind(key.id)
        .bind(&key.key_hash)
        .bind(&key.name)
        .bind(key.enabled)
        .bind(&key.allowed_models)
        .bind(&key.denied_models)
        .bind(key.rpm_limit.map(|v| v as i32))
        .bind(key.tpm_limit.map(|v| v as i32))
        .bind(key.monthly_budget_cents)
        .bind(budget_mode_str)
        .bind(&key.provider_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.into())
    }

    pub async fn get_by_hash(&self, key_hash: &str) -> Result<Option<VirtualKey>, sqlx::Error> {
        let result = sqlx::query_as::<_, VirtualKeyRow>(
            r#"
            SELECT id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key, created_at, updated_at
            FROM virtual_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.into()))
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<VirtualKey>, sqlx::Error> {
        let result = sqlx::query_as::<_, VirtualKeyRow>(
            r#"
            SELECT id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key, created_at, updated_at
            FROM virtual_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.into()))
    }

    pub async fn list(&self) -> Result<Vec<VirtualKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, VirtualKeyRow>(
            r#"
            SELECT id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key, created_at, updated_at
            FROM virtual_keys
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(&self, key: &VirtualKey) -> Result<VirtualKey, sqlx::Error> {
        let budget_mode_str = match key.budget_mode {
            BudgetMode::Hard => "hard",
            BudgetMode::Soft => "soft",
        };

        let result = sqlx::query_as::<_, VirtualKeyRow>(
            r#"
            UPDATE virtual_keys
            SET name = $2, enabled = $3, allowed_models = $4, denied_models = $5, rpm_limit = $6, tpm_limit = $7, monthly_budget_cents = $8, budget_mode = $9, provider_key = $10
            WHERE id = $1
            RETURNING id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key, created_at, updated_at
            "#,
        )
        .bind(key.id)
        .bind(&key.name)
        .bind(key.enabled)
        .bind(&key.allowed_models)
        .bind(&key.denied_models)
        .bind(key.rpm_limit.map(|v| v as i32))
        .bind(key.tpm_limit.map(|v| v as i32))
        .bind(key.monthly_budget_cents)
        .bind(budget_mode_str)
        .bind(&key.provider_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.into())
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM virtual_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Internal row type for database mapping
#[derive(sqlx::FromRow)]
struct VirtualKeyRow {
    id: Uuid,
    key_hash: String,
    name: String,
    enabled: bool,
    allowed_models: Vec<String>,
    denied_models: Vec<String>,
    rpm_limit: Option<i32>,
    tpm_limit: Option<i32>,
    monthly_budget_cents: Option<i64>,
    budget_mode: String,
    provider_key: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<VirtualKeyRow> for VirtualKey {
    fn from(row: VirtualKeyRow) -> Self {
        VirtualKey {
            id: row.id,
            key_hash: row.key_hash,
            name: row.name,
            enabled: row.enabled,
            allowed_models: row.allowed_models,
            denied_models: row.denied_models,
            rpm_limit: row.rpm_limit.map(|v| v as u32),
            tpm_limit: row.tpm_limit.map(|v| v as u32),
            monthly_budget_cents: row.monthly_budget_cents,
            budget_mode: match row.budget_mode.as_str() {
                "soft" => BudgetMode::Soft,
                _ => BudgetMode::Hard,
            },
            provider_key: row.provider_key,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
