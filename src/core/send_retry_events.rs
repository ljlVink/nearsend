use std::sync::atomic::{AtomicBool, Ordering};

static SEND_RETRY_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_send_retry() {
    SEND_RETRY_REQUESTED.store(true, Ordering::Release);
}

pub fn take_send_retry_requested() -> bool {
    SEND_RETRY_REQUESTED.swap(false, Ordering::AcqRel)
}
