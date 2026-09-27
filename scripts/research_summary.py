"""Index experiment measurements and compare finalist glyphs with one common target."""
import collections,csv,json,math,pathlib
root=pathlib.Path('visual/research')
entries=[]
for path in sorted(root.rglob('metrics.txt')):
 d=dict(line.split('=',1) for line in path.read_text().splitlines() if '=' in line)
 if 'method' in d:entries.append({'capture':str(path.parent.relative_to(root)),**d})
keys=['capture']+sorted(set().union(*(d.keys() for d in entries))-{'capture'})
with (root/'experiments.csv').open('w') as f:
 w=csv.DictWriter(f,lineterminator="\n",fieldnames=keys);w.writeheader();w.writerows(entries)
def pgm(path):
 b=path.read_bytes();magic,size,maximum,pixels=b.split(b'\n',3)
 assert magic==b'P5' and maximum==b'255'
 width,height=map(int,size.split());assert len(pixels)==width*height
 return width,height,[x/255 for x in pixels]
glyphs=" .,'`:;!il|/\\-_~=+*xX&%#MW@<>^v[]()"
records=[]
for view in ['doorway','crates','stairs']:
 aw,ah,a=pgm(root/'final/matched'/view/'atlas.pgm');assert aw==len(glyphs)*3 and ah==6
 masks={c:[a[y*aw+k*3+x] for y in range(6) for x in range(3)] for k,c in enumerate(glyphs)}
 width,height,target=pgm(root/'final/matched'/view/'source.pgm')
 for backend in ['fast','matched','enriched']:
  directory=root/'final'/backend/view;lines=(directory/'frame.txt').read_text().splitlines();cells=''.join(lines);counts=collections.Counter(cells);n=len(cells)
  recon=[0.]*len(target)
  for i,c in enumerate(cells):
   mask=masks[c]
   for y in range(6):
    for x in range(3):recon[(i//120*6+y)*width+i%120*3+x]=mask[y*3+x]
  row=dict(backend=backend,view=view,occupancy=1-counts[' ']/n,entropy=-sum((v/n)*math.log2(v/n) for v in counts.values()),atlas_density=sum(recon)/len(recon),common_target_mse=sum((a-b)**2 for a,b in zip(target,recon))/len(recon),common_target_mae=sum(abs(a-b) for a,b in zip(target,recon))/len(recon))
  records.append(row)
  (directory/'statistics.json').write_text(json.dumps({**row,'distribution':dict(sorted(counts.items())),'target':'same PGM-quantized enriched source and 3x6 atlas for all three backends'},indent=2)+'\n')
with (root/'final/statistics.csv').open('w') as f:
 w=csv.DictWriter(f,lineterminator="\n",fieldnames=list(records[0]));w.writeheader();w.writerows(records)
print('Indexed',len(entries),'experiments and',len(records),'finalist frames')
