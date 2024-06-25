use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in seconds
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
