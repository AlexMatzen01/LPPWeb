//! HTTP response building.

use crate::buffer::buf_to_string;
use crate::state::{make_buf_id, set_last_error};

/// web_response_status_line(status_code) -> Int64 (new buffer id)
#[no_mangle]
pub extern "C" fn web_response_status_line(
    a0: i64,
    _a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let status = match a0 {
        200 => "200 OK",
        201 => "201 Created",
        400 => "400 Bad Request",
        404 => "404 Not Found",
        500 => "500 Internal Server Error",
        _ => "200 OK",
    };
    set_last_error(0);
    make_buf_id(&format!("HTTP/1.1 {status}"))
}

/// web_response_build(status_buf_id, headers_buf_id, body_buf_id) -> Int64 (new buffer id)
#[no_mangle]
pub extern "C" fn web_response_build(
    a0: i64,
    a1: i64,
    a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let status = match buf_to_string(a0) {
        Some(s) => s,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let headers = match buf_to_string(a1) {
        Some(s) => s,
        None => String::new(),
    };
    let body = match buf_to_string(a2) {
        Some(s) => s,
        None => String::new(),
    };
    let mut response = String::new();
    response.push_str(&status);
    response.push_str("\r\n");
    if !headers.is_empty() {
        response.push_str(&headers);
        response.push_str("\r\n");
    }
    response.push_str("Content-Length: ");
    response.push_str(&body.len().to_string());
    response.push_str("\r\n");
    response.push_str("Connection: close");
    response.push_str("\r\n");
    response.push_str("\r\n");
    response.push_str(&body);
    set_last_error(0);
    make_buf_id(&response)
}
