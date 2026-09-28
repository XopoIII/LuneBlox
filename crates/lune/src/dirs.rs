use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::BaseDirs;

const CACHE_ENV_VAR: &str = "LUNEBLOX_CACHE";
const STATE_ENV_VAR: &str = "LUNEBLOX_STATE";

const XDG_DIR_NAME: &str = "luneblox";
const HOME_DIR_NAME: &str = ".luneblox";

pub const HISTORY_FILE_NAME: &str = ".luneblox_history";

/**
    Get the cache directory for Lune, respecting `XDG_CACHE_HOME`

    Implements backwards compatibility by preferring existing legacy
    directories over new XDG locations to minimize user friction.

    # Errors

    Returns an error if the system's base directories cannot be determined.
*/
pub fn cache_dir() -> Result<PathBuf> {
    if let Some(custom) = std::env::var_os(CACHE_ENV_VAR) {
        return Ok(PathBuf::from(custom));
    }
    let dirs = BaseDirs::new().context("Unable to find cache directory")?;
    Ok(resolve_cache_dir(dirs.home_dir(), dirs.cache_dir()))
}

/**
    Get the state directory for Lune, respecting `XDG_STATE_HOME`

    Implements backwards compatibility by preferring existing legacy
    directories over new XDG locations to minimize user friction.

    # Errors

    Returns an error if the system's base directories cannot be determined.
*/
pub fn state_dir() -> Result<PathBuf> {
    if let Some(custom) = std::env::var_os(STATE_ENV_VAR) {
        return Ok(PathBuf::from(custom));
    }
    let dirs = BaseDirs::new().context("Unable to find base directories")?;
    let base = dirs.state_dir().unwrap_or(dirs.cache_dir());
    Ok(resolve_state_dir(dirs.home_dir(), base))
}

/**
    Get the typedefs directory (always home-based for LSP compatibility)

    # Errors

    Returns an error if the home directory cannot be determined.
*/
pub fn typedefs_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().context("Unable to find home directory")?;
    Ok(dirs.home_dir().join(HOME_DIR_NAME).join(".typedefs"))
}

fn resolve_cache_dir(home: &Path, cache: &Path) -> PathBuf {
    let xdg_cache = cache.join(XDG_DIR_NAME);
    let legacy_cache = home.join(HOME_DIR_NAME);

    if legacy_cache.join("target").exists() && !xdg_cache.join("target").exists() {
        legacy_cache
    } else {
        xdg_cache
    }
}

fn resolve_state_dir(home: &Path, state: &Path) -> PathBuf {
    let xdg_state = state.join(XDG_DIR_NAME);

    if home.join(HISTORY_FILE_NAME).exists() && !xdg_state.join(HISTORY_FILE_NAME).exists() {
        home.to_path_buf()
    } else {
        xdg_state
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    struct Fixture {
        _root: TempDir,
        home: PathBuf,
        base: PathBuf,
    }

    fn fixture() -> Fixture {
        let root = TempDir::new().unwrap();
        let home = root.path().join("home");
        let base = root.path().join("base");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&base).unwrap();
        Fixture {
            _root: root,
            home,
            base,
        }
    }

    #[test]
    fn cache_defaults_to_xdg() {
        let f = fixture();
        let result = resolve_cache_dir(&f.home, &f.base);
        assert_eq!(result, f.base.join(XDG_DIR_NAME));
    }

    #[test]
    fn cache_uses_existing_legacy_dir() {
        let f = fixture();
        fs::create_dir_all(f.home.join(HOME_DIR_NAME).join("target")).unwrap();
        let result = resolve_cache_dir(&f.home, &f.base);
        assert_eq!(result, f.home.join(HOME_DIR_NAME));
    }

    #[test]
    fn cache_prefers_xdg_when_both_exist() {
        let f = fixture();
        fs::create_dir_all(f.home.join(HOME_DIR_NAME).join("target")).unwrap();
        fs::create_dir_all(f.base.join(XDG_DIR_NAME).join("target")).unwrap();
        let result = resolve_cache_dir(&f.home, &f.base);
        assert_eq!(result, f.base.join(XDG_DIR_NAME));
    }

    #[test]
    fn state_defaults_to_xdg() {
        let f = fixture();
        let result = resolve_state_dir(&f.home, &f.base);
        assert_eq!(result, f.base.join(XDG_DIR_NAME));
    }

    #[test]
    fn state_uses_existing_legacy_history() {
        let f = fixture();
        fs::write(f.home.join(HISTORY_FILE_NAME), "").unwrap();
        let result = resolve_state_dir(&f.home, &f.base);
        assert_eq!(result, f.home);
    }

    #[test]
    fn state_prefers_xdg_when_both_exist() {
        let f = fixture();
        fs::write(f.home.join(HISTORY_FILE_NAME), "").unwrap();
        fs::create_dir_all(f.base.join(XDG_DIR_NAME)).unwrap();
        fs::write(f.base.join(XDG_DIR_NAME).join(HISTORY_FILE_NAME), "").unwrap();
        let result = resolve_state_dir(&f.home, &f.base);
        assert_eq!(result, f.base.join(XDG_DIR_NAME));
    }

    #[test]
    fn typedefs_dir_is_home_based() {
        let path = typedefs_dir().unwrap();
        assert!(path.ends_with(Path::new(HOME_DIR_NAME).join(".typedefs")));
    }
}
