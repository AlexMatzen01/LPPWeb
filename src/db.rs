//! Database functions (file-backed, SQL-like).

use crate::buffer::buf_to_string;
use crate::state::{set_last_error, WebState};
use serde_json::Value;
use std::fs;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////
// Open / close
////////////////////////////////////////////////////////////////////////////////

/// web_db_open(path_buf_id) -> Int64 (db handle id, or -1)
#[no_mangle]
pub extern "C" fn web_db_open(
    a0: i64,
    _a1: i64,
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
    let path = Path::new(&path);
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    let id = state.next_db_id;
    state.next_db_id += 1;
    let handle = WebState::load_db(path);
    state.databases.insert(id, handle);
    set_last_error(0);
    id as i64
}

/// web_db_close(db_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_close(
    a0: i64,
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
    let removed = state.databases.remove(&(a0 as u64)).is_some();
    set_last_error(0);
    if removed { 1 } else { 0 }
}

////////////////////////////////////////////////////////////////////////////////
// exec / query
////////////////////////////////////////////////////////////////////////////////

/// web_db_exec(db_id, sql_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_exec(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let sql = match buf_to_string(a1) {
        Some(s) => s,
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
    let result = true;
    set_last_error(0);
    if result { 1 } else { 0 }
}

/// web_db_query(db_id, sql_buf_id) -> Int64 (new buffer id with JSON result)
#[no_mangle]
pub extern "C" fn web_db_query(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let sql = match buf_to_string(a1) {
        Some(s) => s,
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
    let handle = match state.databases.get(&(a0 as u64)) {
        Some(h) => h,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let result = handle_query(handle, &sql);
    set_last_error(0);
    crate::state::make_buf_id(&serde_json::to_string(&result).unwrap_or_default())
}

////////////////////////////////////////////////////////////////////////////////
// insert / select / update / delete / count
////////////////////////////////////////////////////////////////////////////////

/// web_db_insert(db_id, table_buf_id, row_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_insert(
    a0: i64,
    a1: i64,
    a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let row = match buf_to_string(a2) {
        Some(r) => r,
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
    {
        let handle = match state.databases.get_mut(&(a0 as u64)) {
            Some(h) => h,
            None => {
                set_last_error(1);
                return 0;
            }
        };
        let row_val: Value = serde_json::from_str(&row).unwrap_or(Value::Null);
        handle.tables.entry(table).or_default().rows.push(row_val);
        handle.dirty = true;
        WebState::save_db(handle);
    }
    set_last_error(0);
    1
}

/// web_db_select(db_id, table_buf_id, where_buf_id) -> Int64 (new buffer id)
#[no_mangle]
pub extern "C" fn web_db_select(
    a0: i64,
    a1: i64,
    a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let where_sql = match buf_to_string(a2) {
        Some(w) => w,
        None => String::new(),
    };
    let state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return -1;
        }
    };
    let handle = match state.databases.get(&(a0 as u64)) {
        Some(h) => h,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let result = handle_select(handle, &table, &where_sql);
    set_last_error(0);
    crate::state::make_buf_id(&serde_json::to_string(&result).unwrap_or_default())
}

/// web_db_update(db_id, table_buf_id, set_buf_id, where_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_update(
    a0: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let set_sql = match buf_to_string(a2) {
        Some(s) => s,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let where_sql = match buf_to_string(a3) {
        Some(w) => w,
        None => String::new(),
    };
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return 0;
        }
    };
    let result = {
        let handle = match state.databases.get_mut(&(a0 as u64)) {
            Some(h) => h,
            None => {
                set_last_error(1);
                return 0;
            }
        };
        let ok = handle_update(handle, &table, &set_sql, &where_sql);
        if ok {
            WebState::save_db(handle);
        }
        ok
    };
    set_last_error(0);
    if result { 1 } else { 0 }
}

/// web_db_delete(db_id, table_buf_id, where_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_delete(
    a0: i64,
    a1: i64,
    a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
        None => {
            set_last_error(1);
            return 0;
        }
    };
    let where_sql = match buf_to_string(a2) {
        Some(w) => w,
        None => String::new(),
    };
    let mut state = match crate::state::state() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(1);
            return 0;
        }
    };
    let result = {
        let handle = match state.databases.get_mut(&(a0 as u64)) {
            Some(h) => h,
            None => {
                set_last_error(1);
                return 0;
            }
        };
        let ok = handle_delete(handle, &table, &where_sql);
        if ok {
            WebState::save_db(handle);
        }
        ok
    };
    set_last_error(0);
    if result { 1 } else { 0 }
}

/// web_db_count(db_id, table_buf_id) -> Int64
#[no_mangle]
pub extern "C" fn web_db_count(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
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
    let handle = match state.databases.get(&(a0 as u64)) {
        Some(h) => h,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let count = match handle.tables.get(&table) {
        Some(t) => t.rows.len() as i64,
        None => 0,
    };
    set_last_error(0);
    count
}

/// web_db_create_table(db_id, table_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_create_table(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
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
    {
        let handle = match state.databases.get_mut(&(a0 as u64)) {
            Some(h) => h,
            None => {
                set_last_error(1);
                return 0;
            }
        };
        handle.tables.entry(table).or_default();
        handle.dirty = true;
        WebState::save_db(handle);
    }
    set_last_error(0);
    1
}

