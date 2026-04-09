use std::process::Command;

#[test]
fn gmail_ai_news_help_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_gmail_ai_news"))
        .arg("--help")
        .output()
        .expect("gmail_ai_news binary should run");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gmail_ai_news"));
}
