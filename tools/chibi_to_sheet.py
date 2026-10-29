"""Turns the chibi pose sheet into assets/player.png.

Usage: python tools/chibi_to_sheet.py <pose sheet> assets/player.png [sheet_width]
The source can have a transparent background, or a flat grey one.
sheet_width is how many pixels wide the whole source sheet becomes (sets the sprite size).
Needs Pillow and NumPy. Prints the frame size / feet values for player.txt.
"""
import sys
import numpy as np
from PIL import Image

SRC, DST = sys.argv[1], sys.argv[2]
SHEET_WIDTH = float(sys.argv[3]) if len(sys.argv) > 3 else 164
COLORS = 40
OUTLINE = (42, 34, 54, 255)
# Poses per row of the source sheet, top to bottom.
ROW_COUNTS = [3, 4, 4, 4, 3]
# Bits of a neighbouring pose's slash that touch this pose's sword, so they
# can't be told apart automatically: (row, col, x0, y0, x1, y1) in cell pixels
# at sheet_width 164 (scaled for other sizes).
# Only pale (slash-coloured) pixels inside the box are erased.
ERASE = [(2, 1, 14, 23, 25, 39), (2, 3, 20, 24, 30, 38), (3, 2, 18, 20, 31, 36)]

source = Image.open(SRC).convert('RGBA')
img = np.asarray(source)[..., :3].astype(int)
alpha_in = np.asarray(source)[..., 3]
h, w, _ = img.shape
SCALE = w / SHEET_WIDTH
if (alpha_in < 128).mean() > 0.1:
    mask = alpha_in >= 128
else:
    # Flat background: anything far enough from the corner colour.
    bg = np.median(np.concatenate([img[:20, :20].reshape(-1, 3), img[-20:, -20:].reshape(-1, 3)]), axis=0)
    mask = np.abs(img - bg).sum(axis=2) > 40
r, g, b = img[..., 0], img[..., 1], img[..., 2]
hair = (r > 200) & (g > 160) & (b < 140) & mask
lum = img.mean(axis=2)


def runs(flags):
    out, start = [], None
    for i, f in enumerate(flags):
        if f and start is None:
            start = i
        if not f and start is not None:
            out.append((start, i))
            start = None
    if start is not None:
        out.append((start, len(flags)))
    return out


bands = runs(mask.sum(axis=1) > 0)
assert len(bands) == len(ROW_COUNTS), bands

frames = []  # (row, rgba crop, anchor x, anchor y) in source pixels
for row, ((y0, y1), count) in enumerate(zip(bands, ROW_COUNTS)):
    # One blonde head per pose: split the band halfway between heads.
    # Merge hair runs separated by small gaps (face, highlights) into one head.
    merged = []
    for a, b in runs(hair[y0:y1].sum(axis=0) > 0):
        if merged and a - merged[-1][1] < 25 * w / 737:
            merged[-1] = (merged[-1][0], b)
        else:
            merged.append((a, b))
    heads = [s for s in merged if s[1] - s[0] > 15 * w / 737]
    heads = sorted(heads, key=lambda s: s[1] - s[0], reverse=True)[:count]
    centers = sorted((a + b) / 2 for a, b in heads)
    assert len(centers) == count, (row, heads)
    edges = [0] + [int((a + b) / 2) for a, b in zip(centers, centers[1:])] + [w]
    for x0, x1 in zip(edges, edges[1:]):
        m = mask[y0:y1, x0:x1]
        dark = (lum[y0:y1, x0:x1] < 70) & m
        ys = np.nonzero(dark.any(axis=1))[0]
        feet_y = ys.max()
        bottom = dark[max(0, feet_y - round(12 * w / 737)):feet_y + 1]
        feet_x = np.nonzero(bottom.any(axis=0))[0].mean()
        rgba = np.dstack([img[y0:y1, x0:x1], m * 255]).astype(np.uint8)
        frames.append((row, rgba, x0, y0, feet_x, feet_y))

def drop_border_specks(opaque, feet_x):
    """Removes small blobs at the crop edges: bits of a neighbour's slash."""
    h, w = opaque.shape
    seen = np.zeros_like(opaque)
    for sy in range(h):
        for sx in range(w):
            if not opaque[sy, sx] or seen[sy, sx]:
                continue
            stack, blob = [(sy, sx)], []
            seen[sy, sx] = True
            while stack:
                y, x = stack.pop()
                blob.append((y, x))
                for ny, nx in ((y + 1, x), (y - 1, x), (y, x + 1), (y, x - 1)):
                    if 0 <= ny < h and 0 <= nx < w and opaque[ny, nx] and not seen[ny, nx]:
                        seen[ny, nx] = True
                        stack.append((ny, nx))
            touches_edge = any(x <= 1 or x >= w - 2 for _, x in blob)
            behind = len(blob) < 80 and max(x for _, x in blob) < feet_x - 10
            if (len(blob) < 150 and touches_edge) or behind:
                for y, x in blob:
                    opaque[y, x] = False
    return opaque


