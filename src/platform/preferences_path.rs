use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

use napi_derive_ohos::napi;
use napi_ohos::{Error, Result};

static PREFERENCES_PATH: LazyLock<RwLock<Option<PathBuf>>> = LazyLock::new(|| RwLock::new(None));

#[napi]
pub fn set_preferences_path(preferences_path: String) -> Result<()> {
    let trimmed = preferences_path.trim();
    if trimmed.is_empty() {
        return Err(Error::from_reason("preferences_path is empty"));
    }

    let mut guard = PREFERENCES_PATH
        .write()
        .map_err(|_| Error::from_reason("failed to lock PREFERENCES_PATH"))?;
    guard.replace(PathBuf::from(trimmed));
    Ok(())
}

pub fn get_preferences_path() -> PathBuf {
    PREFERENCES_PATH
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or_else(std::env::temp_dir)
}

pub fn get_preferences_file_path(file_name: &str) -> PathBuf {
    get_preferences_path().join(file_name)
}
