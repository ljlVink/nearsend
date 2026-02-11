use gpui::{AssetSource, Result, SharedString};
use gpui_component_assets::Assets as ComponentAssets;
use std::borrow::Cow;

/// Asset source that adds near-send icons on top of gpui-component assets.
pub struct NearSendAssets(pub ComponentAssets);

const CUSTOM_ICONS: &[&str] = &[
    "icons/wifi.svg",
    "icons/send-horizontal.svg",
    "icons/settings.svg",
    "icons/history.svg",
    "icons/logo.svg",
    "icons/info.svg",
    "icons/file.svg",
    "icons/book-open.svg",
    "icons/folder.svg",
    "icons/close.svg",
    "icons/plus.svg",
    "icons/loader.svg",
    "icons/heart.svg",
    "icons/check.svg",
    "icons/x.svg",
    "icons/download.svg",
    "icons/upload.svg",
    "icons/trash.svg",
    "icons/external-link.svg",
    "icons/smartphone.svg",
    "icons/monitor.svg",
    "icons/globe.svg",
    "icons/server.svg",
    "icons/arrow-left.svg",
    "icons/refresh.svg",
    "icons/image.svg",
    "icons/target.svg",
    "icons/more-horizontal.svg",
];

impl AssetSource for NearSendAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes = match path {
            "icons/wifi.svg" => Some(include_bytes!("../assets/icons/wifi.svg").as_slice()),
            "icons/send-horizontal.svg" => {
                Some(include_bytes!("../assets/icons/send-horizontal.svg").as_slice())
            }
            "icons/settings.svg" => Some(include_bytes!("../assets/icons/settings.svg").as_slice()),
            "icons/history.svg" => Some(include_bytes!("../assets/icons/history.svg").as_slice()),
            "icons/logo.svg" => Some(include_bytes!("../assets/icons/logo.svg").as_slice()),
            "icons/info.svg" => Some(include_bytes!("../assets/icons/info.svg").as_slice()),
            "icons/file.svg" => Some(include_bytes!("../assets/icons/file.svg").as_slice()),
            "icons/book-open.svg" => {
                Some(include_bytes!("../assets/icons/book-open.svg").as_slice())
            }
            "icons/folder.svg" => Some(include_bytes!("../assets/icons/folder.svg").as_slice()),
            "icons/close.svg" => Some(include_bytes!("../assets/icons/close.svg").as_slice()),
            "icons/plus.svg" => Some(include_bytes!("../assets/icons/plus.svg").as_slice()),
            "icons/loader.svg" => Some(include_bytes!("../assets/icons/loader.svg").as_slice()),
            "icons/heart.svg" => Some(include_bytes!("../assets/icons/heart.svg").as_slice()),
            "icons/check.svg" => Some(include_bytes!("../assets/icons/check.svg").as_slice()),
            "icons/x.svg" => Some(include_bytes!("../assets/icons/x.svg").as_slice()),
            "icons/download.svg" => Some(include_bytes!("../assets/icons/download.svg").as_slice()),
            "icons/upload.svg" => Some(include_bytes!("../assets/icons/upload.svg").as_slice()),
            "icons/trash.svg" => Some(include_bytes!("../assets/icons/trash.svg").as_slice()),
            "icons/external-link.svg" => {
                Some(include_bytes!("../assets/icons/external-link.svg").as_slice())
            }
            "icons/smartphone.svg" => {
                Some(include_bytes!("../assets/icons/smartphone.svg").as_slice())
            }
            "icons/monitor.svg" => Some(include_bytes!("../assets/icons/monitor.svg").as_slice()),
            "icons/globe.svg" => Some(include_bytes!("../assets/icons/globe.svg").as_slice()),
            "icons/server.svg" => Some(include_bytes!("../assets/icons/server.svg").as_slice()),
            "icons/arrow-left.svg" => {
                Some(include_bytes!("../assets/icons/arrow-left.svg").as_slice())
            }
            "icons/refresh.svg" => Some(include_bytes!("../assets/icons/refresh.svg").as_slice()),
            "icons/image.svg" => Some(include_bytes!("../assets/icons/image.svg").as_slice()),
            "icons/target.svg" => Some(include_bytes!("../assets/icons/target.svg").as_slice()),
            "icons/more-horizontal.svg" => {
                Some(include_bytes!("../assets/icons/more-horizontal.svg").as_slice())
            }
            _ => None,
        };
        if let Some(b) = bytes {
            return Ok(Some(Cow::Borrowed(b)));
        }
        self.0.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut list = self.0.list(path)?;
        if path == "icons" || path == "icons/" {
            for name in CUSTOM_ICONS {
                let s: SharedString = (*name).into();
                if !list.contains(&s) {
                    list.push(s);
                }
            }
        }
        Ok(list)
    }
}
