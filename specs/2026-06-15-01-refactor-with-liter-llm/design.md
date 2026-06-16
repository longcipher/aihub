# Design: Refactor aihub with liter-llm + BYOK Virtual Keys

| Metadata | Details |
| :--- | :--- |
| **Status** | Draft |
| **Created** | 2026-06-15 |
| **Scope** | Full |

## Executive Summary

Refactor the aihub LLM gateway from a monolithic single-crate architecture into a proper Cargo workspace (`hub-core`, `hub-gateway`, `hub-management`, `bin/hub`), replace all custom provider implementations with `liter-llm` (143+ providers), eliminate forbidden dependencies (`reqwest`, `anyhow`, `log`), and add full BYOK virtual key management with per-key model restrictions, RPM/TPM rate limiting, and budget enforcement.

## Source Inputs & Normalization

**Source material:**

1. User requirement: "Refactor according to AGENTS.md, use liter-llm for LLM providers, support BYOK (ref: BYOKEY)"
2. `AGENTS.md` — workspace rules, forbidden deps, engineering principles
3. Current codebase — monolithic `hub` crate with 5 custom provider implementations
4. `liter-llm` v1.5.1 docs — `DefaultClient`, `LlmClient` trait, `ClientConfig`
5. BYOKEY reference — virtual key management, OAuth flows, per-key budgets

**Assumptions:**

- `liter-llm`'s `ClientConfig` supports `base_url` override for custom endpoints (confirmed from docs)
- The existing DB schema for management API will be extended (not replaced) for virtual keys
- The existing `config.yaml` format will remain supported alongside new BYOK config
- `hpx` crate is available on crates.io and compatible with the project's MSRV (1.88)

## Requirements & Goals

### Functional Goals

| ID | Requirement |
|---|---|
| R1 | Split monolithic crate into workspace: `hub-core`, `hub-gateway`, `hub-management`, `bin/hub` |
| R2 | Replace 5 custom provider implementations with `liter-llm` `DefaultClient` |
| R3 | Support 143+ LLM providers via `liter-llm` model prefix routing |
| R4 | Implement virtual API key system: gateway keys map to provider keys with restrictions |
| R5 | Per-key model allowlists/denylists |
| R6 | Per-key RPM (requests per minute) and TPM (tokens per minute) rate limiting |
| R7 | Per-key budget enforcement (monthly spend limits, hard/soft modes) |
| R8 | Request-level BYOK: clients can pass their own API key via `Authorization` header |
| R9 | Hot-reload configuration (existing DB polling + new YAML-based key config) |
| R10 | Maintain OpenAI-compatible API surface (`/v1/chat/completions`, `/v1/embeddings`, etc.) |
| R11 | Management API CRUD for virtual keys, providers, models, pipelines |

### Non-Functional Goals

| ID | Constraint |
|---|---|
| NF1 | Replace `reqwest` → `hpx`, `anyhow` → `eyre`/`thiserror`, `log` → `tracing` |
| NF2 | Use `arc-swap` for read-heavy config state, `scc` for concurrent maps |
| NF3 | `clippy::pedantic` + `clippy::nursery` + `clippy::cargo` clean |
| NF4 | All docs/comments/commits in English only |
| NF5 | Use `ecdysis` for graceful restart in the binary |
| NF6 | OpenAPI docs via `utoipa` |

### Out of Scope

