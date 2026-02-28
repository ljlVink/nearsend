//! Centralized UI route paths.

pub const HOME: &str = "/";
pub const SETTINGS_ABOUT: &str = "/settings/about";
pub const SETTINGS_DONATE: &str = "/settings/donate";
pub const SETTINGS_OPEN_SOURCE_LICENSES: &str = "/settings/open-source-licenses";
pub const RECEIVE_HISTORY: &str = "/receive/history";
pub const RECEIVE_INCOMING: &str = "/receive/incoming";
pub const TRANSFER_PROGRESS: &str = "/transfer/progress";
pub const SEND_FILES: &str = "/send/files";
pub const SEND_LINK: &str = "/send/link";

/// Convert an absolute URL path into a `gpui-router` route pattern.
/// `gpui-router` route patterns should not start with `/`.
pub fn to_pattern(path: &str) -> &str {
    if path == HOME {
        ""
    } else {
        path.trim_start_matches('/')
    }
}
