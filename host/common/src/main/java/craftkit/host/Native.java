package craftkit.host;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;

/**
 * One Rust mod's shared library, bound through the FFM API.
 *
 * <p>This is the only Java that knows there is a native. Every function of the C ABI
 * ({@code sdk/src/abi.rs}, exported by {@code craftkit::export_mod!}) takes UTF-8 bytes by
 * pointer and length and answers a NUL-terminated JSON string that {@code craftkit_free} frees,
 * so each binding here is the same few lines: copy the text into a confined arena, call, read the
 * answer, free it. Errors come back as JSON ({@code {"error": ...}}) for the caller to read;
 * nothing here throws for a mod that answered.
 */
public final class Native implements AutoCloseable {
	/** The ABI this host speaks; a library built against another is refused. Bumped with {@code sdk/src/lib.rs}'s {@code ABI_VERSION}. */
	public static final int ABI_VERSION = 54;

	private static final Linker LINKER = Linker.nativeLinker();

	private final Arena arena = Arena.ofShared();
	private final MethodHandle abiVersion;
	private final MethodHandle info;
	private final MethodHandle init;
	private final MethodHandle warm;
	private final MethodHandle status;
	private final MethodHandle shutdown;
	private final MethodHandle free;
	private final MethodHandle worldOpen;
	private final MethodHandle worldObserve;
	private final MethodHandle worldTick;
	private final MethodHandle worldReport;
	private final MethodHandle worldDebug;
	private final MethodHandle worldSave;
	private final MethodHandle worldClose;

