use std::ffi::c_void;

use windows::Win32::Foundation::{HANDLE, HGLOBAL};

pub mod clipboard;

fn handle_to_hglobal(handle: HANDLE) -> HGLOBAL {
    // SAFETY: HGLOBAL is just a typedef for HANDLE
    HGLOBAL(handle.0 as *mut c_void)
}

fn hglobal_to_handle(hglobal: HGLOBAL) -> HANDLE {
    // SAFETY: HGLOBAL is just a typedef for HANDLE
    HANDLE(hglobal.0 as isize)
}
