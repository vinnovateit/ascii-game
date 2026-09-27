"""Capture the frozen pipeline with each output vocabulary and build theme previews."""
import pathlib
import subprocess
import sys

root=pathlib.Path(__file__).resolve().parents[1]
captures=root/'visual/glyph-sets'
for view in ['doorway','crates','stairs']:
    for glyphs in ['ascii','hybrid']:
        dest=captures/glyphs/view
        subprocess.run([str(root/'target/release/main'),'--glyph-set',glyphs,'--camera-preset',view,'--capture-frame',str(dest)],check=True)
        subprocess.run([sys.executable,str(root/'scripts/visual_preview.py'),str(dest)],check=True)
    output=captures/'comparisons'
    output.mkdir(exist_ok=True)
    for theme in ['dark','light']:
        left=(captures/'ascii'/view/f'{theme}.svg').read_text()
        right=(captures/'hybrid'/view/f'{theme}.svg').read_text().replace('<svg ','<svg x="960" ',1)
        svg=output/f'{view}-{theme}.svg'
        svg.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="640">'+left+right+'</svg>')
        subprocess.run(['magick',str(svg),str(svg.with_suffix('.png'))],check=True,stderr=subprocess.DEVNULL)
print('Comparisons: ASCII left, hybrid right:',captures/'comparisons')
