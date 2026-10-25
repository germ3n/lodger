//! Drives a mod the way the host does, in a test, with no game.
//!
//! [`Harness`] goes through the exported functions of a library, not the [`crate::Mod`] trait,
//! so what a test exercises is the same path the host takes: bytes in, JSON out, panics caught.
//! A mod's own tests build one with [`harness!`] and ask it questions; the SDK's tests do the
//! same with a mod of their own (`tests/exported.rs`).

use std::ffi::c_char;

/// The exported functions of one library, as the host sees them.
pub struct Exports {
    pub abi_version: extern "C" fn() -> u32,
    pub info: extern "C" fn() -> *mut c_char,
    pub init: unsafe extern "C" fn(*const u8, usize) -> *mut c_char,
    pub warm: extern "C" fn() -> *mut c_char,
    pub status: extern "C" fn() -> *mut c_char,
    pub shutdown: extern "C" fn(),
}

/// A mod loaded as the host loads it.
pub struct Harness {
    exports: Exports,
}

impl Harness {
    /// Checks the ABI version as the host does, and refuses a mismatch the same way.
    pub fn load(exports: Exports) -> Result<Self, String> {
        let version = (exports.abi_version)();

        if version != crate::ABI_VERSION {
            return Err(format!(
                "the library speaks ABI {version}, this host ABI {}",
                crate::ABI_VERSION
            ));
        }

        Ok(Self { exports })
    }

    pub fn info(&self) -> serde_json::Value {
        unsafe { crate::abi::take((self.exports.info)()) }
    }

    /// Calls `init` with the config text alone, as a test usually wants.
    pub fn init(&self, config: &str) -> serde_json::Value {
        self.init_with(&crate::Init::with_config(config))
    }

    /// Calls `init` with a full `Init`, as the host does.
    pub fn init_with(&self, init: &crate::Init) -> serde_json::Value {
        let text = serde_json::to_string(init).expect("Init serializes");

        unsafe { crate::abi::take((self.exports.init)(text.as_ptr(), text.len())) }
    }

    /// Calls `warm` and answers `true`, or the error, as the host reads it.
    pub fn warm(&self) -> serde_json::Value {
        unsafe { crate::abi::take((self.exports.warm)()) }
    }

    pub fn status(&self) -> serde_json::Value {
        unsafe { crate::abi::take((self.exports.status)()) }
    }

    pub fn shutdown(&self) {
        (self.exports.shutdown)()
    }
}

/// The [`Exports`] of the library [`crate::export_mod!`] generated in the calling crate.
#[macro_export]
macro_rules! harness {
    () => {
        $crate::harness::Harness::load($crate::harness::Exports {
            abi_version: craftkit_abi_version,
            info: craftkit_info,
            init: craftkit_init,
            warm: craftkit_warm,
            status: craftkit_status,
            shutdown: craftkit_shutdown,
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_library_of_another_abi_version_is_refused_by_name() {
        extern "C" fn other() -> u32 {
            crate::ABI_VERSION + 1
        }
        extern "C" fn none() -> *mut c_char {
            std::ptr::null_mut()
        }
        unsafe extern "C" fn no_init(_: *const u8, _: usize) -> *mut c_char {
            std::ptr::null_mut()
        }
        extern "C" fn nothing() {}

        let err = Harness::load(Exports {
            abi_version: other,
            info: none,
            init: no_init,
            warm: none,
            status: none,
            shutdown: nothing,
        })
        .err()
        .unwrap();

        assert!(
            err.contains(&format!("ABI {}", crate::ABI_VERSION + 1)),
            "{err}"
        );
    }
}
