# AiHub - High-Performance LLM Gateway

<p align="center">
  <p align="center">Open-source, high-performance LLM gateway written in Rust. Connect to any LLM provider with a single API. Observability Included.</p>
</p>

## Features

- **Multi-Provider Support**: 143+ LLM providers via liter-llm integration
- **OpenAI Compatible API**: Drop-in replacement for OpenAI API calls
- **Two Deployment Modes**:
  - **YAML Mode**: Simple static configuration with config files
  - **Database Mode**: Dynamic configuration with PostgreSQL and Management API
- **Built-in Observability**: OpenTelemetry tracing and Prometheus metrics
- **High Performance**: Written in Rust with async/await support
- **Hot Reload**: Dynamic configuration updates (Database mode)
- **Pipeline System**: Extensible request/response processing
- **Workspace Architecture**: Modular design with `hub-core`, `hub-gateway`, `hub-management`

## Quick Start

### Using Cargo

```bash
# Clone and build
git clone https://github.com/longcipher/aihub.git
cd aihub
cargo build --release

# YAML Mode
./target/release/hub

# Database Mode  
HUB_MODE=database DATABASE_URL=postgresql://user:pass@host:5432/db ./target/release/hub
```

## Architecture

The project uses a Cargo workspace architecture:

```
aihub/
├── bin/hub/                    # Binary crate
│   └── src/main.rs             # Application entry point
├── crates/
│   ├── hub-core/               # Core library
│   │   ├── config/             # Configuration management
│   │   ├── provider/           # LLM provider adapter
│   │   ├── state.rs            # Application state management
│   │   └── types/              # Shared type definitions
│   ├── hub-gateway/            # Gateway library
│   │   ├── routes.rs           # HTTP routing
│   │   └── pipeline/           # Request processing pipelines
│   └── hub-management/         # Management API library
│       ├── api/                # REST API endpoints
│       ├── db/                 # Database models and repositories
│       ├── services/           # Business logic
│       └── dto.rs              # Data transfer objects
├── migrations/                 # Database migrations
├── tests/                      # Integration tests
└── config.yaml                 # Example configuration
```

## Configuration Modes

### YAML Mode

Perfect for simple deployments and development environments.

**Features:**

- Static configuration via `config.yaml`
- No external dependencies
- Simple provider and model setup
- No management API
- Single port (3000)

**Example config.yaml:**

```yaml
providers:
  - key: openai
    type: openai
    api_key: sk-...

models:
  - key: gpt-4
    type: gpt-4
    provider: openai

pipelines:
  - name: chat
    type: Chat
    plugins:
      - ModelRouter:
          models: [gpt-4]
```

### Database Mode

Ideal for production environments requiring dynamic configuration.

**Features:**

- PostgreSQL-backed configuration
- REST Management API (`/api/v1/management/*`)
- Hot reload without restarts
- Configuration polling and synchronization
- Dual ports (3000 for Gateway, 8080 for Management)

**Setup:**

1. Set up PostgreSQL database
2. Run migrations: `sqlx migrate run`
3. Set environment variables:

   ```bash
   HUB_MODE=database
   DATABASE_URL=postgresql://user:pass@host:5432/db
   ```

## API Endpoints

### Core LLM Gateway (Both Modes)

**Port 3000:**

- `POST /api/v1/chat/completions` - Chat completions
- `POST /api/v1/completions` - Text completions  
- `POST /api/v1/embeddings` - Text embeddings
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /swagger-ui` - OpenAPI documentation

### Management API (Database Mode Only)

**Port 8080:**

- `GET /health` - Management API health check
- `GET|POST|PUT|DELETE /api/v1/management/providers` - Provider management
- `GET|POST|PUT|DELETE /api/v1/management/model-definitions` - Model management
- `GET|POST|PUT|DELETE /api/v1/management/pipelines` - Pipeline management

## Provider Configuration

### OpenAI

```yaml
providers:
  - key: openai
    type: openai
    api_key: sk-...
    # Optional
    organization_id: org-...
    base_url: https://api.openai.com/v1
```

### Anthropic

```yaml
providers:
  - key: anthropic
    type: anthropic
    api_key: sk-ant-...
```

### Azure OpenAI

```yaml
providers:
  - key: azure
    type: azure
    api_key: your-key
    resource_name: your-resource
    api_version: "2023-05-15"
```

### AWS Bedrock

```yaml
providers:
  - key: bedrock
    type: bedrock
    region: us-east-1
    # Uses IAM roles or AWS credentials
```

### Google VertexAI

Supports two authentication modes that route to different Google APIs:

```yaml
# Option 1: API Key (uses Gemini Developer API)
providers:
  - key: vertexai
    type: vertexai
    api_key: your-gemini-api-key

# Option 2: Service Account (uses Vertex AI)
providers:
  - key: vertexai
    type: vertexai
    project_id: your-project
    location: us-central1
    credentials_path: /path/to/service-account.json
```

| Auth Method | API Endpoint | Use Case |
|-------------|--------------|----------|
| API Key | `generativelanguage.googleapis.com` | Simple setup, development |
| Service Account | `{location}-aiplatform.googleapis.com` | Enterprise, GCP-integrated |

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HUB_MODE` | Deployment mode: `yaml` or `database` | `yaml` | No |
| `CONFIG_FILE_PATH` | Path to YAML config file | `config.yaml` | YAML mode |
| `DATABASE_URL` | PostgreSQL connection string | - | Database mode |
| `DB_POLL_INTERVAL_SECONDS` | Config polling interval | `30` | No |
| `PORT` | Gateway server port | `3000` | No |
| `MANAGEMENT_PORT` | Management API port | `8080` | Database mode |

## Development

### Prerequisites

- Rust 1.88+
- PostgreSQL 12+ (for database mode)
- `sqlx-cli` (for migrations)

### Commands

```bash
# Build
cargo build

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy

# Run YAML mode
cargo run

# Run database mode
HUB_MODE=database DATABASE_URL=postgresql://... cargo run
```

### Database Setup (for Database Mode)

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

## Observability

### OpenTelemetry Tracing

Configure OTLP endpoint for distributed tracing:

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

### Prometheus Metrics

Available at `/metrics`:

- Request counts and latencies
- Provider-specific metrics
- Error rates
- Active connections

### OpenObserve Integration

For metrics monitoring, use OpenObserve:

1. Deploy OpenObserve
2. Configure Prometheus remote write to OpenObserve
3. Query metrics in OpenObserve dashboard

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client App    │───▶│     AiHub        │───▶│   LLM Provider  │
└─────────────────┘    │                  │    │  (OpenAI, etc.) │
                       │  ┌─────────────┐ │    └─────────────────┘
                       │  │ Config Mode │ │
                       │  │ YAML | DB   │ │    ┌─────────────────┐
                       │  └─────────────┘ │───▶│   Observability │
                       │                  │    │ (OTel, Metrics) │
                       │  ┌─────────────┐ │    └─────────────────┘
                       │  │ Management  │ │
                       │  │ API (DB)    │ │
                       │  └─────────────┘ │
                       └──────────────────┘
```

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! Please see our Contributing Guide for details.
