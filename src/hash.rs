//! Hashing: SHA-256 (cryptographic) and DJB2 (non-cryptographic).

use crate::buffer::{buf_read, make_buf};
use crate::state::set_last_error;
use sha2::Digest;

/// web_sha256_buf(buf_id) -> Int64 (new buffer id with hex digest)
#[no_mangle]
pub extern "C" fn web_sha256_buf(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let data = match buf_read(a0, 1_048_576) {
        Some(d) => d,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let mut hasher = sha2::Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let hex = format!("{:x}", result);
    set_last_error(0);
    make_buf(&hex)
}

/// web_djb2_buf(buf_id) -> Int64 (the djb2 hash value)
#[no_mangle]
pub extern "C" fn web_djb2_buf(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let data = match buf_read(a0, 1_048_576) {
        Some(d) => d,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let mut hash: u64 = 5381;
    for b in data {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    set_last_error(0);
    hash as i64
}
