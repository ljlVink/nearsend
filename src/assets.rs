use gpui::{AssetSource, Result, SharedString};
use gpui_component_assets::Assets as ComponentAssets;
use std::borrow::Cow;

/// Asset source that adds near-send icons (e.g. send-horizontal) on top of gpui-component assets.
pub struct NearSendAssets(pub ComponentAssets);

const BOTTOM_NAV_ICONS: &[&str] = &[
    "icons/wifi.svg",
    "icons/send-horizontal.svg",
    "icons/settings.svg",
];

impl AssetSource for NearSendAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes = match path {
            "icons/wifi.svg" => Some(include_bytes!("../assets/icons/wifi.svg").as_slice()),
            "icons/send-horizontal.svg" => {
                Some(include_bytes!("../assets/icons/send-horizontal.svg").as_slice())
            }
            "icons/settings.svg" => Some(include_bytes!("../assets/icons/settings.svg").as_slice()),
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
            for name in BOTTOM_NAV_ICONS {
                let s: SharedString = (*name).into();
                if !list.contains(&s) {
                    list.push(s);
                }
            }
        }
        Ok(list)
    }
}
