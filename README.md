# LLM Pulse CLI

`llmpulse` is the native Rust command-line client for the [LLM Pulse API](https://llmpulse.ai/api-docs). It queries visibility metrics, mentions, citations, sentiment, AI traffic, Search Console, recommendations, GEO Writer tasks, and webhooks from a terminal or CI job.

## Install

Download the archive for your platform from [GitHub Releases](https://github.com/LLM-Pulse/llmpulse-cli/releases). Extract it, then move `llmpulse` or `llmpulse.exe` to a directory on your `PATH`.

To install from source with Cargo:

```bash
cargo install --git https://github.com/LLM-Pulse/llmpulse-cli --tag v2.0.0 --locked
```

You can also run the container locally:

```bash
docker build -t llmpulse-cli .
docker run --rm -e LLMPULSE_API_KEY llmpulse-cli ping
```

Each release includes archives for macOS on Intel and Apple silicon, Linux on x86-64, and Windows on x86-64. Registry and Homebrew publishing remain separate release steps.

## Set up the CLI

Get an API key from the [LLM Pulse dashboard](https://llmpulse.ai/app/api_keys), then run:

```bash
llmpulse login
llmpulse status
```

`login` verifies the key, asks you to choose a default project, and writes `~/.llmpulse/config.json`. Existing configuration and profiles from the TypeScript CLI continue to work.

## Common commands

```bash
# Visibility summary for the default project
llmpulse metrics summary --range 28

# Every citation in CSV format
llmpulse dimensions citations --all --format csv > citations.csv

# AI referral traffic from connected analytics
llmpulse metrics ai-traffic --range 30

# Top Google Search Console queries
llmpulse search-console queries --range 30 --per-page 50

# Launch and inspect recommendations
llmpulse recommendations launch --recommendation-type ai_visibility
llmpulse recommendations list

# Subscribe a webhook endpoint
llmpulse webhooks create \
  --event-type mention.created \
  --target-url https://example.com/webhooks/llmpulse
```

## Command groups

| Command | Purpose |
| --- | --- |
| `status`, `diff`, `report`, `watch` | Project summaries and terminal reports |
| `dimensions` | Projects, competitors, prompts, executions, sources, mentions, citations, locales, models, and bots |
| `metrics` | Timeseries, summary, prompt performance, share of voice, top sources, AI crawler traffic, and AI referral traffic |
| `search-console` | Search Console summary, timeseries, queries, and pages |
| `citations` | Grouped citation intelligence, citing-domain mention share, URL details, occurrences, and cached content |
| `answers`, `sentiments` | AI responses and sentiment records |
| `recommendations` | Recommendation runs and generation |
| `intelligence-tasks` | GEO Writer tasks |
| `projects`, `prompts`, `competitors`, `collections` | Project setup and resource management |
| `annotations` | Timeline annotation management |
| `reports` | Technical GEO report creation |
| `webhooks` | Subscriptions and sample payloads |
| `export` | Full JSON and CSV exports |
| `api` | Direct access to any REST path |

Run `llmpulse <command> --help` for command-specific flags.

## Output

Commands accept `--format json`, `--format table`, or `--format csv`. List commands also accept `--page`, `--per-page`, and `--all`.

```bash
llmpulse metrics summary --format json | jq '.summary'
llmpulse dimensions mentions --format table
llmpulse answers list --all --format csv > answers.csv
```

CSV output prefixes cells that begin with spreadsheet formula characters. This prevents exported API text from being evaluated as a formula when opened in spreadsheet software.

## Configuration

Configuration lives in `~/.llmpulse/config.json`. Profile files live in `~/.llmpulse/profiles/`.

```bash
llmpulse config set api_key llmpulse_xxx
llmpulse config set default_project_id 123
llmpulse config use production
llmpulse config show
```

Resolution order is command flag, environment variable, active profile, base configuration, then the default value.

| Setting | Environment variable | Default |
| --- | --- | --- |
| `api_key` | `LLMPULSE_API_KEY` | None |
| `base_url` | `LLMPULSE_BASE_URL` | `https://api.llmpulse.ai/api/v1` |
| `default_project_id` | `LLMPULSE_PROJECT_ID` | None |

The CLI accepts HTTPS API URLs. Plain HTTP is limited to localhost for local testing so an API key cannot be sent over an unencrypted remote connection.

## Direct REST calls

The typed commands cover all REST operations in the OpenAPI document. The `api` command is useful for new parameters or automation that needs the response envelope unchanged.

```bash
llmpulse api get /metrics/summary \
  --query project_id=123 \
  --query range=28

llmpulse api post /recommendations \
  --body '{"project_id":123,"recommendation_type":"ai_visibility"}'
```

Paths must be relative to the configured API base URL.

## Shell completions

```bash
llmpulse completions bash > ~/.local/share/bash-completion/completions/llmpulse
llmpulse completions zsh > ~/.zfunc/_llmpulse
llmpulse completions fish > ~/.config/fish/completions/llmpulse.fish
```

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --locked
```

The API parity test reads the bundled `openapi.json` snapshot and fails when a REST operation is missing from the Rust manifest. In the LLM Pulse monorepo, it also checks that the snapshot matches `public/openapi.json`. MCP and OAuth endpoints are protocol surfaces and are intentionally excluded from CLI parity.

## License

MIT. See [LICENSE](LICENSE).
