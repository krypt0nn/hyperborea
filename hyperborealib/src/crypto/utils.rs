use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{SeedableRng, RngCore};

#[inline]
/// Generate random u64 number.
/// 
/// ```rust
/// use hyperborealib::crypto::safe_random_u64;
/// 
/// assert!(safe_random_u64() >= 0);
/// assert!(safe_random_u64() <= u64::MAX);
/// ```
pub fn safe_random_u64() -> u64 {
    ChaCha20Rng::from_entropy().next_u64()
}

#[inline]
/// Generate random u64 number with exactly 64 bits used.
/// 
/// This method uses `safe_random_u64` function and ensures
/// that the really first major bit is 1, making the number
/// start from `2^63`.
/// 
/// ```rust
/// use hyperborealib::crypto::safe_random_u64_long;
/// 
/// assert!(safe_random_u64_long() >= 1 << 63);
/// assert!(safe_random_u64_long() <= u64::MAX);
/// ```
pub fn safe_random_u64_long() -> u64 {
    (1 << 63) | (safe_random_u64() >> 1)
}
