"""Build a self-contained synchronized temporal review (all 180 frames preserved)."""
import base64,gzip,json,pathlib,html,subprocess
root=pathlib.Path('visual/research/temporal')
data={}
for mode in ['fast','matched','hysteresis','topk']:
 data[mode]={}
 for motion in ['yaw','forward','strafe','stairs']:
  p=root/mode/motion
  data[mode][motion]=[f.read_text() for f in sorted(p.glob('frame-*.txt'))]
encoded=base64.b64encode(gzip.compress(json.dumps(data).encode(),mtime=0)).decode()
page='''<!doctype html><meta charset="utf-8"><title>ASCII temporal research</title>
<style>body{background:#141414;color:#ddd;font-family:monospace}main{display:flex;gap:16px;overflow:auto}pre{font:13px/16px monospace;margin:0}section{flex-shrink:0}button,select,input{margin:8px}</style>
<h1>Synchronized temporal research — 120×40, 60 simulation samples/sec</h1>
<p>Compare legitimate surface motion with gratuitous glyph popping. Playback speed can be reduced. Font rendering depends on your browser.</p>
<select id="motion"><option>yaw</option><option>forward</option><option>strafe</option><option>stairs</option></select>
<button id="play">Play/pause</button><select id="speed"><option value="1">1×</option><option value="0.25">¼×</option></select><input id="frame" type="range" min="0" max="179" value="0"><span id="label"></span><main></main>
<script>
const packed='PACKED';
(async()=>{let b=Uint8Array.from(atob(packed),c=>c.charCodeAt(0));let d=JSON.parse(await new Response(new Blob([b]).stream().pipeThrough(new DecompressionStream('gzip'))).text());
let modes=Object.keys(d),main=document.querySelector('main'),t=0,playing=false,last=0;
for(let m of modes){let s=document.createElement('section'),h=document.createElement('h2'),p=document.createElement('pre');h.textContent=m;s.append(h,p);main.append(s);}
let motion=document.querySelector('#motion'),slider=document.querySelector('#frame');
function draw(){let i=Math.floor(t)%180;document.querySelectorAll('pre').forEach((p,j)=>p.textContent=d[modes[j]][motion.value][i]);slider.value=i;document.querySelector('#label').textContent=`Frame ${i}/179`;}
slider.oninput=()=>{t=+slider.value;playing=false;draw()};motion.onchange=()=>{t=0;draw()};document.querySelector('#play').onclick=()=>playing=!playing;
function tick(now){if(playing)t+=(now-last)*.06*+document.querySelector('#speed').value;last=now;draw();requestAnimationFrame(tick)};requestAnimationFrame(tick);
})();</script>'''.replace('PACKED',encoded)
(root/'review.html').write_text(page)
# Closely spaced frames expose popping and structural continuity in a static review.
for motion in ['yaw','stairs']:
 parts=[]
 for y,mode in enumerate(['matched','hysteresis']):
  for x,i in enumerate([60,61,62]):
   rows=data[mode][motion][i].splitlines();pos=' '.join(str(i*8) for i in range(120))
   s=f'<svg x="{x*960}" y="{y*664}" width="960" height="664"><rect width="100%" height="100%" fill="#141414"/><g font-family="DejaVu Sans Mono" font-size="13" fill="#ddd"><text x="4" y="16">{mode} {motion} frame {i}</text>'
   s+=''.join(f'<text x="{pos}" y="{37+16*j}" xml:space="preserve">{html.escape(row)}</text>' for j,row in enumerate(rows))
   parts.append(s+'</g></svg>')
 p=root/f'{motion}-filmstrip.svg';p.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="2880" height="1328">'+''.join(parts)+'</svg>')
 subprocess.run(['magick',str(p),str(p.with_suffix('.png'))],check=True,stderr=subprocess.DEVNULL)
print(root/'review.html')
