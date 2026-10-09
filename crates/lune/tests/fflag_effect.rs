//! A fast flag of the Roblox client changes what the built `luneblox` binary does with a script.
//!
//! The table's own tests prove that every flag is taken. This one proves that taking them reaches
//! the compiler, on the one flag in the table whose effect a script can print.

#![allow(clippy::cargo_common_metadata)]

use std::fs;
use std::process::{Command, Output};

/// `LuauCompileNoFoldVectorEqW`: two vector constants that differ only in a fourth component.
/// The VM keeps three components, so at run time they are equal. At optimization level 2 the
/// compiler folds the comparison, and without the flag it compares all four and folds to `false`.
const SCRIPT: &str = "--!optimize 2\n\
    print(vector.create(1, 2, 3, 4) == vector.create(1, 2, 3, 5))\n\
    print(vector.create(1, 2, 3, 4) ~= vector.create(1, 2, 3, 5))\n\
    print(vector.create(1, 2, 3, 4) == vector.create(1, 2, 4, 4))\n";

fn run_script(fflags: Option<&str>) -> Output {
    let dir = tempfile::tempdir().expect("failed to create a temporary directory");
    let script = dir.path().join("script.luau");
    fs::write(&script, SCRIPT).expect("failed to write the script");

    let mut command = Command::new(env!("CARGO_BIN_EXE_luneblox"));
    command.arg("run").arg(&script);
    command.env_remove("LUNE_ROBLOX_FFLAGS");
    if let Some(value) = fflags {
        command.env("LUNE_ROBLOX_FFLAGS", value);
    }
    command.output().expect("failed to run luneblox")
}

fn assert_stdout(output: &Output, stdout: &str) {
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn vectors_that_differ_only_in_w_compare_as_the_vm_compares_them() {
    assert_stdout(&run_script(None), "true\nfalse\nfalse\n");
}

/// The other half: with Luau's defaults the same script prints the folded answer. When Luau
/// retires the flag this fails, and the test above is then all that is left to keep.
#[test]
fn luau_defaults_still_fold_the_comparison_on_all_four_components() {
    assert_stdout(&run_script(Some("0")), "false\ntrue\nfalse\n");
}
