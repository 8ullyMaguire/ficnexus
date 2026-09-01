#!/usr/bin/env python3
"""Generate FicHub PWA icons (192/512 PNG) in pure Python (zlib+struct).

Design: dark rounded square background (#0f1117) with an indigo 'F'
(#6366f1) composed of rectangles, rendered with 4x supersampling.
"""
import struct, zlib, sys, os

BG = (15, 17, 23)
FG = (99, 102, 241)
CORNER = 0.18  # corner radius as fraction of size


def in_rounded_rect(x, y, x0, y0, x1, y1, r, S):
    # coords in supersampled space; r = corner radius in same space
    if x < x0 or x > x1 or y < y0 or y > y1:
        return False
    cx = min(max(x, x0 + r), x1 - r)
    cy = min(max(y, y0 + r), y1 - r)
    dx, dy = x - cx, y - cy
    return dx * dx + dy * dy <= r * r


def render(size):
    S = 4  # supersample factor
    ss = size * S
    r_corner = CORNER * ss
    # 'F' geometry in supersampled units, centered
    bar_w = 0.16 * ss
    top_h = 0.16 * ss
    mid_h = 0.12 * ss
    gap = 0.09 * ss
    left = (ss - bar_w) / 2 - 0.02 * ss
    right = left + bar_w
    top = 0.16 * ss
    mid_y = top + top_h + gap
    f_r = 0.045 * ss  # rounding of F bars

    rows = []
    for y in range(size):
        row = bytearray([0])  # filter byte
        for x in range(size):
            r = g = b = 0
            for sy in range(S):
                for sx in range(S):
                    px = x * S + sx + 0.5
                    py = y * S + sy + 0.5
                    if in_rounded_rect(px, py, 0, 0, ss, ss, r_corner, S):
                        if in_rounded_rect(px, py, left, top, right, top + top_h, f_r, S) or \
                           in_rounded_rect(px, py, left, mid_y, right, mid_y + mid_h, f_r, S) or \
                           in_rounded_rect(px, py, left, top, left + top_h, mid_y + mid_h, f_r, S):
                            r += FG[0]; g += FG[1]; b += FG[2]
                        else:
                            r += BG[0]; g += BG[1]; b += BG[2]
            n = S * S
            row += bytes((r // n, g // n, b // n))
        rows.append(bytes(row))

    raw = b"".join(rows)

    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    return png


if __name__ == "__main__":
    outdir = sys.argv[1] if len(sys.argv) > 1 else "static"
    os.makedirs(outdir, exist_ok=True)
    for size in (192, 512):
        path = os.path.join(outdir, f"icon-{size}.png")
        with open(path, "wb") as f:
            f.write(render(size))
        print(f"wrote {path} ({size}x{size})")
    # Also generate the tab favicon (replaces the tracked 0-byte favicon.png).
    favicon = os.path.join(outdir, "favicon.png")
    with open(favicon, "wb") as f:
        f.write(render(64))
    print(f"wrote {favicon} (64x64)")
