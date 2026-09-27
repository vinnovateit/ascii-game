"""Preview optional foreground-intensity captures on dark and light backgrounds."""
import pathlib,html,subprocess,sys
for view in ['doorway','crates','stairs']:
 root=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else 'visual/research/intensity')/view
 rows=(root/'frame.txt').read_text().splitlines();alphas=list(map(float,(root/'alphas.txt').read_text().split()))
 parts=[]
 for col,kind in enumerate(['normal','dim-approximation','grayscale']):
  for row,theme in enumerate(['dark','light']):
   bg='#141414' if theme=='dark' else '#f4f1e8';base=222 if theme=='dark' else 40
   s=f'<svg x="{col*960}" y="{row*664}" width="960" height="664"><rect width="100%" height="100%" fill="{bg}"/><g font-family="DejaVu Sans Mono" font-size="13" fill="rgb({base},{base},{base})"><text x="4" y="16">{kind} / {theme}</text>'
   for y,line in enumerate(rows):
    for x,c in enumerate(line):
     if c==' ':continue
     a=alphas[y*len(line)+x]
     if kind=='grayscale':v=round(a*255)
     elif kind=='dim-approximation':v=round((base+(20 if theme=='dark' else 244))/2) if a<.5 else base
     else:v=base
     s+=f'<text x="{x*8}" y="{37+y*16}" fill="rgb({v},{v},{v})">{html.escape(c)}</text>'
   parts.append(s+'</g></svg>')
 svg=root/'comparison.svg';svg.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="2880" height="1328">'+''.join(parts)+'</svg>')
 subprocess.run(['magick',str(svg),str(svg.with_suffix('.png'))],check=True,stderr=subprocess.DEVNULL)
