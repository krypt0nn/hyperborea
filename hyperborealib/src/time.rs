use std::time::{SystemTime, UNIX_EPOCH};

// FIXME: UTC time, not system time
// TODO: from_timestamp

/// Get current UTC timestamp in seconds.
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
