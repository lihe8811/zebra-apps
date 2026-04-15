# zebra-apps

Rust-first workspace for scheduled LLM-powered apps running on one Mac.

## Layout

- `crates/core`: shared document, config, provider, workspace, and run-log primitives
- `crates/apps/capture`: bootstrap binary for ingesting source material
- `crates/apps/gmail_ai_news`: Gmail inbox summarizer for AI-news newsletters
- `crates/apps/summarize`: bootstrap binary for processing inbox docs
- `crates/apps/review`: bootstrap binary for review and escalation flows
- `crates/ops`: bootstrap operational tool for config validation and future `launchd` generation
- `workspace/`: shared markdown exchange folders
- `config/`: example app, provider, and schedule configuration
- `docs/plans/`: project planning documents

## Commands

```bash
cargo build
cargo test
cargo run -p capture
cargo run -p gmail_ai_news
cargo run -p summarize
cargo run -p review
cargo run -p ops
```

## Runtime Workspace

The shared markdown pipeline uses these folders:

- `workspace/inbox`
- `workspace/processing`
- `workspace/done`
- `workspace/review`
- `workspace/archive`
- `workspace/logs`

The intended document front matter fields are:

- `id`
- `type`
- `status`
- `source_app`
- `target_app`
- `created_at`
- `updated_at`
- `model`
- `run_id`

`launchd` generation is planned for a later iteration; the current bootstrap only scaffolds the repo structure and placeholder binaries.

## Gmail AI News App

The first implemented app is `gmail_ai_news`.

- Config: `config/apps/gmail_ai_news.toml`
- Prompt drafts:
  - `config/prompts/gmail_ai_news_summary.md`
  - `config/prompts/gmail_podcast_transcript.md`
- OAuth secret example: `config/gmail/oauth_client_secret.example.json`
- Output folder: `workspace/done/gmail_ai_news`

Before running it, create `config/gmail/oauth_client_secret.json` from the example, then set `OPENAI_API_KEY` for `async-openai`.
The app loads a shared repo `.env` file automatically, so you can put `OPENAI_API_KEY=...` in `/Users/lihe8811/Documents/Code/AI/zebra-apps/.env`.
