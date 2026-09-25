use std::process::Command;

#[test]
fn binary_reports_demo_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_strokelet"))
        .arg("status")
        .output()
        .expect("strokelet binary should start");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "strokelet: demo\n");
    assert!(output.stderr.is_empty());
}
