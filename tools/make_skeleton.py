"""Draws the chibi skeleton enemy into assets/skeleton.png (hand-made pixel art).

Usage: python tools/make_skeleton.py [--crisp]
Row 0: walk (4 frames), row 1: collapse into a bone pile (3 frames),
row 2: archer walk (4 frames), row 3: archer aiming, archer releasing, flying arrow,
and the heart they sometimes drop. Row 4: tank walk (helmeted), row 5: mage walk (book
in hand), row 6: mage casting from the open book. Row 7: shaman walk (hooded), row 8:
shaman calling fire from the sky, arm raised (2 frames).
Faces right.
By default the art is muted, softened and grained to match the AI-generated map;
--crisp keeps the clean hand-drawn pixels.
Needs Pillow and NumPy.
"""
import sys
import numpy as np
from PIL import Image, ImageFilter

CRISP = '--crisp' in sys.argv
PALETTE = {
    'K': (42, 34, 54, 255),     # outline
    'W': (239, 233, 220, 255),  # bone
    'g': (184, 174, 156, 255),  # bone shade
    'E': (26, 20, 36, 255),     # eye socket
    'r': (230, 70, 70, 255),    # eye glow
} if CRISP else {
    'K': (62, 50, 62, 255),
    'W': (205, 194, 172, 255),
    'g': (150, 140, 124, 255),
    'E': (40, 30, 44, 255),
    'r': (190, 70, 64, 255),
}
PALETTE.update({
    'b': (140, 90, 50, 255),    # bow wood
    's': (225, 215, 195, 255),  # bow string
    'a': (150, 110, 70, 255),   # arrow shaft
    'h': (185, 185, 195, 255),  # arrowhead
    'f': (200, 60, 60, 255),    # fletching
    'H': (220, 50, 64, 255),    # heart
    'P': (255, 170, 176, 255),  # heart shine
    'M': (150, 156, 170, 255),  # helmet steel
    'm': (98, 102, 118, 255),   # helmet rim
    'L': (215, 220, 230, 255),  # helmet shine
    'B': (112, 64, 150, 255),   # book cover
    'p': (238, 228, 196, 255),  # book pages
    'G': (255, 205, 100, 255),  # spell glow
    'C': (176, 54, 44, 255),    # shaman hood
    'c': (112, 32, 30, 255),    # hood shade
} if CRISP else {
    'b': (156, 104, 60, 255),
    's': (190, 182, 166, 255),
    'a': (128, 98, 66, 255),
    'h': (160, 160, 168, 255),
    'f': (170, 64, 60, 255),
    'H': (205, 52, 62, 255),
    'P': (240, 160, 164, 255),
    'M': (136, 140, 150, 255),
    'm': (92, 94, 106, 255),
    'L': (190, 194, 200, 255),
    'B': (106, 66, 136, 255),
    'p': (214, 204, 176, 255),
    'G': (236, 190, 100, 255),
    'C': (160, 58, 48, 255),
    'c': (104, 38, 34, 255),
})
FW, FH = 16, 20

