//! File-backed key-value local storage.

use crate::buffer::{buf_to_string, make_buf};
use crate::state::set_last_error;
use serde_json::Value;

/// web_storage_put(key_buf_id, val_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_storage_put(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let key = match buf_to_string(a0) {
        Some(k) => k,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let val = match buf_to_string(a1) {
        Some(v) => v,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return 0;
        }
    };
    state.storage.insert(key, val);
    state.save_storage();
    set_last_error(0);
    1
}

/// web_storage_get(key_buf_id) -> Int64 (new buffer id with value, or -1)
#[no_mangle]
pub extern "C" fn web_storage_get(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let key = match buf_to_string(a0) {
        Some(k) => k,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    match state.storage.get(&key) {
        Some(v) => {
            set_last_error(0);
            make_buf(v)
        }
        None => {
            set_last_error(0);
            -1
        }
    }
}

/// web_storage_delete(key_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_storage_delete(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let key = match buf_to_string(a0) {
        Some(k) => k,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return 0;
        }
    };
    let removed = state.storage.remove(&key).is_some();
    if removed {
        state.save_storage();
    }
    set_last_error(0);
    if removed { 1 } else { 0 }
}

/// web_storage_exists(key_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_storage_exists(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let key = match buf_to_string(a0) {
        Some(k) => k,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return 0;
        }
    };
    set_last_error(0);
    if state.storage.contains_key(&key) { 1 } else { 0 }
}

/// web_storage_list() -> Int64 (new buffer id with JSON array of keys)
#[no_mangle]
pub extern "C" fn web_storage_list(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    let keys: Vec<Value> = state
        .storage
        .keys()
        .map(|k| Value::String(k.clone()))
        .collect();
    set_last_error(0);
    crate::state::make_buf_id(&serde_json::to_string(&keys).unwrap_or_default())
}
