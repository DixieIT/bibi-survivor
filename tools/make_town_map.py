"""Builds the "Tiny Town" map out of Kenney's Tiny Town tiles (CC0).

Usage: python tools/make_town_map.py
Writes assets/maps/town/: map.png, collision.png, map.txt, overlay.png, overlays.txt.
The layout is described below in tile coordinates (64 x 48 tiles of 16 px).
Needs Pillow and NumPy.
"""
import random
import numpy as np
from PIL import Image, ImageDraw

TILESET = 'assets/tilesets/kenney_tiny_town/tilemap_packed.png'
OUT = 'assets/maps/town/'
T = 16
W, H = 64, 48
BLOCK = (255, 0, 0, 160)
# Gloomier colours than the tileset's: share of grey mixed in, brightness, and a cold tint.
DESATURATE = 0.62
BRIGHTNESS = 0.62
TINT, TINT_AMOUNT = (52, 64, 78), 0.18

# Tile ids in tilemap_packed.png (12 tiles per row).
GRASS, GRASS_TUFT, GRASS_FLOWER = 0, 1, 2
STONES, BUSH, SPROUT, MUSHROOMS = 43, 5, 17, 29
WELL, BARREL, WATER_BARREL, TARGET, SIGN, BEEHIVE = 104, 107, 131, 95, 83, 94
TALL_TREES = [(4, 16), (3, 15)]  # (top, bottom): green, orange
SMALL_TREES = [28, 27]
HOUSES = {
    'red': [[52, 53, 53, 54], [64, 65, 65, 66], [72, 84, 85, 75]],
    'red_chimney': [[52, 53, 53, 55], [64, 65, 65, 67], [84, 86, 84, 87]],
    'grey_chimney': [[48, 49, 49, 51], [60, 61, 61, 63], [88, 89, 88, 91]],
    'grey_small': [[48, 49, 50], [60, 61, 62], [76, 78, 79]],
    'red_small': [[52, 53, 54], [64, 65, 66], [73, 74, 73]],
}
# Fence box pieces: corners, edges.
FENCE = {'tl': 44, 't': 45, 'tr': 46, 'l': 56, 'r': 58, 'bl': 68, 'b': 69, 'br': 70}


def autotile(cells, x, y, outside, tiles):
    """Picks an edge/corner tile from a 3x3 autotile set by which neighbours are in `cells`."""
    def has(dx, dy):
        nx, ny = x + dx, y + dy
        return (nx, ny) in cells or (outside and not (0 <= nx < W and 0 <= ny < H))
    n, s, w, e = has(0, -1), has(0, 1), has(-1, 0), has(1, 0)
    (tl, t, tr), (l, c, r), (bl, b, br) = tiles
    if not n and not w: return tl
    if not n and not e: return tr
    if not s and not w: return bl
    if not s and not e: return br
    if not n: return t
    if not s: return b
    if not w: return l
    if not e: return r
    return c


tileset = Image.open(TILESET).convert('RGBA')


