# First rendering milestone

This records the original foundation. The subsequent [visual convergence pass](VISUAL.md)
updates the mapper defaults, theme behavior, captures, tests and measured performance.

## Audit and checkpoints

The original repository contained five Rust source files: a counter state, one-second
counter loop, static launcher, and libloading launcher. `cargo test` and `cargo check`
passed with zero tests. Both launchers ran. The loader declared `unsafe fn(*mut State)
-> State` while the library defined `fn(&mut State) -> ()`; this was an incompatible
Rust ABI call, despite appearing to work. Rebuilding the mapped library in place was
also unsafe operationally. The implementation retains the counter experiment behind
a versioned C interface and validates copied candidates before replacement.

Development checkpoints were: ABI repair; projected triangle and terminal framebuffer;
rotating cube/multiple meshes with clipping and lighting; architectural scene and ASCII;
first-person controls, collision, and lifecycle tests. The existing historical commits
were preserved. Rust/dependencies were updated under the requested aggressive policy.
No CI/CD/README rollout was performed: this VinnovateIT repository is explicitly excluded.
The placeholder README remains; BUILD.md is the project run guide.

## Measured validation

Local Linux container, optimized release build, deterministic rotating camera,
1,000 frames per CPU benchmark:

| Framebuffer | Render + ASCII conversion | CPU-only throughput |
| --- | --- | --- |
| 120×40 | 0.382 ms/frame | 2,620 FPS |
| 200×60 | 0.892 ms/frame | 1,121 FPS |

These timings include a checksum and exclude terminal output. They are not claims
about a particular terminal emulator. The interactive loop is capped at 60 FPS.
The drained Linux PTY test reported 59–60 FPS / approximately 16.8ms with presentation,
input, debug-mode changes and resize events. Real display latency/font rasterization
must still be judged in the user's terminal.

Tests cover projection near/far conventions, all-plane and near-plane clipping,
barycentric orientation, order-independent depth, rotating/multiple meshes, glyph
stability and validation, physical cell aspect, fixed timing, key expiry/release,
wall/crate collision, stair ascent/descent, all debug views, and exact room/stair
snapshots. PTY tests cover movement/look input, 1–5 modes, mouse reports, resizing
including 1×1, and Q/Esc/Ctrl+C exits with exact termios restoration. The hot-reload
test checks rejected ABI v2, continued old-state updates, and a successful ABI v1
C library replacement retaining the counter.

## Limits and next work

- The image is intentionally low-resolution, flat-lit and noisy; it approximates the
  supplied ASCII post-process aesthetic, not Pixatool's exact algorithm.
- Raw luminance/depth debug modes use simple glyph-density ramps in the terminal.
- No triangle-mesh collision, slopes, jumping, moving objects, or physics engine.
  Box collision matches this scene; the renderer itself accepts arbitrary meshes.
- Stair eye height steps in 20cm increments; no camera smoothing is applied.
- Legacy key-repeat and bounded cell-based mouse reports limit input fidelity; see BUILD.md.
- Resize reallocates buffers, but steady-state frames reuse them. Huge terminals are capped.
- Cleanup handles normal exits, returned errors and Rust panics; SIGKILL, aborts, and
  externally terminating the process cannot be guaranteed to restore terminal state.
- This environment validates PTY output and generated snapshots, not a human-operated
  terminal emulator. Final visual acceptance and input feel require that manual run.

Next: evaluate the milestone in the target terminal/font, tune the default glyph
families and contrast there, then add mesh import and ramp/triangle collision while
keeping the software rasterizer independent of gameplay.