- OAuth/device-code flows (like BYOKEY's `byokey login`) — deferred to a future phase
- Desktop TUI
- Audio/speech/image generation endpoints (liter-llm supports them but gateway won't expose them yet)
- WASM bindings

## Requirements Coverage Matrix

| Req | design.md Section | Scenarios | Tasks |
|---|---|---|---|
| R1 | Architecture Overview, Workspace Structure | — | 1.1–1.4 |
| R2 | Detailed Design: Provider Layer | gateway-liter-llm | 2.1–2.3 |
| R3 | Detailed Design: Provider Layer | gateway-liter-llm | 2.2 |
| R4 | Detailed Design: Virtual Key System | byok-virtual-keys | 3.1–3.4 |
| R5 | Detailed Design: Virtual Key System | byok-virtual-keys | 3.2 |
| R6 | Detailed Design: Rate Limiting | byok-virtual-keys | 3.3 |
| R7 | Detailed Design: Budget Enforcement | byok-virtual-keys | 3.4 |
| R8 | Detailed Design: BYOK Passthrough | byok-virtual-keys | 3.5 |
| R9 | Architecture Overview | — | 1.3, 4.2 |
| R10 | Detailed Design: Gateway Routes | gateway-liter-llm | 2.4 |
| R11 | Detailed Design: Management API | — | 4.1–4.3 |
| NF1 | Dependency Migration | — | 1.1 |
| NF2 | Architecture Decisions | — | 1.1, 1.3 |
| NF5 | Binary Crate | — | 1.4 |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     bin/hub (binary)                     │
│  main.rs: CLI args, config loading, server startup       │
│  Uses ecdysis for graceful restart                       │
└──────────┬──────────────────────────────┬────────────────┘
           │                              │
           ▼                              ▼
┌─────────────────────┐    ┌──────────────────────────────┐
│    hub-gateway       │    │       hub-management          │
│                     │    │                              │
│ • Routes (axum)     │    │ • DB-based CRUD API          │
│ • Pipeline steering │    │ • Provider/Model/Pipeline    │
│ • Virtual key auth  │    │ • Virtual key management     │
│ • Rate limiting     │    │ • Config provider service    │
│ • Budget tracking   │    │ • SQLx + Postgres            │
│ • OTEL tracing      │    │                              │
└──────────┬──────────┘    └──────────┬───────────────────┘
           │                          │
           ▼                          ▼
┌─────────────────────────────────────────────────────────┐
│                      hub-core (library)                  │
│                                                         │
│ • Config types (GatewayConfig, Provider, ModelConfig)   │
│ • Virtual key types (VirtualKey, Budget, RateLimit)     │
│ • Error types (thiserror)                               │
│ • liter-llm adapter (LiterProvider wrapping DefaultClient)│
│ • State management (arc-swap based AppState)            │
│ • OTEL setup                                            │
│ • Hash/change detection                                 │
└─────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────┐
│                   liter-llm (external)                   │
│  143+ providers, streaming, tool calling, embeddings     │
└─────────────────────────────────────────────────────────┘
```

## Architecture Decisions

### Inherited Decisions (from AGENTS.md)

- **Error handling**: `eyre` for application layer, `thiserror` for library layer
- **HTTP client**: `hpx` over `reqwest`
- **Concurrent maps**: `scc` over `dashmap`/`RwLock<HashMap>`
- **Read-heavy state**: `arc-swap` over `RwLock`
- **Logging**: `tracing` only, no `log`
- **Config**: `config` crate with TOML preference
- **Observability**: OpenTelemetry OTLP gRPC
- **API docs**: `utoipa` for OpenAPI

### New Decisions

**Pattern: Adapter (for liter-llm integration)**

The existing `Provider` trait will be replaced by an adapter that wraps `liter_llm::DefaultClient`. This is an Adapter pattern — the gateway's internal request/response types adapt to liter-llm's types.

Why not Strategy: The 5 concrete provider implementations are being deleted, not selected at runtime. liter-llm handles provider selection internally via model prefix.

Why not keeping the Provider trait: liter-llm's `LlmClient` trait already provides the polymorphism we need. Adding another trait layer would be unnecessary abstraction.

**Pattern: Decorator (for virtual key middleware)**

Rate limiting and budget enforcement are Tower layers that decorate the gateway router. This matches AGENTS.md's preference for composable Tower middleware.

**Pattern: Factory (for per-request client creation)**

When a BYOK request arrives with a user-provided key, a `ClientConfigFactory` creates a `DefaultClient` with that key. Cached clients avoid repeated construction.

**SRP/DIP Check:**

- `hub-core`: Pure types + adapter, no HTTP/DB dependencies
- `hub-gateway`: HTTP routing + middleware, depends on hub-core
- `hub-management`: DB operations, depends on hub-core
- `bin/hub`: Wiring only, depends on all three

## BDD/TDD Strategy

- **Primary Language:** Rust
- **BDD Runner:** `cucumber-rs`
- **BDD Command:** `cargo test -p hub-gateway --test bdd`
- **Unit Test Command:** `cargo test --all-features`
- **Property Test Tool:** `proptest` for config validation, key matching
- **Fuzz Test Tool:** N/A (no parser/protocol/unsafe-heavy code)
- **Benchmark Tool:** N/A (no explicit latency SLA yet)
- **Feature Files:** `specs/2026-06-15-01-refactor-with-liter-llm/features/*.feature`
- **Outside-in Loop:** Virtual key auth scenarios fail first, then pass after implementation

## Code Simplification Constraints

- **Behavioral Contract:** Preserve existing OpenAI-compatible API surface. Existing YAML config format remains valid.
- **Repo Standards:** Follow AGENTS.md strictly — no `anyhow`, `reqwest`, `log`, `dashmap`.
- **Readability Priorities:** Explicit error types, clear module boundaries, no deep nesting.
- **Refactor Scope:** Delete all of `src/providers/` (openai, anthropic, azure, bedrock, vertexai). Refactor `src/state.rs` to use `arc-swap`. Refactor `src/pipelines/` to use liter-llm adapter.
- **Clarity Guardrails:** No clever abstractions — the liter-llm adapter should be a thin, obvious wrapper.

## Detailed Design

### Module Structure

```
hub-core/src/
├── lib.rs
├── config/
│   ├── mod.rs
│   ├── models.rs          # GatewayConfig, Provider, ModelConfig, Pipeline
│   ├── validation.rs      # Config validation
│   ├── hash.rs            # Config change detection
│   └── constants.rs
├── types/
│   ├── mod.rs
│   ├── virtual_key.rs     # VirtualKey, BudgetConfig, RateLimitConfig
│   └── provider.rs        # ProviderType (kept for OTEL vendor mapping)
├── error.rs               # thiserror error types
├── state.rs               # AppState with arc-swap
├── provider/
│   ├── mod.rs
│   └── adapter.rs         # LiterProvider wrapping liter_llm::DefaultClient
└── otel.rs                # OpenTelemetry setup

hub-gateway/src/
├── lib.rs
├── routes.rs              # Axum routes
├── middleware/
│   ├── mod.rs
│   ├── virtual_key_auth.rs # Extract + validate virtual keys
│   ├── rate_limiter.rs     # Per-key RPM/TPM Tower layer
│   └── budget_enforcer.rs  # Per-key budget Tower layer
├── pipeline/
│   ├── mod.rs
│   └── pipeline.rs        # Pipeline creation + steering
└── openapi.rs

hub-management/src/
├── lib.rs
├── api/
│   └── routes/
│       ├── mod.rs
│       ├── provider_routes.rs
│       ├── model_definition_routes.rs
│       ├── pipeline_routes.rs
│       └── virtual_key_routes.rs  # NEW: CRUD for virtual keys
├── db/
│   ├── mod.rs
│   └── repositories/
│       ├── mod.rs
│       ├── provider_repository.rs
│       ├── model_definition_repository.rs
│       ├── pipeline_repository.rs
│       └── virtual_key_repository.rs  # NEW
├── services/
│   ├── mod.rs
│   ├── config_provider_service.rs
│   ├── provider_service.rs
│   ├── model_definition_service.rs
│   ├── pipeline_service.rs
│   └── virtual_key_service.rs  # NEW
├── dto.rs
├── errors.rs
└── state.rs
```

### Virtual Key Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VirtualKey {
    pub id: Uuid,
    pub key_hash: String,          // SHA-256 hash of "hub-<random>"
    pub name: String,
    pub enabled: bool,
    pub allowed_models: Vec<String>,  // Empty = all models allowed
    pub denied_models: Vec<String>,
    pub rpm_limit: Option<u32>,       // Requests per minute
    pub tpm_limit: Option<u32>,       // Tokens per minute
    pub monthly_budget_cents: Option<i64>, // Hard limit in cents
    pub budget_mode: BudgetMode,      // Hard (reject) or Soft (warn)
    pub provider_key: String,         // Which provider key to use
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BudgetMode {
    Hard,  // Reject requests when budget exceeded
    Soft,  // Allow but log warnings
}
```

### Request Flow with BYOK

```
Client Request
  │
  ├─ Authorization: Bearer hub-<virtual-key>  →  Virtual key lookup
  │     │
  │     ├─ Validate key exists + enabled
  │     ├─ Check model allowlist/denylist
  │     ├─ Check RPM/TPM rate limits
  │     ├─ Check budget
  │     └─ Resolve provider API key from virtual key's provider_key
  │
  ├─ Authorization: Bearer sk-<raw-provider-key>  →  BYOK passthrough
  │     └─ Use client's key directly with liter-llm
  │
  └─ No Authorization  →  Use default provider key from config
```

### liter-llm Adapter

```rust
use hub_core::error::HubError;
use liter_llm::{ClientConfig, DefaultClient, LlmClient};
use std::time::Duration;

pub struct LiterProvider {
    client: DefaultClient,
}

impl LiterProvider {
    pub fn new(api_key: &str, base_url: Option<&str>, model_hint: Option<&str>) -> Result<Self, HubError> {
        let mut config = ClientConfig::new(api_key);
        if let Some(url) = base_url {
            config.base_url = Some(url.to_string());
        }
        config.timeout = Duration::from_secs(120);
        let client = DefaultClient::new(config, model_hint)
            .map_err(|e| HubError::ProviderInit(e.to_string()))?;
        Ok(Self { client })
    }

    pub fn client(&self) -> &impl LlmClient {
        &self.client
    }
}
```

### Config Extension for Virtual Keys (YAML)

```yaml
general:
  trace_content_enabled: true

providers:
  - key: openai
    type: openai
    api_key: ${OPENAI_API_KEY}

  - key: anthropic
    type: anthropic
    api_key: ${ANTHROPIC_API_KEY}

models:
  - key: gpt-4o
    type: gpt-4o
    provider: openai
  - key: claude-sonnet
    type: claude-sonnet-4-20250514
    provider: anthropic

pipelines:
  - name: default
    type: chat
    plugins:
      - model-router:
          models: [gpt-4o, claude-sonnet]

# NEW: Virtual key configuration
virtual_keys:
  - key: hub-team-a
    name: "Team A"
    allowed_models: [gpt-4o]
    rpm_limit: 60
    tpm_limit: 100000
    monthly_budget_cents: 5000  # $50
    budget_mode: hard
    provider_key: openai

  - key: hub-team-b
    name: "Team B"
    allowed_models: []  # All models
    rpm_limit: 120
    monthly_budget_cents: null  # Unlimited
    budget_mode: soft
    provider_key: anthropic
```

## Verification & Testing

1. **Unit tests**: colocated `#[cfg(test)]` in each module
2. **Integration tests**: `tests/` directory, test full request flow through axum
3. **Property tests**: `proptest` for config validation, key hash matching
4. **BDD**: `features/` with `cucumber-rs` for virtual key auth scenarios
5. **Runtime verification**: `curl` health check + chat completion with virtual key

## Implementation Plan

| Phase | Focus | Tasks |
|---|---|---|
| Phase 1 | Workspace setup + dependency migration | 1.1–1.4 |
| Phase 2 | liter-llm integration + provider replacement | 2.1–2.4 |
| Phase 3 | Virtual key system + BYOK | 3.1–3.5 |
| Phase 4 | Management API + DB migration | 4.1–4.3 |
| Phase 5 | Polish + verification | 5.1–5.3 |