SKULL = [
    "....KKKKKKK.....",
    "...KWWWWWWWK....",
    "..KWWWWWWWWWK...",
    ".KWWWWWWWWWWWK..",
    ".KWWWWWWWEEWWK..",
    ".KgWWWWWWErWWK..",
    ".KgWWWWWWEEWWWK.",
    ".KggWWWWWWWWWWK.",
    "..KgggWWKWKWKK..",
    "...KKKgWgWgWK...",
    ".....KKKKKKK....",
]
# Spine, ribs and an arm reaching forward.
TORSO = [
    "......KWK.......",
    ".....KWWWKKKKKK.",
    "....KWKWKWWWWWK.",
    ".....KWKWKKKKKK.",
    "......KgK.......",
]
LEGS = {
    'apart': [
        ".....KWKWK......",
        "....KWK..KWK....",
        "...KWK....KWK...",
        "...KK......KK...",
    ],
    'together': [
        ".....KWKWK......",
        ".....KWKWK......",
        ".....KWKWK......",
        "....KKK.KK......",
    ],
    'lifted': [
        ".....KWKWK......",
        "....KWK.KWK.....",
        "...KWK...KK.....",
        "...KK...........",
    ],
}
# Archer: shorter arm holding the bow out front.
ARCHER_TORSO = [
    "......KWK.......",
    ".....KWWWKKKK...",
    "....KWKWKWWWK...",
    ".....KWKWKKKK...",
    "......KgK.......",
]
# Bow limb, from row 7 down to row 17.
BOW = [
    "............KK..",
    "............KbK.",
    ".............KbK",
    ".............KbK",
    ".............KbK",
    ".............KbK",
    ".............KbK",
    ".............KbK",
    ".............KbK",
    "............KbK.",
    "............KK..",
]
# Dropped heart, sitting on the bottom of its cell.
HEART = [
    "....KK...KK.....",
    "...KHHK.KHHK....",
    "..KHPHHKHHHHK...",
    "..KHPHHHHHHHK...",
    "..KHHHHHHHHHK...",
    "...KHHHHHHHK....",
    "....KHHHHHK.....",
    ".....KHHHK......",
    "......KHK.......",
    ".......K........",
]
# Tank helmet: a steel dome over the skull, its rim low over the eyes.
HELMET = [
    "...KKKKKKKK.....",
    "..KMMMLLLMMK....",
    ".KMMMMMLLMMMK...",
    ".KMMMMMMMMMMK...",
    "KmmmmmmmmmmmmmK.",
]
# Mage's closed book with a gold emblem, held in the hand of the short arm.
BOOK = [
    "..........KKKKKK",
    "..........KBBBpK",
    "..........KBGBpK",
    "..........KBBBpK",
    "..........KBBBpK",
    "..........KKKKKK",
]
# Book held up open while casting.
OPEN_BOOK = [
    ".......KKKK.KKKK",
    ".......KpppKpppK",
    ".......KpppKpppK",
    ".......KBBBBBBBK",
    "........KKKKKKK.",
]
# Shaman hood: covers the back of the skull, leaving the face out.
HOOD = [
    "...KKKKKKKK.....",
    "..KCCCCCCCCK....",
    ".KCCCCCCCCCCK...",
    "KCCCCCCKKKKKK...",
    "KCCCCCK.........",
    "KCCCCK..........",
    "KCCCCK..........",
    "KcCCK...........",
    ".KcK............",
]
# Torso without the reaching arm, for the raised-arm pose.
CALL_TORSO = [
    "......KWK.......",
    ".....KWWWK......",
    "....KWKWKWK.....",
    ".....KWKWK......",
    "......KgK.......",
]
BONES = [
    ".KWWK.......KWWK",
    ".KKKK..KWK..KKKK",
]


def paint(img, rows, ox, oy):
    for y, row in enumerate(rows):
        assert len(row) == FW, (row, len(row))
        for x, ch in enumerate(row):
            if ch in PALETTE and 0 <= oy + y < img.height:
                img.putpixel((ox + x, oy + y), PALETTE[ch])


def put(img, x, y, ch):
    img.putpixel((x, y), PALETTE[ch])


def line(img, x0, y0, x1, y1, ch):
    steps = max(abs(x1 - x0), abs(y1 - y0))
    for i in range(steps + 1):
        put(img, round(x0 + (x1 - x0) * i / steps), round(y0 + (y1 - y0) * i / steps), ch)


def archer(img, ox, oy, legs, dip, aiming):
    """String behind the body, bow in front of it."""
    paint(img, LEGS[legs], ox, oy + 16)
    if aiming:
        # String pulled back to the hand, arrow nocked along the arm.
        line(img, ox + 12, oy + dip + 8, ox + 9, oy + dip + 13, 's')
        line(img, ox + 12, oy + dip + 16, ox + 9, oy + dip + 13, 's')
    else:
        line(img, ox + 12, oy + dip + 8, ox + 12, oy + dip + 16, 's')
    paint(img, SKULL + ARCHER_TORSO, ox, oy + dip)
    paint(img, BOW, ox, oy + dip + 7)
    if aiming:
        line(img, ox + 8, oy + dip + 13, ox + 14, oy + dip + 13, 'a')
        put(img, ox + 15, oy + dip + 13, 'h')
        put(img, ox + 8, oy + dip + 12, 'f')
        put(img, ox + 8, oy + dip + 14, 'f')


sheet = Image.new('RGBA', (FW * 4, FH * 9), (0, 0, 0, 0))
# Walk: legs apart / together / lifted / together, dipping on the stride frames.
for i, legs in enumerate(['apart', 'together', 'lifted', 'together']):
    dip = 0 if legs == 'together' else 1
    paint(sheet, SKULL + TORSO, FW * i, dip)
    paint(sheet, LEGS[legs], FW * i, 16)
# Collapse: sagging, skull falling, bone pile.
paint(sheet, SKULL + TORSO[:3], 0, FH + 4)
paint(sheet, LEGS['apart'], 0, FH + 16)
paint(sheet, BONES, FW, FH + 18)
paint(sheet, SKULL, FW, FH + 6)
paint(sheet, BONES, FW * 2, FH + 18)
paint(sheet, SKULL, FW * 2, FH + 9)
# Archer walk, then aiming / releasing.
for i, legs in enumerate(['apart', 'together', 'lifted', 'together']):
    archer(sheet, FW * i, FH * 2, legs, 0 if legs == 'together' else 1, aiming=False)
