"""Paints the "Ruined Hamlet" map: a burnt-out village at night, all procedural pixel art.

Usage: python tools/make_ruins_map.py [preview.png]
Writes assets/maps/ruins/: map.png, collision.png, map.txt (with the lights), overlay.png, overlays.txt.
Muddy ground and dark puddles, bare twisted trees, burnt and roofless shacks, bones,
broken carts and abandoned weapons. The night, fog and firelight are added by the game
from the settings in map.txt. With a preview path, also saves the map with collision drawn on.
Needs Pillow and NumPy.
"""
import math
import random
import sys
import numpy as np
from PIL import Image, ImageDraw

OUT = 'assets/maps/ruins/'
W, H = 1024, 768
BLOCK = (255, 0, 0, 160)
rng = random.Random(23)
nrng = np.random.default_rng(23)

# Palette: greys, browns and sick, washed-out greens.
OUTLINE = (20, 16, 16)
MUD = [(40, 33, 28), (48, 39, 32), (57, 46, 37), (67, 54, 42)]
GRASS = [(37, 39, 32), (45, 47, 37), (54, 55, 42), (64, 63, 47)]
PATH = [(62, 51, 40), (73, 60, 46), (85, 70, 53), (97, 81, 61)]
WATER = [(20, 24, 30), (27, 32, 40), (36, 43, 52), (58, 68, 78)]
BARK = [(30, 26, 25), (42, 36, 33), (55, 48, 43), (70, 62, 55)]
STONE = [(40, 40, 44), (56, 56, 60), (72, 71, 72), (92, 90, 88), (112, 108, 102)]
WOOD = [(36, 27, 21), (50, 38, 28), (66, 50, 36), (82, 64, 46)]
THATCH = [(52, 44, 32), (66, 56, 40), (80, 68, 48), (96, 82, 58)]
CHAR = [(18, 15, 15), (28, 23, 22), (38, 31, 28)]
EMBER = [(150, 50, 22), (210, 96, 34), (255, 170, 70)]
IVY = [(34, 46, 32), (45, 60, 40), (58, 74, 48)]
BONE = [(118, 114, 102), (162, 158, 142), (204, 200, 184)]
METAL = [(60, 62, 68), (98, 102, 108), (150, 154, 160)]
BLOOD = [(52, 18, 18), (70, 24, 22)]
WHEAT = [(70, 64, 46), (88, 80, 56), (108, 98, 68)]

BAYER = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]) / 16 - 0.47


