# Rust LLM Apps Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bootstrap a Rust-first macOS automation workspace with scheduled LLM apps sharing a markdown document pipeline, starting with a Gmail AI-news summarizer.

**Architecture:** A Cargo workspace hosts a shared `core` library plus app binaries and an `ops` tool. Apps communicate through a filesystem workspace of markdown documents with structured front matter, common integrations such as Gmail live in shared Rust modules, and `launchd` support will be added through the ops crate.

**Tech Stack:** Rust 2024 edition, Cargo workspace, markdown files with front matter, macOS `launchd`, Gmail API with local OAuth, `async-openai` for LLM calls.

## Workspace Layout

- `crates/core`: shared document model, workspace transitions, provider abstraction, Gmail integration utilities, config loading, run logging
- `crates/apps/capture`: ingestion app that writes normalized markdown into `workspace/inbox`
- `crates/apps/summarize`: processing app that claims inbox docs and generates summaries
- `crates/apps/review`: review app that inspects failures or low-confidence output
- `crates/ops`: operational tooling for config validation and future `launchd` plist generation
- `workspace/`: runtime folders for file-based coordination
- `config/`: app, provider, and schedule configuration examples

## Shared Markdown Contract

Each document uses front matter with these fields:

- `id`
- `type`
- `status`
- `source_app`
- `target_app`
- `created_at`
- `updated_at`
- `model`
- `run_id`

The shared lifecycle folders are:

- `workspace/inbox`
- `workspace/processing`
- `workspace/done`
- `workspace/review`
- `workspace/archive`
- `workspace/logs`

Apps must claim a file before processing it and record state transitions atomically.
Each app should write completed artifacts into its own subfolder under `workspace/done`, for example `workspace/done/gmail_ai_news`.

## App Responsibilities

### `capture`

- normalize incoming content into the shared markdown schema
- create valid documents in `workspace/inbox`

### `summarize`

- claim eligible inbox documents
- call the configured LLM provider
- write summary output and route the document to `done` or `review`

### `review`

- inspect failed or ambiguous items
- generate markdown review notes and proposed next actions

## First App: Gmail AI News Summarizer

- add a dedicated app, tentatively named `gmail_ai_news`, that reads Gmail inbox messages from `swyx+ainews@substack.com`
- extend the same app to also process podcast transcript emails from `swyx@substack.com`
- use local OAuth on this Mac for Gmail API access
- search only current inbox messages from configured senders
- fetch the full MIME message body rather than relying on clipped email snippets
- summarize the full content with `async-openai`
- load sender-specific prompts from draft files in `config/` so they can be edited later without code changes
- write one markdown output per processed email into `workspace/done/gmail_ai_news`
- archive each message in Gmail after the summary file is written successfully

## Shared Gmail Utilities

The shared Rust layer should expose reusable Gmail helpers so later Gmail tasks do not duplicate access logic:

- OAuth desktop-flow configuration and token persistence
- Gmail search query construction
- full-message fetch and MIME/body extraction
- message metadata normalization for markdown output
- archive and label mutation helpers
- idempotency helpers so future Gmail tasks can avoid reprocessing the same message

## `launchd` Approach

- define schedules in config with cron-like fields
- have the `ops` crate validate schedule files and later emit `launchd` plists
- install one job per app with predictable log and working-directory settings
- keep this bootstrap limited to placeholders rather than full plist generation

## Test Strategy

- core unit tests for front matter validation and workspace transitions
- smoke tests for each app binary so the scaffold stays runnable
- ops smoke test to keep the operational entrypoint available
- workspace bootstrapping checks for required folders and example configs
