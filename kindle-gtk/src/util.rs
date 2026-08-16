use std::ffi::{c_int, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::ffi::{FALSE, TRUE};

pub(crate) fn abort_on_panic<F: FnOnce()>(callback: F) {
    if catch_unwind(AssertUnwindSafe(callback)).is_err() {
        std::process::abort();
    }
}

pub(crate) fn bool_value(value: bool) -> c_int {
    if value {
        TRUE
    } else {
        FALSE
    }
}

pub(crate) fn c_string(text: &str) -> CString {
    CString::new(text.replace('\0', " ")).expect("NUL bytes were replaced")
}
