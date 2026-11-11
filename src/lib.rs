//! LPPWeb - batteries-included web server package for LPPO.
//!
//! Requires LPPNet. Exposes hashing, encryption, storage, database, security
//! headers, routing, and response-building functions to LPPO via native FFI.
//! String data is exchanged through lppo's shared string buffer registry
//! (`lppo_buf_*` symbols resolved from the main executable).

mod buffer;
mod crypto;
mod db;
mod hash;
mod response;
mod routing;
mod security;
mod storage;
mod state;

use std::sync::Once;

static INIT: Once = Once::new();

/// Ensure the shared state is initialized.
#[no_mangle]
pub extern "C" fn lppweb_init() {
    INIT.call_once(|| {
        // Force initialization of the lazy state.
        let _ = crate::state::state();
    });
}
