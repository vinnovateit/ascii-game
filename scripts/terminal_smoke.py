"""Linux PTY lifecycle/input/resize smoke test; run with uv run --no-project."""
import fcntl
import os
import pty
import re
import select
import signal
import struct
import subprocess
import termios
import time


def smoke(exit_key, glyph_set="ascii"):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 120, 0, 0))
    before = termios.tcgetattr(slave)
    process = subprocess.Popen(['target/release/main', '--telemetry', '--glyph-set', glyph_set], stdin=slave, stdout=slave, stderr=slave)
    output = bytearray()
    width_probes = 0

    def drain(seconds):
        nonlocal width_probes
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            if select.select([master], [], [], .01)[0]:
                data = os.read(master, 65536)
                output.extend(data)
                # Reply as a legacy terminal to capability queries.
                if b'\x1b[6n' in data:
                    width_probes += data.count(b'\x1b[6n')
                    os.write(master, b'\x1b[1;2R')
                if b'\x1b[c' in data:
                    os.write(master, b'\x1b[?1;2c')

    try:
        drain(2.)
        assert b'xyz' in output, 'first frame missing'
        assert width_probes == (8 if glyph_set == 'hybrid' else 0), width_probes
        for key in [b'w', b'a', b's', b'd', b'\x1b[C', b'\x1b[A', b'2', b'3', b'4', b'5', b'1']:
            os.write(master, key)
            drain(.1)
        os.write(master,b'v')
        drain(.15)
        assert b'exposure=' in output, 'tuning selection missing'
        for key in [b'=',b'+',b'-',b'_',b'.',b',',b'p',b'r']:
            os.write(master,key)
            drain(.08)
        assert b'neutral' in output, 'preset switch missing'
        # Reset selected preset and return to the intended default.
        os.write(master,b'prv')
        drain(.1)
        for rows, columns in [(25,80),(1,1),(50,160),(40,120)]:
            fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',rows,columns,0,0))
            process.send_signal(signal.SIGWINCH)
            drain(.15)
        os.write(master,b'\x1b[<35;30;15M\x1b[<35;40;17M')
        drain(.2)
        os.write(master,exit_key)
        drain(.2)
        process.wait(timeout=3)
        assert process.returncode == 0, output[-1000:]
        assert termios.tcgetattr(slave) == before, 'termios not restored'
        assert b'\x1b[?1049h' in output and b'\x1b[?1049l' in output
        assert b'\x1b[?25h' in output and b'\x1b[?7h' in output
        # Default/reset SGR is allowed, explicit foreground/background SGR is not.
        sgr = re.findall(rb'\x1b\[([0-9;]*)m',output)
        assert all(code in (b'',b'0',b'39',b'49',b'39;49') for code in sgr), sgr
        output.decode('utf-8')  # Updated spans must never split multibyte glyphs.
        fps = re.findall(rb'(\d+)fps ([\d.]+)ms',output)
        print('PTY input/resize/exit passed:',glyph_set,repr(exit_key),'telemetry samples:',fps[-3:],'bytes:',len(output))
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        os.close(master)
        os.close(slave)


for key in [b'q',b'\x1b',b'\x03']:
    smoke(key)

smoke(b'q', 'hybrid')

# Verify rejection of terminals that report wide cells or do not support CPR.
for report in [b'\x1b[1;3R', None]:
    master, slave = pty.openpty()
    fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',40,120,0,0))
    before=termios.tcgetattr(slave)
    process=subprocess.Popen(['target/release/main','--glyph-set','hybrid'],stdin=slave,stdout=slave,stderr=slave)
    output=bytearray()
    try:
        deadline=time.monotonic()+6
        while time.monotonic()<deadline and process.poll() is None:
            if select.select([master],[],[],.05)[0]:
                data=os.read(master,65536);output.extend(data)
                if b'\x1b[c' in data:os.write(master,b'\x1b[?1;2c')
                if b'\x1b[6n' in data and report:os.write(master,report)
        process.wait(timeout=1)
        while select.select([master],[],[],.05)[0]:output.extend(os.read(master,65536))
        assert process.returncode!=0
        assert b'use --glyph-set ascii' in output, output
        assert termios.tcgetattr(slave)==before
        assert b'\x1b[?1049l' in output
        print('Hybrid width rejection and restoration passed:',repr(report))
    finally:
        if process.poll() is None:process.kill();process.wait()
        os.close(master);os.close(slave)
