# Plan: standard-nutation Lahiri offset, validated against a NOVAS benchmark

> Import provenance: this document records claims and decisions from the returned
> coding project. Its external benchmark reports and temporary Git history were
> not supplied, and those reported experiments were not independently rerun during
> import. Local checks are reported separately in the import PR.

- **Status:** all phases (0–4) are complete (2026-11-12). Committed: Phases 0–1 in `5970a0c`, Phase 2 in
  `afaa8c3`, Phase 3 results in `416cbfe`, Phase 4 and the `records.json` deletion (D7) in "Add library-generated Lahiri regression fixture". All decisions
  (D1–D7) are made.
- **Library revision:** `7c28758` (export of `feat/lahiri` `bc31f43`)
- **Toolchain:** rustc 1.96.0
- **Research:** [runtime nutation correction (option C)](../research/2026-11-12-runtime-nutation-correction.md)

## 1. Goal

The library reports a Lahiri true ayanamsa that matches a local benchmark: the IAE mean formula
plus IAU 2000B nutation in longitude (Δψ₇₇). The benchmark computes Δψ with NOVAS C 3.1 compiled from USNO's public-domain source, and the library carries its own Rust transcription of the same table (D3).
Lahiri sidereal positions and speeds stay **exactly** as they are today. The library then generates its
own Lahiri regression fixtures. The benchmark stays outside the repo (`~/lahiri-benchmark`, outside the repo).

## 2. Background

- The library's nutation keeps 5 of the 77 IAU 2000B luni-solar terms (`src/astro/frames.rs::iau2000b`).
  Over 1800–2200 its Δψ₅ differs from Δψ₇₇ by up to 0.270″. The reported Lahiri offset (IAE mean + Δψ₅)
  inherits that error. The mean formula agreed with external calibrated records to within a few mas.
- Δψ₅ is also baked into the dataset's tropical longitudes, so it cancels in sidereal longitudes.
  Fixing only the offset (option A) would make Lahiri sidereal Sun and Moon worse: 0.017″ → 0.249″
  against the records. Details and numbers are in the research note.
- **The IAE's own tables confirm full nutation.** IAE 2027 p. 423 tabulates the true ayanamsa
  (= mean + nutation in longitude, 5h29m IST ≈ 0h TT) every 3 days for 2027-01-01..2028-04-25, to 0.1″.
  Against 161 tabulated dates:

  | Candidate | max | rounds to the table |
  |---|---|---|
  | Library today (IAE mean + 5-term) | 0.213″ | 65/161 |
  | IAE mean + NOVAS `iau2000b` | 0.052″ | **158/161** |
  | IAE mean + NOVAS `iau2000a` | 0.052″ | 158/161 |

  The 3 misses are rounding-boundary cases: the computed values end in .x48–.x49″ and the table rounds up.
  So B brings the reported offset in line with the IAE's own published values, not only with a third-party benchmark.
- **Option B** therefore separates two values:
  - the **reported** offset becomes standard (mean + Δψ₇₇);
  - the **projection** offset, used to turn tropical into sidereal, stays mean + Δψ₅.

  Sidereal output is unchanged bit for bit.
- **Documented exception:** for Lahiri, `sidereal = tropical − reported offset + (Δψ₇₇ − Δψ₅)`.
  The extra term is at most 0.270″. Fagan–Bradley, True Sky and tropical output are untouched.
  Option C would remove the exception later.

## 3. Decisions

### D1. Scope: **B** (decided)

### D2. Nutation model: **IAU 2000B, 77 terms** (recommended; no objection raised)

IAU 2000B is within 0.0038″ of IAU 2006/2000A over 1800–2200, uses a small table, and uses the same
linear fundamental arguments as the current code.

### D3. Coefficient source: **NOVAS C 3.1** (decided 2026-11-12)

B still needs the full 77-term table, because the reported offset needs Δψ₇₇. The sidereal path does not.

