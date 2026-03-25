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
