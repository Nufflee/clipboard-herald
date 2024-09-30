use std::{
    ffi::{CStr, CString},
    thread::sleep,
    time::Duration,
};

use anyhow::Error;

use windows::{
    core::HRESULT,
    Win32::{
        Foundation::*,
        System::{DataExchange::*, Memory::*, Ole::CF_TEXT},
    },
};

use super::{handle_to_hglobal, hglobal_to_handle};
use crate::clipboard::Clipboard;

#[derive(Debug, Clone)]
pub struct WindowsClipboard {
    is_open: bool,
}

impl Clipboard for WindowsClipboard {
    fn try_open() -> Result<WindowsClipboard, Error> {
        // Try to open the clipboard 10 times, waiting 100ms between each attempt if we receive ERROR_ACCESS_DENIED.
        // This is due to a race condition where the clipboard is sometimes still locked by another process, and we
        // need to wait for it to be unlocked.
        for _ in 0..10 {
            unsafe {
                match OpenClipboard(None) {
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        if e.code() != ERROR_ACCESS_DENIED.to_hresult() {
                            return Err(Error::new(e));
                        }
                    }
                }
            }

            sleep(Duration::from_millis(100));
        }

        Ok(Self { is_open: true })
    }

    fn is_text_available(&self) -> bool {
        assert!(self.is_open, "clipboard must be open");

        unsafe {
            let mut format = 0;

            loop {
                format = EnumClipboardFormats(format);

                if format == CF_TEXT.0.into() {
                    return true;
                }

                if format == 0 {
                    break;
                }
            }

            false
        }
    }

    fn get_text(&self) -> Result<String, Error> {
        assert!(self.is_open, "clipboard must be open");

        unsafe {
            let data_handle = GetClipboardData(CF_TEXT.0.into())?;

            let data_ptr = GlobalLock(handle_to_hglobal(data_handle));

            // Copy clipboard data into our own memory so we can unlock it without having
            // to worry about it getting moved or something.
            let text = CStr::from_ptr(data_ptr as *const i8)
                .to_string_lossy()
                .to_string();

            GlobalUnlock(handle_to_hglobal(data_handle))?;

            Ok(text)
        }
    }

    fn set_text(&mut self, text: &str) -> Result<(), Error> {
        assert!(self.is_open, "clipboard must be open");

        unsafe {
            let cstr = CString::new(text).unwrap();

            let alloc_handle = GlobalAlloc(GMEM_MOVEABLE, text.len() + 1)?;
            let alloc_ptr = GlobalLock(alloc_handle);

            std::ptr::copy_nonoverlapping(cstr.as_ptr(), alloc_ptr as *mut i8, text.len() + 1);

            // GlobalUnlock returns 0 on success (if we are the last to unlock it)...
            match GlobalUnlock(alloc_handle) {
                Ok(_) => {}
                Err(e) => {
                    if e.code() != HRESULT(0) {
                        panic!("failed to unlock global handle: {:?}", e);
                    }
                }
            }

            EmptyClipboard()?;

            SetClipboardData(CF_TEXT.0.into(), hglobal_to_handle(alloc_handle))?;
        }

        Ok(())
    }
}

impl Drop for WindowsClipboard {
    fn drop(&mut self) {
        unsafe {
            if self.is_open {
                CloseClipboard().unwrap();
            }
        }
    }
}
