use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub struct IncomingFileMeta {
    pub file_id: String,
    pub file_name: String,
    pub file_type: String,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub enum IncomingTransferEvent {
    Prepared {
        session_id: String,
        sender_alias: String,
        sender_device_model: Option<String>,
        sender_fingerprint: String,
        files: Vec<IncomingFileMeta>,
    },
    FileReceived {
        session_id: String,
        file_id: String,
        saved_path: Option<String>,
        text_content: Option<String>,
    },
    Completed {
        session_id: String,
    },
    Cancelled {
        session_id: String,
        reason: Option<String>,
    },
}

static EVENT_QUEUE: OnceLock<Mutex<VecDeque<IncomingTransferEvent>>> = OnceLock::new();

fn queue() -> &'static Mutex<VecDeque<IncomingTransferEvent>> {
    EVENT_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn push_incoming_event(event: IncomingTransferEvent) {
    if let Ok(mut q) = queue().lock() {
        q.push_back(event);
    }
}

pub fn drain_incoming_events() -> Vec<IncomingTransferEvent> {
    if let Ok(mut q) = queue().lock() {
        q.drain(..).collect()
    } else {
        Vec::new()
    }
}
