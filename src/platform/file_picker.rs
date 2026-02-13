use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, RwLock};

use napi_derive_ohos::napi;
use napi_ohos::{
    bindgen_prelude::{Function, PromiseRaw, Unknown},
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
    Error, Result, Status,
};
use tokio::sync::oneshot;

type PickFilesTsfn = ThreadsafeFunction<(), Unknown<'static>, (), Status, false>;
type PickFoldersTsfn = ThreadsafeFunction<(), Unknown<'static>, (), Status, false>;
type PickSaveDirTsfn = ThreadsafeFunction<(), Unknown<'static>, (), Status, false>;

static PICK_FILES_TSFN: LazyLock<RwLock<Option<Arc<PickFilesTsfn>>>> =
    LazyLock::new(|| RwLock::new(None));
static PICK_FOLDERS_TSFN: LazyLock<RwLock<Option<Arc<PickFoldersTsfn>>>> =
    LazyLock::new(|| RwLock::new(None));
static PICK_SAVE_DIR_TSFN: LazyLock<RwLock<Option<Arc<PickSaveDirTsfn>>>> =
    LazyLock::new(|| RwLock::new(None));

#[napi]
pub fn register_file_picker_callbacks(
    pick_files: Function<'static, (), Unknown<'static>>,
    pick_folders: Function<'static, (), Unknown<'static>>,
    pick_save_directory: Function<'static, (), Unknown<'static>>,
) -> Result<()> {
    let files_tsfn = pick_files
        .build_threadsafe_function()
        .callee_handled::<false>()
        .build()?;
    let folders_tsfn = pick_folders
        .build_threadsafe_function()
        .callee_handled::<false>()
        .build()?;
    let save_dir_tsfn = pick_save_directory
        .build_threadsafe_function()
        .callee_handled::<false>()
        .build()?;

    {
        let mut guard = PICK_FILES_TSFN
            .write()
            .map_err(|_| Error::from_reason("failed to lock PICK_FILES_TSFN"))?;
        guard.replace(Arc::new(files_tsfn));
    }
    {
        let mut guard = PICK_FOLDERS_TSFN
            .write()
            .map_err(|_| Error::from_reason("failed to lock PICK_FOLDERS_TSFN"))?;
        guard.replace(Arc::new(folders_tsfn));
    }
    {
        let mut guard = PICK_SAVE_DIR_TSFN
            .write()
            .map_err(|_| Error::from_reason("failed to lock PICK_SAVE_DIR_TSFN"))?;
        guard.replace(Arc::new(save_dir_tsfn));
    }
    Ok(())
}

fn get_pick_files_tsfn() -> Result<Arc<PickFilesTsfn>> {
    PICK_FILES_TSFN
        .read()
        .map_err(|_| Error::from_reason("failed to read PICK_FILES_TSFN"))?
        .as_ref()
        .map(Arc::clone)
        .ok_or_else(|| Error::from_reason("file picker callback is not registered"))
}

fn get_pick_folders_tsfn() -> Result<Arc<PickFoldersTsfn>> {
    PICK_FOLDERS_TSFN
        .read()
        .map_err(|_| Error::from_reason("failed to read PICK_FOLDERS_TSFN"))?
        .as_ref()
        .map(Arc::clone)
        .ok_or_else(|| Error::from_reason("folder picker callback is not registered"))
}

fn get_pick_save_dir_tsfn() -> Result<Arc<PickSaveDirTsfn>> {
    PICK_SAVE_DIR_TSFN
        .read()
        .map_err(|_| Error::from_reason("failed to read PICK_SAVE_DIR_TSFN"))?
        .as_ref()
        .map(Arc::clone)
        .ok_or_else(|| Error::from_reason("save-directory picker callback is not registered"))
}

