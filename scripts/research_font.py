"""Optional development calibration; needs ImageMagick, no bundled font.
Usage: uv run --no-project python scripts/research_font.py FONT_PATH OUTPUT.pgm [WIDTH HEIGHT]
Produces canonical P5 with 95 printable ASCII cells in one row.
"""
import pathlib
import subprocess
import sys
font=pathlib.Path(sys.argv[1]).resolve()
dest=pathlib.Path(sys.argv[2])
w,h=map(int,sys.argv[3:5]) if len(sys.argv)>3 else (6,10)
# Rasterize at 4x the 8x16 preview cell, then area downsample to matching resolution.
cells=[]
for code in range(32,127):
    svg=f'<svg xmlns="http://www.w3.org/2000/svg" width="32" height="64"><rect width="32" height="64" fill="black"/><text x="0" y="52" font-size="52" fill="white">&#{code};</text></svg>'
    p=subprocess.run(['magick','-background','black','-font',str(font),'svg:-','-filter','box','-resize',f'{w}x{h}!','-colorspace','gray','-depth','8','gray:-'],input=svg.encode(),capture_output=True,check=True)
    if len(p.stdout)!=w*h:raise RuntimeError('unexpected raster size')
    cells.append(p.stdout)
data=b''.join(c[y*w:(y+1)*w] for y in range(h) for c in cells)
dest.parent.mkdir(parents=True,exist_ok=True)
dest.write_bytes(f'P5\n{w*95} {h}\n255\n'.encode()+data)
print(dest,'font',font,'cell',w,h)
