"""Side-by-side font previews for research captures (not terminal screenshots).
Usage: uv run --no-project python scripts/research_compare.py OUTPUT.png CAPTURE_DIR...
"""
import html,pathlib,subprocess,sys
light='--light' in sys.argv
if light:sys.argv.remove('--light')
out=pathlib.Path(sys.argv[1]);dirs=[pathlib.Path(p) for p in sys.argv[2:]]
parts=[];left=0;height=0
for path in dirs:
 rows=(path/'frame.txt').read_text().splitlines();w=len(rows[0])*8;h=len(rows)*16+24;height=max(height,h)
 positions=' '.join(str(x*8) for x in range(len(rows[0])))
 weight=700 if 'font_bold_current' in str(path) else 400
 s=f'<svg x="{left}" width="{w}" height="{h}"><rect width="100%" height="100%" fill="#141414"/><g font-family="DejaVu Sans Mono" font-size="13" font-weight="{weight}" fill="#dedede">'
 s+=f'<text x="4" y="16">{html.escape(str(path.relative_to(pathlib.Path("visual/research"))))}</text>'
 s+=''.join(f'<text x="{positions}" y="{37+y*16}" xml:space="preserve">{html.escape(row)}</text>' for y,row in enumerate(rows))
 parts.append(s+'</g></svg>');left+=w
out.parent.mkdir(parents=True,exist_ok=True)
if light:parts=[s.replace('#141414','#f4f1e8').replace('#dedede','#282828') for s in parts]
svg=out.with_suffix('.svg');svg.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{left}" height="{height}">'+''.join(parts)+'</svg>')
subprocess.run(['magick',str(svg),str(out)],check=True,stderr=subprocess.DEVNULL)
print(out)
