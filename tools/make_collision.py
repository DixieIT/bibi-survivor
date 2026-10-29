"""Draws assets/maps/village/collision.png and the foreground overlays for assets/maps/village/map.png.

Usage: python tools/make_collision.py [preview.png]
collision.png: opaque pixels are blocked; can be touched up by hand afterwards.
overlay.png + overlays.txt: bits of the map cut out and drawn over characters
standing behind them (none on this map yet).
Needs Pillow and NumPy.
"""
import sys
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

W, H = Image.open('assets/maps/village/map.png').size
BLOCK = (255, 0, 0, 160)

# Outlines in map pixels.
POLYGONS = {
    # Forest around the village.
    'forest_top': [(0, 0), (480, 0), (470, 60), (300, 95), (230, 140), (110, 120), (100, 0)],
    'forest_top_left': [(0, 0), (110, 0), (110, 330), (95, 560), (60, 650), (0, 660)],
    'forest_top_right': [(860, 0), (1024, 0), (1024, 320), (960, 300), (950, 170), (980, 120), (880, 60)],
    'forest_right': [(1024, 330), (1024, 560), (960, 520), (950, 380)],
    'forest_bottom': [(140, 765), (150, 690), (230, 640), (400, 650), (420, 700), (380, 765)],
    'forest_bottom_right': [(560, 765), (590, 720), (800, 700), (900, 690), (1024, 660), (1024, 765)],
    # Buildings.
    'house_yellow_back': [(240, 140), (290, 80), (400, 75), (455, 150), (455, 290), (330, 292), (300, 300), (250, 270)],
    'house_yellow_front': [(105, 240), (150, 160), (215, 155), (310, 250), (310, 350), (130, 350), (105, 320)],
    'house_red_left': [(90, 380), (150, 330), (240, 340), (310, 420), (300, 525), (250, 555), (170, 555),
                       (100, 560), (75, 500), (75, 450)],
    'house_red_right': [(540, 170), (600, 85), (700, 70), (760, 140), (740, 260), (690, 300), (620, 320),
                        (600, 270), (545, 240)],
    'ruin': [(685, 215), (720, 180), (800, 175), (860, 215), (862, 310), (840, 340), (720, 345), (685, 300)],
    'cart_right': [(795, 305), (865, 305), (865, 370), (795, 370)],
    'barrel_right': [(852, 270), (890, 270), (890, 312), (852, 312)],
    'tower': [(600, 0), (770, 0), (770, 80), (700, 95), (610, 80)],
    # Props.
    'well': [(470, 305), (530, 295), (600, 320), (612, 400), (575, 432), (515, 425), (470, 400)],
    'tent_top_left': [(525, 90), (560, 55), (610, 75), (600, 115), (530, 115)],
    'tent_top_right': [(880, 120), (930, 75), (965, 110), (960, 140), (885, 140)],
    'sacks_top': [(775, 60), (845, 60), (845, 115), (775, 115)],
    'campfire_top': [(825, 110), (870, 110), (870, 140), (825, 140)],
    'tent_bottom_right': [(920, 560), (965, 515), (1010, 560), (1010, 610), (925, 610)],
    'logs_bottom': [(820, 640), (880, 630), (885, 680), (825, 690)],
    'campfire_bottom': [(880, 620), (925, 620), (925, 655), (880, 655)],
    'barrel_plaza': [(300, 440), (332, 440), (332, 475), (300, 475)],
    'farm_fence': [(550, 690), (625, 625), (680, 595), (745, 625), (830, 680), (800, 720), (720, 745),
                   (620, 740), (555, 720)],
    # Shaded water right along the bridge, too dark to be found by colour.
    # Their inner edges follow the planks: W (632,534) N (700,495) E (792,549) S (722,593).
    'river_by_bridge_ne': [(700, 495), (792, 549), (830, 522), (830, 460), (760, 460), (718, 486)],
    'river_by_bridge_sw': [(632, 534), (722, 593), (722, 620), (600, 620), (600, 566), (622, 552)],
}

# The river is found by colour (neutral dark grey) inside this corridor,
# so walls and rocks elsewhere aren't caught. The bridge stays walkable.
RIVER_CORRIDOR = [(1024, 260), (1024, 440), (820, 520), (700, 580), (560, 660), (480, 765), (330, 765),
                  (420, 640), (560, 530), (760, 430), (900, 330)]
# Cut-outs drawn over characters whose feet are above (behind) the baseline:
# 'name': {'polygon': [...], 'baseline': [(x0, y0), (x1, y1)]}.
OVERLAYS = {}
BRIDGE = [(628, 532), (700, 493), (797, 545), (735, 600)]


def river_mask():
    rgb = np.asarray(Image.open('assets/maps/village/map.png').convert('RGB')).astype(int)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    lum = rgb.mean(axis=2)
    grey = (abs(r - g) < 9) & (abs(g - b) < 14) & (lum > 26) & (lum < 115)
    corridor = Image.new('L', (W, H), 0)
    ImageDraw.Draw(corridor).polygon(RIVER_CORRIDOR, fill=255)
    found = Image.fromarray(((grey & (np.asarray(corridor) > 0)) * 255).astype(np.uint8))
    # Close the speckles into one solid band, then pad it a little.
    found = found.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.MinFilter(5))
    # Drop specks narrower than the river (grey rocks and shadows).
    found = found.filter(ImageFilter.MinFilter(9)).filter(ImageFilter.MaxFilter(9))
    ImageDraw.Draw(found).polygon(BRIDGE, fill=0)
    return np.asarray(found) > 0

mask = Image.new('RGBA', (W, H), (0, 0, 0, 0))
draw = ImageDraw.Draw(mask)
for points in POLYGONS.values():
    draw.polygon(points, fill=BLOCK)
pixels = np.asarray(mask).copy()
pixels[river_mask()] = BLOCK
mask = Image.fromarray(pixels, 'RGBA')
mask.save('assets/maps/village/collision.png')

# Overlays: the map's own pixels inside each polygon, transparent elsewhere.
cut = Image.new('L', (W, H), 0)
lines = ['# <name> <baseline x0 y0 x1 y1> <bounds x y w h>, written by tools/make_collision.py']
for name, o in OVERLAYS.items():
    ImageDraw.Draw(cut).polygon(o['polygon'], fill=255)
    xs, ys = zip(*o['polygon'])
    (bx0, by0), (bx1, by1) = o['baseline']
    lines.append(f'{name} {bx0} {by0} {bx1} {by1} {min(xs)} {min(ys)} {max(xs) - min(xs) + 1} {max(ys) - min(ys) + 1}')
overlay = Image.new('RGBA', (W, H), (0, 0, 0, 0))
overlay.paste(Image.open('assets/maps/village/map.png').convert('RGBA'), (0, 0), cut)
overlay.save('assets/maps/village/overlay.png', optimize=True)
open('assets/maps/village/overlays.txt', 'w', newline='\n').write('\n'.join(lines) + '\n')

if len(sys.argv) > 1:
    preview = Image.open('assets/maps/village/map.png').convert('RGBA')
    preview.alpha_composite(mask)
    preview.save(sys.argv[1])