def noise(scale, w=W, h=H, octaves=3):
    """Smooth value noise in [0, 1], `scale` px per lattice cell."""
    total, amp, norm = np.zeros((h, w)), 1.0, 0.0
    for _ in range(octaves):
        cells = nrng.random((h // scale + 3, w // scale + 3)).astype(np.float32)
        img = Image.fromarray(cells, 'F').resize(((w // scale + 3) * scale, (h // scale + 3) * scale), Image.Resampling.BICUBIC)
        total += amp * np.asarray(img)[scale:scale + h, scale:scale + w]
        norm += amp
        amp, scale = amp * 0.5, max(2, scale // 2)
    total /= norm
    return (total - total.min()) / (total.max() - total.min())


def shade(values, palette):
    """Picks palette colours by value with ordered dithering, for a pixel-art gradient."""
    h, w = values.shape
    dither = np.tile(BAYER, (h // 4 + 1, w // 4 + 1))[:h, :w]
    idx = np.clip((values * len(palette) + dither).astype(int), 0, len(palette) - 1)
    return np.array(palette, dtype=np.uint8)[idx]


def dist_to_polyline(points, w=W, h=H):
    ys, xs = np.mgrid[0:h, 0:w]
    best = np.full((h, w), 1e9)
    for (x0, y0), (x1, y1) in zip(points, points[1:]):
        dx, dy = x1 - x0, y1 - y0
        t = np.clip(((xs - x0) * dx + (ys - y0) * dy) / (dx * dx + dy * dy), 0, 1)
        best = np.minimum(best, np.hypot(xs - x0 - t * dx, ys - y0 - t * dy))
    return best


# ---------------------------------------------------------------- ground
# Worked out at half size and doubled, so the ground has chunky 2 px pixels.
HW, HH = W // 2, H // 2
CENTER = (512, 410)
ROADS = [
    [(512, 410), (505, 300), (520, 180), (508, 60), (512, 0)],
    [(512, 410), (500, 520), (480, 640), (492, 768)],
    [(512, 410), (380, 425), (250, 400), (120, 420), (0, 410)],
    [(512, 410), (650, 395), (790, 420), (900, 400), (1024, 396)],
]
SIDE_PATHS = [
    [(380, 425), (330, 330), (300, 220)],
    [(650, 395), (690, 300), (720, 210)],
    [(500, 560), (640, 610), (760, 600)],
]


def half(points):
    return [(x / 2, y / 2) for x, y in points]


road = np.full((HH, HW), 1e9)
for line in ROADS:
    road = np.minimum(road, dist_to_polyline(half(line), HW, HH) - 11)
for line in SIDE_PATHS:
    road = np.minimum(road, dist_to_polyline(half(line), HW, HH) - 7)
ys, xs = np.mgrid[0:HH, 0:HW]
plaza = np.hypot((xs - CENTER[0] / 2) * 0.85, ys - CENTER[1] / 2) - 52
road = np.minimum(road, plaza) + (noise(6, HW, HH) - 0.5) * 14

grassy = noise(24, HW, HH) + (noise(5, HW, HH) - 0.5) * 0.3
ground = shade(noise(10, HW, HH) * 0.55 + noise(3, HW, HH) * 0.5, MUD)
ground = np.where((grassy > 0.52)[..., None], shade(noise(8, HW, HH) * 0.5 + noise(2, HW, HH) * 0.55, GRASS), ground)
# Trampled mud on the roads, with wet cart ruts along them.
ground = np.where((road < 0)[..., None], shade(noise(7, HW, HH) * 0.5 + noise(2, HW, HH) * 0.5, PATH), ground)
ruts = np.full((HH, HW), 1e9)
for line in ROADS:
    for side in (-4, 4):
        ruts = np.minimum(ruts, dist_to_polyline([(x + side, y + side * 0.3) for x, y in half(line)], HW, HH))
wet = (ruts < 1.2) & (road < -3) & (noise(5, HW, HH) > 0.3) & (plaza > -20)
ground[wet] = MUD[1]

# Puddles: (x, y, rx, ry, solid). Big ones block, small ones are just mud.
PUDDLES = [
    (420, 352, 26, 11, False), (560, 330, 30, 13, False), (610, 470, 18, 8, False),
    (300, 600, 64, 30, True), (812, 478, 52, 30, True), (180, 300, 30, 14, True),
    (700, 560, 16, 7, False), (420, 505, 14, 6, False), (880, 300, 22, 10, False),
]
puddle_mask = np.zeros((HH, HW), bool)
for x, y, rx, ry, _ in PUDDLES:
    edge = (noise(3, HW, HH) - 0.5) * 0.5
    d = ((xs - x / 2) / (rx / 2)) ** 2 + ((ys - y / 2) / (ry / 2)) ** 2 + edge
    rim = d < 1.35
    ground[rim] = MUD[0]
    water = d < 1
    depth = np.clip(1 - d, 0, 1)
    ground = np.where(water[..., None], shade(0.15 + depth * 0.25 + noise(4, HW, HH) * 0.4, WATER[:3]), ground)
    puddle_mask |= water
# A few pale glints on the water.
for _ in range(240):
    x, y = rng.randrange(HW), rng.randrange(HH)
    if puddle_mask[y, x] and puddle_mask[y, min(x + 2, HW - 1)]:
        ground[y, x:x + rng.choice([1, 2, 3])] = WATER[3]

ground_img = Image.fromarray(ground, 'RGB').resize((W, H), Image.Resampling.NEAREST).convert('RGBA')
road_full = np.asarray(Image.fromarray((road < 6).astype(np.uint8) * 255).resize((W, H), Image.Resampling.NEAREST)) > 0
puddle_full = np.asarray(Image.fromarray(puddle_mask.astype(np.uint8) * 255).resize((W, H), Image.Resampling.NEAREST)) > 0

# ---------------------------------------------------------------- layers
mapimg = ground_img
overlay = Image.new('RGBA', (W, H))
collision = Image.new('RGBA', (W, H))
cdraw = ImageDraw.Draw(collision)
taken = np.zeros((H, W), bool)     # space claimed by something
objects = []                       # (baseline y, sprite, x, y, overlay?)
overlays = []                      # (name, baseline, bounds)
lights = []                        # (x, y, radius, kind)

for x, y, rx, ry, solid in PUDDLES:
    if solid:
        cdraw.ellipse((x - rx * 0.8, y - ry * 0.7, x + rx * 0.8, y + ry * 0.7), fill=BLOCK)


def claim(x0, y0, x1, y1):
    taken[max(0, int(y0)):max(0, int(y1)), max(0, int(x0)):max(0, int(x1))] = True


def free(x0, y0, x1, y1, roads=True):
    x0, y0, x1, y1 = max(0, int(x0)), max(0, int(y0)), min(W, int(x1)), min(H, int(y1))
    if x0 >= x1 or y0 >= y1:
        return False
    area = taken[y0:y1, x0:x1].any() or puddle_full[y0:y1, x0:x1].any()
    return not area and not (roads and road_full[y0:y1, x0:x1].any())


def outlined(sprite):
    """Adds a 1 px dark outline around a sprite's opaque pixels."""
    a = np.asarray(sprite)[..., 3] > 0
    grown = a.copy()
    grown[1:] |= a[:-1]
    grown[:-1] |= a[1:]
    grown[:, 1:] |= a[:, :-1]
    grown[:, :-1] |= a[:, 1:]
    out = np.asarray(sprite).copy()
    out[grown & ~a] = OUTLINE + (255,)
    return Image.fromarray(out, 'RGBA')


def put(sprite, x, y, baseline=None, name=None, outline=True):
    """Queues a sprite with its top-left at (x, y); with a name it also covers characters behind it."""
    if outline:
        padded = Image.new('RGBA', (sprite.width + 2, sprite.height + 2))
        padded.paste(sprite, (1, 1))
        sprite, x, y = outlined(padded), x - 1, y - 1
    objects.append((baseline if baseline is not None else y + sprite.height, sprite, int(x), int(y), name))


def ground_shadow(cx, cy, rx, ry, alpha=90):
    layer = Image.new('RGBA', (W, H))
    ImageDraw.Draw(layer).ellipse((cx - rx, cy - ry, cx + rx, cy + ry), fill=(10, 8, 10, alpha))
    mapimg.alpha_composite(layer)


# ---------------------------------------------------------------- dead trees
def dead_tree(size):
    """A bare, twisted tree. Returns the sprite and the trunk base in it."""
    w, h = int(70 * size) + 20, int(84 * size) + 10
    img = Image.new('RGBA', (w, h))
    d = ImageDraw.Draw(img)
    base = (w // 2, h - 4)
    segments = []

    def branch(x, y, angle, length, thick, depth):
        steps = 3
        for _ in range(steps):
            angle += rng.uniform(-0.38, 0.38)
            nx, ny = x + math.cos(angle) * length / steps, y + math.sin(angle) * length / steps
            segments.append((x, y, nx, ny, thick))
            x, y = nx, ny
            thick = max(1, thick - (1 if rng.random() < 0.35 else 0))
        if depth == 0 or length < 5:
            return
        for _ in range(rng.choice([2, 2, 3])):
            spread = rng.uniform(0.35, 0.85) * rng.choice([-1, 1])
            branch(x, y, angle + spread, length * rng.uniform(0.55, 0.75), max(1, thick - 1), depth - 1)
        # Little twigs along the way.
        if rng.random() < 0.5:
            branch(x, y, angle + rng.uniform(-1.3, 1.3), length * 0.3, 1, 0)

    trunk = max(3, int(5 * size))
    branch(base[0], base[1], -math.pi / 2 + rng.uniform(-0.2, 0.2), 30 * size, trunk, 4 if size > 0.8 else 3)
    for side in (-1, 1):
        for _ in range(rng.choice([1, 2])):
            ang = math.pi / 2 - side * rng.uniform(1.0, 1.35)
            segments.append((base[0], base[1] - 1, base[0] + math.cos(ang) * rng.uniform(5, 10) * size * side,
                             base[1] + rng.uniform(0, 2), max(1, trunk - 2)))
    for x0, y0, x1, y1, t in segments:
        d.line((x0, y0, x1, y1), fill=BARK[1], width=t)
    # Light catches the left side of the thicker limbs, shadow on the right.
    for x0, y0, x1, y1, t in segments:
        if t >= 3:
            off = t / 2 - 1
            d.line((x0 - off, y0, x1 - off, y1), fill=BARK[3], width=1)
            d.line((x0 + off, y0, x1 + off, y1), fill=BARK[0], width=1)
        elif t == 2:
            d.line((x0 - 0.5, y0, x1 - 0.5, y1), fill=BARK[2], width=1)
    # Knots and a hollow.
    a = np.asarray(img)
    for _ in range(int(6 * size)):
        x, y = rng.randrange(w), rng.randrange(h)
        if a[y, x, 3]:
            img.putpixel((x, y), BARK[0] + (255,))
    return img, base


def plant_tree(cx, cy, size, overlay_ok=True):
    sprite, (bx, by) = dead_tree(size)
    ground_shadow(cx + 3, cy, 10 * size + 4, 4 * size + 2, 70)
    trunk = max(3, int(5 * size))
    cdraw.rectangle((cx - trunk, cy - 4, cx + trunk, cy + 1), fill=BLOCK)
    claim(cx - 12, cy - 10, cx + 12, cy + 6)
    put(sprite, cx - bx, cy - by, baseline=cy, name='tree' if overlay_ok else None)


# ---------------------------------------------------------------- ruined buildings
def bricks(d, x0, y0, x1, y1, palette):
    """Stone courses with staggered joints."""
    d.rectangle((x0, y0, x1, y1), fill=palette[1])
    row = 0
    for y in range(y0, y1 + 1, 4):
        off = 0 if row % 2 else 4
        for x in range(x0 - 8 + off, x1 + 1, 8):
            c = palette[rng.choice([1, 2, 2, 3])]
            if max(x0, x + 1) <= min(x1, x + 7) and y + 1 <= y1:
                d.rectangle((max(x0, x + 1), y + 1, min(x1, x + 7), min(y1, y + 3)), fill=c)
            if x + 1 >= x0:
                d.point((x + 1, y + 1), fill=palette[3])
        row += 1


def planks(d, x0, y0, x1, y1, vertical=True):
    d.rectangle((x0, y0, x1, y1), fill=WOOD[0])
    if vertical:
        for x in range(x0, x1 + 1, 4):
            c = WOOD[rng.choice([1, 2, 2, 3])]
            top = y0 + (rng.randrange(0, 5) if rng.random() < 0.3 else 0)
            d.rectangle((x, top, min(x1, x + 2), y1), fill=c)
            d.point((x + 1, rng.randrange(top, y1 + 1)), fill=WOOD[0])
    else:
        for y in range(y0, y1 + 1, 3):
            d.rectangle((x0, y, x1, min(y1, y + 1)), fill=WOOD[rng.choice([1, 2, 3])])


def jagged(length, low, high):
    """A broken top edge: heights that wander between low and high."""
    hgt, out = rng.uniform(low, high), []
    for _ in range(length):
        hgt = min(high, max(low, hgt + rng.choice([-2, -1, 0, 0, 0, 1, 2]) * (1 if rng.random() < 0.6 else 2)))
        out.append(int(hgt))
    return out


def rubble(d, x0, y0, x1, y1, count, palette=STONE):
    for _ in range(count):
        x, y = rng.uniform(x0, x1), rng.uniform(y0, y1)
        r = rng.choice([1, 1, 2, 2, 3])
        d.rectangle((x - r, y - r + 1, x + r, y + r), fill=palette[rng.choice([1, 2])])
        d.point((x - r + 1, y - r + 1), fill=palette[3])


def char_marks(img, count, embers):
    """Soot streaks and burnt patches; some still glowing."""
    a = np.asarray(img)
    d = ImageDraw.Draw(img)
    h, w = a.shape[:2]
    for _ in range(count):
        x, y = rng.randrange(w), rng.randrange(h)
        if not a[y, x, 3]:
            continue
        r = rng.randint(2, 6)
        for _ in range(r * 4):
            px, py = x + rng.randint(-r, r), y + rng.randint(-r, r // 2)
            if 0 <= px < w and 0 <= py < h and a[py, px, 3]:
                d.point((px, py), fill=CHAR[rng.randrange(3)])
        if embers and rng.random() < 0.6:
            for _ in range(rng.randint(2, 5)):
                px, py = x + rng.randint(-r // 2, r // 2), y + rng.randint(-1, 1)
                if 0 <= px < w and 0 <= py < h and a[py, px, 3]:
                    d.point((px, py), fill=EMBER[rng.randrange(3)])


def ivy(img, count, from_bottom=True):
    a = np.asarray(img)
    d = ImageDraw.Draw(img)
    h, w = a.shape[:2]
    for _ in range(count):
        x = rng.randrange(w)
        y = h - 2 if from_bottom else rng.randrange(h)
        for _ in range(rng.randint(6, 22)):
            if 0 <= x < w and 0 <= y < h and a[y, x, 3]:
                d.point((x, y), fill=IVY[rng.randrange(3)])
                if rng.random() < 0.4 and x + 1 < w:
                    d.point((x + 1, y), fill=IVY[0])
            x += rng.choice([-1, 0, 0, 1])
            y -= rng.choice([0, 1, 1])


def shack(w, depth, wall_h, burning):
    """A timber and stone hut seen from the front and above; its thatch roof caved in."""
    roof_h = depth + 10
    img = Image.new('RGBA', (w + 8, roof_h + wall_h + 6))
    d = ImageDraw.Draw(img)
    x0, x1 = 4, w + 3
    front = roof_h
    bottom = front + wall_h
    # Front wall: stone footing, planks above.
    footing = bottom - wall_h // 3
    planks(d, x0, front, x1, footing)
    bricks(d, x0, footing, x1, bottom, STONE)
    # Roof: a thatch slab with a ridge, sagging and holed.
    roof = [(x0 - 3, front + 2), (x0 + 6, 4), (x1 - 6, 4), (x1 + 3, front + 2)]
    d.polygon(roof, fill=THATCH[1])
    for y in range(6, front + 2, 3):
        for x in range(x0 - 2, x1 + 3, 2):
            if rng.random() < 0.7:
                d.line((x, y, x + rng.choice([0, 1]), y + 2), fill=THATCH[rng.choice([0, 2, 2, 3])])
    d.line((x0 + 6, 4, x1 - 6, 4), fill=THATCH[3])
    d.line((x0 - 3, front + 2, x1 + 3, front + 2), fill=THATCH[0])
    # A caved-in hole: dark inside, charred rim, rafters across.
    hx = rng.randint(x0 + w // 5, x1 - w // 2)
    hw_, hh_ = rng.randint(w // 4, w // 2), rng.randint(depth // 3, depth // 2 + 4)
    hy = rng.randint(8, max(9, front - hh_ - 2))
    hole = [(hx + rng.randint(-3, 3) + i * hw_ / 6, hy + rng.randint(-3, 2)) for i in range(7)]
    hole += [(hx + hw_ - i * hw_ / 6 + rng.randint(-3, 3), hy + hh_ + rng.randint(-2, 3)) for i in range(7)]
    d.polygon(hole, fill=CHAR[2])
    d.line(hole + hole[:1], fill=CHAR[0], width=2)
    d.polygon([(px + 2, py + 2) for px, py in hole[2:-2]], fill=CHAR[0])
    for rx in range(hx + 3, hx + hw_, rng.randint(6, 9)):
        d.line((rx, hy - 2, rx + rng.randint(-3, 3), hy + hh_ + 2), fill=WOOD[1], width=2)
        d.line((rx - 1, hy - 2, rx - 1 + rng.randint(-2, 2), hy + hh_ // 2), fill=WOOD[3])
    # Door and windows, black inside.
    door_x = rng.randint(x0 + 6, x1 - 18)
    d.rectangle((door_x, bottom - 15, door_x + 9, bottom), fill=CHAR[0])
    d.line((door_x + 1, bottom - 14, door_x + 8, bottom - 3), fill=WOOD[2], width=2)
    glow = []
    for wx in range(x0 + 5, x1 - 8, 16):
        if abs(wx - door_x) < 14 or rng.random() < 0.3:
            continue
        d.rectangle((wx, front + 5, wx + 6, front + 10), fill=CHAR[0])
        if burning and rng.random() < 0.6:
            d.rectangle((wx + 1, front + 8, wx + 5, front + 10), fill=EMBER[1])
            d.point((wx + 3, front + 9), fill=EMBER[2])
            glow.append((wx + 3, front + 8))
        d.line((wx - 1, front + 4, wx - 1, front + 11), fill=WOOD[3])
    char_marks(img, 10 if burning else 5, burning)
    ivy(img, 6)
    if burning:
        glow.append((hx + hw_ // 2, hy + hh_ // 2))
    return img, bottom, depth, glow


def roofless(w, depth, wall_h, burning):
    """Stone walls with no roof left: the inside, its floor and fallen beams, is open to the sky."""
    img = Image.new('RGBA', (w + 8, depth + wall_h + 8))
    d = ImageDraw.Draw(img)
    x0, x1 = 4, w + 3
    top = 6
    front = top + depth
    bottom = front + wall_h
    # Floor inside, strewn with rubble and burnt beams.
    d.rectangle((x0 + 4, top + 4, x1 - 4, front), fill=CHAR[2] if burning else (44, 38, 34))
    rubble(d, x0 + 6, top + 6, x1 - 6, front - 2, w // 4)
    for _ in range(3):
        bx = rng.randint(x0 + 6, x1 - 20)
        by = rng.randint(top + 6, front - 4)
        d.line((bx, by, bx + rng.randint(12, 24), by + rng.randint(-6, 6)), fill=CHAR[0] if burning else WOOD[1], width=3)
    # Back and side walls, seen from above: thick tops with broken heights.
    back = jagged(w, 2, 6)
    for i, hgt in enumerate(back):
        d.line((x0 + i, top + 4 - hgt, x0 + i, top + 4), fill=STONE[2])
        d.point((x0 + i, top + 4 - hgt), fill=STONE[4])
    d.rectangle((x0, top + 4, x0 + 3, front), fill=STONE[2])
    d.rectangle((x1 - 3, top + 4, x1, front), fill=STONE[2])
    d.line((x0, top + 4, x0, front), fill=STONE[4])
    d.line((x1 - 3, top + 4, x1 - 3, front), fill=STONE[3])
    # Front wall, broken along its top; a gap where it fell.
    bricks(d, x0, front, x1, bottom, STONE)
    edge = jagged(w, 0, wall_h // 2)
    gap = rng.randint(x0 + w // 4, x1 - w // 3)
    for i, cut in enumerate(edge):
        x = x0 + i
        if gap <= x < gap + w // 5:
            cut = wall_h - 6
        if cut:
            d.line((x, front, x, front + cut - 1), fill=CHAR[2] if burning else (44, 38, 34))
        d.point((x, front + cut), fill=STONE[4])
    rubble(d, gap - 4, bottom - 4, gap + w // 5 + 4, bottom + 2, 10)
    for wx in range(x0 + 8, x1 - 10, 18):
        if gap - 8 < wx < gap + w // 5 + 2:
            continue
        d.rectangle((wx, bottom - wall_h // 2 - 1, wx + 5, bottom - 5), fill=CHAR[0])
    char_marks(img, 14 if burning else 6, burning)
    ivy(img, 10)
    glow = [(gap + w // 10, front + 6)] if burning else []
    return img, bottom, depth + 4, glow


def build(kind, cx, base_y, w, depth, wall_h, burning):
    sprite, bottom, deep, glow = (shack if kind == 'shack' else roofless)(w, depth, wall_h, burning)
    x, y = cx - sprite.width // 2, base_y - bottom
    ground_shadow(cx + 4, base_y - 2, sprite.width // 2 + 4, 7, 80)
    # A spill of rubble and scorched ground around the walls.
    d = ImageDraw.Draw(mapimg)
    if burning:
        for _ in range(60):
            px, py = rng.uniform(x - 6, x + sprite.width + 6), rng.uniform(base_y - 6, base_y + 10)
            d.point((px, py), fill=CHAR[rng.randrange(3)] + (255,))
    rubble(d, x - 4, base_y - 2, x + sprite.width + 4, base_y + 8, sprite.width // 5)
    cdraw.rectangle((x + 4, base_y - deep - 2, x + sprite.width - 4, base_y - 1), fill=BLOCK)
    claim(x - 10, y - 6, x + sprite.width + 10, base_y + 14)
    put(sprite, x, y, baseline=base_y, name=kind)
    for gx, gy in glow:
        lights.append((x + gx, y + gy, 64, 'glow'))


# ---------------------------------------------------------------- props
def bone_pile(d, cx, cy, count):
    for _ in range(count):
        x, y = cx + rng.uniform(-8, 8), cy + rng.uniform(-4, 4)
        ang = rng.uniform(0, math.pi)
        ln = rng.uniform(3, 6)
        x1, y1 = x + math.cos(ang) * ln, y + math.sin(ang) * ln * 0.6
        d.line((x, y, x1, y1), fill=BONE[1])
        for px, py in ((x, y), (x1, y1)):
            d.point((px, py), fill=BONE[2])
            d.point((px + 1, py), fill=BONE[0])


def skull(d, x, y):
    d.rectangle((x, y, x + 4, y + 3), fill=BONE[2])
    d.rectangle((x + 1, y + 3, x + 3, y + 4), fill=BONE[1])
    d.point((x + 1, y + 1), fill=OUTLINE)
    d.point((x + 3, y + 1), fill=OUTLINE)
    d.line((x, y + 3, x + 4, y + 3), fill=BONE[0])


def ribcage(d, x, y):
    d.line((x, y, x + 10, y), fill=BONE[1])
    for i in range(4):
        rx = x + 2 + i * 2
        d.line((rx, y - 3, rx, y + 3), fill=BONE[2] if i % 2 else BONE[1])
    skull(d, x + 11, y - 2)


def sword_in_ground(cx, cy):
    s = Image.new('RGBA', (9, 18))
    d = ImageDraw.Draw(s)
    d.line((4, 0, 4, 11), fill=METAL[2])
    d.line((5, 1, 5, 11), fill=METAL[1])
    d.line((1, 12, 7, 12), fill=METAL[1])
    d.line((4, 13, 4, 16), fill=WOOD[2])
    d.point((4, 17), fill=METAL[2])
    s = s.transpose(Image.Transpose.FLIP_TOP_BOTTOM)
    put(s, cx - 4, cy - 17, baseline=cy)
    ImageDraw.Draw(mapimg).ellipse((cx - 4, cy - 1, cx + 4, cy + 2), fill=MUD[3] + (255,))


def lying_weapon(d, cx, cy):
    kind = rng.choice(['sword', 'spear', 'shield', 'axe'])
    ang = rng.uniform(0, math.pi)
    if kind == 'shield':
        d.ellipse((cx - 6, cy - 4, cx + 6, cy + 4), fill=WOOD[2], outline=METAL[1])
        d.line((cx - 5, cy, cx + 5, cy), fill=WOOD[1])
        d.rectangle((cx - 1, cy - 1, cx + 1, cy + 1), fill=METAL[2])
        return
    ln = {'sword': 13, 'spear': 22, 'axe': 11}[kind]
    x1, y1 = cx + math.cos(ang) * ln, cy + math.sin(ang) * ln * 0.55
    if kind == 'sword':
        d.line((cx, cy, x1, y1), fill=METAL[2])
        d.line((cx + math.sin(ang) * 3, cy - math.cos(ang) * 2, cx - math.sin(ang) * 3, cy + math.cos(ang) * 2), fill=METAL[1])
    else:
        d.line((cx, cy, x1, y1), fill=WOOD[2])
        head = METAL[2] if kind == 'spear' else METAL[1]
        d.rectangle((x1 - 1, y1 - (2 if kind == 'axe' else 1), x1 + 1, y1 + 1), fill=head)


def cart(cx, cy, flip):
    s = Image.new('RGBA', (52, 32))
    d = ImageDraw.Draw(s)
    # Bed, tipped over on one side.
    d.polygon([(6, 8), (40, 4), (44, 18), (8, 22)], fill=WOOD[2])
    for i in range(5):
        y = 7 + i * 3
        d.line((7, y + 1, 42, y - 3 + 1), fill=WOOD[3 if i % 2 else 1])
    d.line((6, 8, 8, 22), fill=WOOD[0], width=2)
    d.line((22, 7, 24, 22), fill=WOOD[0])
    # Broken planks sticking out.
    d.line((40, 4, 47, 0), fill=WOOD[2], width=2)
    d.line((43, 18, 50, 22), fill=WOOD[1], width=2)
    # Wheel still on, spokes and rim.
    d.ellipse((12, 16, 28, 31), fill=None, outline=WOOD[3], width=2)
    for a in range(0, 180, 45):
        r = math.radians(a)
        d.line((20 - math.cos(r) * 6, 23.5 - math.sin(r) * 6, 20 + math.cos(r) * 6, 23.5 + math.sin(r) * 6), fill=WOOD[1])
    d.rectangle((19, 22, 21, 25), fill=METAL[1])
    # The shaft, snapped.
    d.line((0, 20, 8, 16), fill=WOOD[2], width=2)
    if flip:
        s = s.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    ground_shadow(cx, cy - 2, 24, 6, 80)
    # The other wheel fell off and lies in the mud.
    wx = cx + (-34 if flip else 30)
    ImageDraw.Draw(mapimg).ellipse((wx - 7, cy - 3, wx + 7, cy + 3), outline=WOOD[2] + (255,), width=2)
    ImageDraw.Draw(mapimg).point((wx, cy), fill=METAL[1] + (255,))
    cdraw.rectangle((cx - 22, cy - 14, cx + 22, cy - 1), fill=BLOCK)
    claim(cx - 40, cy - 34, cx + 40, cy + 8)
    put(s, cx - 26, cy - 32, baseline=cy - 4, name='cart')


def well(cx, cy):
    s = Image.new('RGBA', (40, 44))
    d = ImageDraw.Draw(s)
    # Stone ring with black water.
    d.ellipse((2, 20, 38, 42), fill=STONE[1])
    d.ellipse((4, 21, 36, 38), fill=STONE[3])
    for a in range(0, 360, 30):
        r = math.radians(a)
        d.point((20 + math.cos(r) * 15, 30 + math.sin(r) * 7), fill=STONE[0])
    d.ellipse((9, 24, 31, 35), fill=WATER[0])
    d.line((13, 28, 17, 28), fill=WATER[2])
    # Posts; the crossbeam has snapped and hangs down.
    d.rectangle((4, 4, 6, 30), fill=WOOD[2])
    d.rectangle((33, 12, 35, 30), fill=WOOD[1])
    d.line((5, 4, 24, 8), fill=WOOD[3], width=2)
    d.line((34, 12, 30, 22), fill=WOOD[2], width=2)
    d.line((22, 8, 22, 22), fill=BONE[0])
    ivy(s, 3)
    ground_shadow(cx + 2, cy - 2, 20, 6, 80)
    cdraw.ellipse((cx - 17, cy - 13, cx + 17, cy), fill=BLOCK)
    claim(cx - 26, cy - 46, cx + 26, cy + 10)
    put(s, cx - 20, cy - 43, baseline=cy - 4, name='well')


def campfire(cx, cy):
    d = ImageDraw.Draw(mapimg)
    d.ellipse((cx - 13, cy - 6, cx + 13, cy + 6), fill=CHAR[1] + (255,))
    for a in range(0, 360, 40):
        r = math.radians(a)
        x, y = cx + math.cos(r) * 11, cy + math.sin(r) * 5
        d.rectangle((x - 2, y - 1, x + 2, y + 2), fill=STONE[2] + (255,))
        d.point((x - 1, y - 1), fill=STONE[4] + (255,))
    d.line((cx - 7, cy + 2, cx + 7, cy - 2), fill=WOOD[1] + (255,), width=3)
    d.line((cx - 7, cy - 2, cx + 7, cy + 2), fill=WOOD[2] + (255,), width=3)
    for _ in range(10):
        d.point((cx + rng.randint(-5, 5), cy + rng.randint(-2, 2)), fill=EMBER[rng.randrange(3)] + (255,))
    cdraw.ellipse((cx - 9, cy - 4, cx + 9, cy + 4), fill=BLOCK)
    claim(cx - 20, cy - 26, cx + 20, cy + 12)
    lights.append((cx, cy, 120, 'fire'))


def torch(cx, cy):
    s = Image.new('RGBA', (6, 20))
    d = ImageDraw.Draw(s)
    d.rectangle((2, 4, 3, 19), fill=WOOD[2])
    d.line((2, 4, 2, 19), fill=WOOD[3])
    d.rectangle((1, 2, 4, 5), fill=METAL[0])
    put(s, cx - 3, cy - 19, baseline=cy)
    cdraw.rectangle((cx - 2, cy - 3, cx + 2, cy), fill=BLOCK)
    claim(cx - 8, cy - 24, cx + 8, cy + 6)
    lights.append((cx, cy - 19, 78, 'torch'))


def grave(d, x, y, cross):
    if cross:
        d.rectangle((x, y - 12, x + 1, y), fill=WOOD[2])
        d.rectangle((x - 3, y - 9, x + 4, y - 8), fill=WOOD[3])
    else:
        d.rectangle((x - 3, y - 9, x + 3, y), fill=STONE[2])
        d.rectangle((x - 2, y - 10, x + 2, y - 10), fill=STONE[2])
        d.line((x - 3, y - 9, x - 3, y), fill=STONE[3])
        d.line((x - 1, y - 6, x + 1, y - 6), fill=STONE[0])
    d.rectangle((x - 4, y + 1, x + 5, y + 9), fill=MUD[3])
    d.line((x - 4, y + 1, x + 5, y + 1), fill=MUD[2])


def fence(d, x0, y0, x1, y1, broken=0.3):
    n = max(1, int(math.hypot(x1 - x0, y1 - y0) / 12))
    pts = [(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n) for i in range(n + 1)]
    for (ax, ay), (bx, by) in zip(pts, pts[1:]):
        if rng.random() > broken:
            d.line((ax, ay - 5, bx, by - 5), fill=WOOD[3])
            d.line((ax, ay - 2, bx, by - 2), fill=WOOD[1])
        elif rng.random() < 0.5:
            d.line((ax, ay - 4, bx - 3, by), fill=WOOD[2])
    for i, (x, y) in enumerate(pts):
        if rng.random() < 0.85:
            lean = rng.choice([0, 0, 1, -1])
            d.line((x, y, x + lean, y - 8), fill=WOOD[2], width=2)
            d.point((x, y - 8), fill=WOOD[3])


# ---------------------------------------------------------------- layout
# Dead forest all around the edge: dense trees on dark ground, not walkable.
edge_noise = noise(16) * 40
ys_f, xs_f = np.mgrid[0:H, 0:W]
edge_dist = np.minimum.reduce([xs_f, W - 1 - xs_f, ys_f * 1.2, (H - 1 - ys_f) * 1.2]) - edge_noise
forest = (edge_dist < 46) & ~(road_full & (edge_dist > 8))
dark = np.asarray(mapimg).copy()
deep = forest[..., None]
dark = np.where(deep, (dark * 0.62).astype(np.uint8), dark)
dark[..., 3] = 255
mapimg = Image.fromarray(dark, 'RGBA')
collision.paste(Image.fromarray(np.where(forest[..., None], np.array(BLOCK, np.uint8), 0).astype(np.uint8), 'RGBA'),
                (0, 0), Image.fromarray((forest * 255).astype(np.uint8)))
cdraw = ImageDraw.Draw(collision)
taken |= forest
claim(CENTER[0] - 60, CENTER[1] - 50, CENTER[0] + 60, CENTER[1] + 50)

# The plaza: a well, a campfire, a torch, a broken cart.
well(458, 352)
campfire(590, 440)
torch(560, 400)
torch(440, 470)
cart(372, 492, False)
cart(690, 300, True)

# Houses along the top, and a burnt row along the bottom.
HOUSES = [
    ('shack', 160, 215, 116, 50, 38, False),
    ('roofless', 330, 200, 120, 56, 30, False),
    ('shack', 650, 195, 128, 52, 38, True),
    ('roofless', 820, 215, 112, 54, 30, True),
    ('shack', 930, 300, 84, 42, 32, False),
    ('roofless', 630, 728, 116, 48, 30, True),
    ('shack', 790, 726, 112, 46, 36, True),
]
for kind, cx, by, w, depth, wall_h, burning in HOUSES:
    build(kind, cx, by, w, depth, wall_h, burning)

d = ImageDraw.Draw(mapimg)
# Graveyard on the right, behind a broken fence.
GY = (835, 560, 975, 650)
claim(GY[0] - 6, GY[1] - 14, GY[2] + 6, GY[3] + 10)
for gx in range(GY[0] + 10, GY[2] - 6, 22):
    for gy in range(GY[1] + 14, GY[3], 26):
        grave(d, gx + rng.randint(-3, 3), gy + rng.randint(-2, 2), rng.random() < 0.5)
        cdraw.rectangle((gx - 3, gy - 2, gx + 3, gy + 1), fill=BLOCK)
fence(d, GY[0], GY[1], GY[2], GY[1])
fence(d, GY[0], GY[1], GY[0], GY[3], 0.5)
torch(GY[0] - 4, GY[3] + 4)

# A dead wheat field on the left, fenced.
WF = (70, 520, 230, 680)
claim(*WF)
field = np.asarray(mapimg).copy()
field[WF[1]:WF[3], WF[0]:WF[2], :3] = (field[WF[1]:WF[3], WF[0]:WF[2], :3] * 0.8).astype(np.uint8)
mapimg = Image.fromarray(field, 'RGBA')
d = ImageDraw.Draw(mapimg)
for y in range(WF[1] + 8, WF[3] - 2, 5):
    for x in range(WF[0] + 4, WF[2] - 4, 3):
        if rng.random() < 0.8:
            h = rng.randint(3, 7)
            lean = rng.choice([-1, 0, 0, 1])
            d.line((x, y, x + lean, y - h), fill=WHEAT[rng.randrange(3)] + (255,))
            if rng.random() < 0.3:
                d.point((x + lean, y - h), fill=WHEAT[2] + (255,))
fence(d, WF[0], WF[1], WF[2], WF[1], 0.25)
fence(d, WF[2], WF[1], WF[2], WF[3], 0.4)
fence(d, WF[0], WF[3], WF[2], WF[3], 0.3)
cdraw.rectangle(WF, fill=BLOCK)
# A scarecrow, more like a gibbet.
sc = Image.new('RGBA', (16, 26))
sd = ImageDraw.Draw(sc)
sd.rectangle((7, 0, 8, 25), fill=WOOD[2])
sd.rectangle((1, 6, 14, 7), fill=WOOD[2])
sd.rectangle((4, 7, 11, 15), fill=(54, 46, 40))
skull(sd, 5, 1)
put(sc, 150 - 8, 600 - 25, baseline=600, name='scarecrow')

# Dead trees: rings of them in the forest edge, scattered ones across the open ground.
built = taken & ~forest
trees = []
for _ in range(900):
    x, y = rng.randrange(8, W - 8), rng.randrange(20, H - 2)
    if edge_dist[y, x] < 46 and not road_full[y, x] and not built[max(0, y - 30):y + 10, max(0, x - 20):x + 20].any() and not any(abs(x - ox) < 22 and abs(y - oy) < 18 for ox, oy in trees):
        trees.append((x, y))
        plant_tree(x, y, rng.uniform(0.75, 1.15), overlay_ok=edge_dist[y, x] > 20)
placed = 0
for _ in range(3000):
    if placed >= 34:
        break
    x, y = rng.randrange(60, W - 60), rng.randrange(80, H - 50)
    if free(x - 26, y - 22, x + 26, y + 16):
        plant_tree(x, y, rng.uniform(0.7, 1.05))
        placed += 1

# Stumps and dry grass tufts.
for _ in range(500):
    x, y = rng.randrange(W), rng.randrange(H)
    if free(x - 2, y - 2, x + 3, y + 2, roads=False) and not forest[y, x]:
        for k in range(rng.randint(2, 4)):
            d.line((x + k, y, x + k + rng.choice([-1, 0, 1]), y - rng.randint(2, 4)), fill=WHEAT[rng.randrange(2)] + (255,))
for _ in range(14):
    x, y = rng.randrange(80, W - 80), rng.randrange(80, H - 60)
    if free(x - 8, y - 8, x + 8, y + 6):
        d.ellipse((x - 5, y - 3, x + 5, y + 3), fill=WOOD[1] + (255,), outline=OUTLINE + (255,))
        d.ellipse((x - 3, y - 2, x + 3, y + 1), fill=WOOD[3] + (255,))
        d.point((x, y - 1), fill=WOOD[1] + (255,))
        cdraw.rectangle((x - 4, y - 2, x + 4, y + 2), fill=BLOCK)
        claim(x - 10, y - 8, x + 10, y + 8)

# Blood, bones, weapons: more of them near the roads and the plaza, where the fighting was.
for _ in range(150):
    x, y = rng.randrange(40, W - 40), rng.randrange(40, H - 40)
    if forest[y, x] or puddle_full[y, x]:
        continue
    near_road = road_full[max(0, y - 30):y + 30, max(0, x - 30):x + 30].any()
    if not near_road and rng.random() < 0.6:
        continue
    roll = rng.random()
    if roll < 0.3:
        for _ in range(rng.randint(6, 16)):
            r = rng.randint(1, 3)
            px, py = x + rng.randint(-8, 8), y + rng.randint(-4, 4)
            d.ellipse((px - r, py - r // 2, px + r, py + r // 2), fill=BLOOD[rng.randrange(2)] + (255,))
    elif roll < 0.55:
        bone_pile(d, x, y, rng.randint(2, 5))
    elif roll < 0.68:
        ribcage(d, x, y)
    elif roll < 0.78:
        skull(d, x, y)
    elif roll < 0.92:
        lying_weapon(d, x, y)
    elif free(x - 6, y - 18, x + 6, y + 3, roads=False):
        sword_in_ground(x, y)
        claim(x - 6, y - 18, x + 6, y + 3)

# ---------------------------------------------------------------- compose
for baseline, sprite, x, y, name in sorted(objects, key=lambda o: o[0]):
    mapimg.alpha_composite(sprite, (max(0, x), max(0, y)), (max(0, -x), max(0, -y)))
    if name:
        overlay.alpha_composite(sprite, (max(0, x), max(0, y)), (max(0, -x), max(0, -y)))
        bx, by = max(0, x), max(0, y)
        bw, bh = min(W, x + sprite.width) - bx, min(H, baseline) - by
        if bw > 0 and bh > 0:
            overlays.append((name, (x, baseline, x + sprite.width, baseline), (bx, by, bw, bh)))

mapimg.convert('RGB').save(OUT + 'map.png', optimize=True)
collision.save(OUT + 'collision.png', optimize=True)
overlay.save(OUT + 'overlay.png', optimize=True)
with open(OUT + 'overlays.txt', 'w', newline='\n') as f:
    f.write('# <name> <baseline x0 y0 x1 y1> <bounds x y w h>, written by tools/make_ruins_map.py\n')
    for name, (x0, y0, x1, y1), (bx, by, bw, bh) in overlays:
        f.write(f'{name} {x0} {y0} {x1} {y1} {bx} {by} {bw} {bh}\n')
with open(OUT + 'map.txt', 'w', newline='\n') as f:
    f.write('# Map settings, written by tools/make_ruins_map.py.\n'
            '# night/fog/ash: 0..1. light <x> <y> <radius> [fire|torch|glow]: lights in the dark.\n'
            'title Ruined Hamlet\n'
            f'spawn {CENTER[0] - 10} {CENTER[1] + 20}\n'
            'night 0.38\nfog 0.5\nash 0.6\n')
    for x, y, r, kind in lights:
        f.write(f'light {int(x)} {int(y)} {r} {kind}\n')

if len(sys.argv) > 1:
    preview = mapimg.copy()
    preview.alpha_composite(collision)
    preview.convert('RGB').save(sys.argv[1])
print(f'{len(overlays)} overlays, {len(lights)} lights')
