use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub app_name: String,
    pub workspace_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    pub provider_name: String,
    pub default_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleConfig {
    pub job_name: String,
    pub cron: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GmailAiNewsConfig {
    pub app_name: String,
    pub workspace_root: PathBuf,
    pub done_dir: PathBuf,
    pub sources: Vec<GmailSourceConfig>,
    pub provider: String,
    pub model: String,
    pub oauth_client_secret_file: PathBuf,
    pub oauth_token_cache_file: PathBuf,
    pub gmail_user_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct GmailSourceConfig {
    pub name: String,
    pub gmail_query: String,
    pub prompt_file: PathBuf,
}

impl GmailAiNewsConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse config file {}", path.display()))
    }

    pub fn load_prompt(&self, source: &GmailSourceConfig) -> Result<String> {
        fs::read_to_string(&source.prompt_file).with_context(|| {
            format!(
                "failed to read prompt file {}",
                source.prompt_file.as_path().display()
            )
        })
    }

    pub fn resolve_relative_paths(&mut self, workspace_root: &Path) {
        self.workspace_root = absolutize_path(workspace_root, &self.workspace_root);
        self.done_dir = absolutize_path(workspace_root, &self.done_dir);
        for source in &mut self.sources {
            source.prompt_file = absolutize_path(workspace_root, &source.prompt_file);
        }
        self.oauth_client_secret_file = absolutize_path(workspace_root, &self.oauth_client_secret_file);
        self.oauth_token_cache_file = absolutize_path(workspace_root, &self.oauth_token_cache_file);
    }
}

pub fn load_shared_env_file(path: impl AsRef<Path>) -> Result<bool> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(false);
    }

    dotenvy::from_path(path)
        .with_context(|| format!("failed to load env file {}", path.display()))?;
    Ok(true)
}

pub fn find_workspace_root(start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current = start.as_ref();
    if current.is_file() {
        current = current.parent()?;
    }

    for dir in current.ancestors() {
        if dir.join("Cargo.toml").exists() {
            return Some(dir.to_path_buf());
        }
    }

    None
}

fn absolutize_path(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}