# Shared palette from every pose at full resolution, so colours stay consistent.
samples = np.concatenate([rgba[..., :3][rgba[..., 3] > 0][::7] for _, rgba, *_ in frames])
pal_img = Image.fromarray(samples.reshape(1, -1, 3), 'RGB').quantize(COLORS, method=Image.Quantize.MAXCOVERAGE)
palette = np.array(pal_img.getpalette()[:COLORS * 3]).reshape(-1, 3)

# Downscale by majority: each small pixel takes the most common palette colour
# of its block (no blending, so edges stay crisp), and is opaque if most of the
# block is.
small = []
for row, rgba, x0, y0, fx, fy in frames:
    fh, fw = rgba.shape[:2]
    idx = np.asarray(Image.fromarray(rgba[..., :3]).quantize(palette=pal_img, dither=Image.Dither.NONE))
    solid = rgba[..., 3] > 0
    sw, sh = max(1, round(fw / SCALE)), max(1, round(fh / SCALE))
    xs = np.linspace(0, fw, sw + 1).astype(int)
    ys = np.linspace(0, fh, sh + 1).astype(int)
    rgb = np.zeros((sh, sw, 3))
    opaque = np.zeros((sh, sw), bool)
    for j in range(sh):
        for i in range(sw):
            block = solid[ys[j]:ys[j + 1], xs[i]:xs[i + 1]]
            if block.mean() > 0.5:
                opaque[j, i] = True
                rgb[j, i] = palette[np.bincount(idx[ys[j]:ys[j + 1], xs[i]:xs[i + 1]][block]).argmax()]
    opaque = drop_border_specks(opaque, fx / SCALE)
    small.append((row, rgb, opaque, fx / SCALE, fy / SCALE))

# Cell size from the largest extent around the feet anchor.
left = max(fx for *_, fx, _ in small)
right = max(op.shape[1] - fx for _, _, op, fx, _ in small)
top = max(fy for *_, fy in small)
below = max(op.shape[0] - fy for _, _, op, _, fy in small)
cell_w = int(np.ceil(max(left, right))) * 2 + 2
cell_h = int(np.ceil(top + below)) + 2
anchor_y = int(np.ceil(top)) + 1

cols = max(ROW_COUNTS)
sheet = Image.new('RGBA', (cell_w * cols, cell_h * len(ROW_COUNTS)), (0, 0, 0, 0))
counters = [0] * len(ROW_COUNTS)
for row, rgb, op, fx, fy in small:
    rgba = np.dstack([rgb, op * 255]).astype(np.uint8)
    # Drop lone pixels left over from JPG noise.
    n = sum(np.roll(np.roll(op, dy, 0), dx, 1) for dy in (-1, 0, 1) for dx in (-1, 0, 1)) - op
    rgba[..., 3] = np.where(op & (n >= 2), 255, 0)
    col = counters[row]
    counters[row] += 1
    ox = col * cell_w + cell_w // 2 - int(round(fx))
    oy = row * cell_h + anchor_y - int(round(fy))
    cell = Image.new('RGBA', (cell_w, cell_h), (0, 0, 0, 0))
    cell.alpha_composite(Image.fromarray(rgba, 'RGBA'), (ox - col * cell_w, oy - row * cell_h))
    px = np.asarray(cell).copy()
    for er, ec, *box_164 in ERASE:
        if (er, ec) == (row, col):
            x0, y0, x1, y1 = (round(v * SHEET_WIDTH / 164) for v in box_164)
            box = px[y0:y1, x0:x1]
            pale = (box[..., :3].astype(int).mean(axis=2) > 110) & (box[..., 2] >= box[..., 0])
            box[pale, 3] = 0
    # Dark 1px outline around the sprite, like the hand-drawn skeleton.
    filled = px[..., 3] > 0
    around = np.zeros_like(filled)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        around |= np.roll(filled, (dy, dx), (0, 1))
    px[around & ~filled] = OUTLINE
    sheet.alpha_composite(Image.fromarray(px, 'RGBA'), (col * cell_w, row * cell_h))

sheet.save(DST)
print(f'frame {cell_w} {cell_h}')
print(f'feet {cell_h - anchor_y - 1}')
