"""Validate the Linux C ABI, rejected reloads, and state continuity."""
import pathlib
import select
import shutil
import subprocess
import tempfile

with tempfile.TemporaryDirectory() as directory:
    root = pathlib.Path(directory)
    library = root / 'logic.so'
    shutil.copyfile('target/release/libascii_game.so', library)
    process = subprocess.Popen(['target/release/hot', str(library)], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    def line():
        assert select.select([process.stdout], [], [], 5)[0], 'hot loader timed out'
        return process.stdout.readline().strip()

    def replace(version):
        source = root / 'candidate.c'
        source.write_text('#include <stdint.h>\n'
                          'struct State { int32_t counter; };\n'
                          f'uint32_t ascii_abi_version(void) {{ return {version}; }}\n'
                          'void fmain(struct State *s) { s->counter += 7; }\n')
        candidate = root / 'candidate.so'
        subprocess.run(['cc', '-shared', '-fPIC', str(source), '-o', str(candidate)], check=True)
        candidate.replace(library)

    try:
        assert line() == 'Hot counter: 1'
        replace(2)
        assert 'Keeping previous library: incompatible hot ABI' in line()
        # Readline may buffer the following line internally.
        assert process.stdout.readline().strip() == 'Hot counter: 2'
        replace(1)
        assert line() == 'Reloaded ABI v1'
        assert process.stdout.readline().strip() == 'Hot counter: 9'
        print('Hot ABI rejection/replacement/state continuity passed')
    finally:
        process.terminate()
        process.wait(timeout=3)
