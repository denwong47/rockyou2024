//! A FFI wrapper around things Go need from Rust.
//!
use std::ffi::{CStr, CString, NulError};
use libc::c_char;

use rockyou2024::config;
use rockyou2024::models::IndexOf;

const LOG_TARGET: &str = "ffi";

#[no_mangle]
/// Get the indices of the given input.
///
/// For use in Go.
///
/// # ``LENGTH`` and ``DEPTH``
///
/// The length and depth of the index always default to the values in the configuration,
/// since the Rust FFI does not support generics.
pub extern "C" fn indices_of(input: *const c_char) -> *mut *mut c_char {
    if input.is_null() {
        rockyou2024::warn!(
            target: LOG_TARGET,
            "Received a null pointer for `input`.",
        );
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{c_str:?}' to a Rust string.",
            );
            return std::ptr::null_mut()
        }
    };

    let mut index_cstrs = match Result::<Vec<_>, _>::from_iter(
        IndexOf::<{config::INDEX_LENGTH}, {config::INDEX_DEPTH}>::from(rust_str.as_bytes())
        .map(|index| Ok::<_, NulError>(CString::new(index)?.into_raw()))
    ) {
        Ok(cstrs) => cstrs,
        Err(err) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{rust_str}' to `CString`: {err}",
            );

            return std::ptr::null_mut()
        }
    };

    index_cstrs.push(std::ptr::null_mut());

    let index_ctrs_ptr = index_cstrs.as_mut_ptr();

    std::mem::forget(index_cstrs);

    index_ctrs_ptr
}
