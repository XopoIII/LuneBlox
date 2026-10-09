//! The built `luneblox` binary writes nothing of its own to stderr.
//!
//! Tools run scripts through `luneblox run` and treat anything on stderr as a finding, so a
//! diagnostic printed at startup - a fast flag that did not apply, say - breaks them all.

#![allow(clippy::cargo_common_metadata)]

use std::fs;
use std::process::{Command, Output};

fn run_script(source: &str, fflags: Option<&str>) -> Output {
    let dir = tempfile::tempdir().expect("failed to create a temporary directory");
    let script = dir.path().join("script.luau");
    fs::write(&script, source).expect("failed to write the script");

    let mut command = Command::new(env!("CARGO_BIN_EXE_luneblox"));
    command.arg("run").arg(&script);
    command.env_remove("LUNE_ROBLOX_FFLAGS");
    if let Some(value) = fflags {
        command.env("LUNE_ROBLOX_FFLAGS", value);
    }
    command.output().expect("failed to run luneblox")
}

fn assert_output(output: &Output, stdout: &str, stderr: &str, code: i32) {
    assert_eq!(String::from_utf8_lossy(&output.stderr), stderr);
    assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
    assert_eq!(output.status.code(), Some(code));
}

#[test]
fn a_script_that_prints_leaves_stderr_empty() {
    let output = run_script("print(\"hello\")\n", None);
    assert_output(&output, "hello\n", "", 0);
}

#[test]
fn an_empty_script_prints_nothing_at_all() {
    let output = run_script("", None);
    assert_output(&output, "", "", 0);
}

#[test]
fn luau_default_flags_leave_stderr_empty_too() {
    let output = run_script("print(\"hello\")\n", Some("0"));
    assert_output(&output, "hello\n", "", 0);
}

// The other direction: stderr is captured for real, so the tests above cannot pass by
// reading a stream nothing is written to.
#[test]
fn what_the_script_writes_to_stderr_reaches_it() {
    let output = run_script("require(\"@lune/stdio\").ewrite(\"careful\\n\")\n", None);
    assert_output(&output, "", "careful\n", 0);
}
