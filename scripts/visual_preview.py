"""Render capture text as SVG on two themes (a font preview, not a terminal screenshot).
Usage: uv run --no-project python scripts/visual_preview.py CAPTURE_DIRECTORY
"""
import html
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
frame = root / 'frame.txt'
rows = (frame if frame.exists() else root / 'ascii.txt').read_text().splitlines()
width = len(rows[0]) * 8
height = len(rows) * 16
positions = ' '.join(str(x*8) for x in range(len(rows[0])))
for theme, foreground, background in [('dark', '#dedede', '#171717'), ('light', '#282828', '#f4f1e8')]:
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">'
    svg += f'<rect width="100%" height="100%" fill="{background}"/><g font-family="DejaVu Sans Mono" font-size="13" fill="{foreground}">'
    svg += ''.join(f'<text x="{positions}" y="{13+y*16}" xml:space="preserve">{html.escape(row)}</text>' for y, row in enumerate(rows))
    svg += '</g></svg>'
    (root / f'{theme}.svg').write_text(svg)
