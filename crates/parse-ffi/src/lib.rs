//! A FFI wrapper around things Go need from Rust.
//!
use libc::c_char;
use std::ffi::{CStr, CString, NulError};

use rockyou2024::config;
use rockyou2024::models::IndexOf;

#[cfg(doc)]
use rockyou2024::models::IndexCollection;

const LOG_TARGET: &str = "ffi";

macro_rules! vec_str_to_mut_mut_c_char {
    ($vec_str:expr) => {
        match Result::<Vec<_>, _>::from_iter(
            $vec_str
            .into_iter()
            .map(|s| Ok::<_, NulError>(CString::new(s)?.into_raw()))
        ) {
            Ok(mut cstrs) => {
                cstrs.push(std::ptr::null_mut());

                let ctrs_ptr = cstrs.as_mut_ptr();

                std::mem::forget(cstrs);

                ctrs_ptr
            },
            Err(err) => {
                rockyou2024::warn!(
                    target: LOG_TARGET,
                    "Could not convert `{vec_str}` to `CString`: {err}",
                    vec_str=stringify!($vec_str),
                    err=err,
                );

                std::ptr::null_mut()
            }
        }
    }
}

#[no_mangle]
/// Get the indices of the given input.
///
/// For use in Go.
///
/// # ``LENGTH`` and ``DEPTH``
///
/// The length and depth of the index always default to the values in the configuration,
/// since the Rust FFI does not support generics.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers; this is unavoidable if we have to
/// pass an array of strings to Go.
///
/// Go will be responsible for freeing the memory allocated; please ensure that
/// `defer C.free(unsafe.Pointer(ptr))` is called for each string in the array.
pub unsafe extern "C" fn indices_of(input: *const c_char) -> *mut *mut c_char {
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
            return std::ptr::null_mut();
        }
    };

    vec_str_to_mut_mut_c_char!(
        IndexOf::<{ config::INDEX_LENGTH }, { config::INDEX_DEPTH }>::from(rust_str.as_bytes())
    )
}

#[no_mangle]
/// Clean the string using the specified search style.
pub unsafe extern "C" fn as_search_string(
    query: *const c_char,
    search_style: *const c_char,
) -> *mut c_char {
    let query_c_str = unsafe { CStr::from_ptr(query) };
    let query_str = match query_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{query_c_str:?}' to a Rust string.",
            );
            return std::ptr::null_mut();
        }
    };

    let search_style_c_str = unsafe { CStr::from_ptr(search_style) };
    let search_style_str = match search_style_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{search_style_c_str:?}' to a Rust string.",
            );
            return std::ptr::null_mut();
        }
    };

    let search_style = match search_style_str {
        "strict" => rockyou2024::search::SearchStyle::Strict,
        "case-insensitive" => rockyou2024::search::SearchStyle::CaseInsensitive,
        "fuzzy" => rockyou2024::search::SearchStyle::Fuzzy,
        _ => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Unknown search style '{search_style_str}'.",
            );
            return std::ptr::null_mut();
        }
    };

    let transformed = search_style.transform_query()(&[query_str])
        .pop()
        .expect("The transformed query should always have at least one element.");

    match CString::new(transformed) {
        Ok(c_str) => c_str.into_raw(),
        Err(err) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert final string to a `CString`: {err}",
                err=err,
            );
            return std::ptr::null_mut();
        }
    }
}

#[no_mangle]
/// Find the lines in the index collection that contain the given query.
///
/// This function is a wrapper around the [`IndexCollection::find_lines_containing`] method, which
/// does not report errors. This function will log any errors and return a null pointer if an error
/// occurs, including:
///
/// - The `dir` pointer is null.
/// - The path given by `dir` is not a directory.
/// - The `query` pointer is null.
/// - The `search_style` pointer is null.
/// - The `search_style` is not one of "strict", "case-insensitive", or "fuzzy".
///
/// For use in Go.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers; this is unavoidable if we have to
/// pass an array of strings to Go.
///
/// Go will be responsible for freeing the memory allocated; please ensure that
/// `defer C.free(unsafe.Pointer(ptr))` is called for each string in the array.
pub unsafe extern "C" fn find_lines_in_index_collection(
    dir: *const c_char,
    query: *const c_char,
    search_style: *const c_char,
) -> *mut *mut c_char {
    // Validate the input.
    if dir.is_null() {
        rockyou2024::warn!(
            target: LOG_TARGET,
            "Received a null pointer for `dir`.",
        );
        return std::ptr::null_mut();
    }

    let dir_c_str = unsafe { CStr::from_ptr(dir) };
    let dir_str = match dir_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{dir_c_str:?}' to a Rust string.",
            );
            return std::ptr::null_mut();
        }
    };

    let path = std::path::Path::new(dir_str);
    if !path.is_dir() {
        rockyou2024::warn!(
            target: LOG_TARGET,
            "The path '{path:?}' is not a directory.",
        );
        return std::ptr::null_mut();
    }

    let query_c_str = unsafe { CStr::from_ptr(query) };
    let query_str = match query_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{query_c_str:?}' to a Rust string.",
            );
            return std::ptr::null_mut();
        }
    };

    let search_style_c_str = unsafe { CStr::from_ptr(search_style) };
    let search_style_str = match search_style_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Could not convert '{search_style_c_str:?}' to a Rust string.",
            );
            return std::ptr::null_mut();
        }
    };

    let search_style = match search_style_str {
        "strict" => rockyou2024::search::SearchStyle::Strict,
        "case-insensitive" => rockyou2024::search::SearchStyle::CaseInsensitive,
        "fuzzy" => rockyou2024::search::SearchStyle::Fuzzy,
        _ => {
            rockyou2024::warn!(
                target: LOG_TARGET,
                "Unknown search style '{search_style_str}'.",
            );
            return std::ptr::null_mut();
        }
    };

    // Perform the search.
    let index_collection = rockyou2024::models::IndexCollection::<
        { config::INDEX_LENGTH },
        { config::INDEX_DEPTH },
    >::new(path.to_path_buf());

    let found = index_collection.find_lines_containing(query_str, search_style);

    vec_str_to_mut_mut_c_char!(<rockyou2024::models::IndexCollectionResult as Clone>::clone(&found))
}
