use std::process::Command;

#[test]
fn ops_binary_runs() {
    let output = Command::new(env!("CARGO_BIN_EXE_ops"))
        .output()
        .expect("ops binary should run");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ops bootstrap placeholder"));
}
