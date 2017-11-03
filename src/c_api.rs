use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;
use std::panic::catch_unwind;
use std::sync::Mutex;

use sequences::Sequence;

type SequenceHandle = u32;
type ResultCodeRaw = u8;

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

#[no_mangle]
pub extern fn parse(s: &str) -> ResultCodeRaw {
    let mut ids = IDSBYNAME.lock().unwrap();
    let mut sequences = SEQSBYID.lock().unwrap();
    ::parse_assignments(s, &mut *ids, &mut *sequences);

    ResultCode::Success.value()
}

#[no_mangle]
pub extern fn lookup(name: &str, handle_ptr: *mut SequenceHandle) -> ResultCodeRaw {
    if handle_ptr.is_null() {
        return ResultCode::NullPointer.value()
    }

    let mut ids = IDSBYNAME.lock().unwrap();

    if let Occupied(entry) = ids.entry(name.into()) {
        let id = *entry.get() as SequenceHandle;

        unsafe {
            *handle_ptr = id;
        };

        ResultCode::Success.value()
    } else {
        ResultCode::NotFound.value()
    }
}

#[no_mangle]
pub extern fn next(handle: SequenceHandle, result_ptr: *mut u32) -> ResultCodeRaw {
    if result_ptr.is_null() {
        return ResultCode::NullPointer.value()
    }

    let mut sequences = SEQSBYID.lock().unwrap();

    let idx = handle as usize;
    if sequences.is_empty() || idx > sequences.len() - 1 {
        ResultCode::NotFound.value()
    } else {
        let value = sequences[idx].next();
        unsafe { *result_ptr = value; };

        ResultCode::Success.value()
    }
}

#[cfg(test)]
mod tests {
    mod parse {
        use super::super::*;
        use std::collections::hash_map::Entry::Occupied;

        #[test]
        fn basic() {
            assert!(IDSBYNAME.lock().unwrap().is_empty());
            assert!(SEQSBYID.lock().unwrap().is_empty());

            let result_code = parse("a=5;");
            assert_eq!(result_code, ResultCode::Success.value());

            let mut ids = IDSBYNAME.lock().unwrap();
            let mut sequences = SEQSBYID.lock().unwrap();
            assert!(ids.contains_key("a"));
            if let Occupied(entry) = ids.entry("a".into()) {
                let id = entry.get();
                let value = sequences[*id].next();
                assert_eq!(value, 5);
            }

            ids.clear();
            sequences.clear();
        }

        #[test]
        fn range() {
            assert!(IDSBYNAME.lock().unwrap().is_empty());
            assert!(SEQSBYID.lock().unwrap().is_empty());

            let result_code = parse("a=[0,1];");
            assert_eq!(result_code, ResultCode::Success.value());

            let mut ids = IDSBYNAME.lock().unwrap();
            let mut sequences = SEQSBYID.lock().unwrap();
            assert!(ids.contains_key("a"));
            if let Occupied(entry) = ids.entry("a".into()) {
                let id = entry.get();
                let value = sequences[*id].next();
                assert!(value == 0 || value == 1);
            }

            ids.clear();
            sequences.clear();
        }
    }

    mod lookup {
        use super::super::*;
        use std::ptr;

        #[test]
        fn not_found() {
            assert!(IDSBYNAME.lock().unwrap().is_empty());
            assert!(SEQSBYID.lock().unwrap().is_empty());

            let mut handle: SequenceHandle = 0;
            let handle_ptr: *mut SequenceHandle = &mut handle;
            assert_eq!(lookup("a", handle_ptr), ResultCode::NotFound.value());
        }

        #[test]
        fn found() {
            assert!(IDSBYNAME.lock().unwrap().is_empty());
            assert!(SEQSBYID.lock().unwrap().is_empty());

            let result_code = parse("a=5;");
            assert_eq!(result_code, 0);

            let mut handle: SequenceHandle = 0;
            let handle_ptr: *mut SequenceHandle = &mut handle;
            assert_eq!(lookup("a", handle_ptr), ResultCode::Success.value());

            IDSBYNAME.lock().unwrap().clear();
            SEQSBYID.lock().unwrap().clear();
        }

        #[test]
        fn null_handle() {
            let handle_ptr: *mut SequenceHandle = ptr::null_mut();
            let result_code = lookup("a", handle_ptr);
            assert_eq!(result_code, ResultCode::NullPointer.value());
        }
    }

    mod next {
        use super::super::*;
        use std::ptr;

        #[test]
        fn found() {
            assert!(IDSBYNAME.lock().unwrap().is_empty());
            assert!(SEQSBYID.lock().unwrap().is_empty());

            let result_code = parse("a=5;");
            assert_eq!(result_code, ResultCode::Success.value());

            let mut handle: SequenceHandle = 0;
            let handle_ptr: *mut SequenceHandle = &mut handle;
            let result_code = lookup("a", handle_ptr);
            assert_eq!(result_code, ResultCode::Success.value());

            let mut value: u32 = 0;
            let value_ptr: *mut u32 = &mut value;
            let result_code = next(handle, value_ptr);
            assert_eq!(result_code, ResultCode::Success.value());
            assert_eq!(value, 5);

            IDSBYNAME.lock().unwrap().clear();
            SEQSBYID.lock().unwrap().clear();
        }

        #[test]
        fn not_found() {
            assert!(IDSBYNAME.lock().unwrap().is_empty());
            assert!(SEQSBYID.lock().unwrap().is_empty());

            let handle = 0;
            let mut value: u32 = 0;
            let value_ptr: *mut u32 = &mut value;
            let result_code = next(handle, value_ptr);
            assert_eq!(result_code, ResultCode::NotFound.value());
            assert_eq!(value, 0);
        }

        #[test]
        fn null_result() {
            let handle = 0;
            let value_ptr: *mut u32 = ptr::null_mut();
            let result_code = next(handle, value_ptr);
            assert_eq!(result_code, ResultCode::NullPointer.value());
        }
    }
}