| Source | Reachable here | Terms | Notes |
|---|---|---|---|
| McCarthy & Luzum (2003), *Cel. Mech. Dyn. Astron.* 85, 37–49; IERS Conventions 2003/2010 Table 5.3a | no (IERS/SOFA sites blocked) | scientific publication | Primary source. Its fixed "planetary bias" offsets are Δψ −1.5835 mas / Δε +1.6339 mas. |
| **NOVAS C 3.1** `Cdist/nutation.c::iau2000b` (USNO), in the `novas` 3.1.1.6 sdist (sha-verified from PyPI) | yes | **public domain**: "produced by the United States Naval Observatory … not susceptible to copyright … it cannot be licensed; it is simply free" (package README) | 77 rows. Cites McCarthy & Luzum (2003) and IERS Conventions (2003). Uses Luzum (2001) offsets −0.135 / +0.388 mas. Same fundamental arguments (Simon et al. 1994) as the current code. |
| ERFA `nut00b.c` in the pyerfa 2.0.1.5 sdist (sha-verified from PyPI) | yes | BSD-3 (NumFOCUS), derived with permission from SOFA | Same offsets. This is the benchmark's own implementation. |
| Astronomy Engine 2.1.19 (npm and PyPI, MIT) | yes | MIT (already attributed) | **Only the same 5 terms** the library has; no 77-term table. Other functions in it are marked "adapted from NOVAS C 3.1". |
| `sofars` 0.6.1 (already in `Cargo.lock` via ANISE) | yes | labelled MIT | A port of SOFA; the MIT grant's relationship to SOFA's licence terms is unclear. Not recommended as a source. |

**Cross-check done (2026-11-12):**
- The NOVAS and ERFA tables are identical in all 77 rows × 11 values (exact comparison of the parsed C sources).
- The library's 5 current terms are exactly NOVAS rows 1–5, which is consistent with Astronomy Engine deriving them from NOVAS.

**The offsets must match the benchmark.** Use the Luzum (2001) values, which NOVAS and ERFA both use.
Taking the paper's values would give a constant ~1.45 mas offset and fail a 1 µas check.

**Licensing, in brief (not legal advice):**
- In the US, facts are not protected by copyright (*Feist v. Rural*, 1991). A compilation is protected only
  for original selection or arrangement, and a licence clearly covers a file's *expression*, not the numbers.
- The EU and UK also have a *sui generis* database right.
- NOVAS sidesteps the question. As a US government work it has no US copyright (17 U.S.C. § 105), and USNO
  states it is unlicensed and free. Its status outside the US isn't addressed by that statement.

**Recommendation:**
- Transcribe the 77 rows from NOVAS C 3.1 into our own table layout, and cite McCarthy & Luzum (2003) and
  NOVAS C 3.1 (USNO) in the source comment. Citation is good practice, not a licence condition.
- No new notice is needed in `THIRD_PARTY_NOTICES.md`.
- Validate against the benchmark at 1 µas. The benchmark runs NOVAS's original C, so any error in our
  Rust transcription is caught. An error in NOVAS's table itself is ruled out by the exact match with ERFA
  recorded above (a one-time check; ERFA is no longer used by the benchmark).

### D4. Public `lahiri_ayanamsa(epoch) -> Result<f64, CalculationError>` (recommended)

This mirrors `fagan_bradley_ayanamsa` and returns the **reported** (standard) offset. It lets the
benchmark cover all of 1800–2200, where charts need ephemeris coverage (~1850–2150).

### D5. Convention revision: **no bump** (decided 2026-11-12)

Lahiri on `feat/lahiri` is an unreleased draft.

### D6. Mean-formula constants: confirmed from IAE 2027; decision (a)

Source: IAE 2027, printed p. 382 (PDF p. 389), section "AYANAMSA". I checked the rendered page, not only
the text layer. It says the polynomial is the precession in longitude of Capitaine et al. (2003), in use
since 2021:

> Mean Ayanamsa = 23°51′25″.53 + 5028″.796195·T + 1″.1054348·T² + 0″.00007964·T³ − 0″.00023857·T⁴ − 0″.0000000383·T⁵,
> T = (JD − 2451545)/36525. "Ayanamsha for J2000.0 is taken as 23°51′25″.53."

- **Anchor: 25.53″.** This is confirmed; `src/sidereal.rs` matches, and the records' implied 25.532″ is not
  supported. The 1956 definition on p. 381 (PDF p. 388) reads 23°15′00″ at 0h, 21 March 1956. My recalled
  23°15′00.658″ was wrong.
