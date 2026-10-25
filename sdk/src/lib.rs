//! craftkit (a working name): write Minecraft mods in Rust.
//!
//! A Rust mod is a crate that implements [`Mod`] and exports it with [`export_mod!`], built as a
//! shared library and shipped in a content jar beside its assets and a manifest. The craftkit
//! host, a Java mod for Fabric and NeoForge (`../host`), loads the library through Java's FFM
//! API and talks to it through the C ABI in [`abi`]: once a tick it hands the mod what the world
//! did ([`protocol`]'s observations) and takes back what the mod wants done (its commands). The
//! mod never touches Minecraft's classes, so it is tested with [`harness`] in `cargo test`, and
//! survives Minecraft versions the host is updated for.
//!
//! The phases of `../docs/todo.md` build this up; phase 0 is the ABI's shape, `init`, and the
//! harness calling through the exported functions as the host would.

pub mod abi;
pub mod harness;
pub mod protocol;

pub use protocol::{InitReply, ModInfo};

/// The version of the SDK crate, which [`ModInfo`] carries so the host can log it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the C ABI. The host refuses a library built against another one, so a change
/// to any exported function's signature or any JSON shape bumps this and `docs/sdk.md`'s table.
pub const ABI_VERSION: u32 = 1;

/// What a Rust mod implements. One value per process, made by [`Mod::init`] when the host
/// starts the mod, and asked by the host through the functions [`export_mod!`] exports.
///
/// Later phases add the per-world calls (`open`, `observe`, `tick`, `report`, `save`, `close`)
/// as `docs/architecture.md` lays them out; phase 0 has `init` and `info`.
pub trait Mod: Sized + Send + 'static {
    /// The mod's name and version for the host's log and `/craftkit status`.
    fn info() -> ModInfo;

    /// Reads the mod's config file as text (empty when there is none) and makes the mod. The
    /// reply carries the effective config, every key present, which the host writes to a
    /// missing file, and the unknown keys, which the host logs once. An `Err` is logged and the
    /// mod is not loaded.
    fn init(config: &str) -> Result<(Self, InitReply), String>;
}

/// Exports a [`Mod`] as the C ABI the host loads. Exactly one call per shared library.
///
/// ```ignore
/// struct MyMod;
/// impl craftkit::Mod for MyMod { /* ... */ }
/// craftkit::export_mod!(MyMod);
/// ```
#[macro_export]
macro_rules! export_mod {
    ($mod:ty) => {
        /// The slot the exported functions keep the one mod in.
        static CRAFTKIT_MOD: ::std::sync::Mutex<Option<$mod>> = ::std::sync::Mutex::new(None);

        #[no_mangle]
        pub extern "C" fn craftkit_abi_version() -> u32 {
            $crate::ABI_VERSION
        }

        #[no_mangle]
        pub extern "C" fn craftkit_info() -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| Ok(<$mod as $crate::Mod>::info()))
        }

        /// # Safety
        /// `config` points to `len` readable bytes, or is null with `len` zero.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_init(
            config: *const u8,
            len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let text = unsafe { $crate::abi::text(config, len) }?;
                let (made, reply) = <$mod as $crate::Mod>::init(text)?;
                let mut slot = CRAFTKIT_MOD.lock().unwrap_or_else(|err| err.into_inner());
                *slot = Some(made);

                Ok(reply)
            })
        }

        #[no_mangle]
        pub extern "C" fn craftkit_shutdown() {
            let mut slot = CRAFTKIT_MOD.lock().unwrap_or_else(|err| err.into_inner());
            *slot = None;
        }

        /// # Safety
        /// `s` was returned by this library's ABI and not yet freed, or is null.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_free(s: *mut ::std::ffi::c_char) {
            unsafe { $crate::abi::free(s) }
        }
    };
}
