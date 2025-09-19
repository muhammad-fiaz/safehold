use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn bin() -> Command { Command::cargo_bin("safehold").unwrap() }

#[test]
fn create_unlocked_and_add_get() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let home = tmp.path().to_string_lossy().into_owned();

    // create set
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).arg("create").arg("project1");
    cmd.assert().success().stdout(predicate::str::contains("Created project"));

    // add key
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["add", "-p", "project1", "-k", "API_KEY", "-v", "abc123"]);
    cmd.assert().success().stdout(predicate::str::contains("Added"));

    // get key
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["get", "-p", "project1", "-k", "API_KEY"]);
    cmd.assert().success().stdout("abc123\n");

    // list keys
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["list", "-p", "project1"]);
    cmd.assert().success().stdout(predicate::str::contains("API_KEY=abc123"));
}

#[test]
fn create_locked_and_add_get_with_env_password() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let home = tmp.path().to_string_lossy().into_owned();

    // create locked set with password flag
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["create", "project2", "--lock", "--password", "p@ssw0rd"]);
    cmd.assert().success();

    // use SAFEHOLD_PASSWORD to avoid prompt
    let password = "p@ssw0rd";

    // add
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).env("SAFEHOLD_PASSWORD", password).args(["add", "-p", "project2", "-k", "TOKEN", "-v", "xyz"]);
    cmd.assert().success();

    // get
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).env("SAFEHOLD_PASSWORD", password).args(["get", "-p", "project2", "-k", "TOKEN"]);
    cmd.assert().success().stdout("xyz\n");
}
