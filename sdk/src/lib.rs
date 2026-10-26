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
pub mod base64;
pub mod debug;
pub mod harness;
pub mod protocol;
pub mod world;

pub use debug::{DebugReply, DebugRequest};
pub use protocol::{Init, InitReply, License, ModInfo, Resource, Status};
pub use world::{Command, Now, Observations, Report, WorldInfo};

/// The version of the SDK crate, which [`ModInfo`] carries so the host can log it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The version of the C ABI. The host refuses a library built against another one, so a change
/// to any exported function's signature or any JSON shape bumps this and `docs/sdk.md`'s table.
pub const ABI_VERSION: u32 = 6;

/// What a Rust mod implements. One value per process, made by [`Mod::init`] when the host
/// starts the mod, and asked by the host through the functions [`export_mod!`] exports.
///
/// Later phases add the per-world calls (`open`, `observe`, `tick`, `report`, `save`, `close`)
/// as `docs/architecture.md` lays them out; phase 0 has `init` and `info`.
pub trait Mod: Sized + Send + Sync + 'static {
    /// The mod's name and version for the host's log and `/craftkit status`.
    fn info() -> ModInfo;

    /// Makes the mod from what the host hands it: the config file's text (empty when there is
    /// none) and the game's directories. The reply carries the effective config, every key
    /// present, which the host writes to a missing file, the unknown keys, which the host logs
    /// once, and the resources the host must fetch. An `Err` is logged and the mod is not loaded.
    fn init(init: &Init) -> Result<(Self, InitReply), String>;

    /// Loads whatever the mod loads before play (the lodger: the model), blocking until it is
    /// loaded or failed. The host calls it on a thread of its own once the resources are there,
    /// and polls [`Mod::status`] meanwhile. A mod with nothing to load does nothing.
    fn warm(&self) -> Result<(), String> {
        Ok(())
    }

    /// Whether the mod is ready to play, and one line saying so or why not.
    fn status(&self) -> Status {
        Status {
            ready: true,
            line: "Ready".to_string(),
            ..Status::default()
        }
    }

    /// The mod's state for one world, made by [`Mod::open`] and dropped by [`Mod::close`].
    type World: Send + 'static;

    /// A world loaded, with the bytes [`Mod::save`] last gave for it, or none for a new one.
    fn open(&self, info: &WorldInfo, saved: &[u8]) -> Result<Self::World, String>;

    /// What the host saw since the last tick. Called before [`Mod::tick`] on the same tick.
    fn observe(&self, world: &mut Self::World, observations: &Observations);

    /// One tick: the commands to carry out now, usually none.
    fn tick(&self, world: &mut Self::World, now: &Now) -> Vec<Command>;

    /// How the last tick's commands went.
    fn report(&self, world: &mut Self::World, reports: &[Report]);

    /// The world's state as bytes for the save. Called on every world save.
    fn save(&self, world: &Self::World) -> Vec<u8>;

    /// The world unloaded. The default drops it.
    fn close(&self, world: Self::World) {
        drop(world);
    }

    /// The debug screen's question about one world ([`debug`]): its panel, or an event forced,
    /// a stage set, a button pressed. The default has no page; the host's own panel still shows.
    fn debug(&self, world: &mut Self::World, request: &DebugRequest) -> DebugReply {
        let _ = world;
        match request {
            DebugRequest::Snapshot => DebugReply::default(),
            _ => DebugReply::message("this mod has no debug page"),
        }
    }
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
        /// The slot the exported functions keep the one mod in. An `Arc`, so `warm`, which
        /// blocks for seconds, holds the mod and not the lock, and `status` answers meanwhile.
        static CRAFTKIT_MOD: ::std::sync::Mutex<Option<::std::sync::Arc<$mod>>> =
            ::std::sync::Mutex::new(None);

        #[no_mangle]
        pub extern "C" fn craftkit_abi_version() -> u32 {
            $crate::ABI_VERSION
        }

        #[no_mangle]
        pub extern "C" fn craftkit_info() -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| Ok(<$mod as $crate::Mod>::info()))
        }

        /// # Safety
        /// `init` points to `len` readable bytes of `Init` JSON, or is null with `len` zero.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_init(
            init: *const u8,
            len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let text = unsafe { $crate::abi::text(init, len) }?;
                let init: $crate::Init = $crate::abi::parse(text)?;
                let (made, reply) = <$mod as $crate::Mod>::init(&init)?;
                let mut slot = CRAFTKIT_MOD.lock().unwrap_or_else(|err| err.into_inner());
                *slot = Some(::std::sync::Arc::new(made));

                Ok(reply)
            })
        }

        /// Blocks until the mod has loaded what it loads, or failed. Called on a host thread.
        #[no_mangle]
        pub extern "C" fn craftkit_warm() -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let made = {
                    let slot = CRAFTKIT_MOD.lock().unwrap_or_else(|err| err.into_inner());
                    ::std::sync::Arc::clone(
                        slot.as_ref()
                            .ok_or_else(|| "the mod is not initialized".to_string())?,
                    )
                };
                <$mod as $crate::Mod>::warm(&made)?;

                Ok($crate::abi::Ok::yes())
            })
        }

        #[no_mangle]
        pub extern "C" fn craftkit_status() -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let made = {
                    let slot = CRAFTKIT_MOD.lock().unwrap_or_else(|err| err.into_inner());
                    ::std::sync::Arc::clone(
                        slot.as_ref()
                            .ok_or_else(|| "the mod is not initialized".to_string())?,
                    )
                };

                Ok(<$mod as $crate::Mod>::status(&made))
            })
        }

        /// The open worlds by handle. One lock around the map and each world: the host calls
        /// every world function from the server thread, so nothing waits on it.
        static CRAFTKIT_WORLDS: ::std::sync::Mutex<
            Option<::std::collections::HashMap<u64, <$mod as $crate::Mod>::World>>,
        > = ::std::sync::Mutex::new(None);
        static CRAFTKIT_NEXT_WORLD: ::std::sync::atomic::AtomicU64 =
            ::std::sync::atomic::AtomicU64::new(1);

        fn craftkit_the_mod() -> Result<::std::sync::Arc<$mod>, String> {
            let slot = CRAFTKIT_MOD.lock().unwrap_or_else(|err| err.into_inner());

            slot.as_ref()
                .map(::std::sync::Arc::clone)
                .ok_or_else(|| "the mod is not initialized".to_string())
        }

        fn craftkit_with_world<T>(
            handle: u64,
            body: impl FnOnce(&::std::sync::Arc<$mod>, &mut <$mod as $crate::Mod>::World) -> T,
        ) -> Result<T, String> {
            let made = craftkit_the_mod()?;
            let mut worlds = CRAFTKIT_WORLDS
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            let world = worlds
                .as_mut()
                .and_then(|map| map.get_mut(&handle))
                .ok_or_else(|| format!("no open world with handle {handle}"))?;

            Ok(body(&made, world))
        }

        /// Opens a world from its `WorldInfo` JSON and saved bytes; answers `{"handle": n}`.
        /// # Safety
        /// The two pointers point to their lengths of readable bytes, or are null with zero.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_world_open(
            info: *const u8,
            info_len: usize,
            saved: *const u8,
            saved_len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let info: $crate::WorldInfo =
                    $crate::abi::parse(unsafe { $crate::abi::text(info, info_len) }?)?;
                let saved = unsafe { $crate::abi::bytes(saved, saved_len) }?;
                let made = craftkit_the_mod()?;
                let world = <$mod as $crate::Mod>::open(&made, &info, saved)?;
                let handle =
                    CRAFTKIT_NEXT_WORLD.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed);
                let mut worlds = CRAFTKIT_WORLDS
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                worlds
                    .get_or_insert_with(::std::collections::HashMap::new)
                    .insert(handle, world);

                Ok($crate::abi::Handle { handle })
            })
        }

        /// # Safety
        /// `json` points to `len` readable bytes.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_world_observe(
            handle: u64,
            json: *const u8,
            len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let observations: $crate::Observations =
                    $crate::abi::parse(unsafe { $crate::abi::text(json, len) }?)?;
                craftkit_with_world(handle, |made, world| {
                    <$mod as $crate::Mod>::observe(made, world, &observations)
                })?;

                Ok($crate::abi::Ok::yes())
            })
        }

        /// Answers the tick's commands as a JSON list.
        /// # Safety
        /// `json` points to `len` readable bytes of `Now`.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_world_tick(
            handle: u64,
            json: *const u8,
            len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let now: $crate::Now =
                    $crate::abi::parse(unsafe { $crate::abi::text(json, len) }?)?;

                craftkit_with_world(handle, |made, world| {
                    <$mod as $crate::Mod>::tick(made, world, &now)
                })
            })
        }

        /// # Safety
        /// `json` points to `len` readable bytes of a `Report` list.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_world_report(
            handle: u64,
            json: *const u8,
            len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let reports: Vec<$crate::Report> =
                    $crate::abi::parse(unsafe { $crate::abi::text(json, len) }?)?;
                craftkit_with_world(handle, |made, world| {
                    <$mod as $crate::Mod>::report(made, world, &reports)
                })?;

                Ok($crate::abi::Ok::yes())
            })
        }

        /// Answers a `DebugReply` to a `DebugRequest`.
        /// # Safety
        /// `json` points to `len` readable bytes of a `DebugRequest`.
        #[no_mangle]
        pub unsafe extern "C" fn craftkit_world_debug(
            handle: u64,
            json: *const u8,
            len: usize,
        ) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let request: $crate::DebugRequest =
                    $crate::abi::parse(unsafe { $crate::abi::text(json, len) }?)?;

                craftkit_with_world(handle, |made, world| {
                    <$mod as $crate::Mod>::debug(made, world, &request)
                })
            })
        }

        /// Answers `{"bytes": "<base64>"}`, the world's state for the save.
        #[no_mangle]
        pub extern "C" fn craftkit_world_save(handle: u64) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let bytes = craftkit_with_world(handle, |made, world| {
                    <$mod as $crate::Mod>::save(made, world)
                })?;

                Ok($crate::abi::Saved {
                    bytes: $crate::base64::encode(&bytes),
                })
            })
        }

        #[no_mangle]
        pub extern "C" fn craftkit_world_close(handle: u64) -> *mut ::std::ffi::c_char {
            $crate::abi::answer(|| {
                let made = craftkit_the_mod()?;
                let world = {
                    let mut worlds = CRAFTKIT_WORLDS
                        .lock()
                        .unwrap_or_else(|err| err.into_inner());
                    worlds
                        .as_mut()
                        .and_then(|map| map.remove(&handle))
                        .ok_or_else(|| format!("no open world with handle {handle}"))?
                };
                <$mod as $crate::Mod>::close(&made, world);

                Ok($crate::abi::Ok::yes())
            })
        }

        #[no_mangle]
        pub extern "C" fn craftkit_shutdown() {
            {
                let mut worlds = CRAFTKIT_WORLDS
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                if let (Some(map), Ok(made)) = (worlds.take(), craftkit_the_mod()) {
                    for (_, world) in map {
                        <$mod as $crate::Mod>::close(&made, world);
                    }
                }
            }
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
