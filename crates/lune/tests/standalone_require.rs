//! A binary made with `luneblox build` says why it cannot require a file.
//!
//! It holds only the script it was built from, so `require("./module")` has nothing to load.
//! Luau's own message for that, "require is not supported in this context", named neither the
//! cause nor the way out.

#![allow(clippy::cargo_common_metadata)]

use std::fs;
use std::process::Command;

#[test]
fn a_standalone_binary_explains_a_file_require() {
    let dir = tempfile::tempdir().expect("failed to create a temporary directory");
    let script = dir.path().join("main.luau");
    let binary = dir.path().join("main-standalone");
    fs::write(
        &script,
        "print(require(\"@lune/process\").os ~= nil)\n\
         print(pcall(require, \"./module\"))\n",
    )
    .expect("failed to write the script");
    fs::write(dir.path().join("module.luau"), "return 1\n").expect("failed to write the module");

    let built = Command::new(env!("CARGO_BIN_EXE_luneblox"))
        .arg("build")
        .arg(&script)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("failed to run luneblox build");
    assert!(built.status.success(), "{built:?}");

    let output = Command::new(&binary)
        .current_dir(dir.path())
        .output()
        .expect("failed to run the standalone binary");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The standard libraries are registered by name and keep working
    assert!(stdout.starts_with("true\nfalse"), "{stdout}");
    assert!(
        stdout.contains("cannot require a file from 'STANDALONE'")
            && stdout.contains("bundle the modules it requires"),
        "{stdout}"
    );
    assert_eq!(output.status.code(), Some(0));
}
