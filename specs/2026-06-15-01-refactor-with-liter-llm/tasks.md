# Refactor aihub with liter-llm + BYOK Virtual Keys — Tasks

| Metadata | Details |
| :--- | :--- |
| **Design Doc** | specs/2026-06-15-01-refactor-with-liter-llm/design.md |
| **Status** | Planning |

## Phase 1: Workspace Setup + Dependency Migration

### Task 1.1: Create Workspace Structure

> **Context:** Current project is a single crate. Need to split into `hub-core`, `hub-gateway`, `hub-management`, `bin/hub`.
> **Verification:** `cargo check --workspace` succeeds.
> **Requirement Coverage:** R1, NF1
> **Scenario Coverage:** N/A (infrastructure)

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior
- **Simplification Focus:** Clean module boundaries
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (infrastructure task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo check --workspace` succeeds
- [x] Create root `Cargo.toml` with `[workspace]` and `members = ["bin/hub", "crates/hub-core", "crates/hub-gateway", "crates/hub-management"]`
- [x] Create `crates/hub-core/Cargo.toml` with `edition.workspace = true`
- [x] Create `crates/hub-gateway/Cargo.toml` with dependencies on `hub-core`
- [x] Create `crates/hub-management/Cargo.toml` with dependencies on `hub-core` and `sqlx`
- [x] Create `bin/hub/Cargo.toml` as binary crate depending on `hub-gateway` and `hub-management`
- [x] Move `src/types/` → `crates/hub-core/src/types/`
- [x] Move `src/config/` → `crates/hub-core/src/config/`
- [x] Move `src/models/` → `crates/hub-core/src/models/`
- [x] Verification: `cargo check --workspace`

### Task 1.2: Replace Forbidden Dependencies

> **Context:** AGENTS.md forbids `reqwest`, `anyhow`, `log`, `dashmap`. Replace with `hpx`, `eyre`/`thiserror`, `tracing`, `scc`.
> **Verification:** `cargo check --workspace` + `cargo clippy --all` pass with no forbidden deps.
> **Requirement Coverage:** NF1, NF2
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior
- **Simplification Focus:** Remove forbidden deps, use idiomatic alternatives
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (infrastructure task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo check --workspace && cargo clippy --all -- -D warnings` passes
- [x] `cargo add hpx --workspace` with `rustls` feature
- [x] `cargo add eyre --workspace` (app layer)
- [x] `cargo add thiserror --workspace` (lib layer)
- [x] `cargo add arc-swap --workspace`
- [x] `cargo add scc --workspace`
- [x] `cargo add liter-llm --workspace` with `native-http` feature
- [x] `cargo add ecdysis --workspace` (binary only)
- [x] Replace all `anyhow::Result` → `eyre::Result` in `bin/hub`
- [x] Replace all `anyhow::Error` → `thiserror` enums in `hub-core` and `hub-management`
- [x] Replace `reqwest` → `hpx` in all HTTP client usage
- [x] Replace `log` → `tracing` in all logging macros
- [x] Remove `anyhow`, `reqwest`, `log` from all `Cargo.toml`
- [x] Verification: `cargo check --workspace && cargo clippy --all -- -D warnings`

### Task 1.3: Refactor State Management

> **Context:** `AppState` uses `Arc<RwLock<InnerAppState>>`. Replace with `arc-swap` for lock-free reads.
> **Verification:** All existing state tests pass.
> **Requirement Coverage:** NF2, R9
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior
- **Simplification Focus:** Lock-free reads, simpler code
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (infrastructure task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test --workspace` passes
- [x] Replace `Arc<RwLock<InnerAppState>>` with `ArcSwap<InnerAppState>` in `hub-core/src/state.rs`
- [x] Replace `Arc<RwLock<Arc<Router>>>` with `ArcSwap<Router>` for current_router
- [x] Update `update_config()` to use `arc_swap::store()` instead of `write().unwrap()`
- [x] Update `get_current_router()` to use `arc_swap::load()` instead of `read().unwrap()`
- [x] Move pipeline steering logic to `hub-gateway`
- [x] Verification: `cargo test --workspace` (existing tests pass)

### Task 1.4: Binary Crate with ecdysis

> **Context:** Move `main.rs` to `bin/hub/`, add `ecdysis` for graceful restart.
> **Verification:** `cargo run -p hub -- --help` works. Server starts and responds to `/health`.
> **Requirement Coverage:** NF5, R1
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior
- **Simplification Focus:** Clean binary entry point
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (infrastructure task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo run -p hub -- --help` works
  - [x] Server starts and responds to `/health`
- [x] Create `bin/hub/src/main.rs` wiring hub-gateway and hub-management
- [x] Integrate `ecdysis` for graceful restart/reload
- [x] Move config loading logic to `bin/hub`
- [x] Move DB mode detection to `bin/hub`
- [x] Verification: `cargo run -p hub -- --help && cargo run -p hub` starts server

## Phase 2: liter-llm Integration + Provider Replacement

### Task 2.1: Create liter-llm Provider Adapter

> **Context:** Replace 5 custom provider implementations (openai, anthropic, azure, bedrock, vertexai) with a single adapter wrapping `liter_llm::DefaultClient`.
> **Verification:** Unit tests create adapter with mock config, verify it constructs successfully.
> **Requirement Coverage:** R2, R3
> **Scenario Coverage:** gateway-liter-llm

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing API surface
- **Simplification Focus:** Delete ~1500 lines of custom provider code, replace with ~80 lines
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-core` passes
- [x] Create `crates/hub-core/src/provider/mod.rs`
- [x] Create `crates/hub-core/src/provider/adapter.rs` with `LiterProvider` struct
- [x] Implement `LiterProvider::new(api_key, base_url, model_hint)` → `Result<Self>`
- [x] Implement `LiterProvider::chat()` wrapping `LlmClient::chat()`
- [x] Implement `LiterProvider::chat_stream()` wrapping `LlmClient::chat_stream()`
- [x] Implement `LiterProvider::embed()` wrapping `LlmClient::embed()`
- [x] Add type conversion: `hub-core` chat types ↔ `liter_llm` chat types
- [x] Add unit tests for adapter construction and type conversion
- [x] Delete `src/providers/openai/`, `src/providers/anthropic/`, `src/providers/azure/`, `src/providers/bedrock/`, `src/providers/vertexai/`
- [x] Delete `src/providers/provider.rs`, `src/providers/registry.rs`
- [x] Verification: `cargo test -p hub-core`

### Task 2.2: Update Model Registry for liter-llm

> **Context:** `ModelRegistry` currently holds `Arc<dyn Provider>` instances. Replace with `Arc<LiterProvider>`.
> **Verification:** Model registry tests pass.
> **Requirement Coverage:** R2, R3
> **Scenario Coverage:** gateway-liter-llm

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior
- **Simplification Focus:** Simpler model resolution
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-core` passes
- [x] Update `ModelInstance` to hold `Arc<LiterProvider>` instead of `Arc<dyn Provider>`
- [x] Update `ModelRegistry::new()` to create `LiterProvider` per provider config
- [x] Update `ModelRegistry` to use `scc::HashMap` instead of `std::collections::HashMap`
- [x] Update model info response to include provider prefix (e.g., `openai/gpt-4o`)
- [x] Update existing model registry tests
- [x] Verification: `cargo test -p hub-core`

### Task 2.3: Update Pipeline to Use liter-llm

> **Context:** Pipeline handler calls `model.chat_completions()` which goes through the old Provider trait. Route through new adapter.
> **Verification:** Pipeline tests pass with mock provider.
> **Requirement Coverage:** R2, R10
> **Scenario Coverage:** gateway-liter-llm

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior
- **Simplification Focus:** Cleaner request flow
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway` passes
- [x] Update `hub-gateway/src/pipeline/pipeline.rs` to use new `ModelInstance` API
- [x] Update `chat_completions()` handler to call adapter's `chat()` / `chat_stream()`
- [x] Update `embeddings()` handler to call adapter's `embed()`
- [x] Keep SSE streaming support via `chat_stream()`
- [x] Update pipeline tests
- [x] Verification: `cargo test -p hub-gateway`

### Task 2.4: Update Config for liter-llm Provider Format

> **Context:** Config `Provider.type` currently uses `openai`, `anthropic`, etc. With liter-llm, the model name prefix handles routing. Update config to be simpler.
> **Verification:** Config loading + validation tests pass.
> **Requirement Coverage:** R2, R3
> **Scenario Coverage:** gateway-liter-llm

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Existing YAML config remains valid, new fields optional
- **Simplification Focus:** Simpler provider config
- **Advanced Test Coverage:** `proptest` for config validation
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] `proptest` for config round-trip serialization passes
- **Runtime Verification:**
  - [x] `cargo test -p hub-core` passes
- [x] Update `Provider` type to support optional `base_url` field
- [x] Keep `ProviderType` enum for backward compatibility but map to liter-llm internally
- [x] Update config validation to accept liter-llm provider format
- [x] Update `config-example.yaml` with new format
- [x] Add `proptest` for config round-trip serialization
- [x] Verification: `cargo test -p hub-core`

## Phase 3: Virtual Key System + BYOK

### Task 3.1: Virtual Key Data Model

> **Context:** Define `VirtualKey`, `BudgetMode`, `RateLimitConfig` types in hub-core.
> **Verification:** Types serialize/deserialize correctly.
> **Requirement Coverage:** R4
> **Scenario Coverage:** byok-virtual-keys

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Clear type definitions
- **Advanced Test Coverage:** `proptest` for serialization round-trip
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] `proptest` for serialization round-trip passes
- **Runtime Verification:**
  - [x] `cargo test -p hub-core` passes
- [x] Create `crates/hub-core/src/types/virtual_key.rs`
- [x] Define `VirtualKey` struct with all fields (id, key_hash, name, enabled, allowed_models, denied_models, rpm_limit, tpm_limit, monthly_budget_cents, budget_mode, provider_key)
- [x] Define `BudgetMode` enum (Hard, Soft)
- [x] Define `RateLimitConfig` struct
- [x] Add `#[derive(sqlx::FromRow)]` support
- [x] Add serde serialization/deserialization
- [x] Add utoipa ToSchema derive
- [x] Add `proptest` for serialization round-trip
- [x] Verification: `cargo test -p hub-core`

### Task 3.2: Virtual Key Auth Middleware

> **Context:** Tower middleware that extracts `Authorization: Bearer hub-<key>`, validates against stored virtual keys, and injects resolved provider key into request extensions.
> **Verification:** Middleware correctly validates keys and rejects invalid ones.
> **Requirement Coverage:** R4, R5, R8
> **Scenario Coverage:** byok-virtual-keys

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Clear auth flow
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] Scenario "Virtual key authentication succeeds" passes
  - [x] Scenario "Virtual key authentication fails with invalid key" passes
  - [x] Scenario "Virtual key authentication fails with disabled key" passes
  - [x] Scenario "Model allowlist enforcement" passes
  - [x] Scenario "Model allowlist blocks unauthorized models" passes
  - [x] Scenario "Empty allowlist permits all models" passes
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway` passes
- [x] Create `crates/hub-gateway/src/middleware/virtual_key_auth.rs`
- [x] Implement `VirtualKeyAuth` Tower layer
- [x] Extract `Authorization` header, detect `hub-` prefix
- [x] Look up virtual key by hash in `scc::HashMap`
- [x] Check `enabled` flag
- [x] Check model allowlist/denylist against request model
- [x] Inject resolved provider API key into request extensions
- [x] Support BYOK passthrough: keys without `hub-` prefix passed directly
- [x] Support fallback to default provider key when no auth header
- [x] Add unit tests for all auth paths
- [x] BDD: Implement virtual key auth scenarios
- [x] Verification: `cargo test -p hub-gateway`

### Task 3.3: Rate Limiting Middleware

> **Context:** Per-key RPM and TPM rate limiting using token bucket algorithm.
> **Verification:** Rate limiter correctly throttles requests.
> **Requirement Coverage:** R6
> **Scenario Coverage:** byok-virtual-keys

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Simple token bucket
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] Scenario "Rate limiting enforcement - RPM" passes
  - [x] Scenario "Rate limiting enforcement - TPM" passes
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway` passes
- [x] Create `crates/hub-gateway/src/middleware/rate_limiter.rs`
- [x] Implement per-key token bucket using `scc::HashMap<String, TokenBucket>`
- [x] RPM check: reject with 429 when exceeded
- [x] TPM check: track tokens from response usage, reject when exceeded
- [x] Token bucket refill based on time elapsed
- [x] Add `X-RateLimit-Remaining` and `X-RateLimit-Reset` response headers
- [x] Add unit tests for rate limiting logic
- [x] BDD: Implement rate limiting scenarios
- [x] Verification: `cargo test -p hub-gateway`

### Task 3.4: Budget Enforcement Middleware

> **Context:** Per-key monthly budget tracking with hard/soft enforcement.
> **Verification:** Budget middleware correctly blocks/warns when exceeded.
> **Requirement Coverage:** R7
> **Scenario Coverage:** byok-virtual-keys

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Simple cost tracking
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] Scenario "Budget enforcement - hard mode blocks requests" passes
  - [x] Scenario "Budget enforcement - soft mode allows requests" passes
  - [x] Scenario "Monthly budget reset" passes
  - [x] Scenario "Unlimited budget key" passes
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway` passes
- [x] Create `crates/hub-gateway/src/middleware/budget_enforcer.rs`
- [x] Track per-key monthly spend using `scc::HashMap<String, BudgetTracker>`
- [x] Calculate cost from response usage (input/output tokens × model pricing)
- [x] Hard mode: reject with 402 when budget exceeded
- [x] Soft mode: allow but log warning via `tracing::warn!`
- [x] Reset monthly counters on month boundary
- [x] Add `X-Budget-Remaining` response header
- [x] Add unit tests for budget logic
- [x] BDD: Implement budget enforcement scenarios
- [x] Verification: `cargo test -p hub-gateway`

### Task 3.5: BYOK Request-Level Key Passthrough

> **Context:** When client sends `Authorization: Bearer sk-<raw-key>` (no `hub-` prefix), create a per-request `DefaultClient` with that key.
> **Verification:** BYOK requests route correctly to the provider.
> **Requirement Coverage:** R8
> **Scenario Coverage:** byok-virtual-keys

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Simple passthrough
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] Scenario "Hot-reload adds new virtual key" passes
  - [x] Scenario "Hot-reload disables virtual key" passes
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway` passes
- [x] In virtual key auth middleware, detect non-`hub-` prefixed keys
- [x] Create per-request `LiterProvider` with the user's key
- [x] Cache `DefaultClient` instances using `scc::HashMap` with TTL eviction
- [x] Route request through the BYOK provider instead of configured provider
- [x] Add unit tests for BYOK passthrough
- [x] BDD: Implement BYOK passthrough scenarios
- [x] Verification: `cargo test -p hub-gateway`

