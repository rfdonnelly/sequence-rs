//! Sequence C API
//!
//! Provides a C API for parsing and evaluating sequences.
//!
//! # Examples
//!
//! ```
//! // Define a sequence "a" as a constant value 5.
//! let char_str = CString::new("a=5;").unwrap().as_ptr();
//! let result_code = sequence_parse(char_str);
//! assert_eq!(result_code, 0);
//!
//! // Find the sequence "a"
//! let char_str = CString::new("a").unwrap().as_ptr();
//! let handle = 0;
//! let result_code = sequence_find(char_str, &mut handle);
//! assert_eq!(result_code, 0);
//!
//! // Evaluate the sequence "a"
//! let result = 0;
//! let result_code = sequence_next(handle, &mut result);
//! assert_eq!(result_code, 0);
//! assert_eq!(result, 5);
//!
//! sequence_clear();
//! ```

use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;
use std::panic::catch_unwind;
use std::sync::Mutex;
use std::ops::Deref;

use sequences::Sequence;

use libc::uint32_t;
use libc::c_char;

use std::ffi::CStr;

type SequenceHandle = uint32_t;
type ResultCodeRaw = uint32_t;

enum ResultCode {
    Success,
    NotFound,
    NullPointer,
}

impl ResultCode {
    fn value(&self) -> ResultCodeRaw {
        match *self {
            ResultCode::Success => 0,
            ResultCode::NotFound => 1,
            ResultCode::NullPointer => 2,
        }
    }
}

lazy_static! {
    static ref IDSBYNAME: Mutex<HashMap<String, usize>> = {
        Mutex::new(HashMap::new())
    };

    static ref SEQSBYID: Mutex<Vec<Box<Sequence>>> = {
        Mutex::new(Vec::new())
    };
}

