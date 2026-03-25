# Rust LLM Apps Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bootstrap a Rust-first macOS automation workspace with three scheduled LLM apps sharing a markdown document pipeline.

**Architecture:** A Cargo workspace hosts a shared `core` library plus app binaries for `capture`, `summarize`, `review`, and an `ops` tool. Apps communicate through a filesystem workspace of markdown documents with structured front matter, and `launchd` support will be added through the ops crate.

**Tech Stack:** Rust 2024 edition, Cargo workspace, markdown files with front matter, macOS `launchd`, API-based LLM providers.

## Workspace Layout

- `crates/core`: shared document model, workspace transitions, provider abstraction, config loading, run logging
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