async fn invoke_picker(tsfn: Arc<PickFilesTsfn>) -> Result<Vec<String>> {
    let (tx, rx) = oneshot::channel::<Result<Vec<String>>>();
    let status = tsfn.call_with_return_value(
        (),
        ThreadsafeFunctionCallMode::NonBlocking,
        move |result, _| {
            match result {
                Ok(value) => {
                    let tx_cell = Rc::new(Cell::new(Some(tx)));
                    let tx_in_catch = tx_cell.clone();
                    let promise = unsafe { value.cast::<PromiseRaw<'static, Vec<String>>>() }?;
                    promise
                        .then(move |ctx| {
                            if let Some(sender) = tx_cell.replace(None) {
                                let _ = sender.send(Ok(ctx.value));
                            }
                            Ok(())
                        })?
                        .catch(
                            move |ctx: napi_ohos::bindgen_prelude::CallbackContext<Unknown>| {
                                if let Some(sender) = tx_in_catch.replace(None) {
                                    let _ = sender.send(Err(ctx.value.into()));
                                }
                                Ok(())
                            },
                        )?;
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                }
            }
            Ok(())
        },
    );

    if status != Status::Ok {
        return Err(Error::from_reason(format!(
            "call picker callback failed with status: {:?}",
            status
        )));
    }

    rx.await
        .map_err(|_| Error::from_reason("picker callback receiver dropped"))?
}

pub async fn pick_files() -> Result<Vec<String>> {
    invoke_picker(get_pick_files_tsfn()?).await
}

pub async fn pick_folders() -> Result<Vec<String>> {
    invoke_picker(get_pick_folders_tsfn()?).await
}

pub async fn pick_save_directory() -> Result<Option<PathBuf>> {
    let tsfn = get_pick_save_dir_tsfn()?;
    let (tx, rx) = oneshot::channel::<Result<String>>();
    let status = tsfn.call_with_return_value(
        (),
        ThreadsafeFunctionCallMode::NonBlocking,
        move |result, _| {
            match result {
                Ok(value) => {
                    let tx_cell = Rc::new(Cell::new(Some(tx)));
                    let tx_in_catch = tx_cell.clone();
                    let promise = unsafe { value.cast::<PromiseRaw<'static, String>>() }?;
                    promise
                        .then(move |ctx| {
                            if let Some(sender) = tx_cell.replace(None) {
                                let _ = sender.send(Ok(ctx.value));
                            }
                            Ok(())
                        })?
                        .catch(
                            move |ctx: napi_ohos::bindgen_prelude::CallbackContext<Unknown>| {
                                if let Some(sender) = tx_in_catch.replace(None) {
                                    let _ = sender.send(Err(ctx.value.into()));
                                }
                                Ok(())
                            },
                        )?;
                }
                Err(err) => {
                    let _ = tx.send(Err(err));
                }
            }
            Ok(())
        },
    );
    if status != Status::Ok {
        return Err(Error::from_reason(format!(
            "call save-directory picker callback failed with status: {:?}",
            status
        )));
    }
    let uri = rx
        .await
        .map_err(|_| Error::from_reason("save-directory picker callback receiver dropped"))??;
    Ok(picker_uri_to_path(&uri))
}

/// Convert picker output (URI or path) to PathBuf.
/// On OpenHarmony, prefer `ohos-fileuri-binding` to resolve URIs to native paths.
pub fn picker_uri_to_path(uri: &str) -> Option<PathBuf> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return None;
    }
    uri_to_native_path(trimmed)
}

#[cfg(target_env = "ohos")]
fn uri_to_native_path(uri: &str) -> Option<PathBuf> {
    match ohos_fileuri_binding::get_path_from_uri(uri) {
        Ok(path) => Some(PathBuf::from(path)),
        Err(err) => {
            log::warn!(
                "failed to convert picker uri via ohos-fileuri-binding: {}",
                err
            );
            if let Some(path) = uri.strip_prefix("file://") {
                return Some(PathBuf::from(path));
            }
            Some(PathBuf::from(uri))
        }
    }
}

#[cfg(not(target_env = "ohos"))]
fn uri_to_native_path(uri: &str) -> Option<PathBuf> {
    if let Some(path) = uri.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }
    Some(PathBuf::from(uri))
}