/// web_db_drop_table(db_id, table_buf_id) -> Bool
#[no_mangle]
pub extern "C" fn web_db_drop_table(
    a0: i64,
    a1: i64,
    _a2: i64,
    _a3: i64,
    _a4: i64,
    _a5: i64,
    _a6: i64,
    _a7: i64,
) -> i64 {
    let table = match buf_to_string(a1) {
        Some(t) => t,
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
    {
        let handle = match state.databases.get_mut(&(a0 as u64)) {
            Some(h) => h,
            None => {
                set_last_error(1);
                return 0;
            }
        };
        let removed = handle.tables.remove(&table).is_some();
        if removed {
            let _ = fs::remove_file(handle.path.join(format!("{table}.json")));
            handle.dirty = true;
            WebState::save_db(handle);
        }
        set_last_error(0);
        if removed { 1 } else { 0 }
    }
}

/// web_db_list_tables(db_id) -> Int64 (new buffer id with JSON array)
#[no_mangle]
pub extern "C" fn web_db_list_tables(
    a0: i64,
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
    let handle = match state.databases.get(&(a0 as u64)) {
        Some(h) => h,
        None => {
            set_last_error(1);
            return -1;
        }
    };
    let tables: Vec<Value> = handle
        .tables
        .keys()
        .map(|k| Value::String(k.clone()))
        .collect();
    set_last_error(0);
    crate::state::make_buf_id(&serde_json::to_string(&Value::Array(tables)).unwrap_or_default())
}

////////////////////////////////////////////////////////////////////////////////
// Minimal SQL-like query engine
////////////////////////////////////////////////////////////////////////////////

fn handle_query(handle: &crate::state::DbHandle, sql: &str) -> Value {
    let sql = sql.trim();
    let upper = sql.to_uppercase();
    if upper.starts_with("SELECT ") {
        let rest = &sql[7..];
        let from_pos = match rest.find(" FROM ") {
            Some(p) => p,
            None => return Value::Array(vec![]),
        };
        let _columns = rest[..from_pos].trim();
        let after_from = rest[from_pos + 6..].trim();
        let (table_part, where_part) = match after_from.find(" WHERE ") {
            Some(p) => (after_from[..p].trim(), Some(after_from[p + 7..].trim())),
            None => (after_from.trim(), None),
        };
        let table_name = table_part.trim_end_matches(';').trim();
        let rows = match handle.tables.get(table_name) {
            Some(t) => t.rows.clone(),
            None => return Value::Array(vec![]),
        };
        if let Some(cond) = where_part {
            Value::Array(filter_rows(&rows, cond))
        } else {
            Value::Array(rows)
        }
    } else {
        Value::Array(vec![])
    }
}

fn handle_select(
    handle: &crate::state::DbHandle,
    table: &str,
    where_sql: &str,
) -> Value {
    let rows = match handle.tables.get(table) {
        Some(t) => t.rows.clone(),
        None => return Value::Array(vec![]),
    };
    if where_sql.is_empty() {
        return Value::Array(rows);
    }
    Value::Array(filter_rows(&rows, where_sql))
}

fn handle_update(
    handle: &mut crate::state::DbHandle,
    table: &str,
    set_sql: &str,
    where_sql: &str,
) -> bool {
    let rows = match handle.tables.get_mut(table) {
        Some(t) => &mut t.rows,
        None => return false,
    };
    let set_pairs = parse_kv_pairs(set_sql);
    for row in rows.iter_mut() {
        if row_matches(row, where_sql) {
            if let Value::Object(map) = row {
                for (k, v) in &set_pairs {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
    }
    handle.dirty = true;
    true
}

fn handle_delete(
    handle: &mut crate::state::DbHandle,
    table: &str,
    where_sql: &str,
) -> bool {
    let rows = match handle.tables.get_mut(table) {
        Some(t) => &mut t.rows,
        None => return false,
    };
    let len_before = rows.len();
    rows.retain(|r| !row_matches(r, where_sql));
    handle.dirty = true;
    rows.len() < len_before
}

fn filter_rows(rows: &[Value], cond: &str) -> Vec<Value> {
    rows.iter().filter(|r| row_matches(r, cond)).cloned().collect()
}

fn parse_kv_pairs(s: &str) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if let Some(eq_pos) = part.find('=') {
            let k = part[..eq_pos].trim().to_string();
            let v = part[eq_pos + 1..].trim().trim_matches('"').to_string();
            out.push((k, Value::String(v)));
        }
    }
    out
}

fn row_matches(row: &Value, cond: &str) -> bool {
    let cond = cond.trim();
    if cond.is_empty() {
        return true;
    }
    if let Some(eq_pos) = cond.find('=') {
        let k = cond[..eq_pos].trim();
        let v = cond[eq_pos + 1..].trim().trim_matches('"');
        match row {
            Value::Object(map) => match map.get(k) {
                Some(Value::String(s)) => s == v,
                _ => false,
            },
            _ => false,
        }
    } else {
        true
    }
}
