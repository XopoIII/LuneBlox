//! `stdio.prompt` with no terminal to ask in fails at once, and says why.
//!
//! dialoguer draws its prompts on stderr. When stderr is not a terminal - a test runner, a git
//! hook, CI - it used to fail with a bare `IO error: not a terminal`, which does not name the
//! prompt at all.

#![allow(clippy::cargo_common_metadata)]

use std::fs;
use std::process::{Command, Stdio};

#[test]
fn a_prompt_without_a_terminal_names_what_it_needs() {
    let dir = tempfile::tempdir().expect("failed to create a temporary directory");
    let script = dir.path().join("script.luau");
    fs::write(
        &script,
        "local ok, err = pcall(require(\"@lune/stdio\").prompt, \"confirm\", \"Continue?\")\n\
         print(ok, err)\n",
    )
    .expect("failed to write the script");

    let output = Command::new(env!("CARGO_BIN_EXE_luneblox"))
        .arg("run")
        .arg(&script)
        .stdin(Stdio::null())
        .output()
        .expect("failed to run luneblox");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("false") && stdout.contains("stdio.prompt needs a terminal"),
        "{stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}
