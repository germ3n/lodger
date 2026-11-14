# Upgrading

How to take the lodger and craftkit to a new Minecraft, Fabric, NeoForge or Java version, step
by step, and why the work is as small as it is. Written at the owner's ask ("a doc
that describes the process of upgrading the code to a new mc/fabric/neoforge version"), from
what moving to 26.3 took.

## Why the Rust side never moves

Nothing in `sdk/` or `core/` names a Minecraft class. The mind sees the world as the protocol's
JSON (`sdk/src/world.rs`): players, entities, blocks by id, events by kind, and it answers with
commands by kind. Every field has a default, so a host built for the next game version and a
mod built last year read each other for as long as the ABI version holds. A new Minecraft
version is therefore a change to the host alone, and inside the host to its Minecraft-facing
edge, which is kept to a few known places:

| Where | What touches the game there |
| --- | --- |
| `host/common/.../senses/Senses.java`, `MovementTracker.java` | Reading players, entities, blocks, light, biomes, kills into observations; the debug panel's host half. |
| `host/common/.../world/Executors.java` and the rest of `world/` (`Looks`, `Poses`, `Surroundings`, `Later`, `Figures`, `Fights`, `Warps`, `OtherHouse` among them) | Carrying commands out: blocks, containers, signs, items, sounds, mobs' looks, stands' poses, frames, paintings, the player list, chat lines, the surroundings pass, the figures, the fight, the other house. |
| `host/common/.../world/Worlds.java`, `WorldData.java` | The per-tick loop, saved data, the clock. |
| `host/common/.../client/` | The preparing screen, the title gate, the settings screen, the glitch renderers (`Glitches.java`, `Atmospheres.java`), the debug screen (`client/debug/DebugScreen.java`). |
| `host/common/.../mixin/` | The render hooks (below). |
| `host/common/.../net/` | The payload records and their stream codecs. |
| `host/common/.../Host.java` | The `/craftkit` commands and the permission check. |
| `host/fabric/`, `host/neoforge/` | Each loader's entry points, events, networking registration, key mapping, HUD layer, and the Fabric mixin for block placement. |

Everything else in the host (`Native`, `LoadedMod`, `Manifest`, `Downloader`, `Preparation`,
`Acceptance`, `Natives`, `ConfigFile`, `client/debug/DebugModel`) is plain Java and compiles
against no game version; it has JUnit tests that run without one.

## The versions, in one place each

| Version | Where it is pinned | Checked by |
| --- | --- | --- |
| Minecraft | `host/gradle.properties` `minecraft_version`, `minecraft_version_range`; `mod/gradle.properties` the same two | `VersionsTest` (the two property files agree) |
| NeoForm (the vanilla mappings `common/` compiles against) | `host/gradle.properties` `neo_form_version` | the build |
| Fabric Loader, Fabric API | `host/gradle.properties` `fabric_loader_version`, `fabric_version`; `mod/gradle.properties` `fabric_loader_version` | `VersionsTest` |
| NeoForge | `host/gradle.properties` `neoforge_version`, `neoforge_loader_version_range`; `mod/gradle.properties` the range | `VersionsTest` |
| Loom, ModDevGradle | `host/settings.gradle` (plugin versions) | the build |
| Gradle | `host/gradle/wrapper/gradle-wrapper.properties`, `mod/gradle/wrapper/...` | the wrapper |
| Java | `host/gradle.properties` `java_version`; `host/build-logic/` toolchain; `fabric.mod.json` `java` | the build; the toolchain download |
| The CI's runners and JDK | `.github/workflows/ci.yml` | the CI |

