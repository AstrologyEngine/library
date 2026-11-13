# Research: correcting the truncated nutation at runtime (option C)

> Import provenance: this document records claims and decisions from the returned
> coding project. Its external benchmark reports and temporary Git history were
> not supplied, and those reported experiments were not independently rerun during
> import. Local checks are reported separately in the import PR.

- **Status:** findings only, not scheduled. Option B was chosen on 2026-11-12; see
  [the plan](../plans/2026-11-12-lahiri-full-nutation.md). This note records what option C would
  involve, so it can be picked up later.
- **Measured at:** library `7c28758`, rustc 1.96.0, bundled `cheb.bin` `06d4c26e…`, NOVAS C 3.1 `iau2000b` (first measured with pyerfa 2.0.1.5; the figures are identical, as NOVAS and ERFA agree bit for bit).

## Summary

The library's nutation keeps 5 of the 77 luni-solar terms of IAU 2000B
(`src/astro/frames.rs::iau2000b`, adapted from Astronomy Engine). Over 1800–2200 its Δψ differs
from full IAU 2000B by up to **0.270″** (RMS 0.076″), and its Δε by up to **0.087″**.

The truncated Δψ is baked into the dataset's tropical longitudes and also used for the runtime
ayanamsas. It therefore cancels in sidereal longitudes, which today are nutation-free, while tropical
longitudes and reported ayanamsas carry the error. Option C would remove the error from tropical
output instead of cancelling it. Everything except stored declinations can be corrected exactly at
runtime, without regenerating the dataset.

## Where the truncated model enters

| Path | Location | Effect |
|---|---|---|
| Dataset generation | `tools/dataset-builder/src/astro/apparent.rs`: `ecliptic_of_date`, `eqd_declination`, reached through `astrology_engine::tooling::frames` | Δψ₅ in stored tropical longitudes; Δψ₅ and Δε₅ in stored declinations |
| Lahiri true ayanamsa | `src/sidereal.rs::lahiri_true` | offset = IAE mean + Δψ₅ |
| Fagan–Bradley true ayanamsa | `src/sidereal.rs` line 42 | offset = mean + Δψ₅ |
| True obliquity | `src/astro/frames.rs::e_tilt`, then `true_obliquity` | node declination (`src/cheb/eval.rs::node_pair`), houses (`src/astro/houses.rs`) |
| Equation of the equinoxes | `src/astro/frames.rs` (`15 · ee`) | apparent sidereal time, which feeds the angles and houses |
| Not affected | `src/conventions.rs::true_sky_offset` | linear Chimenti offset with no nutation term, so True Sky sidereal positions carry Δψ₅ |

The dataset builder uses the library's own `tooling::frames`. Changing `e_tilt` therefore also changes
what future datasets bake in.

## Measurements

### Positions against external calibrated records (recorded; source data deleted)

Method: 133 distinct instants (1851–2100). The ephemeris time is set to the records' TT
(`Epoch::from_jde_tdb`). Longitudes are compared with the records' tropical and Lahiri longitudes.
Δψ₇₇ is NOVAS `iau2000b`. Values are in arcseconds.

| | SUN max / rms | MOON max / rms | MARS max / rms |
|---|---|---|---|
| Tropical, library − records | 0.251 / 0.078 | 0.246 / 0.078 | 0.261 / 0.084 |
| Correlation with Δψ₅ − Δψ₇₇ | 0.998 | 0.999 | 0.933 |
| Tropical after removing Δψ₅ − Δψ₇₇ | 0.020 / 0.008 | 0.018 / 0.010 | 0.142 / 0.030 |
| Sidereal (Lahiri) today | 0.017 / 0.006 | 0.013 / 0.007 | 0.145 / 0.030 |
| Sidereal, option A (fix only the ayanamsa) | 0.249 / 0.077 | 0.244 / 0.078 | 0.258 / 0.083 |
| Sidereal, options B and C | 0.017 / 0.006 | 0.013 / 0.007 | 0.145 / 0.030 |

Mars keeps a 0.14″ residual after the nutation is removed. It has nothing to do with nutation
(light-time, aberration or ephemeris differences) and was not investigated.

### Size of the correction (dense grid)

Method: the library's 5-term nutation at TT steps of 0.25 d from 1800-01-01 to 2200-01-01
(584,389 epochs), compared with NOVAS `iau2000b`.

| Quantity | 5-term − IAU 2000B |
|---|---|
| Δψ (= tropical longitude and ayanamsa correction) | max 0.270″, rms 0.076″ |
| Δψ centered-day rate (= speed correction) | max 0.078″/day |
| Δε (= true obliquity) | max 0.087″, rms 0.026″ |
| Equation of the equinoxes | max 0.0165 s of time (0.248″) |
| Declination, worst case over right ascension | max 0.109″ |

