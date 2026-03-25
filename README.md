# zebra-apps

Rust-first workspace for scheduled LLM-powered apps running on one Mac.

## Layout

- `crates/core`: shared document, config, provider, workspace, and run-log primitives
- `crates/apps/capture`: bootstrap binary for ingesting source material
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