The docs that name versions: this page's history below, the [README](../README.md) (what it
takes to run), [Development](development.md#toolchain), and the todo's Done lines.

## The process

Do it on a branch, one version at a time (Minecraft first, then each loader), and keep every
step's build green before the next.

1. **Find the versions.** Fabric: `https://meta.fabricmc.net/v2/versions/loader/<mc>` and the
   Fabric API's Modrinth or Maven listing for the `+<mc>` build. NeoForge:
   `https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml`, the
   `<mc minor>.<mc patch>.*` line; NeoForm is listed beside it. Loom and ModDevGradle: their
   changelogs say which game versions they support; a new Minecraft often needs a new Loom.
   Java: Mojang's `version_manifest` names the `javaVersion` the client wants.
2. **Pin them.** The two `gradle.properties` files, `settings.gradle` for the plugins, the
   wrapper if Gradle itself must move. `./gradlew :common:test` in `host/`: `VersionsTest` fails
   until every file agrees.
3. **Build the host.** `./gradlew build` in `host/`. What breaks is in the table above and
   nowhere else. The new names are found in the decompiled sources the build writes:
   `host/neoforge/build/moddev/artifacts/*-sources.jar` and
   `host/common/build/moddev/artifacts/vanilla-*-sources.jar`, or with `javap -cp
   ~/.gradle/caches/fabric-loom/<mc>/minecraft-merged.jar <class>` for a quick signature. Grep
   for the old name, read the new class, fix the call. The things that moved for 26.3 are the
   history below; expect the same kinds of change: a method renamed, a parameter object added
   (`MouseButtonEvent`, `KeyEvent`), a registry access moved, a packet's constructor changed.
   Reflection into game classes (`Executors.withEntries` sets the player-list packet's private
   entries) is the first thing to re-check: the test is `/craftkit lodger force tab_entry` in a
   dev client.
4. **Run the tests.** `./gradlew test` in `host/` (JUnit, about a hundred tests); with
   `target/release/liblodger_core.dylib` built (`cargo build --release -p lodger-core`) the
   real-native tests load the mind through FFM too. Nothing on the Rust side needs to move, so
   `cargo test --workspace` is a check that nothing did.
5. **Rebuild the lodger's jars.** `./gradlew build` in `mod/`: it builds the native, packages
   each loader's jar with the host nested. The content jar's manifests take the Minecraft
   range from `mod/gradle.properties`, so they moved in step 2.
6. **Run it.** `./gradlew :fabric:runClient` in `host/` with the lodger's jar in
   `host/fabric/runs/client/mods/` (its nested host removed, since the dev run has the host
   from its classes; see [Running a stay quickly](development.md#running-a-stay-quickly)). On
   the title screen: the license screen and the download bar if the model is not there. In a
   world: F8 opens the debug screen; the Senses tab must show your meters moving, the Events
   tab must fire `open_door`, `open_chest`, `frame_rotate`, `tab_entry`, `says_back`,
   `flicker` and `cut_music` with the world and the screen doing what each says. Then
   `:neoforge:runClient` the same. Then a dedicated server of each loader with the plain jar,
   as the [queue's](queue.md#rules-for-every-session) smoke test has it.
7. **Write it down.** A line under Done in [Todo](todo.md) with the versions and what
   moved, and a row in the history below. The README's "What it will take to run" if the
   versions a player needs changed.

A Fabric-only or NeoForge-only bump (a loader release for the same Minecraft) is steps 2, 3,
4 and 6 for that loader alone; the content jar needs rebuilding only if the manifest's range
moved.

## What a game version is most likely to break

- **Input events** (`Screen.mouseClicked`, `keyPressed`): parameter objects since 1.21.9; the
  debug screen hit-tests `MouseButtonEvent.x()`/`y()`.
- **GUI drawing** (`GuiGraphicsExtractor` since 26.x): `fill`, `text`, `centeredText`,
  `enableScissor`, `guiWidth`/`guiHeight`; the HUD element interface of each loader wraps it.
- **Networking**: the payload type registries (Fabric's `PayloadTypeRegistry`, NeoForge's
  `PayloadRegistrar`), and `ByteBufCodecs.stringUtf8` for the debug JSON.
- **Permissions**: `Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)` as a predicate over a
  `CommandSourceStack`, and `MinecraftServer.isSingleplayerOwner(NameAndId)` for the owner of
  a world with cheats off.
- **The clock and weather**: 26.3 moved time to `LevelData.getGameTime()` and weather to
  `ServerLevel.getWeatherData()`.
- **Entities the executors touch**: `ItemFrame.setRotation`, `Painting` (its variant is final,
  so a swap spawns a new one), `ArmorStand.setHeadPose(Rotations)`, `Sheep.setColor`,
  `Entity.teleportTo`, `ChestBlockEntity` lid via `Level.blockEvent(pos, block, 1, count)`,
  the `ClientboundPlayerInfoUpdatePacket.Entry` constructor (nine arguments in 26.3).
- **Saved data**: `SavedDataType` and the codec in `WorldData`.
- **Events the loaders hook**: block use, break and place (the Fabric mixin on `BlockItem`),
  sleep, death (and the killer, read off the damage source), respawn, dimension change,
  attack, chat, and a block struck (Fabric's `AttackBlockCallback`, NeoForge's
  `LeftClickBlock` at `START`) for the `knocked` sense.
- **The horn's hook** (`InstrumentItemMixin`, server side): `InstrumentItem.use` at its return, a
  horn blown queued for the senses' `used_item` (`UsedItems`).
- **The render hooks** (`host/common/src/main/java/craftkit/host/mixin/`, the
  `craftkit.common.mixins.json` both loaders name,): each is one line into
  `craftkit.host.client.Atmospheres`, and with `defaultRequire` 1 a hook that no longer matches
  stops the client at the title screen, so a broken one is found at once.
  - `FogRenderer.setupFog` at its return: the `FogData` (start, end, colour) before the caller
    uploads it; the atmospheres close and tint it.
  - `SkyRenderer.extractRenderState` at its tail: the `SkyRenderState` fields (`starBrightness`,
    `moonPhase`, the sun, moon and star angles, `skyColor`).
  - `WeatherEffectRenderer.prepareInstances`: its `ARGB.white(float)` call, the drops' colour,
    redirected for red rain; and `extractRenderState` at its tail, the columns of rain and snow
    as gathered, one left out for `dry_beside_you` and the snow held for `snow_hangs`.
  - `CompassAngleState.calculate`: its `CompassTarget.get` call, redirected for
    `compass_points`.
  - `PaintingRenderer.submit` at its head: the pose turned half round on the wall's normal for
    `painting_flip` (`PoseStack.rotate`).
  - `LivingEntityRenderer.extractRenderState` at its tail (the entity's id kept for its render
    state) and `ModelFeatureRenderer.prepareModel`'s `Model.setupAnim` call, redirected: the
    fight's warped animals bent as they are posed (`WarpedModels`).
  - `SubtitleOverlay.onPlaySound`: its `WeighedSoundEvents.getSubtitle()` call, redirected so
    a ghast's subtitle reads as a name for `ghast_name` (into `Glitches.subtitleOf`).
  - `SoundManager.play(SoundInstance)` at its head: the sound about to play, swapped for
    another while a `sound_swap` hint names it (`wrong_footing`, into `Glitches.swapSound`).
  The HUD's `heart_skip` and `subtitle` read `Hud`'s private `tickCount`, `healthBlinkTime` and
  `subtitleOverlay` by name, and `frames_show` and `bobber` the private `ItemFrame.DATA_ITEM`,
  `FishingHook.DATA_BITING`, `nibble`, `timeUntilHooked` and `timeUntilLured`; a renamed field
  only loses that effect, with a warning in the log. Common code compiles
  against Fabric's `sponge-mixin` for the annotations alone.

## History

| Bump | From | To | What moved |
| --- | --- | --- | --- |
| First | the design, for 1.21.x | Minecraft 26.3, Java 25, Fabric Loader 0.20.1, Fabric API 0.165.2+26.3, NeoForge 26.3.0.101, NeoForm 26.3-1, Loom 1.19.4, ModDevGradle 2.0.148, Gradle 9.8.2 | First build. Found by reading 26.3's sources: `Commands.hasPermission`, `PayloadTypeRegistry.clientboundConfiguration`, `IModFile.getContents` (JarContents) in NeoForge, `GuiGraphicsExtractor`, `Blaze3D.openUri`, `getGameTime` for the clock, `SavedDataType` with a null extra, `ItemEntity.setUnlimitedLifetime`, `ClientboundSetActionBarTextPacket`, the four-argument `SignText`, `ServerLevelEvents`, `ServerEntityLevelChangeEvents`, NeoForge's `BreakBlockEvent` under `event/level/block/`, `KeyMapping.Category`, `InputConstants.Type.KEYBOARD`, `Minecraft.gui.screen()`. |

## The sound list

`core/tests/data/sounds-26.3.txt` is every sound event id of the game version, read from its
own `sounds.json` (the assets index in Loom's cache, `~/.gradle/caches/fabric-loom/assets/`,
`minecraft/sounds.json`). `every_sound_the_mind_names_is_one_the_game_has` checks every sound
the mind names against it. On a version bump, write the new version's list beside it and point
the test at it: a renamed sound then fails a test, not a player's world (26.3 dropped
`entity.wolf.howl`).
