//! Bottom navigation rendering for the home page.

use super::*;

impl HomePage {
    pub(super) fn navigate_to(&self, pathname: &str, cx: &mut Context<Self>) {
        let current = RouterState::global(cx).location.pathname.clone();
        if current.as_ref() == pathname {
            return;
        }

        let pathname_owned = pathname.to_string();
        crate::ui::router_history::RouterHistoryState::global_mut(cx)
            .history
            .push(crate::ui::router_history::HistoryEntry::new(
                pathname_owned.clone(),
            ));
        RouterState::global_mut(cx).location.pathname = pathname_owned.into();
        cx.notify();
    }

    pub(super) fn render_bottom_nav(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let items: [(TabType, &'static str, &'static str); 3] = [
            (TabType::Receive, "接收", "icons/wifi.svg"),
            (TabType::Send, "发送", "icons/send-horizontal.svg"),
            (TabType::Settings, "设置", "icons/settings.svg"),
        ];

        h_flex()
            .w_full()
            .items_center()
            .children(items.iter().map(|(tab, label, icon_path)| {
                div()
                    .flex_1()
                    .child(self.render_bottom_nav_item(*tab, label, *icon_path, cx))
            }))
            .into_any_element()
    }

    fn render_bottom_nav_item(
        &mut self,
        tab: TabType,
        label: &'static str,
        icon_path: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.current_tab == tab;
        let tab_id = format!("tab-{:?}", tab);
        let text_color = if selected {
            cx.theme().primary
        } else {
            cx.theme().muted_foreground
        };
        let icon_el = Icon::default()
            .path(icon_path)
            .text_color(text_color)
            .with_size(gpui_component::Size::Large);

        div()
            .id(tab_id)
            .w_full()
            .h(px(56.))
            .py(px(6.))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(move |this, _event, _window, _cx| {
                this.current_tab = tab;
            }))
            .child(
                v_flex()
                    .items_center()
                    .gap(px(2.))
                    .text_color(text_color)
                    .child(icon_el)
                    .child(
                        div()
                            .when(selected, |this| this.text_base())
                            .when(!selected, |this| this.text_sm())
                            .child(label),
                    ),
            )
            .into_any_element()
    }
}