## Phase 4: Management API + DB Migration

### Task 4.1: Virtual Key DB Schema + Repository

> **Context:** Add `virtual_keys` table to PostgreSQL schema. Create repository for CRUD.
> **Verification:** Migration applies cleanly, repository CRUD tests pass.
> **Requirement Coverage:** R4, R11
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Standard CRUD patterns
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-management` passes
- [x] Create migration `migrations/NNNN_create_virtual_keys.sql`
- [x] Table: `virtual_keys` (id UUID PK, key_hash TEXT UNIQUE, name TEXT, enabled BOOL, allowed_models TEXT[], denied_models TEXT[], rpm_limit INT, tpm_limit INT, monthly_budget_cents BIGINT, budget_mode TEXT, provider_key TEXT, created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ)
- [x] Create `crates/hub-management/src/db/repositories/virtual_key_repository.rs`
- [x] Implement `create()`, `get_by_hash()`, `list()`, `update()`, `delete()`
- [x] Add unit tests with testcontainers
- [x] Verification: `cargo test -p hub-management`

### Task 4.2: Virtual Key Service + API Routes

> **Context:** Create service layer and REST API for virtual key management.
> **Verification:** API endpoints return correct responses.
> **Requirement Coverage:** R4, R11
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** Standard service patterns
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-management` passes
- [x] Create `crates/hub-management/src/services/virtual_key_service.rs`
- [x] Implement `create_key()`, `get_key()`, `list_keys()`, `update_key()`, `delete_key()`, `rotate_key()`
- [x] Key generation: `hub-` prefix + 48 random bytes base64url-encoded
- [x] Key hashing: SHA-256 of the full key for storage
- [x] Create `crates/hub-management/src/api/routes/virtual_key_routes.rs`
- [x] POST `/api/v1/management/virtual-keys` — create
- [x] GET `/api/v1/management/virtual-keys` — list
- [x] GET `/api/v1/management/virtual-keys/:id` — get
- [x] PUT `/api/v1/management/virtual-keys/:id` — update
- [x] DELETE `/api/v1/management/virtual-keys/:id` — delete
- [x] POST `/api/v1/management/virtual-keys/:id/rotate` — rotate key
- [x] Add OpenAPI docs via `utoipa`
- [x] Verification: `cargo test -p hub-management`