archer(sheet, 0, FH * 3, 'apart', 1, aiming=True)
archer(sheet, FW, FH * 3, 'apart', 0, aiming=False)
# Flying arrow, pointing right, centered on (8, 10) of its cell.
ax, ay = FW * 2, FH * 3
line(sheet, ax + 3, ay + 10, ax + 11, ay + 10, 'a')
put(sheet, ax + 12, ay + 10, 'h')
put(sheet, ax + 11, ay + 9, 'h')
put(sheet, ax + 11, ay + 11, 'h')
put(sheet, ax + 3, ay + 9, 'f')
put(sheet, ax + 3, ay + 11, 'f')
put(sheet, ax + 4, ay + 9, 'f')
put(sheet, ax + 4, ay + 11, 'f')
paint(sheet, HEART, FW * 3, FH * 3 + FH - len(HEART))
# Tank and mage walks: the plain walk with a helmet, the archer's short arm with a book.
for i, legs in enumerate(['apart', 'together', 'lifted', 'together']):
    dip = 0 if legs == 'together' else 1
    paint(sheet, SKULL + TORSO, FW * i, FH * 4 + dip)
    paint(sheet, HELMET, FW * i, FH * 4 + dip)
    paint(sheet, LEGS[legs], FW * i, FH * 4 + 16)
    paint(sheet, SKULL + ARCHER_TORSO, FW * i, FH * 5 + dip)
    paint(sheet, BOOK, FW * i, FH * 5 + dip + 10)
    paint(sheet, LEGS[legs], FW * i, FH * 5 + 16)
# Casting: book raised and open, sparks rising from the pages.
paint(sheet, SKULL + ARCHER_TORSO, 0, FH * 6)
paint(sheet, OPEN_BOOK, 0, FH * 6 + 9)
paint(sheet, LEGS['apart'], 0, FH * 6 + 16)
for x, y in [(12, 8), (13, 8), (13, 7), (14, 7), (14, 6), (15, 5), (15, 6), (11, 8)]:
    put(sheet, x, FH * 6 + y, 'G')
# Shaman walk: the plain walk under a hood.
for i, legs in enumerate(['apart', 'together', 'lifted', 'together']):
    dip = 0 if legs == 'together' else 1
    paint(sheet, SKULL + TORSO, FW * i, FH * 7 + dip)
    paint(sheet, HOOD, FW * i, FH * 7 + dip)
    paint(sheet, LEGS[legs], FW * i, FH * 7 + 16)
# Calling: arm thrust up in front of the face, hand glowing; the second frame shakes it.
for i, (hx, hy) in enumerate([(14, 2), (13, 1)]):
    ox, oy = FW * i, FH * 8
    paint(sheet, SKULL + CALL_TORSO, ox, oy)
    paint(sheet, HOOD, ox, oy)
    paint(sheet, LEGS['apart'], ox, oy + 16)
    line(sheet, ox + 8, oy + 12, ox + hx - 1, oy + hy, 'K')
    line(sheet, ox + 10, oy + 12, ox + hx + 1, oy + hy, 'K')
    line(sheet, ox + 9, oy + 12, ox + hx, oy + hy, 'W')
    for gx, gy in [(0, -1), (-1, -1), (1, -1), (0, -2), (-1, 0), (1, 0)][:4 + 2 * i]:
        if 0 <= hy + gy and 0 <= hx + gx < FW:
            put(sheet, ox + hx + gx, oy + hy + gy, 'G')


def grunge(cell, seed):
    """Softens a frame like a picture shrunk from a bigger one, then adds grain."""
    big = cell.resize((FW * 4, FH * 4), Image.Resampling.NEAREST)
    rgba = np.asarray(big).astype(float)
    alpha = rgba[..., 3:4] / 255.
    pre = np.dstack([rgba[..., :3] * alpha, rgba[..., 3:4]]).astype(np.uint8)
    blurred = Image.fromarray(pre, 'RGBA').filter(ImageFilter.GaussianBlur(1.3))
    small = np.asarray(blurred.resize((FW, FH), Image.Resampling.BOX)).astype(float)
    a = small[..., 3:4] / 255.
    rgb = np.where(a > 0, small[..., :3] / np.maximum(a, 1e-6), 0)
    rgb += np.random.default_rng(seed).normal(0, 7, rgb.shape)
    alpha = np.where(small[..., 3] > 100, 255, 0)
    return Image.fromarray(np.dstack([rgb.clip(0, 255), alpha]).astype(np.uint8), 'RGBA')


if not CRISP:
    for y in range(0, sheet.height, FH):
        for x in range(0, sheet.width, FW):
            box = (x, y, x + FW, y + FH)
            sheet.paste(grunge(sheet.crop(box), x * 31 + y), box)
sheet.save('assets/skeleton.png')
