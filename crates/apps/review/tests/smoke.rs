use std::process::Command;

#[test]
fn review_binary_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_review"))
        .output()
        .expect("review binary should run");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("review app bootstrap placeholder"));
}
