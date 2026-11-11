//! Shared in-process state for LPPWeb: storage, databases, headers, errors.

use crate::buffer::make_buf;
use crate::buffer::make_buf_from_value;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};

const STORAGE_FILE: &str = ".lppweb_storage.json";

#[derive(Clone, Default)]
pub struct Table {
    pub rows: Vec<Value>,
}

#[derive(Clone, Default)]
pub struct DbHandle {
    pub path: PathBuf,
    pub tables: HashMap<String, Table>,
    pub dirty: bool,
}

pub struct WebState {
    pub storage: HashMap<String, String>,
    pub storage_path: PathBuf,
    pub databases: HashMap<u64, DbHandle>,
    pub next_db_id: u64,
    pub headers: Vec<(String, String)>,
}

impl WebState {
    pub fn storage_path() -> PathBuf {
        std::env::current_dir()
            .map(|p| p.join(STORAGE_FILE))
            .unwrap_or_else(|_| PathBuf::from(STORAGE_FILE))
    }

    pub fn load_storage() -> HashMap<String, String> {
        let path = Self::storage_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&contents) {
                return map;
            }
        }
        HashMap::new()
    }

    pub fn save_storage(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.storage) {
            let _ = fs::write(&self.storage_path, json);
        }
    }

    pub fn load_db(path: &std::path::Path) -> DbHandle {
        let mut tables = HashMap::new();
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.extension().map(|e| e == "json").unwrap_or(false) {
                        if let Ok(contents) = fs::read_to_string(&entry_path) {
                            if let Ok(rows) = serde_json::from_str::<Vec<Value>>(&contents) {
                                let name = entry_path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                tables.insert(name, Table { rows });
                            }
                        }
                    }
                }
            }
        }
        DbHandle {
            path: path.to_path_buf(),
            tables,
            dirty: false,
        }
    }

    pub fn save_db(handle: &DbHandle) {
        let _ = fs::create_dir_all(&handle.path);
        for (name, table) in &handle.tables {
            let file_path = handle.path.join(format!("{name}.json"));
            if let Ok(json) = serde_json::to_string_pretty(&table.rows) {
                let _ = fs::write(&file_path, json);
            }
        }
    }
}

pub struct StateGuard<'a> {
    guard: std::sync::MutexGuard<'a, WebState>,
}

impl<'a> std::ops::Deref for StateGuard<'a> {
    type Target = WebState;
    fn deref(&self) -> &WebState {
        &self.guard
    }
}

impl<'a> std::ops::DerefMut for StateGuard<'a> {
    fn deref_mut(&mut self) -> &mut WebState {
        &mut self.guard
    }
}

static STATE: once_cell::sync::Lazy<std::sync::Mutex<WebState>> =
    once_cell::sync::Lazy::new(|| {
        std::sync::Mutex::new(WebState {
            storage: WebState::load_storage(),
            storage_path: WebState::storage_path(),
            databases: HashMap::new(),
            next_db_id: 1,
            headers: Vec::new(),
        })
    });

static LAST_ERROR: AtomicI64 = AtomicI64::new(0);

pub fn state() -> Result<StateGuard<'static>, String> {
    let guard = STATE.lock().map_err(|e| e.to_string())?;
    Ok(StateGuard { guard })
}

pub fn set_last_error(code: i64) {
    LAST_ERROR.store(code, Ordering::Relaxed);
}

pub fn last_error() -> i64 {
    LAST_ERROR.load(Ordering::Relaxed)
}

pub fn make_buf_id(data: &str) -> i64 {
    make_buf(data)
}