- **The t⁴ coefficient: the IAE prints −0.00023857**, and `src/sidereal.rs` transcribes it faithfully. The
  source the IAE cites, Capitaine et al. (2003) p_A (ERFA `p06e.c`), has −0.000023857. It is most likely a
  typo in the IAE.
  - The IAE's own tables can't decide: in 2027 the difference is 1e-6″.
  - Over 1800–2200 the difference is at most 0.0034″ (0.0011″ over 1850–2150), below the IAE's 0.01″ precision.
- **The time scale is not stated.** The tables at 5h29m IST (≈ 0h TT) imply TT, which matches the library.
- **Unexplained, sub-0.01″:** the stated mean values on p. 423 (24°14′03″.22 at 2027.0 and 24°14′53″.48 at
  2028.0, reproduced at Jan 0.0 TT) are 0.007″ above the printed formula (3.213″ and 53.473″). Together with
  the 3 rounding misses, this suggests the IAE's software differs from its printed formula by a few mas.

**Decision (a), 2026-11-12: keep the IAE's printed t⁴ value.** `src/sidereal.rs` therefore stays as it is,
the discrepancy with Capitaine (2003) is documented, and Lahiri sidereal output stays bit-identical.
The benchmark uses the same literal value. The gap to P03 is 0.0034″ over 1800–2200.

### D7. `records.json`: **deleted** (decided 2026-11-12)

The NOVAS service replaced it as the benchmark, and the IAE publication replaced it as evidence for the
constants. The file and every derived copy (the research position extract and scratch files) were deleted.
The research note's position tables remain as recorded measurements but can no longer be reproduced.

## 4. Work plan

### Phase 0: benchmark service (outside the repo) — DONE 2026-11-12

- **Nutation: NOVAS C 3.1**, compiled from the public-domain source in the `novas` 3.1.1.6 sdist
  (SHA-256 `196191ec…`, verified) into `libnovas.so` and called through `ctypes`. Only `iau2000b` and
  `iau2000a` are used.
  - The Python `novas` package is not used: it builds with `distutils`, which Python 3.12 removed.
  - Compiler warnings come only from star-catalog string handling and the JPL file reader, which the
    benchmark never calls.
- **Verified 2026-11-12:** NOVAS `iau2000b`/`iau2000a` and ERFA `nut00b`/`nut00a` agree **bit for bit**
  over 25,001 epochs from 1800 to 2200.