	private Native(Path library) {
		SymbolLookup lookup = SymbolLookup.libraryLookup(library, arena);
		abiVersion = LINKER.downcallHandle(symbol(lookup, "craftkit_abi_version"), FunctionDescriptor.of(ValueLayout.JAVA_INT));
		info = LINKER.downcallHandle(symbol(lookup, "craftkit_info"), FunctionDescriptor.of(ValueLayout.ADDRESS));
		init = LINKER.downcallHandle(symbol(lookup, "craftkit_init"), FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
		warm = LINKER.downcallHandle(symbol(lookup, "craftkit_warm"), FunctionDescriptor.of(ValueLayout.ADDRESS));
		status = LINKER.downcallHandle(symbol(lookup, "craftkit_status"), FunctionDescriptor.of(ValueLayout.ADDRESS));
		shutdown = LINKER.downcallHandle(symbol(lookup, "craftkit_shutdown"), FunctionDescriptor.ofVoid());
		free = LINKER.downcallHandle(symbol(lookup, "craftkit_free"), FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
		worldOpen = LINKER.downcallHandle(symbol(lookup, "craftkit_world_open"), FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG));
		FunctionDescriptor handleAndText = FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.JAVA_LONG);
		worldObserve = LINKER.downcallHandle(symbol(lookup, "craftkit_world_observe"), handleAndText);
		worldTick = LINKER.downcallHandle(symbol(lookup, "craftkit_world_tick"), handleAndText);
		worldReport = LINKER.downcallHandle(symbol(lookup, "craftkit_world_report"), handleAndText);
		worldDebug = LINKER.downcallHandle(symbol(lookup, "craftkit_world_debug"), handleAndText);
		FunctionDescriptor handleOnly = FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.JAVA_LONG);
		worldSave = LINKER.downcallHandle(symbol(lookup, "craftkit_world_save"), handleOnly);
		worldClose = LINKER.downcallHandle(symbol(lookup, "craftkit_world_close"), handleOnly);
	}

	/** Opens a world from its {@code WorldInfo} JSON and saved bytes; {@code {"handle": n}} or an error. */
	public String worldOpen(String infoJson, byte[] saved) {
		try (Arena call = Arena.ofConfined()) {
			MemorySegment info = utf8(call, infoJson);
			MemorySegment bytes = call.allocate(Math.max(saved.length, 1));
			MemorySegment.copy(saved, 0, bytes, ValueLayout.JAVA_BYTE, 0, saved.length);

			return take((MemorySegment) worldOpen.invokeExact(info, (long) infoJson.getBytes(StandardCharsets.UTF_8).length, bytes, (long) saved.length));
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_world_open", t);
		}
	}

	public String worldObserve(long handle, String json) {
		return handleAndText(worldObserve, "craftkit_world_observe", handle, json);
	}

	/** The tick's commands as a JSON list, or an error. */
	public String worldTick(long handle, String nowJson) {
		return handleAndText(worldTick, "craftkit_world_tick", handle, nowJson);
	}

	public String worldReport(long handle, String json) {
		return handleAndText(worldReport, "craftkit_world_report", handle, json);
	}

	/** A {@code DebugReply} to a {@code DebugRequest}, for the debug screen, or an error. */
	public String worldDebug(long handle, String requestJson) {
		return handleAndText(worldDebug, "craftkit_world_debug", handle, requestJson);
	}

	/** {@code {"bytes": "<base64>"}} or an error. */
	public String worldSave(long handle) {
		try {
			return take((MemorySegment) worldSave.invokeExact(handle));
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_world_save", t);
		}
	}

	public String worldClose(long handle) {
		try {
			return take((MemorySegment) worldClose.invokeExact(handle));
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_world_close", t);
		}
	}

	private String handleAndText(MethodHandle function, String name, long handle, String json) {
		try (Arena call = Arena.ofConfined()) {
			byte[] bytes = json.getBytes(StandardCharsets.UTF_8);
			MemorySegment text = call.allocate(Math.max(bytes.length, 1));
			MemorySegment.copy(bytes, 0, text, ValueLayout.JAVA_BYTE, 0, bytes.length);

			return take((MemorySegment) function.invokeExact(handle, text, (long) bytes.length));
		} catch (Throwable t) {
			throw new IllegalStateException(name, t);
		}
	}

	private static MemorySegment utf8(Arena arena, String text) {
		byte[] bytes = text.getBytes(StandardCharsets.UTF_8);
		MemorySegment segment = arena.allocate(Math.max(bytes.length, 1));
		MemorySegment.copy(bytes, 0, segment, ValueLayout.JAVA_BYTE, 0, bytes.length);

		return segment;
	}

	/** Loads the library at {@code path}; it stays loaded until {@link #close()}. Throws when a symbol is missing. */
	public static Native load(Path path) {
		return new Native(path);
	}

	private static MemorySegment symbol(SymbolLookup lookup, String name) {
		return lookup.find(name).orElseThrow(() -> new IllegalStateException("the library has no " + name + "; is it a craftkit mod?"));
	}

	public int abiVersion() {
		try {
			return (int) abiVersion.invokeExact();
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_abi_version", t);
		}
	}

	/** {@code {"id": ..., "name": ..., "version": ..., "sdk": ...}}. */
	public String info() {
		try {
			return take((MemorySegment) info.invokeExact());
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_info", t);
		}
	}

	/** Hands the mod its {@code Init} JSON (config text, directories, id) and answers its JSON: the effective config and unknown keys, or an error. */
	public String init(String initJson) {
		try (Arena call = Arena.ofConfined()) {
			byte[] bytes = initJson.getBytes(StandardCharsets.UTF_8);
			MemorySegment text = call.allocate(Math.max(bytes.length, 1));
			MemorySegment.copy(bytes, 0, text, ValueLayout.JAVA_BYTE, 0, bytes.length);

			return take((MemorySegment) init.invokeExact(text, (long) bytes.length));
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_init", t);
		}
	}

	/** Blocks while the mod loads what it loads (the lodger: its model, a second or two). {@code true}, or {@code {"error": ...}}. Call on a worker thread. */
	public String warm() {
		try {
			return take((MemorySegment) warm.invokeExact());
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_warm", t);
		}
	}

	/** {@code {"ready": ..., "line": ..., "loading": ..., "error": ...}}. Safe from any thread. */
	public String status() {
		try {
			return take((MemorySegment) status.invokeExact());
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_status", t);
		}
	}

	public void shutdown() {
		try {
			shutdown.invokeExact();
		} catch (Throwable t) {
			throw new IllegalStateException("craftkit_shutdown", t);
		}
	}

	private String take(MemorySegment answer) throws Throwable {
		if (answer.equals(MemorySegment.NULL)) {
			return "{\"error\":\"the mod answered null\"}";
		}

		try {
			return answer.reinterpret(Long.MAX_VALUE).getString(0, StandardCharsets.UTF_8);
		} finally {
			free.invokeExact(answer);
		}
	}

	@Override
	public void close() {
		arena.close();
	}
}
