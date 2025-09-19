use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::cargo_bin("safehold").unwrap();
    cmd
}

#[test]
fn create_unlocked_and_add_get() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let home = tmp.path().to_string_lossy().into_owned();

    // create set
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).arg("create").arg("project1");
    cmd.assert().success().stdout(predicate::str::contains("Created set"));

    // add key
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["add", "-s", "project1", "-k", "API_KEY", "-v", "abc123"]);
    cmd.assert().success().stdout(predicate::str::contains("Added"));

    // get key
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["get", "-s", "project1", "-k", "API_KEY"]);
    cmd.assert().success().stdout("abc123\n");

    // list keys
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).args(["list", "-s", "project1"]);
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
    cmd.env("SAFEHOLD_HOME", &home).env("SAFEHOLD_PASSWORD", password).args(["add", "-s", "project2", "-k", "TOKEN", "-v", "xyz"]);
    cmd.assert().success();

    // get
    let mut cmd = bin();
    cmd.env("SAFEHOLD_HOME", &home).env("SAFEHOLD_PASSWORD", password).args(["get", "-s", "project2", "-k", "TOKEN"]);
    cmd.assert().success().stdout("xyz\n");
}