### Task 4.3: Config Provider Integration

> **Context:** Update `ConfigProviderService` to include virtual keys in the live config.
> **Verification:** DB-polling picks up new virtual keys.
> **Requirement Coverage:** R4, R9
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing behavior + extend
- **Simplification Focus:** N/A
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-management` passes
- [x] Update `ConfigProviderService::fetch_live_config()` to include virtual keys
- [x] Update `GatewayConfig` to include `virtual_keys: Vec<VirtualKey>`
- [x] Update config hash calculation to include virtual keys
- [x] Update DB poller to detect virtual key changes
- [x] Verification: `cargo test -p hub-management`

## Phase 5: Polish + Verification

### Task 5.1: Update OpenAPI Spec

> **Context:** OpenAPI spec must reflect new virtual key endpoints and liter-llm model format.
> **Verification:** `cargo test` for OpenAPI spec generation passes.
> **Requirement Coverage:** NF6, R10
> **Scenario Coverage:** N/A

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** Preserve existing + extend
- **Simplification Focus:** N/A
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (TDD-only task)
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway` passes
- [x] Update `utoipa` schemas for new types
- [x] Add virtual key endpoints to OpenAPI
- [x] Update existing endpoint schemas
- [x] Verification: `cargo test -p hub-gateway`

### Task 5.2: BDD Feature File Implementation

