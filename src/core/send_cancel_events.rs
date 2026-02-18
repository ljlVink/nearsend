use std::sync::atomic::{AtomicBool, Ordering};

static SEND_CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_send_cancel() {
    SEND_CANCEL_REQUESTED.store(true, Ordering::Release);
}

pub fn take_send_cancel_requested() -> bool {
    SEND_CANCEL_REQUESTED.swap(false, Ordering::AcqRel)
}
