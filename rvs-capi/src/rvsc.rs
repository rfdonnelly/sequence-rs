//! Rvs C API
//!
//! Provides a C API for parsing and evaluating random variables.
//!
//! # Examples
//!
//! ```
//! use rvsc::*;
//! use std::ffi::CString;
//!
//! // Create a new context and error
//! let context = rvs_context_new();
//! let error = rvs_error_new();
//!
//! // Define a variable "a" as a constant value 5.
//! rvs_parse(context, CString::new("a=5;").unwrap().as_ptr(), error);
//! assert_eq!(rvs_error_code(error), ErrorKind::None.code());
//!
//! // Find the variable "a"
//! let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
//! assert!(handle != 0);
//!
//! // Evaluate the variable "a"
//! let result = rvs_next(context, handle);
//! assert_eq!(result, 5);
//!
//! // Free the error and context
//! rvs_error_free(error);
//! rvs_context_free(context);
//! ```

use std::panic::catch_unwind;
use libc::uint32_t;
use libc::c_char;
use std::ffi::CStr;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use rvs::types::RvC;
use rvs::types::Context;
use rvs::types::Seed;
use rvs::parse_rvs;

use error::Error;
use error::ErrorKind;

type SequenceHandle = uint32_t;

/// Allocates and returns a new context.
///
/// The caller owns the context and must call `rvs_context_free()` to free the context.
///
/// # Examples
///
/// ```
/// # use rvsc::*;
/// let context = rvs_context_new();
/// // ...
/// rvs_context_free(context);
/// ```
#[no_mangle]
pub extern fn rvs_context_new() -> *mut Context {
    Box::into_raw(Box::new(
        Context::new()
    ))
}

/// Frees a context.
///
/// # Examples
///
/// ```
/// # use rvsc::*;
/// let context = rvs_context_new();
/// // ...
/// rvs_context_free(context);
/// ```
#[no_mangle]
pub extern fn rvs_context_free(context: *mut Context) {
    if context.is_null() { return }
    unsafe { Box::from_raw(context); }
}

/// Sets the seed for all future calls to `rvs_parse()`.
///
/// Should be called before `rvs_parse()`.
///
/// # Examples
///
/// ```
/// # use rvsc::*;
/// let context = rvs_context_new();
/// rvs_seed(context, 0);
/// // ...
/// rvs_context_free(context);
/// ```
#[no_mangle]
pub extern fn rvs_seed(context: *mut Context, seed: u32) {
    assert!(!context.is_null());

    let context = unsafe { &mut *context };
    context.seed = Seed::from_u32(seed);
}

/// Parses a semicolon delimited string of Rvs statements and/or Rvs files.
///
/// A terminating semicolon is optional.
///
/// # Errors
///
/// * Returns ResultCode::Success on success
/// * Returns ResultCode::ParseError if string is not valid Rvs DSL.
///
/// # Panics
///
/// If any pointer arguments are null.
///
/// # Examples
///
/// A single Rvs statement:
///
/// "a = 5"
///
/// A single Rvs file:
///
/// "example.rvs"
///
/// An Rvs file and an Rvs statement:
///
/// "example.rvs; a = 5;"
#[no_mangle]
pub extern fn rvs_parse(
    context: *mut Context,
    s: *const c_char,
    error: *mut Error
) {
    assert!(!context.is_null());
    assert!(!s.is_null());

    let c_str = unsafe { CStr::from_ptr(s) };
    let r_str = c_str.to_str().unwrap();

    let mut context = unsafe { &mut *context };

    for entry in r_str.split(';') {
        if !entry.is_empty() {
            let is_file = !entry.contains("=") && !entry.contains("require");

            let parser_string =
                if is_file {
                    let path = Path::new(&entry);
                    if !path.exists() {
                        panic!("path does not exist: {}", path.display());
                    }

                    let mut file = match File::open(&path) {
                        Err(e) => panic!("could not open {}: {}", path.display(), ::std::error::Error::description(&e)),
                        Ok(file) => file,
                    };

                    let mut contents = String::new();
                    match file.read_to_string(&mut contents) {
                        Err(e) => panic!("could not read {}: {}", path.display(), ::std::error::Error::description(&e)),
                        Ok(_) => (),
                    };

                    contents
                } else {
                    entry.to_owned() + ";"
                };

            match parse_rvs(&parser_string, &mut context) {
                Ok(_) => (),
                Err(e) => {
                    unsafe {
                        if !error.is_null() {
                            *error = Error::new(ErrorKind::Parse(e))
                        }
                        // FIXME: Add the following to the error:
                        // println!("{}", e);
                        // println!("{}", parser_string.lines().nth(e.line - 1).unwrap());
                        // for _ in 0..e.column-1 { print!(" "); }
                        // println!("^");
                        //
                        // To do so:
                        //
                        // 1. Convert rust-peg ParseError into an Rvs ParseError
                        // 2. Pass the Rvs ParseError
                    }
                },
            }
        }
    }
}

