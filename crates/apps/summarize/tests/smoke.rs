use std::process::Command;

#[test]
fn summarize_binary_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_summarize"))
        .output()
        .expect("summarize binary should run");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("summarize app bootstrap placeholder"));
}
