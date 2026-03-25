use zebra_core::config::AppConfig;

fn main() {
    let app = AppConfig {
        app_name: "capture".to_string(),
        workspace_root: "workspace".to_string(),
    };

    println!(
        "{} app bootstrap placeholder. Workspace root: {}",
        app.app_name, app.workspace_root
    );
}
