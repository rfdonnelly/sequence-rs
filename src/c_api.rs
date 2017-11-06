//! Sequence C API
//!
//! Provides a C API for parsing and evaluating sequences.
//!
//! # Examples
//!
//! ```
//! // Create a new context
//! let context = sequence_context_new();
//!
//! // Define a sequence "a" as a constant value 5.
//! let char_str = CString::new("a=5;").unwrap().as_ptr();
//! let result_code = sequence_parse(context, char_str);
//! assert_eq!(result_code, 0);
//!
//! // Find the sequence "a"
//! let char_str = CString::new("a").unwrap().as_ptr();
//! let handle = 0;
//! let result_code = sequence_find(context, char_str, &mut handle);
//! assert_eq!(result_code, 0);
//!
//! // Evaluate the sequence "a"
//! let result = 0;
//! let result_code = sequence_next(context, handle, &mut result);
//! assert_eq!(result_code, 0);
//! assert_eq!(result, 5);
//!
//! // Free the context
//! sequence_context_free(context);
//! ```

use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;
use std::panic::catch_unwind;

use sequences::Sequence;
use ::parse_assignments;

use libc::uint32_t;
use libc::c_char;

use std::ffi::CStr;

type SequenceHandle = uint32_t;
type ResultCodeRaw = uint32_t;

enum ResultCode {
    Success,
    NotFound,
    ParseError,
}

impl ResultCode {
    fn value(&self) -> ResultCodeRaw {
        match *self {
            ResultCode::Success => 0,
            ResultCode::NotFound => 1,
            ResultCode::ParseError => 2,
        }
    }
}

pub struct Context {
    sequences: Vec<Box<Sequence>>,
    ids: HashMap<String, usize>,
}

/// Allocates and returns a new context.
///
/// The caller owns the context and must call `sequence_context_free()` to free the context.
#[no_mangle]
pub extern fn sequence_context_new() -> *mut Context {
    Box::into_raw(Box::new(
        Context {
            sequences: Vec::new(),
            ids: HashMap::new(),
        }
    ))
}

/// Frees a context.
#[no_mangle]
pub extern fn sequence_context_free(context: *mut Context) {
    if context.is_null() { return }
    unsafe { Box::from_raw(context); }
}

/// Passes a string to the sequence parser.
///
/// The string is expected to be valid Sequence DSL.
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::ParseError if string is not valid Sequence DSL.
///
/// # Panics
///
/// If any pointer arguments are null.
#[no_mangle]
pub extern fn sequence_parse(context: *mut Context, s: *const c_char) -> ResultCodeRaw {
    assert!(!context.is_null());
    assert!(!s.is_null());

    let c_str = unsafe { CStr::from_ptr(s) };
    let r_str = c_str.to_str().unwrap();

    let mut context = unsafe { &mut *context };
    match parse_assignments(r_str, &mut context.ids, &mut context.sequences) {
        Ok(_) => ResultCode::Success.value(),
        Err(e) => {
            println!("{}", e);
            println!("{}", r_str.lines().nth(e.line - 1).unwrap());
            for _ in 0..e.column-1 { print!(" "); }
            println!("^");

            ResultCode::ParseError.value()
        },
    }
}

/// Returns the handle of a sequence via the handle pointer
///
/// The callee owns the handle.  The handle is valid until one of the following occurs:
///
/// * `sequence_clear()` is called
/// * The process terminates
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::NotFound if the sequence name is not found.
///
/// # Panics
///
/// If any pointer arguments are null.
#[no_mangle]
pub extern fn sequence_find(context: *mut Context, name: *const c_char, handle_ptr: *mut SequenceHandle) -> ResultCodeRaw {
    assert!(!context.is_null());
    assert!(!name.is_null());
    assert!(!handle_ptr.is_null());

    let c_str = unsafe { CStr::from_ptr(name) };
    let r_str = c_str.to_str().unwrap();

    let mut context = unsafe { &mut *context };
    if let Occupied(entry) = context.ids.entry(r_str.into()) {
        let id = *entry.get() as SequenceHandle;

        unsafe {
            *handle_ptr = id + 1;
        };

        ResultCode::Success.value()
    } else {
        ResultCode::NotFound.value()
    }
}

/// Returns the next value of a sequence via the result pointer
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::NotFound if the handle is not valid
///
/// # Panics
///
/// If any pointer arguments are null.
#[no_mangle]
pub extern fn sequence_next(context: *mut Context, handle: SequenceHandle, result_ptr: *mut u32) -> ResultCodeRaw {
    assert!(!context.is_null());
    assert!(!result_ptr.is_null());

    let mut context = unsafe { &mut *context };
    let idx = match handle_to_idx(&context.sequences, handle) {
        Some(x) => x,
        None => { return ResultCode::NotFound.value(); },
    };

    let value = context.sequences[idx].next();
    unsafe { *result_ptr = value; };

    ResultCode::Success.value()
}

