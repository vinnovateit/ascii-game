"""Build side-by-side font previews from checked-in before/after text captures.
Requires ImageMagick only for PNG output; SVGs can be opened directly.
"""
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
output = root / 'visual' / 'comparisons'
output.mkdir(exist_ok=True)
for view in ['doorway', 'crates', 'stairs']:
    for stage in ['before', 'after']:
        subprocess.run([sys.executable,str(root/'scripts/visual_preview.py'),str(root/'visual'/stage/view)],check=True)
    for theme in ['dark', 'light']:
        left=(root/'visual/before'/view/f'{theme}.svg').read_text()
        right=(root/'visual/after'/view/f'{theme}.svg').read_text()
        # Nested SVGs retain the exact 8x16 cell aspect. Left = before; right = after.
        right=right.replace('<svg ','<svg x="960" ',1)
        pair='<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="640">'+left+right+'</svg>'
        svg=output/f'{view}-{theme}.svg'
        svg.write_text(pair)
        subprocess.run(['magick',str(svg),str(svg.with_suffix('.png'))],check=True,stderr=subprocess.DEVNULL)