For scale, IAU 2000B itself differs from IAU 2006/2000A by ≤0.0038″ over the same span.

## What option C would change

1. **A full nutation module**, shared with option B's Phase 1: the IAU 2000B table plus Δψ/Δε.
2. **`e_tilt` switches to the full model.** This fixes true obliquity, the equation of the equinoxes
   (and so angles and houses), the node declination, and both true ayanamsas.
3. **Runtime longitude correction** for datasets generated with the 5-term model: add Δψ₇₇ − Δψ₅ to every
   tropical body longitude read from the dataset, nodes included, before speed differencing and projection.
   - *This is exact:* nutation moves the equator, not the ecliptic. The true equinox is displaced by Δψ along the
     ecliptic of date, so the ecliptic longitude of date changes by exactly Δψ and latitude is unchanged.
   - Speeds are centered differences of longitude (`src/cheb/eval.rs::fd_speed`, ±0.5 d), so they pick up the
     correction without further changes.
4. **Dataset declaration** to avoid double correction. Datasets generated after step 2 bake in the full
   model. The runtime must apply step 3 only to 5-term datasets, so the nutation model has to be
   declared, for example in `data/bundled/manifest.json` / provenance. The blob format stays unchanged, and a
   missing declaration is treated as 5-term. A regenerated dataset without the declaration should fail the build.
5. **What stays:** the stored declination series. With no latitude stored, they cannot be re-derived
   exactly at runtime. They stay on the 5-term model (error ≤0.109″) until the dataset is regenerated.
   That needs JPL kernels, which are not in this export. Sidereal and tropical declinations are identical,
   so this does not affect the ayanamsa.

### Expected output changes under C

| Output | Change |
|---|---|
| Tropical longitudes (bodies, nodes, angles, cusps) | ≤0.270″, toward the standard |
| Tropical speeds | ≤0.078″/day |
| Lahiri and Fagan–Bradley offsets | ≤0.270″ |
| Lahiri and Fagan–Bradley sidereal longitudes | unchanged (the correction cancels); the contract `sidereal = tropical − offset` still holds |
| True Sky sidereal longitudes | ≤0.270″ (follow tropical) |
| Body declinations (stored) | unchanged until regeneration |
| Node declination, houses | change through the true obliquity (≤0.087″) and sidereal time |

## Consequences

- **Frozen fixtures.** Tropical, True Sky and Fagan–Bradley parity rows are compared exactly
  (`tools/verification/conventions.md`, `parity.py`, `tests/conventions.rs`, and the ignored
  full-chart and Fagan–Bradley legacy tests). They would need a documented migration.
  - The policy forbids regenerating from changed code, so expected values would be frozen value +
    correction, with the correction computed independently (NOVAS), not by the library.
  - The full-chart runners need the captured dataset (`37773a57…`), which is not in this environment.
- **Convention revision.** Tropical output changes, so `CONVENTION_REVISION` would become
  `astrology-engine-conventions-v2`. Its pin in `examples/configurable-chart/verify.ts` changes with it.
- **Fagan–Bradley legacy comparison.** `verify-fagan-bradley.ts` compares `ayanamsa_degrees`. Its
  tolerance needs checking against a ≤0.27″ change.
- **Licensing.** This is the same table question as in option B (plan D3).

## Related findings (separate issues)

- **Two time bases.** Ephemeris evaluation uses hifitime TDB for the epoch. Nutation, the ayanamsa
  and houses use `AstroTime::from_tdb`, which takes the *UTC* Julian day as UT and adds the library's ΔT.
  In the sample above, the two TT values differed by −38.5 to +133.5 s. The effect is small on the
  ayanamsa (Δψ moves ≤0.00016″ per minute; precession moves 0.0001″ per minute), but it is large for
  anything that depends on UT, such as angles.
- **Sparse sampling understates the maxima.** The 133-instant sample showed 0.237″ for the Δψ
  difference, while the dense grid shows 0.270″. Acceptance checks should use dense grids.

## Reproduction

The tools live outside the repo, in `~/lahiri-benchmark`:

- `research/probe`: a Rust probe. `grid` mode needs no external data.
- `research/analyze.py`: NOVAS via `../novas/libnovas.so`, numpy in `.venv`. It prints the grid figures.

```sh
cd ~/lahiri-benchmark/research/probe && cargo run --offline --release -- grid > ../grid.csv
cd .. && ../.venv/bin/python analyze.py
```

**The "Positions against external calibrated records" figures are no longer reproducible.** The records
(SHA-256 `71c5a699…`) and every file derived from them were deleted on 2026-11-12 at the owner's request,
so those tables stand as recorded measurements only. The dense-grid figures ("Size of the correction")
depend only on the library and NOVAS, and were re-run after the deletion with identical results.
