//! What [`crate::export_mod!`]'s functions are built from: reading the host's bytes, answering as
//! a C string the host frees, and turning panics into errors.
//!
//! Every exported function takes UTF-8 bytes by pointer and length and returns a NUL-terminated
//! UTF-8 JSON string the host frees with `craftkit_free`, so the Java side is a few method
//! handles and no struct layouts. Errors are JSON too ([`Failure`]), never a panic across the
//! boundary: each function runs under `catch_unwind`. `serde_json` never emits a NUL byte, so
//! the C string is always whole.

use crate::protocol::Failure;
use serde::Serialize;
use std::ffi::{c_char, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Turns a byte pointer and length into a `&str`, or an error message. A null pointer with a
/// length of zero is the empty string.
///
/// # Safety
/// `ptr` must point to `len` readable bytes, or be null with `len` zero.
pub unsafe fn text<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    if ptr.is_null() {
        return if len == 0 {
            Ok("")
        } else {
            Err("null pointer with a length".to_string())
        };
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };

    std::str::from_utf8(bytes).map_err(|err| format!("not UTF-8: {err}"))
}

/// A bare byte slice the host sent: a world's saved bytes.
///
/// # Safety
/// `ptr` must point to `len` readable bytes, or be null with `len` zero.
pub unsafe fn bytes<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], String> {
    if ptr.is_null() {
        return if len == 0 {
            Ok(&[])
        } else {
            Err("null pointer with a length".to_string())
        };
    }

    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// `{"ok": true}`, the answer to a call that has nothing to say but that it worked. An object,
/// like every other answer, so the host reads one shape.
#[derive(Serialize, serde::Deserialize)]
pub struct Ok {
    pub ok: bool,
}

impl Ok {
    pub fn yes() -> Self {
        Self { ok: true }
    }
}

/// `{"handle": n}`, the answer to opening a world.
#[derive(Serialize, serde::Deserialize)]
pub struct Handle {
    pub handle: u64,
}

/// `{"bytes": "<base64>"}`, the answer to saving a world.
#[derive(Serialize, serde::Deserialize)]
pub struct Saved {
    pub bytes: String,
}

/// Reads JSON the host sent, naming what was wrong when it is not what was expected.
pub fn parse<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
    serde_json::from_str(text).map_err(|err| format!("bad JSON from the host: {err}"))
}

/// Runs `body`, turning a panic or an `Err` into `{"error": ...}` and any result into a C string
/// the host owns until it calls `craftkit_free`.
pub fn answer<T: Serialize>(body: impl FnOnce() -> Result<T, String>) -> *mut c_char {
    let json = match catch_unwind(AssertUnwindSafe(body)) {
        Ok(Ok(value)) => serde_json::to_string(&value)
            .unwrap_or_else(|err| failure(format!("could not encode the answer: {err}"))),
        Ok(Err(error)) => failure(error),
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".to_string());
            failure(format!("the mod panicked: {message}"))
        }
    };

    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"error":"the answer held a NUL byte"}"#).unwrap())
        .into_raw()
}

fn failure(error: String) -> String {
    serde_json::to_string(&Failure { error }).unwrap_or_else(|_| r#"{"error":"error"}"#.to_string())
}

/// Frees a string [`answer`] returned. Null is fine and does nothing.
///
/// # Safety
/// `s` must have come from [`answer`] and not yet been freed, or be null.
pub unsafe fn free(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}

/// Reads an answer back as JSON and frees it, as the host does. For tests and the harness.
///
/// # Safety
/// `s` must have come from [`answer`] and not yet been freed.
pub unsafe fn take(s: *mut c_char) -> serde_json::Value {
    assert!(!s.is_null(), "an exported function answered null");
    let json = unsafe { std::ffi::CStr::from_ptr(s) }
        .to_str()
        .expect("answers are UTF-8")
        .to_string();
    unsafe { free(s) };

    serde_json::from_str(&json).expect("answers are JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call<T: Serialize>(body: impl FnOnce() -> Result<T, String>) -> serde_json::Value {
        unsafe { take(answer(body)) }
    }

    #[test]
    fn a_value_is_answered_as_its_json() {
        assert_eq!(call(|| Ok(vec![1, 2]))["0"], serde_json::Value::Null);
        assert_eq!(call(|| Ok(vec![1, 2])), serde_json::json!([1, 2]));
    }

    #[test]
    fn an_error_is_answered_as_json_with_the_message() {
        assert_eq!(call::<()>(|| Err("nope".to_string()))["error"], "nope");
    }

    #[test]
    fn a_panic_is_answered_as_an_error_with_its_message() {
        let answer = call::<()>(|| panic!("on purpose"));

        assert_eq!(answer["error"], "the mod panicked: on purpose");
    }

    #[test]
    fn text_reads_bytes_and_refuses_what_is_not_utf8() {
        let bytes = "héllo".as_bytes();
        assert_eq!(
            unsafe { text(bytes.as_ptr(), bytes.len()) }.unwrap(),
            "héllo"
        );

        let bad = [0xffu8, 0xfe];
        assert!(unsafe { text(bad.as_ptr(), bad.len()) }
            .unwrap_err()
            .contains("UTF-8"));
    }

    #[test]
    fn a_null_pointer_is_the_empty_string_only_with_no_length() {
        assert_eq!(unsafe { text(std::ptr::null(), 0) }.unwrap(), "");
        assert!(unsafe { text(std::ptr::null(), 3) }.is_err());
    }

    #[test]
    fn freeing_null_does_nothing() {
        unsafe { free(std::ptr::null_mut()) };
    }
}
