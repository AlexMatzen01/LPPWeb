//! Buffer accessors shared between LPPO and native packages.
//!
//! Resolves lppo's exported `lppo_buf_*` symbols from the main executable
//! and provides safe helpers to read/write string buffers.

use once_cell::sync::Lazy;
use std::ffi::OsStr;
use std::sync::Mutex;

struct BufferAccessors {
    buf_get: unsafe extern "C" fn(i64, *mut u8, i64) -> i64,
    buf_set: unsafe extern "C" fn(i64, *const u8, i64) -> i64,
    buf_len: unsafe extern "C" fn(i64) -> i64,
    buf_new: unsafe extern "C" fn() -> i64,
    buf_clear: unsafe extern "C" fn(i64) -> i64,
    buf_append: unsafe extern "C" fn(i64, *const u8, i64) -> i64,
}

unsafe impl Send for BufferAccessors {}
unsafe impl Sync for BufferAccessors {}

static BUFFER_ACCESSORS: Lazy<Mutex<Option<&'static BufferAccessors>>> =
    Lazy::new(|| Mutex::new(None));

fn buffer_accessors() -> Result<&'static BufferAccessors, String> {
    let mut guard = BUFFER_ACCESSORS.lock().unwrap();
    if let Some(acc) = guard.as_ref() {
        return Ok(*acc);
    }
    let exe = OsStr::new("");
    unsafe {
        let lib = match libloading::Library::new(exe) {
            Ok(l) => Box::leak(Box::new(l)),
            Err(e) => return Err(format!("failed to open main executable: {e}")),
        };
        macro_rules! resolve {
            ($name:literal, $ty:ty) => {
                lib.get::<$ty>($name).map(|s| *s).map_err(|e| {
                    format!(
                        "failed to resolve {}: {e}",
                        std::str::from_utf8($name).unwrap_or("")
                    )
                })?
            };
        }
        let buf_get = resolve!(
            b"lppo_buf_get\0",
            unsafe extern "C" fn(i64, *mut u8, i64) -> i64
        );
        let buf_set = resolve!(
            b"lppo_buf_set\0",
            unsafe extern "C" fn(i64, *const u8, i64) -> i64
        );
        let buf_len = resolve!(b"lppo_buf_len\0", unsafe extern "C" fn(i64) -> i64);
        let buf_new = resolve!(b"lppo_buf_new\0", unsafe extern "C" fn() -> i64);
        let buf_clear = resolve!(b"lppo_buf_clear\0", unsafe extern "C" fn(i64) -> i64);
        let buf_append = resolve!(
            b"lppo_buf_append\0",
            unsafe extern "C" fn(i64, *const u8, i64) -> i64
        );
        let accessors = Box::leak(Box::new(BufferAccessors {
            buf_get,
            buf_set,
            buf_len,
            buf_new,
            buf_clear,
            buf_append,
        }));
        *guard = Some(accessors);
        Ok(accessors)
    }
}

/// Read an lppo buffer into a Vec<u8>. Returns None if unknown id.
pub fn buf_read(id: i64, max_len: usize) -> Option<Vec<u8>> {
    let acc = buffer_accessors().ok()?;
    let len = unsafe { (acc.buf_len)(id) };
    if len < 0 {
        return None;
    }
    let cap = (len as usize).min(max_len);
    let mut vec = Vec::with_capacity(cap);
    let written = unsafe { (acc.buf_get)(id, vec.as_mut_ptr(), cap as i64) };
    if written < 0 {
        return None;
    }
    unsafe { vec.set_len(written as usize) };
    Some(vec)
}

/// Write bytes into an lppo buffer (overwriting). Returns true on success.
pub fn buf_write(id: i64, data: &[u8]) -> bool {
    let Ok(acc) = buffer_accessors() else {
        return false;
    };
    let s = std::str::from_utf8(data).unwrap_or("");
    let result = unsafe { (acc.buf_set)(id, s.as_ptr(), s.len() as i64) };
    result == 1
}

/// Append bytes to an lppo buffer. Returns true on success.
pub fn buf_append(id: i64, data: &[u8]) -> bool {
    let Ok(acc) = buffer_accessors() else {
        return false;
    };
    let s = std::str::from_utf8(data).unwrap_or("");
    let result = unsafe { (acc.buf_append)(id, s.as_ptr(), s.len() as i64) };
    result == 1
}

/// Create a new empty buffer. Returns its id, or -1 on failure.
pub fn buf_new() -> i64 {
    let Ok(acc) = buffer_accessors() else {
        return -1;
    };
    unsafe { (acc.buf_new)() }
}

/// Clear a buffer's contents. Returns true on success.
pub fn buf_clear(id: i64) -> bool {
    let Ok(acc) = buffer_accessors() else {
        return false;
    };
    let result = unsafe { (acc.buf_clear)(id) };
    result == 1
}

/// Read a buffer as a String. Returns None if unknown id.
pub fn buf_to_string(id: i64) -> Option<String> {
    buf_read(id, 1_048_576).map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// Create a new buffer from a string. Returns its id, or -1 on failure.
pub fn make_buf(data: &str) -> i64 {
    let id = buf_new();
    if id < 0 {
        return -1;
    }
    if !buf_write(id, data.as_bytes()) {
        return -1;
    }
    id
}

/// Create a new buffer from a serde_json Value. Returns its id, or -1.
pub fn make_buf_from_value(v: &serde_json::Value) -> i64 {
    make_buf(&serde_json::to_string(v).unwrap_or_default())
}

/// Write a serde_json Value into an existing buffer (overwriting).
pub fn buf_write_value(id: i64, v: &serde_json::Value) -> bool {
    buf_write(id, serde_json::to_string(v).unwrap_or_default().as_bytes())
}