> **Context:** Implement step definitions for Gherkin scenarios.
> **Verification:** `cargo test -p hub-gateway --test bdd` passes.
> **Requirement Coverage:** R4–R8
> **Scenario Coverage:** byok-virtual-keys, gateway-liter-llm

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** New behavior
- **Simplification Focus:** N/A
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] All scenarios in `byok-virtual-keys.feature` pass
  - [x] All scenarios in `gateway-liter-llm.feature` pass
- **Advanced Test Verification:**
  - [x] N/A (no advanced tests)
- **Runtime Verification:**
  - [x] `cargo test -p hub-gateway --test bdd` passes
- [x] Create `crates/hub-gateway/tests/bdd/` with cucumber-rs runner
- [x] Implement step definitions for virtual key auth scenarios
- [x] Implement step definitions for rate limiting scenarios
- [x] Implement step definitions for budget enforcement scenarios
- [x] Implement step definitions for BYOK passthrough scenarios
- [x] Verification: `cargo test -p hub-gateway --test bdd`

### Task 5.3: End-to-End Verification

> **Context:** Full integration verification.
> **Verification:** All tests pass. Server starts. Chat completion works with virtual key.
> **Requirement Coverage:** All
> **Scenario Coverage:** All

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** All behaviors verified
- **Simplification Focus:** N/A
- **Advanced Test Coverage:** N/A
- **Status:** 🟢 DONE
- **BDD Verification:**
  - [x] N/A (verification task)
