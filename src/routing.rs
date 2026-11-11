//! Routing helpers: match paths against patterns, extract parameters.

use crate::buffer::{buf_to_string, make_buf_from_value};
use crate::state::{make_buf_id, set_last_error};
use serde_json::Value;

/// web_route_match(path_buf_id, pattern_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_route_match(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let path = match buf_to_string(a0) {
        Some(p) => p,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let pattern = match buf_to_string(a1) {
        Some(p) => p,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    set_last_error(0);
    if route_match(&path, &pattern) { 1 } else { 0 }
}

/// web_route_extract(path_buf_id, pattern_buf_id) -> Int64 (new buffer id with JSON params)
#[no_mangle]
pub extern "C" fn web_route_extract(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let path = match buf_to_string(a0) {
        Some(p) => p,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let pattern = match buf_to_string(a1) {
        Some(p) => p,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    set_last_error(0);
    match route_extract(&path, &pattern) {
        Some(params) => make_buf_id(&serde_json::to_string(&params).unwrap_or_default()),
        None => -1,
    }
}

fn route_match(path: &str, pattern: &str) -> bool {
    route_extract(path, pattern).is_some()
}

fn route_extract(path: &str, pattern: &str) -> Option<Value> {
    let path_parts: Vec<&str> = path.split('/').collect();
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    if path_parts.len() != pattern_parts.len() {
        return None;
    }
    let mut params = serde_json::Map::new();
    for (p_part, pat_part) in path_parts.iter().zip(pattern_parts.iter()) {
        if let Some(rest) = pat_part.strip_prefix(':') {
            params.insert(rest.to_string(), Value::String(p_part.to_string()));
        } else if p_part != pat_part {
            return None;
        }
    }
    Some(Value::Object(params))
}