/// Returns the handle of a variable
///
/// The callee owns the handle.  The handle is valid until `rvs_context_free()` is called.
///
/// # Errors
///
/// * Returns 0 if variable does not exist
///
/// # Panics
///
/// * If any pointer arguments are null
#[no_mangle]
pub extern fn rvs_find(context: *mut Context, id: *const c_char) -> SequenceHandle {
    assert!(!context.is_null());
    assert!(!id.is_null());

    let id_cstr = unsafe { CStr::from_ptr(id) };
    let id_rstr = id_cstr.to_str().unwrap();

    let context = unsafe { &mut *context };
    if let Some(handle) = context.handles.get(id_rstr) {
        let handle = *handle as SequenceHandle;
        handle + 1
    } else {
        0
    }
}

/// Returns the next value of a variable via the result pointer
///
/// # Errors
///
/// Returns 0 if handle is invalid.
///
/// # Panics
///
/// * If any pointer arguments are null
/// * If handle doesn't exist
#[no_mangle]
pub extern fn rvs_next(context: *mut Context, handle: SequenceHandle) -> u32 {
    assert!(!context.is_null());

    let context = unsafe { &mut *context };
    assert!(handle > 0 && handle <= (context.variables.len() as u32));

    match handle_to_idx(&context.variables, handle) {
        Some(idx) => context.variables[idx].next(),
        None => 0,
    }
}

/// Returns the previous value of a variable
///
/// # Errors
///
/// * Returns 0 if handle is invalid
/// * Returns 0 if `rvs_next` has not been called
///
/// # Panics
///
/// * If any pointer arguments are null
/// * If handle doesn't exist
#[no_mangle]
pub extern fn rvs_prev(context: *mut Context, handle: SequenceHandle) -> u32 {
    assert!(!context.is_null());

    let context = unsafe { &mut *context };
    assert!(handle > 0 && handle <= (context.variables.len() as u32));

    match handle_to_idx(&context.variables, handle) {
        Some(idx) => context.variables[idx].prev(),
        None => 0,
    }
}

/// Returns the done value of a variable via the result pointer
///
/// # Errors
///
/// * Returns false if handle is invalid
/// * Returns false if `rvs_next` has not been called
///
/// # Panics
///
/// * If any pointer arguments are null
/// * If handle doesn't exist
#[no_mangle]
pub extern fn rvs_done(context: *mut Context, handle: SequenceHandle) -> bool {
    assert!(!context.is_null());

    let context = unsafe { &mut *context };
    assert!(handle > 0 && handle <= (context.variables.len() as u32));

    match handle_to_idx(&context.variables, handle) {
        Some(idx) => context.variables[idx].done(),
        None => false,
    }
}