/// Passes a string to the sequence parser.
///
/// The string is expected to be valid Sequence DSL.
///
/// Returns ResultCode::Success on success.
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::NullPointer if the string is null.
/// * **[FIXME]** Retunrs ??? if string is not valid Sequence DSL.
#[no_mangle]
pub extern fn sequence_parse(s: *const c_char) -> ResultCodeRaw {
    if s.is_null() {
        return ResultCode::NullPointer.value()
    }

    let c_str = unsafe { CStr::from_ptr(s) };
    let r_str = c_str.to_str().unwrap();

    let mut ids = IDSBYNAME.lock().unwrap();
    let mut sequences = SEQSBYID.lock().unwrap();
    ::parse_assignments(r_str, &mut *ids, &mut *sequences);

    ResultCode::Success.value()
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
/// * Returns ResultCode::NullPointer if the name string or the handle pointer are null.
/// * Returns ResultCode::NotFound if the sequence name is not found.
#[no_mangle]
pub extern fn sequence_find(name: *const c_char, handle_ptr: *mut SequenceHandle) -> ResultCodeRaw {
    if name.is_null() {
        return ResultCode::NullPointer.value()
    }

    if handle_ptr.is_null() {
        return ResultCode::NullPointer.value()
    }

    let c_str = unsafe { CStr::from_ptr(name) };
    let r_str = c_str.to_str().unwrap();

    let mut ids = IDSBYNAME.lock().unwrap();

    if let Occupied(entry) = ids.entry(r_str.into()) {
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
/// * Returns ResultCode::NullPointer if the result pointer is null
/// * Returns ResultCode::NotFound if the handle is not valid
#[no_mangle]
pub extern fn sequence_next(handle: SequenceHandle, result_ptr: *mut u32) -> ResultCodeRaw {
    if result_ptr.is_null() {
        return ResultCode::NullPointer.value()
    }

    let mut sequences = SEQSBYID.lock().unwrap();

    let idx = match handle_to_idx(sequences.deref(), handle) {
        Some(x) => x,
        None => { return ResultCode::NotFound.value(); },
    };

    let value = sequences[idx].next();
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
/// * Returns ResultCode::NullPointer if the result pointer is null
/// * Returns ResultCode::NotFound if the handle is not valid
#[no_mangle]
pub extern fn sequence_prev(handle: SequenceHandle, result_ptr: *mut u32) -> ResultCodeRaw {
    if result_ptr.is_null() {
        return ResultCode::NullPointer.value()
    }

    let sequences = SEQSBYID.lock().unwrap();

    let idx = match handle_to_idx(sequences.deref(), handle) {
        Some(x) => x,
        None => { return ResultCode::NotFound.value(); },
    };

    let value = sequences[idx].prev();
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
/// * Returns ResultCode::NullPointer if the result pointer is null
/// * Returns ResultCode::NotFound if the handle is not valid
#[no_mangle]
pub extern fn sequence_done(handle: SequenceHandle, result_ptr: *mut bool) -> ResultCodeRaw {
    if result_ptr.is_null() {
        return ResultCode::NullPointer.value()
    }

    let sequences = SEQSBYID.lock().unwrap();

    let idx = match handle_to_idx(sequences.deref(), handle) {
        Some(x) => x,
        None => { return ResultCode::NotFound.value(); },
    };

    let value = sequences[idx].done();
    unsafe { *result_ptr = value; };

    ResultCode::Success.value()
}

/// Clears all state and all parsed sequences.
#[no_mangle]
pub extern fn sequence_clear() {
    IDSBYNAME.lock().unwrap().clear();
    SEQSBYID.lock().unwrap().clear();
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

    fn assert_empty() {
        assert!(IDSBYNAME.lock().unwrap().is_empty());
        assert!(SEQSBYID.lock().unwrap().is_empty());
    }

    mod sequence_parse {
        use super::*;

        use std::ffi::CString;
        use std::collections::hash_map::Entry::Occupied;

        #[test]
        fn basic() {
            assert_empty();

            let result_code = sequence_parse(CString::new("a=5;").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::Success.value());

            {
                let mut ids = IDSBYNAME.lock().unwrap();
                assert!(ids.contains_key("a"));
                if let Occupied(entry) = ids.entry("a".into()) {
                    let id = entry.get();
                    let mut sequences = SEQSBYID.lock().unwrap();
                    let value = sequences[*id].next();
                    assert_eq!(value, 5);
                }
            }

            sequence_clear();
        }

        #[test]
        fn range() {
            assert_empty();

            let result_code = sequence_parse(CString::new("a=[0,1];").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::Success.value());

            {
                let mut ids = IDSBYNAME.lock().unwrap();
                assert!(ids.contains_key("a"));
                if let Occupied(entry) = ids.entry("a".into()) {
                    let id = entry.get();
                    let mut sequences = SEQSBYID.lock().unwrap();
                    let value = sequences[*id].next();
                    assert!(value == 0 || value == 1);
                }
            }

            sequence_clear();
        }
    }

    mod sequence_find {
        use super::*;

        use std::ptr;
        use std::ffi::CString;

        #[test]
        fn not_found() {
            assert_empty();

            let mut handle: SequenceHandle = 0;
            let result_code = sequence_find(CString::new("a").unwrap().as_ptr(), &mut handle);
            assert_eq!(result_code, ResultCode::NotFound.value());
        }

        #[test]
        fn found() {
            assert_empty();

            let result_code = sequence_parse(CString::new("a=5;").unwrap().as_ptr());
            assert_eq!(result_code, 0);

            let mut handle: SequenceHandle = 0;
            let result_code = sequence_find(CString::new("a").unwrap().as_ptr(), &mut handle);
            assert_eq!(handle, 1);
            assert_eq!(result_code, ResultCode::Success.value());

            sequence_clear();
        }

        #[test]
        fn null_handle() {
            let handle_ptr: *mut SequenceHandle = ptr::null_mut();
            let result_code = sequence_find(CString::new("a").unwrap().as_ptr(), handle_ptr);
            assert_eq!(result_code, ResultCode::NullPointer.value());
        }
    }

    mod sequence_next {
        use super::*;

        use std::ptr;
        use std::ffi::CString;

        #[test]
        fn found() {
            assert_empty();

            let result_code = sequence_parse(CString::new("a=5;").unwrap().as_ptr());
            assert_eq!(result_code, ResultCode::Success.value());

            let mut handle: SequenceHandle = 0;
            let result_code = sequence_find(CString::new("a").unwrap().as_ptr(), &mut handle);
            assert_eq!(result_code, ResultCode::Success.value());

            let mut value: u32 = 0;
            let result_code = sequence_next(handle, &mut value);
            assert_eq!(result_code, ResultCode::Success.value());
            assert_eq!(value, 5);

            sequence_clear();
        }

        #[test]
        fn not_found() {
            assert_empty();

            let handle = 1;
            let mut value: u32 = 0;
            let result_code = sequence_next(handle, &mut value);
            assert_eq!(result_code, ResultCode::NotFound.value());
            assert_eq!(value, 0);
        }

        #[test]
        fn null_result() {
            let handle = 1;
            let value_ptr: *mut u32 = ptr::null_mut();
            let result_code = sequence_next(handle, value_ptr);
            assert_eq!(result_code, ResultCode::NullPointer.value());
        }
    }
}
