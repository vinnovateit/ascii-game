"""Serial staged CPU measurements; run after a release build, without other CPU jobs."""
import csv,pathlib,subprocess
root=pathlib.Path('visual/research/performance');root.mkdir(parents=True,exist_ok=True)
rows=[]
for width,height in [(120,40),(200,60),(240,80)]:
 for trial in range(1,4):
  for name,method,sw,sh in [('fast','fast',3,6),('matched','gradient',3,6),('matched-low','gradient',2,4),('enriched','fast-source',3,6)]:
   out=pathlib.Path('/tmp/ascii-research-performance')/f'{width}x{height}'/name/str(trial)
   args=['target/release/research','--method',method,'--width',str(width),'--height',str(height),'--sample-width',str(sw),'--sample-height',str(sh),'--frames','120','--motion','yaw','--output',str(out)]
   if name!='fast':args+=['--noise','.22','--lighting','1','--black','.055','--white','.7','--hysteresis','.035']
   subprocess.run(args,check=True,capture_output=True,text=True)
   stats=dict(line.split('=',1) for line in (out/'metrics.txt').read_text().splitlines())
   row={'name':name,'trial':trial,'width':width,'height':height,**{k:stats[k] for k in ['render_ms','source_ms','convert_ms','cpu_ms']}}
   rows.append(row);print(row,flush=True)
   with (root/'trials.csv').open('w') as f:
    writer=csv.DictWriter(f,lineterminator="\n",fieldnames=list(row));writer.writeheader();writer.writerows(rows)