fn handle_to_idx(variables: &Vec<Box<RvC>>, handle: SequenceHandle) -> Option<usize> {
    let handle = handle as usize;
    if variables.is_empty() || handle == 0 || handle > variables.len() {
        Option::None
    } else {
        Some(handle - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::CString;

    use error::{
        rvs_error_new,
        rvs_error_free,
        rvs_error_code,
    };

    fn next_by_name(context: *mut Context, name: &str) -> u32 {
        let handle = rvs_find(context, CString::new(name).unwrap().as_ptr());
        assert!(handle != 0);

        rvs_next(context, handle)
    }

    mod rvs_seed {
        use super::*;

        fn next(seed: u32, s: &str) -> u32 {
            let context = rvs_context_new();
            let error = rvs_error_new();
            rvs_seed(context, seed);

            let s = format!("a = {};", s);
            rvs_parse(context, CString::new(s).unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
            assert!(handle != 0);

            let value = rvs_next(context, handle);

            rvs_error_free(error);
            rvs_context_free(context);

            value
        }

        #[test]
        fn basic() {
            let s = "[0, 0xffff_ffff]";

            let seed0_value0 = next(0, s);
            let seed1_value0 = next(1, s);
            let seed0_value1 = next(0, s);
            let seed1_value1 = next(1, s);

            assert!(seed0_value0 != seed1_value0);
            assert_eq!(seed0_value0, seed0_value1);
            assert_eq!(seed1_value0, seed1_value1);
        }
    }

    mod rvs_parse {
        use super::*;

        // FIXME Cannot test this due to error parameter being mutable while std::ptr::null() is
        // not.
        // #[test]
        // fn null_error_struct() {
        //     use std::ptr;
        //
        //     let context = rvs_context_new();
        //     rvs_parse(context, CString::new("a=5;").unwrap().as_ptr(), ptr::null());
        //     assert_eq!(next_by_name(context, "a"), 5);
        // }

        #[test]
        fn require() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("require '../examples/require.rvs'").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            assert_eq!(next_by_name(context, "a"), 5);
            assert_eq!(next_by_name(context, "b"), 1);

            rvs_error_free(error);
            rvs_context_free(context);
        }

        #[test]
        fn basic() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("a=5;").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handles = unsafe { &mut (*context).handles };
            let variables = unsafe { &mut (*context).variables };
            assert!(handles.contains_key("a"));
            let id = handles.get("a").unwrap();
            let value = variables[*id].next();
            assert_eq!(value, 5);

            rvs_error_free(error);
            rvs_context_free(context);
        }

        #[test]
        fn range() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("a=[0,1];").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handles = unsafe { &mut (*context).handles };
            let variables = unsafe { &mut (*context).variables };
            assert!(handles.contains_key("a"));
            let id = handles.get("a").unwrap();
            let value = variables[*id].next();
            assert!(value == 0 || value == 1);

            rvs_error_free(error);
            rvs_context_free(context);
        }

        #[test]
        fn parse_error() {
            use rvs::grammar::ParseError;

            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("a = 1;\n1 = b;").unwrap().as_ptr(), error);
            // FIXME: Check error message
            // println!("{}", unsafe { *error });
            // assert_eq!(rvs_error_code(error), ErrorKind::Parse(ParseError::new()).code());
            assert!(rvs_error_code(error) != ErrorKind::None.code());

            rvs_error_free(error);
            rvs_context_free(context);
        }

        #[test]
        fn file() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("../examples/basic.rvs;b = 3").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
            assert!(handle != 0);

            let value = rvs_next(context, handle);
            assert_eq!(value, 5);

            let handle = rvs_find(context, CString::new("b").unwrap().as_ptr());
            assert!(handle != 0);

            let value = rvs_next(context, handle);
            assert_eq!(value, 3);

            rvs_error_free(error);
            rvs_context_free(context);
        }

        #[test]
        fn override_rv() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("a = 0;a = 1").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            rvs_parse(context, CString::new("a = 2").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
            assert!(handle != 0);

            let value = rvs_next(context, handle);
            assert_eq!(value, 2);

            rvs_error_free(error);
            rvs_context_free(context);
        }
    }

    mod rvs_find {
        use super::*;

        #[test]
        fn not_found() {
            let context = rvs_context_new();

            let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
            assert_eq!(handle, 0);

            rvs_context_free(context);
        }

        #[test]
        fn found() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("a=5;").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
            assert!(handle != 0);

            rvs_error_free(error);
            rvs_context_free(context);
        }
    }

    mod rvs_next {
        use super::*;

        #[test]
        fn found() {
            let context = rvs_context_new();
            let error = rvs_error_new();

            rvs_parse(context, CString::new("a=5;").unwrap().as_ptr(), error);
            assert_eq!(rvs_error_code(error), ErrorKind::None.code());

            let handle = rvs_find(context, CString::new("a").unwrap().as_ptr());
            assert!(handle != 0);

            let value = rvs_next(context, handle);
            assert_eq!(value, 5);

            rvs_error_free(error);
            rvs_context_free(context);
        }

        #[test]
        #[should_panic]
        fn not_found() {
            let context = rvs_context_new();

            let handle = 1;
            let value = rvs_next(context, handle);
            assert_eq!(value, 0);

            rvs_context_free(context);
        }
    }
}
