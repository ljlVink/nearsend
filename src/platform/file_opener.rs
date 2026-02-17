use std::path::Path;

#[cfg(target_env = "ohos")]
use std::sync::{Arc, LazyLock, RwLock};

#[cfg(target_env = "ohos")]
use napi_derive_ohos::napi;
#[cfg(target_env = "ohos")]
use napi_ohos::{
    bindgen_prelude::{Function, PromiseRaw, Unknown},
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
    Error, Result, Status,
};

#[cfg(target_env = "ohos")]
type OpenFileTsfn = ThreadsafeFunction<String, Unknown<'static>, String, Status, false>;

#[cfg(target_env = "ohos")]
static OPEN_FILE_TSFN: LazyLock<RwLock<Option<Arc<OpenFileTsfn>>>> =
    LazyLock::new(|| RwLock::new(None));

#[cfg(target_env = "ohos")]
#[napi]
pub fn register_open_file_callback(
    open_file: Function<'static, String, Unknown<'static>>,
) -> Result<()> {
    let open_file_tsfn = open_file
        .build_threadsafe_function()
        .callee_handled::<false>()
        .build()?;

    let mut guard = OPEN_FILE_TSFN
        .write()
        .map_err(|_| Error::from_reason("failed to lock OPEN_FILE_TSFN"))?;
    guard.replace(Arc::new(open_file_tsfn));
    Ok(())
}

#[cfg(target_env = "ohos")]
fn get_open_file_tsfn() -> Result<Arc<OpenFileTsfn>> {
    OPEN_FILE_TSFN
        .read()
        .map_err(|_| Error::from_reason("failed to read OPEN_FILE_TSFN"))?
        .as_ref()
        .map(Arc::clone)
        .ok_or_else(|| Error::from_reason("open-file callback is not registered"))
}

#[cfg(target_env = "ohos")]
fn normalize_to_file_uri(path: &Path) -> String {
    let raw = path.to_string_lossy();
    match ohos_fileuri_binding::get_uri_from_path(raw.as_ref()) {
        Ok(uri) => uri,
        Err(_) => {
            if raw.starts_with("file://") {
                raw.to_string()
            } else {
                format!("file://{}", raw)
            }
        }
    }
}

#[cfg(target_env = "ohos")]
fn canonicalize_ohos_uri(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // `file:///path` (missing bundleName) -> build canonical URI via fileuri API.
    if let Some(rest) = trimmed.strip_prefix("file://") {
        if rest.starts_with('/') {
            if let Ok(uri) = ohos_fileuri_binding::get_uri_from_path(rest) {
                return uri;
            }
        }
        return trimmed.to_string();
    }

    // Native path input -> build canonical URI via fileuri API.
    if trimmed.starts_with('/') {
        if let Ok(uri) = ohos_fileuri_binding::get_uri_from_path(trimmed) {
            return uri;
        }
    }

    trimmed.to_string()
}

#[cfg(target_env = "ohos")]
pub fn open_saved_uri(uri: &str) -> anyhow::Result<()> {
    let tsfn = get_open_file_tsfn().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let target_uri = canonicalize_ohos_uri(uri);
    if target_uri.is_empty() {
        return Err(anyhow::anyhow!("empty uri"));
    }
    let status = tsfn.call_with_return_value(
        target_uri,
        ThreadsafeFunctionCallMode::NonBlocking,
        move |result, _| {
            match result {
                Ok(value) => {
                    let promise = unsafe { value.cast::<PromiseRaw<'static, bool>>() }?;
                    promise.then(|_| Ok(()))?.catch(
                        move |ctx: napi_ohos::bindgen_prelude::CallbackContext<Unknown>| {
                            let _ = ctx;
                            log::warn!("open-file callback rejected");
                            Ok(())
                        },
                    )?;
                }
                Err(err) => {
                    log::warn!("open-file callback invoke error: {}", err);
                }
            }
            Ok(())
        },
    );

    if status != Status::Ok {
        return Err(anyhow::anyhow!(
            "call open-file callback failed with status: {:?}",
            status
        ));
    }
    Ok(())
}

#[cfg(not(target_env = "ohos"))]
pub fn open_saved_uri(uri: &str) -> anyhow::Result<()> {
    if let Some(path) = uri.strip_prefix("file://") {
        return open_saved_file(Path::new(path));
    }
    open_saved_file(Path::new(uri))
}

#[cfg(target_env = "ohos")]
pub fn open_saved_file(path: &Path) -> anyhow::Result<()> {
    open_saved_uri(&normalize_to_file_uri(path))
}

#[cfg(not(target_env = "ohos"))]
pub fn open_saved_file(path: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(path);
        c
    };

    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(path);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]).arg(path);
        c
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return Err(anyhow::anyhow!(
            "open file is not supported on this platform"
        ));
    }

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        let status = cmd.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "open file command exited with status {}",
                status
            ))
        }
    }
}
