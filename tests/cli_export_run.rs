use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn bin() -> Command { Command::cargo_bin("safehold").unwrap() }

#[test]
fn export_and_run_with_global_merge() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let home = tmp.path().to_string_lossy().into_owned();

    // create set
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["create", "proj"]);
    cmd.assert().success();

    // add to global and set
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["add", "-s", "global", "-k", "SHARED", "-v", "gval"]);
    cmd.assert().success();

    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["add", "-s", "proj", "-k", "LOCAL", "-v", "lval"]);
    cmd.assert().success();

    // export set to file
    let file = tmp.child(".env");
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["export", "-s", "proj", "--file", file.path().to_str().unwrap()]);
    cmd.assert().success().stdout(predicate::str::contains(".env written"));
    file.assert(predicate::str::contains("LOCAL=lval"));

    // run with global merge; print env var to stdout
    #[cfg(windows)]
    let echo_cmd = vec!["cmd".to_string(), "/C".to_string(), "set LOCAL".to_string()];
    #[cfg(not(windows))]
    let echo_cmd = vec!["/usr/bin/env".to_string(), "sh".to_string(), "-c".to_string(), "printf %s $LOCAL".to_string()];

    let mut cmd = bin();
    let mut args = vec!["run".to_string(), "-s".to_string(), "proj".to_string(), "--with-global".to_string(), "--".to_string()];
    args.extend(echo_cmd);
    cmd.env("SAFEHOLD_HOME", &home).args(args);
    #[cfg(windows)]
    cmd.assert().success().stdout(predicate::str::contains("LOCAL=lval"));
    #[cfg(not(windows))]
    cmd.assert().success().stdout("lval");
}
