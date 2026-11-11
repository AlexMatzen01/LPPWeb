//! Symmetric encryption (XOR cipher).

use crate::buffer::{buf_read, make_buf};
use crate::state::set_last_error;

fn xor_crypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

/// web_xor_encrypt(key_buf_id, data_buf_id) -> Int64 (new buffer id)
#[no_mangle]
pub extern "C" fn web_xor_encrypt(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let key = match buf_read(a0, 65536) {
        Some(d) => d,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let data = match buf_read(a1, 1_048_576) {
        Some(d) => d,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let result = xor_crypt(&key, &data);
    let s = String::from_utf8_lossy(&result).into_owned();
    set_last_error(0);
    make_buf(&s)
}

/// web_xor_decrypt(key_buf_id, data_buf_id) -> Int64 (new buffer id)
#[no_mangle]
pub extern "C" fn web_xor_decrypt(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    // XOR is symmetric: decrypt == encrypt
    web_xor_encrypt(a0, a1, 0, 0, 0, 0, 0, 0)
}