- **Advanced Test Verification:**
  - [x] N/A (verification task)
- **Runtime Verification:**
  - [x] All commands pass, runtime probes succeed
- [x] `just format`
- [x] `just lint`
- [x] `just test`
- [x] `just test-all`
- [x] Manual: `cargo run -p hub` → `curl /health` → `curl /v1/chat/completions` with virtual key
- [x] Verify OTEL spans are emitted
- [x] Verify rate limit headers in response
- [x] Verification: All commands pass, runtime probes succeed

## Summary & Timeline

| Phase | Tasks | Estimated Effort |
|---|---|---|
| Phase 1: Workspace + Deps | 4 tasks | Large |
| Phase 2: liter-llm Integration | 4 tasks | Large |
| Phase 3: Virtual Keys + BYOK | 5 tasks | Large |
| Phase 4: Management API | 3 tasks | Medium |
| Phase 5: Polish | 3 tasks | Small |

## Definition of Done

- All `cargo check`, `cargo test`, `cargo clippy` pass
- No forbidden dependencies (`reqwest`, `anyhow`, `log`, `dashmap`)
- liter-llm handles all provider communication
- Virtual key auth works end-to-end
- Rate limiting and budget enforcement functional
- BYOK passthrough works
- Management API CRUD for virtual keys
- All existing tests updated and passing
- BDD scenarios implemented and passing