- **NOVAS can't serve the mean formula.** It has no IAU 2006 p_A, only the two-term approximation `apa`
  used as a planetary-nutation argument. Deriving p_A geometrically from its precession (the J2000
  equinox's longitude on the ecliptic of date) differs from p_A by up to 2 mas.
  - So the benchmark evaluates the P03 polynomial, transcribed from ERFA `p06e.c` (Capitaine et al. 2003),
    with the anchor and t⁴ coefficient as recorded parameters (D6).
- **No ERFA at runtime.** The ERFA/pyerfa startup self-check was removed on 2026-11-12 (BSD library not
  wanted in the benchmark), and pyerfa was uninstalled. Two one-time checks made before removal are recorded
  as evidence:
  - NOVAS `iau2000b`/`iau2000a` equal ERFA `nut00b`/`nut00a` (0 and 3.6e-15″ over 1,025 epochs).
  - The transcribed polynomial, with the P03 t⁴, equals ERFA `p06e` p_A to 8e-12″.
  The service is now Python stdlib plus NOVAS only.
- `POST /lahiri` takes `{"jdTT": [...]}` and returns, per epoch: the mean, Δψ/Δε (2000B, 2000A) and the
  true ayanamsa per model.
- `GET /manifest` returns the NOVAS sdist, `libnovas.so` and server hashes, and the reference parameters.
- The `compare/` client uses this service (`--mode nutation|ayanamsa|all`) and evaluates at the library's
  internal TT, writing a JSON report.
- **Outcome:** the service
  reproduces IAE 2027 p. 423 at 158/161 dates, max 0.052″. Sample set: 16,503 TT epochs.

### Phase 1: nutation module, no behaviour change — DONE 2026-11-12

- New `src/astro/nutation.rs`: the 77-row table with our own layout, full Δψ/Δε, and the Luzum (2001)
  bias offsets. Cite the model source and the transcription source (D3).
- Document the existing 5-term function as a truncation. Its behaviour is unchanged.
- Unit tests:
  - five fixed NOVAS `iau2000b` values, generated from NOVAS itself
  - the first five table rows reproduce the existing truncated function (`frames.rs`)
  - the full series differs from the truncation by 0.2–1.0″
  - table shape: 77 rows, multipliers within −2..4.

**Outcome.** `src/astro/nutation.rs` is generated from NOVAS `nutation.c` by
`~/lahiri-benchmark/tools/gen_nutation_table.py` (idempotent, fmt-stable), so no coefficient was typed by
hand. `accumulate` sums in reverse order, as NOVAS does. Supporting edits: `astro/mod.rs` registers the
module, `frames::ASEC360` became `pub(crate)`, `frames::iau2000b` gained a doc comment marking it as the
truncation, and `tooling` re-exports `nutation`. Checks: 34 tests pass (2 ignored), `cargo fmt`,
`clippy -D warnings` and the wasm32 build are clean; the release rlib grows 38 KB (the table is ~4 KB).
Benchmark: the library's `iau2000b_full` matches NOVAS to **3.6e-15″** over 16,503 epochs.
The ayanamsa check fails as expected before Phase 2 (max 0.268″, RMS 0.075″).

### Phase 2: switch (B) — DONE 2026-11-12

- **Capture first.** Before touching `sidereal.rs`/`chart.rs`, record a baseline of today's Lahiri
  charts: bodies, speeds, declinations, cusps, sectors. Use the bundled dataset over a 1850–2150 grid
  plus edge cases, stored outside the repo.
- **`src/sidereal.rs`:** apply the D6 outcome to `lahiri_mean`, which affects both offsets. `lahiri_offset_and_rate` returns the reported offset (mean + Δψ₇₇) plus today's
  projection offset (mean + Δψ₅) and today's rate. Add `lahiri_ayanamsa` (D4).
- **`src/chart.rs`:** for Lahiri, `project_longitude` uses the projection offset and
  `reference_offset_degrees` reports the reported offset. Speeds keep today's rate. No other reference changes.
- **Docs:** the `ChartValues::reference_offset_degrees` docs state the Lahiri exception and its bound;
  `sidereal.rs` docs and `README.md` updated.

**Outcome.**
- **Code:**
  - `sidereal.rs` splits Lahiri into `lahiri_reported` (mean + Δψ₇₇) and `lahiri_projection` (mean + Δψ₅;
    the former `lahiri_true`, with an identical expression). `lahiri_offsets` returns both plus the unchanged
    rate. The new public `lahiri_ayanamsa` returns the reported value.
  - `chart.rs` projects with the projection offset and reports the reported one; the other references are
    unchanged. The field, module and README docs state the exception and its 0.27″ bound.
  - `lahiri_mean` is unchanged (D6 (a)); only its comment changed.
- **Tests (38 pass, 2 ignored; 4 new):**
  - reported − projection = Δψ₇₇ − Δψ₅
  - the IAE 2027 Jan 1 entry (24°14′13″.9)
  - charts report exactly `lahiri_ayanamsa`, and `tropical − sidereal` stays within 0.27″ of it
  - (generation feature) the gap is exactly Δψ₇₇ − Δψ₅, to 1e-9″.
  - `fmt`, `clippy -D warnings` (with and without `generation`) and wasm32 are clean.
- **Baseline:** captured at `5970a0c` before any change. 661,740 charts = 11,029 epochs × 4 references × 3 house
  systems × 5 locations, including two high-latitude sites.
  - A control run on the unchanged code gave 0 digest mismatches, and the offset check failed as intended.
  - After the change: **every chart is bit-identical** (tropical, Fagan–Bradley, True Sky, and Lahiri
    excluding the reported offset), and the Lahiri reported offset changed at all 11,029 epochs by exactly
    Δψ₇₇ − Δψ₅ (max error 1.3e-11″).
- **Benchmark** (14,503 epochs over 1800–2200, no held-out set):
  - `lahiri_ayanamsa` vs mean + NOVAS `iau2000b`: **max 2.2e-10″**, with no unsupported epochs.
  - Epochs ±1 s outside the range are rejected. IAU 2000A − 2000B: 0.0035″.
- **IAE p. 423 through the public API:** 65/161 → **158/161**, max 0.213″ → 0.052″.
- **Held-out set:** the first seed (`0x5eed1a4120261112`) had been included in the Phase 0/1 runs, so it is
  retired. Phase 3 uses a fresh seed, `0xf1a120261112b0b5`, fixed before any Phase 2 comparison and
  generated only with `--final`.

### Phase 3: validation (reported, not tuned) — DONE 2026-11-12

- **Samples, fixed before running:**
  - a 10.5-day grid from 1800-01-01 to 2200-01-01 TT (~13,900 epochs)
  - 2026-01-01 to 2026-03-01 every 0.1 d (short-period terms)
  - edges: 1800-01-01 and 2200-01-01 TT, J2000, B1950, and ±1 s outside the range (must error)
  - a held-out set of 2,000 epochs from a recorded seed, evaluated only in the final run.
- **Checks:**
  - **IAE table consistency (p. 423, 161 dates):** the reported offset at 0h TT vs the table, rounded to 0.1″.
    This is outside the repo because the extract stays in `~/lahiri-benchmark/iae`.
    **This tolerance was set after inspecting the data:** max |d| ≤ 0.055″ (0.05″ rounding + 0.005″ for the
    observed few-mas IAE internal difference), with ≥158/161 exact roundings.
  - `lahiri_ayanamsa` against the benchmark.
  - Over 1850–2150: chart `reference_offset_degrees` equals `lahiri_ayanamsa`.
  - All Phase 2 baseline outputs are bit-identical.
  - The documented exception holds: (tropical − sidereal) − reported offset = −(Δψ₇₇ − Δψ₅), checked through `tooling`.
- **Negative controls**, each of which must fail:
  - the 5-term model as the reported offset
  - one perturbed coefficient
  - a perturbed projection (sidereal not identical)
  - NaN, a missing epoch, a duplicate epoch
  - a benchmark manifest/hash mismatch.

**Outcome.**
- **Negative controls**, all demonstrated and passing both ways (the failing path really fails, the
  passing path really passes):
  - 9 unit tests in `compare/src/lib.rs` (`cargo test`): a real perturbed value (the measured 0.268″
    5-term gap) fails a 1 µas tolerance; a boundary case just inside/outside tolerance; NaN/±∞; a
    missing or an extra result; a duplicate epoch; a missing or mismatched `libnovas` hash; and the
    corresponding good-path cases.
  - Live: the service refuses to start with `--expect-libnovas-sha256` set to the wrong hash, and the
    client rejects the service's manifest the same way. The service returns HTTP 400 for an empty,
    duplicate, non-finite or malformed request.
- **IAE table wired into the report** (`iaeTable` section): 158/161, max 0.052″, against the tolerance
  declared in D6/here (0.055″, ≥158/161) — set after inspecting the data, and recorded as such.
- **Final run** (`--final`, fresh seed `0xf1a120261112b0b5`, 16,503 epochs including 2,000 held-out):
  **PASS**. Value max 2.18e-10″, nutation max 3.6e-15″, model spread max 0.0038″, IAE table 158/161.
  Unchanged from the non-final run, as expected: nothing was tuned on the held-out epochs.
- Range edges (±1 s outside 1800–2200 TT) are rejected; command: `--mode all --final --report
  lahiri-report-final.json` in `~/lahiri-benchmark`.

### Phase 4: library-generated regression fixtures (after Phase 3 passes) — DONE 2026-11-12

- Generator `examples/lahiri_regression_fixture.rs` writes `tests/fixtures/lahiri-regression.json`.
  - Contents: UTC inputs and the reported offset.
  - Metadata: revision, nutation model, generator command, and the benchmark report hash and NOVAS sdist hash as
    validation provenance. The benchmark itself is not committed.
- `tests/lahiri_regression.rs` compares the public API with the fixture within 1 µas, which tolerates platform `libm` differences.
- Policy text, aligned with `tools/verification/conventions.md`: the fixture detects regressions, does not
  establish accuracy, and is never regenerated just to make tests pass.
- `THIRD_PARTY_NOTICES.md` and the packaging attribution follow the D3 outcome (with NOVAS: no change).

**Outcome.**
- `tests/fixtures/lahiri-regression.json` (191 KB): 1,593 UTC inputs — a 97-day grid over 1800–2200, 80
  inputs at 8.88-hour steps in 2026 (short-period nutation), J2000, the IAE 2027 January 1 instant, the
  1956-03-21 definition date, both range edges (inside), and one must-error input beyond each end.
  Metadata records the policy, definition, tolerance (1 µas), generator command and revision, and the
  validation: the final report's SHA-256, the validated revision (`afaa8c3`), the NOVAS sdist and
  `libnovas.so` hashes, the held-out seed, max 2.18e-10″ and the IAE table result 158/161.
- Generator guards: it refuses without a report, with a non-final or failing report, or when `src/` differs
  from the validated revision (all four demonstrated). It asserts the out-of-range inputs still error.
  Regeneration from unchanged code is byte-identical (checked twice, including after a refactor).
  **Correction (2026-11-13):** that held only at the same commit. `generator.sourceRevision` recorded
  `HEAD`, so any later commit, even docs-only, changed one line. `scripts/review.sh` caught this. It now
  records the last commit that changed `src/` (`afaa8c3`, equal to the validated revision), which is
  stable across unrelated commits. That one metadata line was regenerated; no values changed.
- `tests/lahiri_regression.rs`: the public API reproduces every value within 1 µas, and the fixture is
  validated (format, policy, ≥1,500 cases, unique inputs, finite values, exactly two must-error cases,
  recorded `pass`).
- Negative controls, each restored afterwards: a value perturbed by 2 µas, a non-numeric value, a
  duplicated case and a truncated fixture all fail; temporarily reverting the reported offset to the
  five-term nutation fails 1,591 of 1,593 cases (worst 0.256″).
- Policy text added to `tools/verification/conventions.md` ("Lahiri regression fixture").
- Sanity: at 1956-03-21 0h UTC the library gives 23°15′00.79″, against the IAE's stated historical
  definition of 23°15′00″.
- Checks: 40 tests pass (2 ignored; 22 with default features), `fmt`, `clippy -D warnings` (with and
  without `generation`), wasm32 and the prohibited-usage check are clean.

## 5. Acceptance criteria (tolerances declared before implementation)

| Check | Tolerance |
|---|---|
| Library Δψ/Δε (77-term) vs NOVAS `iau2000b` | 1 µas |
| `lahiri_ayanamsa` vs benchmark (mean + NOVAS `iau2000b`), same instant | 1 µas |
| vs IAU 2000A (NOVAS `iau2000a`; model difference; informational but gated) | 0.005″ |
| Lahiri sidereal longitudes, speeds, declinations, cusps, sectors vs the Phase 2 baseline | bit-identical, except a confirmed mean-formula correction (D6), whose change must equal −Δmean(t) exactly (1 µas) |
| Tropical, Fagan–Bradley, True Sky output | unchanged (existing tests) |
| Range 1800-01-01..2200-01-01 TT inclusive; error outside | exact |
| `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`, prohibited-usage check | pass |
| Wasm build (`wasm32-unknown-unknown`) | builds; size delta reported |

The rate is not benchmarked. Under B it only drives sidereal speeds, which must stay bit-identical, and no
public output exposes it. The Bun-based Worker/browser checks need Bun 1.3.14, which is not installed here.
I'll install it if you want those run.

## 6. Out of scope

- Option C and the related findings (two time bases, the Mars residual, stored declinations): see the research note.
- A global IAU 2006/2000A upgrade.

## 7. Risks

- **The exception can confuse consumers who compute tropical − sidereal.** Mitigations: API docs, the
  stated bound, and the test that pins the relation.
- **Shared upstream (D3).** The library transcribes NOVAS and the benchmark runs NOVAS. An error in NOVAS's
  table would not be caught by the benchmark alone, but it is excluded by the recorded exact match with
  ERFA (an independent SOFA-lineage transcription). Both trace back to the same IAU 2000B definition, and
  matching that standard is the goal.
