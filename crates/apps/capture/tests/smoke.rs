use std::process::Command;

#[test]
fn capture_binary_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_capture"))
        .output()
        .expect("capture binary should run");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("capture app bootstrap placeholder"));
}
