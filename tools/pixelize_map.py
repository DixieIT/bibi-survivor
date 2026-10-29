"""Cleans an AI-generated / JPG map into crisp pixel art on a regular grid.

Usage: python tools/pixelize_map.py source.jpg assets/map.png [block] [colors]
Shrinks the picture so each `block` x `block` square becomes one pixel, snaps
colours to a shared palette (removing JPG noise), then scales it back up with
hard edges, so the output keeps the source size and every collision position.
Needs Pillow.
"""
import sys
from PIL import Image

src, dst = sys.argv[1], sys.argv[2]
block = int(sys.argv[3]) if len(sys.argv) > 3 else 3
colors = int(sys.argv[4]) if len(sys.argv) > 4 else 96

img = Image.open(src).convert('RGB')
w, h = img.size
small = img.resize((round(w / block), round(h / block)), Image.Resampling.LANCZOS)
small = small.quantize(colors, method=Image.Quantize.MEDIANCUT, kmeans=3, dither=Image.Dither.NONE).convert("RGB")
small.resize((w, h), Image.Resampling.NEAREST).save(dst, optimize=True)
