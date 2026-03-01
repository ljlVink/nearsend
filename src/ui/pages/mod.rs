pub mod about;
pub mod donate;
pub mod history;
pub mod home;
pub mod open_source_licenses;
pub mod progress;
pub mod receive_dialog;
pub mod receive_incoming;
pub mod selected_files;
pub mod web_send;

pub use about::AboutPage;
pub use donate::DonatePage;
pub use history::HistoryPage;
#[allow(unused_imports)]
pub use home::{
    ColorMode, HomePage, IncomingTransferRequest, QuickSaveMode, ReceivePageState,
    SelectedFileInfo, SendMode, SendPageState, SettingsPageState, TabType, ThemeMode,
};
pub use open_source_licenses::OpenSourceLicensesPage;
pub use progress::ProgressPage;
#[allow(unused_imports)]
pub use receive_dialog::ReceiveDialog;
pub use receive_incoming::ReceiveIncomingPage;
pub use selected_files::SelectedFilesPage;
pub use web_send::WebSendPage;
