use std::process::Command;

#[test]
fn binary_reports_scaffold_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_strokelet"))
        .output()
        .expect("strokelet binary should start");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "strokelet: scaffold only\n"
    );
    assert!(output.stderr.is_empty());
}
