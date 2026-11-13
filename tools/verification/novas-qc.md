# Independent benchmark quality control

This QC is based on `fix/lahiri-full-nutation-import` at
`9f84a6e9ec815fb52418f26d98a6ccdd78d688dd`. It adds tooling only: library
calculations, fitted datasets and regression fixtures are unchanged.

Start the independent MIT benchmark from the exact commit recorded in
`novas-lock.json`, using its setup command and pinned JPL inputs. The runner
checks the service-source, dependency and dataset identities, and requires the
same native executable identity throughout the run. The benchmark and the
library must remain separate repositories.

```sh
node tools/verification/novas-qc.mjs http://127.0.0.1:8788 target/offset-qc.json
node tools/verification/novas-qc.mjs http://127.0.0.1:8788 target/chart-qc.json /path/to/cheb.bin --wasm
```

The runner requires Rust 1.96.0. With `--wasm`, it also requires Bun, the wasm32
Rust target and wasm-bindgen-cli 0.2.122. It uses `target/qc-tools/bin/wasm-bindgen`
if installed there, or WASM_BINDGEN/an executable on PATH. It rebuilds both the
native capture and Wasm adapter, records executable/glue hashes and a source
snapshot hash, and rejects source/dataset changes during the run.

Exit 0 means all requested checks passed; 1 means completed numerical failures;
2 means setup, transport, identity or contract failure. Reports retain failure
rows. Do not change formulas, thresholds or frozen fixtures merely to make QC
pass. The manually dispatched `Explicit independent benchmark QC` workflow can
run offset-only, or native plus Wasm charts with an explicitly SHA-pinned dataset.
It is a separate opt-in gate, not a required baseline PR check.

## Scope and time conventions

Offset-only covers 408 cases: a two-year grid over 1801–2199, 80 short-interval
2026 cases, and 128 deterministic held-out instants (seed 0x4e4f5641).
Original IAU 2000B longitude/obliquity and reported Lahiri offset use a fixed
one-microarcsecond tolerance. The source library's 77 terms and the benchmark's
ERFA routine share the same standard; this checks implementation and integration,
not independent evidence that the standard itself is correct.

With a dataset, 375 additional chart cases cover 125 dates in 1851–2099 across
tropical, Lahiri and Fagan-Bradley. Eleven major/derived bodies supply longitude,
speed and declination; limits are one arcsecond and one arcsecond/day. The runner
uses captured effective library TT for matched offset checks, and the physical
TT corresponding to each UTC input for end-to-end body checks. The latter retain
time-model effects rather than silently correcting candidate values.

The benchmark projects its full-nutation tropical positions with a matching
full-nutation offset. The library retains its five-term projection while reporting
the original 77-term offset. Neither is substituted for the other during QC.

Reports explicitly set `fullChartCoverage: false`: houses, nodes, Ceres, Chiron
and True Sky are still outside this client. Whole Sign is used to obtain real
chart outputs, but its cusps are not yet validated. Unsupported coverage and
calculation errors are failures, never silently skipped successful evidence.