def tile_image(i):
    return tileset.crop(((i % 12) * T, (i // 12) * T, (i % 12) * T + T, (i // 12) * T + T))


rng = random.Random(7)
ground = {(x, y): rng.choices([GRASS, GRASS_TUFT, GRASS_FLOWER], [84, 12, 4])[0] for x in range(W) for y in range(H)}
path, forest = set(), set()
objects = []          # (tile, x, y) drawn over the ground, in order
solid_tiles = set()   # whole tiles that block
solid_rects = []      # pixel rects that block (tree trunks)
overlays = []         # (name, polygon, baseline)
crown_tiles = []      # (tile, x, y) of every tree, for the overlay layer
taken = set()         # tiles not available for random decoration


def area(x0, y0, x1, y1):
    return {(x, y) for x in range(x0, x1 + 1) for y in range(y0, y1 + 1)}


def place(tile, x, y, solid=True):
    objects.append((tile, x, y))
    taken.add((x, y))
    if solid:
        solid_tiles.add((x, y))


def house(kind, x, y):
    """`y` is the roof's top row; the door row faces south."""
    rows = HOUSES[kind]
    for r, row in enumerate(rows):
        for c, t in enumerate(row):
            place(t, x + c, y + r)
    taken.update(area(x - 1, y - 1, x + len(rows[0]), y + len(rows)))


def fence_box(x0, y0, x1, y1, gaps=()):
    for x in range(x0, x1 + 1):
        for y in range(y0, y1 + 1):
            if (x, y) in gaps or x0 < x < x1 and y0 < y < y1:
                continue
            key = ('t' if y == y0 else 'b' if y == y1 else '') + ('l' if x == x0 else 'r' if x == x1 else '')
            place(FENCE[key], x, y)
    taken.update(area(x0, y0, x1, y1))


def tree(x, y, kind=0, tall=True):
    """`y` is the trunk tile. The lower part blocks so nobody stands inside the tree;
    the crown above covers whoever walks behind it."""
    if tall:
        top, bottom = TALL_TREES[kind]
        place(top, x, y - 1, solid=False)
        place(bottom, x, y, solid=False)
        crown_tiles.extend([(top, x, y - 1), (bottom, x, y)])
        crown_top = (y - 1) * T
        solid_rects.append((x * T + 3, y * T + 6, 10, 10))
    else:
        # Small trees are all crown: block nearly the whole tile.
        place(SMALL_TREES[kind], x, y, solid=False)
        crown_tiles.append((SMALL_TREES[kind], x, y))
        crown_top = y * T
        solid_rects.append((x * T + 2, y * T + 3, 12, 13))
    px, base = x * T, y * T + 15
    overlays.append((f'tree_{x}_{y}', [(px, crown_top), (px + T, crown_top), (px + T, base + 1), (px, base + 1)],
                     [(px, base), (px + T, base)]))
    taken.update(area(x - 1, y - 2, x + 1, y + 1))


# --- Layout ---------------------------------------------------------------
# Forest all around, plus two thicker patches.
forest |= {(x, y) for x in range(W) for y in range(H) if x < 3 or x >= W - 3 or y < 3 or y >= H - 3}
forest |= area(52, 3, 60, 8) | area(3, 38, 9, 44)
# Main roads crossing at the plaza, side streets.
path |= area(3, 22, 60, 24) | area(30, 3, 32, 44) | area(25, 19, 37, 27)
path |= area(3, 11, 29, 13) | area(33, 34, 58, 36)
taken |= forest | path

# Plaza.
place(WELL, 31, 20)
for x, t in ((27, BARREL), (28, WATER_BARREL), (35, BARREL)):
    place(t, x, 20)
place(SIGN, 33, 26)

# Houses facing the main road (roofs on rows 18-19, doors on row 20).
for kind, x in (('red', 5), ('grey_chimney', 11), ('red_small', 17), ('red_chimney', 21),
                ('red_chimney', 39), ('grey_small', 45), ('red', 50), ('grey_chimney', 55)):
    house(kind, x, 18)
# North-west street.
for kind, x in (('grey_chimney', 5), ('red_small', 11), ('red_chimney', 16), ('red', 22)):
    house(kind, x, 7)
# South-east street.
for kind, x in (('red', 36), ('grey_small', 42), ('red_chimney', 47), ('grey_chimney', 53)):
    house(kind, x, 31)

# Training yard with targets.
fence_box(36, 5, 50, 15, gaps={(43, 15)})
for x in (39, 43, 47):
    place(TARGET, x, 8)
place(SIGN, 42, 16)

# Vegetable garden with a beehive.
fence_box(6, 27, 18, 36, gaps={(12, 27)})
place(BEEHIVE, 8, 29)
for y in (30, 32, 34):
    for x in range(10, 17, 2):
        place(SPROUT, x, y, solid=False)
place(MUSHROOMS, 16, 29, solid=False)

# Orchard south of the south-east street.
for x in range(35, 58, 4):
    for y in (39, 42):
        tree(x + (y % 3), y, kind=1, tall=False)

# Scattered trees, bushes and stones on the remaining grass.
free = [(x, y) for x in range(4, W - 4) for y in range(5, H - 4) if (x, y) not in taken]
rng.shuffle(free)
for x, y in free:
    if (x, y) in taken:
        continue
    roll = rng.random()
    if roll < 0.10:
        tree(x, y, kind=rng.random() < 0.25)
    elif roll < 0.13:
        place(BUSH, x, y)
        taken.update(area(x - 1, y - 1, x + 1, y + 1))
    elif roll < 0.15:
        ground[(x, y)] = STONES

# --- Render ---------------------------------------------------------------
img = Image.new('RGBA', (W * T, H * T))
for (x, y), t in ground.items():
    img.alpha_composite(tile_image(t), (x * T, y * T))
for x, y in path:
    img.alpha_composite(tile_image(autotile(path, x, y, False, [[12, 13, 14], [24, 25, 26], [36, 37, 38]])), (x * T, y * T))
for x, y in sorted(forest, key=lambda p: p[1]):
    img.alpha_composite(tile_image(autotile(forest, x, y, True, [[6, 7, 8], [18, 19, 20], [30, 31, 32]])), (x * T, y * T))
for t, x, y in objects:
    img.alpha_composite(tile_image(t), (x * T, y * T))


def gloom(image):
    """Applies the map's colour grading, keeping transparency."""
    px = np.asarray(image).astype(float)
    rgb = px[..., :3]
    rgb = rgb * (1 - DESATURATE) + rgb.mean(axis=2, keepdims=True) * DESATURATE
    rgb = rgb * BRIGHTNESS * (1 - TINT_AMOUNT) + np.array(TINT) * TINT_AMOUNT
    return Image.fromarray(np.dstack([rgb.clip(0, 255), px[..., 3]]).astype(np.uint8), 'RGBA')


img = gloom(img)
img.convert('RGB').save(OUT + 'map.png', optimize=True)

mask = Image.new('RGBA', img.size, (0, 0, 0, 0))
draw = ImageDraw.Draw(mask)
for x, y in solid_tiles | forest:
    draw.rectangle([x * T, y * T, x * T + T - 1, y * T + T - 1], fill=BLOCK)
for x, y, w, h in solid_rects:
    draw.rectangle([x, y, x + w - 1, y + h - 1], fill=BLOCK)
mask.save(OUT + 'collision.png', optimize=True)

# Overlay layer: only the trees' own pixels (no grass around them), graded like the map.
crowns = Image.new('RGBA', img.size, (0, 0, 0, 0))
for t, x, y in crown_tiles:
    crowns.alpha_composite(tile_image(t), (x * T, y * T))
gloom(crowns).save(OUT + 'overlay.png', optimize=True)
lines = ['# <name> <baseline x0 y0 x1 y1> <bounds x y w h>, written by tools/make_town_map.py']
for name, polygon, ((bx0, by0), (bx1, by1)) in overlays:
    xs, ys = zip(*polygon)
    lines.append(f'{name} {bx0} {by0} {bx1} {by1} {min(xs)} {min(ys)} {max(xs) - min(xs) + 1} {max(ys) - min(ys) + 1}')
open(OUT + 'overlays.txt', 'w', newline='\n').write('\n'.join(lines) + '\n')

open(OUT + 'map.txt', 'w', newline='\n').write(
    '# Map settings, written by tools/make_town_map.py.\n'
    '# Tiles: "Tiny Town" by Kenney (www.kenney.nl), CC0.\n'
    'title Tiny Town\n'
    f'spawn {31 * T + 8} {25 * T + 12}\n')
