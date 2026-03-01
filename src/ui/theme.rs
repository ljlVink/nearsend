// Mobile-first theme and styling utilities

/// Mobile-first spacing constants
pub mod spacing {
    use gpui::px;

    pub const XS: gpui::Pixels = px(4.);
    pub const SM: gpui::Pixels = px(8.);
    pub const MD: gpui::Pixels = px(16.);
    pub const LG: gpui::Pixels = px(24.);
    #[allow(dead_code)]
    pub const XL: gpui::Pixels = px(32.);
}

/// Mobile-first sizing constants
pub mod sizing {
    use gpui::px;

    #[allow(dead_code)]
    pub const BUTTON_HEIGHT: gpui::Pixels = px(48.); // Large touch target
    pub const CARD_PADDING: gpui::Pixels = px(16.);
    #[allow(dead_code)]
    pub const CARD_BORDER_RADIUS: gpui::Pixels = px(12.);
    #[allow(dead_code)]
    pub const TAB_BAR_HEIGHT: gpui::Pixels = px(56.);
}
