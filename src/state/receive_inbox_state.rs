use crate::core::receive_events::IncomingTransferEvent;

#[derive(Clone, Debug, Default)]
pub struct ReceiveItem {
    pub file_id: String,
    pub file_name: String,
    pub file_type: String,
    pub size: u64,
    pub saved_path: Option<String>,
    pub text_content: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ReceiveSession {
    pub session_id: String,
    pub sender_alias: String,
    pub sender_device_model: Option<String>,
    pub sender_fingerprint: String,
    pub items: Vec<ReceiveItem>,
    pub completed: bool,
    pub cancelled: bool,
    pub is_message_only: bool,
}

#[derive(Default)]
pub struct ReceiveInboxState {
    pub active: Option<ReceiveSession>,
}

impl ReceiveInboxState {
    pub fn apply_event(&mut self, event: IncomingTransferEvent) {
        match event {
            IncomingTransferEvent::Prepared {
                session_id,
                sender_alias,
                sender_device_model,
                sender_fingerprint,
                files,
            } => {
                let is_message_only = files.len() == 1
                    && files
                        .first()
                        .map(|f| f.file_type.starts_with("text/"))
                        .unwrap_or(false);
                let items = files
                    .into_iter()
                    .map(|f| ReceiveItem {
                        file_id: f.file_id,
                        file_name: f.file_name,
                        file_type: f.file_type,
                        size: f.size,
                        saved_path: None,
                        text_content: None,
                    })
                    .collect();
                self.active = Some(ReceiveSession {
                    session_id,
                    sender_alias,
                    sender_device_model,
                    sender_fingerprint,
                    items,
                    completed: false,
                    cancelled: false,
                    is_message_only,
                });
            }
            IncomingTransferEvent::FileReceived {
                session_id,
                file_id,
                saved_path,
                text_content,
            } => {
                if let Some(active) = self.active.as_mut() {
                    if active.session_id != session_id {
                        return;
                    }
                    if let Some(item) = active.items.iter_mut().find(|x| x.file_id == file_id) {
                        item.saved_path = saved_path;
                        if text_content.is_some() {
                            item.text_content = text_content;
                        }
                    }
                }
            }
            IncomingTransferEvent::Completed { session_id } => {
                if let Some(active) = self.active.as_mut() {
                    if active.session_id == session_id {
                        active.completed = true;
                    }
                }
            }
            IncomingTransferEvent::Cancelled { session_id, .. } => {
                if let Some(active) = self.active.as_mut() {
                    if active.session_id == session_id {
                        active.cancelled = true;
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.active = None;
    }
}
