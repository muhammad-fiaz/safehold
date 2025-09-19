use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn bin() -> Command {
    Command::cargo_bin("safehold").unwrap()
}

#[test]
fn setup_add_path_dry_run_and_launch_without_gui() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let home = tmp.path().to_string_lossy().into_owned();

    // setup with add-path in dry-run mode (do not mutate host PATH)
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home)
        .env("SAFEHOLD_PATH_DRY_RUN", "1")
        .args(["setup", "--add-path"]);
    cmd.assert().success().stdout(predicate::str::contains(
        "PATH update (dry run) would be applied",
    ));

    // launch --gui should print a helpful hint when GUI feature is not compiled
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["launch", "--gui"]);
    // Depending on build flags, either the reinstall hint (stderr) or generic info (stdout) is printed.
    let assert = cmd.assert().success();
    let hint = predicate::str::contains("GUI is not installed");
    let generic = predicate::str::contains("Use --gui to launch");
    // Accept either stderr hint or stdout generic info
    let out = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let err = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(hint.eval(&err) || generic.eval(&out));
}
