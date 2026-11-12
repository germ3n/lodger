//! Drives a mod the way the host does, in a test, with no game.
//!
//! [`Harness`] goes through the exported functions of a library, not the [`crate::Mod`] trait,
//! so what a test exercises is the same path the host takes: bytes in, JSON out, panics caught.
//! A mod's own tests build one with [`crate::harness!`] and ask it questions; the SDK's tests do the
//! same with a mod of their own (`tests/exported.rs`).

use std::ffi::c_char;

/// The exported functions of one library, as the host sees them.
pub struct Exports {
    pub abi_version: extern "C" fn() -> u32,
    pub info: extern "C" fn() -> *mut c_char,
    pub init: unsafe extern "C" fn(*const u8, usize) -> *mut c_char,
    pub warm: extern "C" fn() -> *mut c_char,
    pub status: extern "C" fn() -> *mut c_char,
    pub world_open: unsafe extern "C" fn(*const u8, usize, *const u8, usize) -> *mut c_char,
    pub world_observe: unsafe extern "C" fn(u64, *const u8, usize) -> *mut c_char,
    pub world_tick: unsafe extern "C" fn(u64, *const u8, usize) -> *mut c_char,
    pub world_report: unsafe extern "C" fn(u64, *const u8, usize) -> *mut c_char,
    pub world_debug: unsafe extern "C" fn(u64, *const u8, usize) -> *mut c_char,
    pub world_save: extern "C" fn(u64) -> *mut c_char,
    pub world_close: extern "C" fn(u64) -> *mut c_char,
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

    /// Opens a world as the host does and answers a driver for it, or the error.
    pub fn open_world(&self, info: &crate::WorldInfo, saved: &[u8]) -> Result<World<'_>, String> {
        let text = serde_json::to_string(info).expect("WorldInfo serializes");
        let answer = unsafe {
            crate::abi::take((self.exports.world_open)(
                text.as_ptr(),
                text.len(),
                saved.as_ptr(),
                saved.len(),
            ))
        };

        if let Some(error) = answer.get("error") {
            return Err(error.as_str().unwrap_or("error").to_string());
        }

        Ok(World {
            harness: self,
            handle: answer["handle"].as_u64().expect("a handle"),
        })
    }

    pub fn shutdown(&self) {
        (self.exports.shutdown)()
    }
}

/// One open world, driven as the host drives it: observe, tick, report, save, close.
pub struct World<'a> {
    harness: &'a Harness,
    pub handle: u64,
}

impl World<'_> {
    pub fn observe(&self, observations: &crate::Observations) -> serde_json::Value {
        let text = serde_json::to_string(observations).expect("Observations serialize");

        unsafe {
            crate::abi::take((self.harness.exports.world_observe)(
                self.handle,
                text.as_ptr(),
                text.len(),
            ))
        }
    }

    /// Ticks and answers the commands, or panics on an error answer, which a test wants to see.
    pub fn tick(&self, now: &crate::Now) -> Vec<crate::Command> {
        let text = serde_json::to_string(now).expect("Now serializes");
        let answer = unsafe {
            crate::abi::take((self.harness.exports.world_tick)(
                self.handle,
                text.as_ptr(),
                text.len(),
            ))
        };

        if let Some(error) = answer.get("error") {
            panic!("tick failed: {error}");
        }

        serde_json::from_value(answer).expect("a list of commands")
    }

    pub fn report(&self, reports: &[crate::Report]) -> serde_json::Value {
        let text = serde_json::to_string(reports).expect("Reports serialize");

        unsafe {
            crate::abi::take((self.harness.exports.world_report)(
                self.handle,
                text.as_ptr(),
                text.len(),
            ))
        }
    }

    /// Asks the debug question as the screen does and answers the reply, or panics on an error
    /// answer.
    pub fn debug(&self, request: &crate::DebugRequest) -> crate::DebugReply {
        let text = serde_json::to_string(request).expect("DebugRequest serializes");
        let answer = unsafe {
            crate::abi::take((self.harness.exports.world_debug)(
                self.handle,
                text.as_ptr(),
                text.len(),
            ))
        };

        if let Some(error) = answer.get("error") {
            panic!("debug failed: {error}");
        }

        serde_json::from_value(answer).expect("a DebugReply")
    }

    /// The world's saved bytes, decoded from the ABI's base64.
    pub fn save(&self) -> Vec<u8> {
        let answer = unsafe { crate::abi::take((self.harness.exports.world_save)(self.handle)) };

        crate::base64::decode(answer["bytes"].as_str().expect("saved bytes")).expect("base64")
    }

    pub fn close(self) -> serde_json::Value {
        unsafe { crate::abi::take((self.harness.exports.world_close)(self.handle)) }
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
            world_open: craftkit_world_open,
            world_observe: craftkit_world_observe,
            world_tick: craftkit_world_tick,
            world_report: craftkit_world_report,
            world_debug: craftkit_world_debug,
            world_save: craftkit_world_save,
            world_close: craftkit_world_close,
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
        unsafe extern "C" fn no_open(
            _: *const u8,
            _: usize,
            _: *const u8,
            _: usize,
        ) -> *mut c_char {
            std::ptr::null_mut()
        }
        unsafe extern "C" fn no_world(_: u64, _: *const u8, _: usize) -> *mut c_char {
            std::ptr::null_mut()
        }
        extern "C" fn no_handle(_: u64) -> *mut c_char {
            std::ptr::null_mut()
        }

        let err = Harness::load(Exports {
            abi_version: other,
            info: none,
            init: no_init,
            warm: none,
            status: none,
            world_open: no_open,
            world_observe: no_world,
            world_tick: no_world,
            world_report: no_world,
            world_debug: no_world,
            world_save: no_handle,
            world_close: no_handle,
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
