//! The `rng` module.
//!
//! A very simple shared PRNG that uses the *split-mix64* algorithm on a single 64-bit word of state.
//! This module is private to the crate and is not exported.

#![allow(dead_code)]

// `SystemTime` is used to seed the PRNG on first use.
use std::time::SystemTime;

// Create a thread-safe singleton instance of `SplitMix64` seeded on first use from the system clock.
// `LazyLock` items are immutable but the PRNG needs to be mutable so we wrap it in a `Mutex`.
use std::sync::{
    LazyLock,
    Mutex,
};
static RNG: LazyLock<Mutex<SplitMix64>> = LazyLock::new(|| Mutex::new(SplitMix64::new()));

/// Crate-only function that returns a random 64-bit unsigned integer using the static singleton instance of the PRNG.
#[inline]
pub(crate) fn u64() -> u64 { RNG.lock().unwrap().u64() }

/// Crate-only function that returns a random boolean value using the static singleton instance of the PRNG.
#[inline]
pub(crate) fn bool() -> bool { RNG.lock().unwrap().bool() }

/// Crate-only function that returns the current seed of the static singleton instance of the PRNG.
#[inline]
pub(crate) fn seed() -> u64 { RNG.lock().unwrap().seed() }

/// Crate-only function that sets the seed of the static singleton instance of the PRNG.
pub(crate) fn set_seed(seed: u64) { RNG.lock().unwrap().set_seed(seed) }

/// A very simple PRNG that uses the *split-mix64* algorithm on a single 64-bit word of state.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Create a new generator that uses a scrambled version of the current time as the seed.
    pub fn new() -> Self { Self::from_seed(seed_from_clock()) }

    /// Create a new generator with a user-specified seed.
    pub fn from_seed(seed: u64) -> Self { Self { state: seed } }

    /// Returns the current seed of the PRNG.
    #[inline]
    pub fn seed(&self) -> u64 { self.state }

    /// Set the seed of the PRNG.
    #[inline]
    pub fn set_seed(&mut self, seed: u64) { self.state = seed }

    /// Returns a random 64-bit unsigned integer.
    #[inline]
    pub fn u64(&mut self) -> u64 { self.next() }

    /// Returns a random boolean value.
    #[inline]
    pub fn bool(&mut self) -> bool { self.next() & 1 == 1 }

    /// Generate the next 64-bit unsigned word using the split-mix64 algorithm.
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// Returns a single unsigned 64-bit value based on a scrambled version of the current time.
fn seed_from_clock() -> u64 {
    // Get the current time: By itself this is monotonic with low entropy so we need to scramble it.
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

    // Grab that duration as a seconds part (a u64) and a nanosecond part (a u32).
    let secs = now.as_secs();
    let ns = now.subsec_nanos();

    // The high order bits in `secs` change very slowly so we replace them with the nanosecond part.
    let seed = u64::from(ns) << 32 | secs;

    // Do a fast mix of that seed to get a reasonably random starting point for the PRNG.
    murmur64(seed)
}

/// Returns a scrambled version of the 64-bit input `x` using the Murmur3 hash function.
fn murmur64(x: u64) -> u64 {
    let mut x = x;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^= x >> 33;
    x
}
