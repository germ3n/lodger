"""The lodger's icon: a door left ajar at night, two pale eyes in the gap.
Drawn on a 32x32 grid and scaled without smoothing; a PNG written by hand (no image library).
Run from the repository: python3 mod/art/icon.py
It writes mod/art/icon-512.png (Modrinth) and the jar's mod/src/main/resources/assets/lodger/icon.png (128)."""
import os
import struct
import zlib

N = 32
HERE = os.path.dirname(os.path.abspath(__file__))


def hexc(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


grid = [[(0, 0, 0)] * N for _ in range(N)]

# The night: a dark blue that deepens toward the top, with a few stars.
top, bottom = hexc("#070913"), hexc("#18203a")
for y in range(N):
    t = y / (N - 1)
    row = tuple(round(top[i] + (bottom[i] - top[i]) * t) for i in range(3))
    for x in range(N):
        grid[y][x] = row
for x, y in [(1, 5), (27, 1), (5, 9), (29, 7), (25, 12), (2, 15), (13, 1), (20, 2)]:
    grid[y][x] = hexc("#8e96b8") if (x + y) % 3 else hexc("#c9cde0")

# The wall: dark stone bricks around the doorway, every other row offset half a brick.
stone = [hexc("#2c2f3a"), hexc("#33363f"), hexc("#282a33")]
mortar = hexc("#1c1d24")
for y in range(4, 29):
    for x in range(4, 28):
        brick_row = (y - 4) // 3
        offset = 3 if brick_row % 2 else 0
        if (y - 4) % 3 == 2 or (x + offset) % 6 == 5:
            grid[y][x] = mortar
        else:
            grid[y][x] = stone[(x // 6 + brick_row) % 3]
for x in range(4, 28):
    grid[3][x] = hexc("#3d404c")

# The frame: dark oak around the opening.
oak = hexc("#3a2716")
oak_light = hexc("#4a331d")
for y in range(6, 29):
    grid[y][9] = oak
    grid[y][22] = oak_light
for x in range(9, 23):
    grid[6][x] = oak_light if x % 2 else oak
    grid[5][x] = hexc("#2a1c10")

# The dark inside.
for y in range(7, 29):
    for x in range(10, 22):
        grid[y][x] = hexc("#040406")

# The door, swung in and seen narrow, the gap beside it.
plank = [hexc("#6b4a2b"), hexc("#5c3f24")]
for y in range(7, 29):
    for x in range(10, 17):
        grid[y][x] = plank[(x - 10) // 2 % 2]
    grid[y][16] = hexc("#3f2b17")
for x in range(10, 16):
    grid[11][x] = hexc("#7a5633")
    grid[22][x] = hexc("#7a5633")
for y in (9, 25):
    grid[y][10] = hexc("#5d6066")
    grid[y][11] = hexc("#4a4c52")
grid[17][15] = hexc("#8a8d94")
grid[18][15] = hexc("#5d6066")

# The eyes in the gap, with the faintest glow beside them.
glow = hexc("#0d0e15")
for x, y in [(18, 13), (20, 13)]:
    for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
        if 17 <= x + dx <= 21:
            grid[y + dy][x + dx] = glow
for x, y in [(18, 13), (20, 13)]:
    grid[y][x] = hexc("#e9e4d4")

# The ground, and a stone step at the door.
for x in range(N):
    grid[29][x] = hexc("#2c4a24") if x % 3 else hexc("#335428")
    for y in (30, 31):
        grid[y][x] = hexc("#3b2a1c") if (x + y) % 4 else hexc("#33241a")
for x in range(9, 23):
    grid[29][x] = hexc("#45474f")

# A vignette.
for y in range(N):
    for x in range(N):
        d = ((x - 15.5) ** 2 + (y - 15.5) ** 2) ** 0.5 / 22.6
        k = 1.0 - max(0.0, d - 0.55) * 0.9
        grid[y][x] = tuple(max(0, round(c * k)) for c in grid[y][x])


def write_png(path, pixels, scale):
    size = len(pixels) * scale
    raw = bytearray()
    for row in pixels:
        line = bytearray([0])
        for c in row:
            line += bytes(c) * scale
        for _ in range(scale):
            raw += line

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    png += chunk(b"IEND", b"")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(png)


write_png(os.path.join(HERE, "icon-512.png"), grid, 16)
write_png(os.path.join(HERE, "..", "src", "main", "resources", "assets", "lodger", "icon.png"), grid, 4)
