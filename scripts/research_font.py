"""Optional development calibration; needs ImageMagick, no bundled font.
Usage: uv run --no-project python scripts/research_font.py FONT_PATH OUTPUT.pgm [WIDTH HEIGHT]
Produces canonical P5 with 95 printable ASCII cells in one row.
"""
import pathlib
import subprocess
import sys
font=pathlib.Path(sys.argv[1]).resolve()
if not font.is_file(): raise FileNotFoundError(font)
dest=pathlib.Path(sys.argv[2])
w,h=map(int,sys.argv[3:5]) if len(sys.argv)>3 else (6,10)
# Rasterize at 4x the 8x16 preview cell, then area downsample to matching resolution.
cells=[]
for code in range(32,127):
    literal=chr(code).replace('\\','\\\\').replace('%','%%')
    p=subprocess.run(['magick','-size','32x64','xc:black','-font',str(font),'-pointsize','52','-fill','white','-annotate','+0+52',literal,'-filter','box','-resize',f'{w}x{h}!','-colorspace','gray','-depth','8','gray:-'],capture_output=True,check=True)
    if len(p.stdout)!=w*h:raise RuntimeError('unexpected raster size')
    cells.append(p.stdout)
data=b''.join(c[y*w:(y+1)*w] for y in range(h) for c in cells)
dest.parent.mkdir(parents=True,exist_ok=True)
dest.write_bytes(f'P5\n{w*95} {h}\n255\n'.encode()+data)
print(dest,'font',font,'cell',w,h)
