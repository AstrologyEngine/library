# Independent QC findings

This is a report of the pinned library branch and supplied immutable dataset,
not a change to calculation behavior or expected regression fixtures.

Dataset SHA-256:
`06d4c26e589207bf479dbd217fafb3e4d6c559626425b156912e39a4d859487b`.
Reference: independent SuperNOVAS service revision in `novas-lock.json`.
Reproduce with the native/Wasm command in [the QC guide](novas-qc.md).

The expanded sweep includes 831 cases, of which 423 request charts. It
completes 48,525 numerical checks without numerical failures. This includes
4,932 node checks, whose maximum absolute residual is 0.391780 arcseconds
(or arcseconds/day for rates), within the unchanged one-arcsecond limits.

The overall QC gate **fails with exit 1**: twelve polar inputs are rejected
by both native and Wasm with the library's overlapping-house error. Each of
these four spacetime combinations is tested in tropical, Lahiri and
Fagan-Bradley, at longitude zero with Placidus:

| UTC | Latitude |
| --- | ---: |
| 2026-03-20 18:00 | -85 |
| 2026-03-20 18:00 | -70 |
| 2026-03-20 06:00 | 70 |
| 2026-03-20 06:00 | 85 |

Each target completes 411 charts and rejects 12, producing 24 recorded
calculation errors. The independent reference constructs an ordered Porphyry
fallback for these inputs. The report preserves that geometry alongside the
errors. No row is fabricated for an unavailable library chart, and native
failure does not prevent a Wasm attempt.

The library already documents rejection of overlapping polar house sets.
These results expose that coverage limitation; they do not by themselves
justify changing its convention. Any library house change requires a separate
implementation and review. The benchmark gate remains strict in the meantime.