/// Returns the previous value of a sequence via the result pointer
///
/// If `sequence_next()` has not been called on the same sequence handle previously, the result
/// with be `0`.
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::NotFound if the handle is not valid
///
/// # Panics
///
/// If any pointer arguments are null.
#[no_mangle]
pub extern fn sequence_prev(context: *mut Context, handle: SequenceHandle, result_ptr: *mut u32) -> ResultCodeRaw {
    assert!(!context.is_null());
    assert!(!result_ptr.is_null());

    let context = unsafe { &mut *context };
    let idx = match handle_to_idx(&context.sequences, handle) {
        Some(x) => x,
        None => { return ResultCode::NotFound.value(); },
    };

    let value = context.sequences[idx].prev();
    unsafe { *result_ptr = value; };

    ResultCode::Success.value()
}

/// Returns the done value of a sequence via the result pointer
///
/// If `sequence_next()` has not been called on the same sequence handle previously, the result
/// with be `0`.
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::NotFound if the handle is not valid
///
/// # Panics
///
/// If any pointer arguments are null.
#[no_mangle]
pub extern fn sequence_done(context: *mut Context, handle: SequenceHandle, result_ptr: *mut bool) -> ResultCodeRaw {
    assert!(!context.is_null());
    assert!(!result_ptr.is_null());

    let context = unsafe { &mut *context };
    let idx = match handle_to_idx(&context.sequences, handle) {
        Some(x) => x,
        None => { return ResultCode::NotFound.value(); },
    };

    let value = context.sequences[idx].done();
    unsafe { *result_ptr = value; };

    ResultCode::Success.value()
}

/// Clears all state and all parsed sequences.
#[no_mangle]
pub extern fn sequence_clear(context: *mut Context) {
    let mut context = unsafe { &mut *context };
    context.ids.clear();
    context.sequences.clear();
}

fn handle_to_idx(sequences: &Vec<Box<Sequence>>, handle: SequenceHandle) -> Option<usize> {
    let handle = handle as usize;
    if sequences.is_empty() || handle == 0 || handle > sequences.len() {
        Option::None
    } else {
        Some(handle - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sequence_parse {
        use super::*;

        use std::ffi::CString;
        use std::collections::hash_map::Entry::Occupied;

        #[test]
        fn basic() {
            let context = sequence_context_new();

            let result_code = sequence_parse(context, CString::new("a=5;").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::Success.value());

            let mut ids = unsafe { &mut (*context).ids };
            let mut sequences = unsafe { &mut (*context).sequences };
            assert!(ids.contains_key("a"));
            if let Occupied(entry) = ids.entry("a".into()) {
                let id = entry.get();
                let value = sequences[*id].next();
                assert_eq!(value, 5);
            }

            sequence_context_free(context);
        }

        #[test]
        fn range() {
            let context = sequence_context_new();

            let result_code = sequence_parse(context, CString::new("a=[0,1];").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::Success.value());

            let mut ids = unsafe { &mut (*context).ids };
            let mut sequences = unsafe { &mut (*context).sequences };
            assert!(ids.contains_key("a"));
            if let Occupied(entry) = ids.entry("a".into()) {
                let id = entry.get();
                let value = sequences[*id].next();
                assert!(value == 0 || value == 1);
            }

            sequence_context_free(context);
        }

        #[test]
        fn parse_error() {
            let context = sequence_context_new();

            let result_code = sequence_parse(context, CString::new("a = 1;\n1 = b;").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::ParseError.value());

            sequence_context_free(context);
        }
    }

    mod sequence_find {
        use super::*;

        use std::ffi::CString;

        #[test]
        fn not_found() {
            let context = sequence_context_new();

            let mut handle: SequenceHandle = 0;
            let result_code = sequence_find(context, CString::new("a").unwrap().as_ptr(), &mut handle);
            assert_eq!(result_code, ResultCode::NotFound.value());

            sequence_context_free(context);
        }

        #[test]
        fn found() {
            let context = sequence_context_new();

            let result_code = sequence_parse(context, CString::new("a=5;").unwrap().as_ptr());
            assert_eq!(result_code, 0);

            let mut handle: SequenceHandle = 0;
            let result_code = sequence_find(context, CString::new("a").unwrap().as_ptr(), &mut handle);
            assert_eq!(handle, 1);
            assert_eq!(result_code, ResultCode::Success.value());

            sequence_context_free(context);
        }
    }

    mod sequence_next {
        use super::*;

        use std::ffi::CString;

        #[test]
        fn found() {
            let context = sequence_context_new();

            let result_code = sequence_parse(context, CString::new("a=5;").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::Success.value());

            let mut handle: SequenceHandle = 0;
            let result_code = sequence_find(context, CString::new("a").unwrap().as_ptr(), &mut handle);
            assert_eq!(result_code, ResultCode::Success.value());

            let mut value: u32 = 0;
            let result_code = sequence_next(context, handle, &mut value);
            assert_eq!(result_code, ResultCode::Success.value());
            assert_eq!(value, 5);

            sequence_context_free(context);
        }

        #[test]
        fn not_found() {
            let context = sequence_context_new();

            let handle = 1;
            let mut value: u32 = 0;
            let result_code = sequence_next(context, handle, &mut value);
            assert_eq!(result_code, ResultCode::NotFound.value());
            assert_eq!(value, 0);

            sequence_context_free(context);
        }
    }
}
