"""Measure opt-in backends in a PTY; verify resize, input, default colors and restoration."""
import fcntl,os,pty,re,select,signal,statistics,struct,subprocess,termios,time,pathlib,json
# Presentation emits changed spans: parsing raw "6fps" fragments would lose the
# unchanged leading digit. Reconstruct the screen before reading telemetry.
class Screen:
 def __init__(self,w,h):self.w=w;self.h=h;self.rows=[[' ']*w for _ in range(h)];self.x=0;self.y=0;self.escape='';self.samples=[]
 def feed(self,data):
  for c in data.decode('ascii'):
   if self.escape:
    self.escape+=c
    if self.escape=='\x1b[':continue
    if self.escape.startswith('\x1b[') and '@'<=c<='~':
     raw=self.escape[2:-1];args=[int(n or '1') for n in raw.split(';')] if not raw.startswith('?') else []
     if c in 'Hf':self.y=(args[0] if args else 1)-1;self.x=(args[1] if len(args)>1 else 1)-1
     if c=='J' and raw=='2':self.rows=[[' ']*self.w for _ in range(self.h)]
     self.escape=''
    elif len(self.escape)==2 and c!='[':self.escape=''
   elif c=='\x1b':self.escape=c
   elif c=='\r':self.x=0
   elif c=='\n':self.y=min(self.y+1,self.h-1)
   elif ord(c)>=32:
    if 0<=self.x<self.w and 0<=self.y<self.h:self.rows[self.y][self.x]=c
    self.x=min(self.x+1,self.w-1)
  found=re.match(r'(\d+)fps ([\d.]+)ms',''.join(self.rows[-1]))
  if found:self.samples.append((float(found[1]),float(found[2])))
probe=Screen(20,2);probe.feed(b'\x1b[2;1H56fps 17.8ms');probe.feed(b'\x1b[2;');probe.feed(b'2H7fps 17.5ms');assert probe.samples[-1]==(57.,17.5)
results=[]
for backend,samples in [('fast','3x6'),('matched','3x6'),('matched','2x4'),('enriched','3x6')]:
 for width,height in [(120,40),(200,60)]:
  master,slave=pty.openpty();fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',height,width,0,0));before=termios.tcgetattr(slave)
  p=subprocess.Popen(['target/release/main','--telemetry','--ascii-backend',backend,'--match-samples',samples],stdin=slave,stdout=slave,stderr=slave)
  output=bytearray();screen=Screen(width,height)
  def drain(seconds):
   end=time.monotonic()+seconds
   while time.monotonic()<end:
    if select.select([master],[],[],.01)[0]:
     data=os.read(master,65536);output.extend(data);screen.feed(data)
     if b'\x1b[c' in data:os.write(master,b'\x1b[?1;2c')
  try:
   drain(2)
   for _ in range(20):os.write(master,b'w\x1b[C');drain(.1)
   measured=screen.samples
   assert measured and b'xyz' in output,output[-1000:]
   # Measured window precedes debug/resize probes.
   median=statistics.median(fps for fps,_ in measured[-40:])
   for key in [b'2',b'3',b'4',b'5',b'1']:os.write(master,key);drain(.12)
   for h,w in [(1,1),(25,80),(height,width)]:
    fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',h,w,0,0));p.send_signal(signal.SIGWINCH);drain(.2)
   os.write(master,b'q');drain(.4);p.wait(timeout=3)
   assert p.returncode==0,output[-1000:]
   assert termios.tcgetattr(slave)==before
   assert b'\x1b[?1049l' in output and b'\x1b[?25h' in output
   output.decode('utf-8')
   assert all(c in [b'',b'0',b'39',b'49',b'39;49'] for c in re.findall(rb'\x1b\[([0-9;]*)m',output))
   result=dict(backend=backend,samples=samples,width=width,height=height,median_reported_fps=median,bytes=len(output));results.append(result);print(result,flush=True)
  finally:
   if p.poll() is None:p.kill();p.wait()
   os.close(master);os.close(slave)
path=pathlib.Path('visual/research/performance/pty.json');path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(results,indent=2)+'\n')
