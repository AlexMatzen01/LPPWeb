//! Security measures: response headers, CORS, HSTS, CSP, X-Frame-Options, etc.

use crate::buffer::buf_to_string;
use crate::state::set_last_error;

////////////////////////////////////////////////////////////////////////////////
// Headers
////////////////////////////////////////////////////////////////////////////////

/// web_header_set(name_buf_id, value_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_header_set(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let name = match buf_to_string(a0) {
        Some(n) => n,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let value = match buf_to_string(a1) {
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
    if let Some(entry) = state.headers.iter_mut().find(|(n, _)| n == &name) {
        entry.1 = value;
    } else {
        state.headers.push((name, value));
    }
    set_last_error(0);
    1
}

/// web_header_clear() -> Bool
#[no_mangle]
pub extern "C" fn web_header_clear(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return 0;
        }
    };
    state.headers.clear();
    set_last_error(0);
    1
}

////////////////////////////////////////////////////////////////////////////////
// Pre-built security header bundles
////////////////////////////////////////////////////////////////////////////////

/// web_security_headers() -> Int64 (new buffer id with standard security headers)
#[no_mangle]
pub extern "C" fn web_security_headers(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    state.headers.push(("X-Content-Type-Options".to_string(), "nosniff".to_string()));
    state.headers.push(("X-Frame-Options".to_string(), "DENY".to_string()));
    state.headers.push((
        "Content-Security-Policy".to_string(),
        "default-src 'self'".to_string(),
    ));
    state.headers.push((
        "Strict-Transport-Security".to_string(),
        "max-age=31536000; includeSubDomains".to_string(),
    ));
    set_last_error(0);
    headers_to_buf(&state.headers)
}

/// web_cors(origin_buf_id) -> Int64 (new buffer id with CORS headers)
#[no_mangle]
pub extern "C" fn web_cors(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let origin = match buf_to_string(a0) {
        Some(o) => o,
        None => "*".to_string(),
    };
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    state.headers.push((
        "Access-Control-Allow-Origin".to_string(),
        origin,
    ));
    state.headers.push((
        "Access-Control-Allow-Methods".to_string(),
        "GET, POST, PUT, DELETE, OPTIONS".to_string(),
    ));
    state.headers.push((
        "Access-Control-Allow-Headers".to_string(),
        "Content-Type, Authorization".to_string(),
    ));
    set_last_error(0);
    headers_to_buf(&state.headers)
}

/// web_hsts() -> Int64
#[no_mangle]
pub extern "C" fn web_hsts(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    state.headers.push((
        "Strict-Transport-Security".to_string(),
        "max-age=31536000; includeSubDomains".to_string(),
    ));
    set_last_error(0);
    headers_to_buf(&state.headers)
}

/// web_csp() -> Int64
#[no_mangle]
pub extern "C" fn web_csp(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    state.headers.push((
        "Content-Security-Policy".to_string(),
        "default-src 'self'".to_string(),
    ));
    set_last_error(0);
    headers_to_buf(&state.headers)
}

/// web_x_frame_options() -> Int64
#[no_mangle]
pub extern "C" fn web_x_frame_options(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    state.headers.push(("X-Frame-Options".to_string(), "DENY".to_string()));
    set_last_error(0);
    headers_to_buf(&state.headers)
}

/// web_x_content_type_options() -> Int64
#[no_mangle]
pub extern "C" fn web_x_content_type_options(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    state.headers.push(("X-Content-Type-Options".to_string(), "nosniff".to_string()));
    set_last_error(0);
    headers_to_buf(&state.headers)
}

fn headers_to_buf(headers: &[(String, String)]) -> i64 {
    let mut lines = Vec::new();
    for (name, value) in headers {
        lines.push(format!("{name}: {value}"));
    }
    crate::state::make_buf_id(&lines.join("\r\n"))
}

////////////////////////////////////////////////////////////////////////////////
// Error
////////////////////////////////////////////////////////////////////////////////

/// web_last_error() -> Int64
#[no_mangle]
pub extern "C" fn web_last_error(
    _a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    crate::state::last_error()
}
