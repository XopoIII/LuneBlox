use std::sync::Once;

use mlua::prelude::*;

use super::roblox_fflags::{ROBLOX_AARCH64_FFLAGS, ROBLOX_FFLAGS};

/**
    Sets Luau's fast flags the way the live Roblox client sets them, once per process.

    Luau ships new compiler and VM work behind flags that default to off, and Roblox
    turns them on remotely, so the same Luau source runs differently here and there
    unless the flags match. `LUNE_ROBLOX_FFLAGS=0` keeps Luau's own defaults instead.

    [`Runtime::new`](super::Runtime::new) calls this; anything that compiles Luau without a runtime, such as
    building a standalone binary, should call it first.
*/
pub fn apply_roblox_fflags() {
    static APPLY: Once = Once::new();
    APPLY.call_once(|| {
        if std::env::var_os("LUNE_ROBLOX_FFLAGS").is_some_and(|value| value == "0") {
            return;
        }
        // The table is generated against the same Luau version and holds only flags the
        // runtime links, so a failure means the table has drifted from the VM in a
        // partial update, and that should not pass silently. Flags of the ARM64 code
        // generator are a table of their own, empty where that backend is not linked.
        let failed = set_fflags(ROBLOX_FFLAGS)
            .into_iter()
            .chain(set_fflags(ROBLOX_AARCH64_FFLAGS));
        for name in failed {
            eprintln!("warning: Luau fast flag `{name}` did not apply");
        }
    });
}

/**
    Sets each flag to its value and returns the names of the flags Luau did not take.

    Luau takes a flag only when the code that declares it is linked into this binary.
*/
fn set_fflags(flags: &[(&'static str, bool)]) -> Vec<&'static str> {
    flags
        .iter()
        .filter(|(name, enabled)| Lua::set_fflag(name, *enabled).is_err())
        .map(|(name, _)| *name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ROBLOX_AARCH64_FFLAGS, ROBLOX_FFLAGS, set_fflags};

    // Fails on any target where a flag in a table is declared in Luau code that the linker
    // left out of the binary; `scripts/generate_roblox_fflags.luau` lists such sources as
    // unlinked, and puts the ARM64 backend's flags in a table only aarch64 builds have. This
    // test binary links the whole runtime, as the CLI does.
    #[test]
    fn every_roblox_fflag_applies() {
        // The table is not empty: the fast flags guide documents this entry, which Roblox
        // turned off with client 0.744
        assert!(ROBLOX_FFLAGS.contains(&("DebugCodegenOptSize", false)));
        assert_eq!(set_fflags(ROBLOX_FFLAGS), Vec::<&str>::new());
        assert_eq!(set_fflags(ROBLOX_AARCH64_FFLAGS), Vec::<&str>::new());
    }

    // On aarch64 the ARM64 table holds what the generator found there; elsewhere it is
    // empty, since Luau does not register those flags.
    #[test]
    fn the_arm64_table_is_aarch64_only() {
        if cfg!(target_arch = "aarch64") {
            assert!(ROBLOX_AARCH64_FFLAGS.contains(&("LuauCodegenA64ForgLoopArray", true)));
        } else {
            assert_eq!(ROBLOX_AARCH64_FFLAGS, &[] as &[(&str, bool)]);
        }
    }

    #[test]
    fn roblox_fflags_are_sorted_and_unique() {
        for table in [ROBLOX_FFLAGS, ROBLOX_AARCH64_FFLAGS] {
            for pair in table.windows(2) {
                assert!(pair[0].0 < pair[1].0, "{} before {}", pair[0].0, pair[1].0);
            }
        }
        for (name, _) in ROBLOX_AARCH64_FFLAGS {
            assert!(
                !ROBLOX_FFLAGS.iter().any(|(other, _)| other == name),
                "{name} twice"
            );
        }
    }

    // `LuauCallFeedback` is in the table with this value, so setting it changes nothing.
    #[test]
    fn a_flag_luau_does_not_register_is_reported() {
        assert!(ROBLOX_FFLAGS.contains(&("LuauCallFeedback", true)));
        assert_eq!(
            set_fflags(&[
                ("LuauCallFeedback", true),
                ("LuneBloxNoSuchFlag", true),
                // Declared in Luau's Ast/src/PrettyPrinter.cpp, which the runtime does not link
                ("LuauPrettyPrintVisualizeIndexerAccess", true),
            ]),
            vec![
                "LuneBloxNoSuchFlag",
                "LuauPrettyPrintVisualizeIndexerAccess"
            ]
        );
    }
}
