# Astrology Engine

High-precision, Wasm-first astrology engine in Rust. Calculates planetary
positions, angles, and houses for the browser, Cloudflare Workers, and server
runtimes.

## What it calculates

- Planetary and derived body positions, longitudinal speeds, and declinations,
  plus the Ascendant, Midheaven, Descendant, and Imum Coeli.
- Tropical, Fagan–Bradley sidereal, and Chimenti True Sky references, with sign
  assignments and equal or True Sky constellation divisions.
- Equal, Whole Sign, and Placidus houses, with twelve cusps and explicit reporting
  when Placidus falls back to Porphyry.
- Sun-longitude crossings, including the design-date search used in Human Design.

One `calculate_chart` function takes explicit zodiac and house settings and
returns positions, signs, cusps, and calculation metadata. See the
[chart conventions](docs/chart-conventions.md) for reference models, supported
combinations, speed conventions, and polar-house safeguards. Fagan–Bradley's
model covers 1800–2200 TT; every chart also requires coverage in its dataset.

## Use it in your runtime

The runtime accepts caller-supplied ephemeris bytes. It performs no network or
filesystem access; your application loads the data and resolves place names and
local birth times into coordinates and an unambiguous instant.

- **Browser/Wasm:** the [standalone chart example](examples/configurable-chart/README.md)
  builds Rust to Wasm and provides interactive zodiac and house settings.
- **Cloudflare Workers:** the [HTTP example](examples/cloudflare-worker/) loads
  data from R2. Follow the [Worker runbook](docs/worker-runbook.md) for local
  testing and deployment. Its HTTP interface uses tropical charts.
- **Native/server Rust:** parse the dataset once with `Ephemeris::parse` and reuse
  it across calls, as in the quickstart below.

## Calculate a natal chart

Use the Rust toolchain specified in [`rust-toolchain.toml`](rust-toolchain.toml).
You also need an ephemeris dataset: a binary file of prepared celestial-position
data in the engine's `HDCHEB01` format. The examples call this file `cheb.bin`.
Dataset files are supplied separately and are not included in the repository.
Follow [Build a dataset](tools/README.md#build-a-dataset), then run:

```sh
cargo run --locked --example natal_chart -- /absolute/path/to/cheb.bin
```

This example calculates a tropical chart for **1 January 2000 at 12:00 UTC**,
London coordinates, and Equal houses. Convert local birth times to UTC before
calling the engine. The dataset must support the instant and samples 12 hours
before and after it for speed calculations.

The complete source is [`examples/natal_chart.rs`](examples/natal_chart.rs):

```rust
// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

use astrology_engine::{
    calculate_chart, ChartInput, Ephemeris, Epoch, HouseSystem, Location, SignDivisions,
    ZodiacConfiguration, ZodiacReference,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dataset = std::env::args_os()
        .nth(1)
        .ok_or("Usage: cargo run --example natal_chart -- /path/to/cheb.bin")?;
    let ephemeris = Ephemeris::parse(std::fs::read(dataset)?)
        .map_err(|error| format!("Cannot load ephemeris: {error}"))?;

    let birth_moment = Epoch::from_gregorian_str("2000-01-01T12:00:00 UTC")?;
    let chart = calculate_chart(
        &ephemeris,
        ChartInput {
            epoch: birth_moment,
            location: Location {
                latitude: 51.5074,
                longitude: -0.1278,
            },
            house_system: HouseSystem::Equal,
            zodiac: ZodiacConfiguration {
                reference: ZodiacReference::Tropical,
                divisions: SignDivisions::Equal,
                ophiuchus: None,
            },
        },
    )?;

    println!("Birth moment: {birth_moment}");
    println!("Longitudes in degrees:");
    for body in &chart.bodies {
        println!("{:<20} {:>12.6}", body.values.name, body.values.longitude);
    }
    println!("House cusps in degrees: {:?}", chart.cusps);

    Ok(())
}
```

`ChartValues` contains bodies with numerical values and sign assignments,
twelve house cusps, effective zodiac settings, and the requested and actual
house methods. Body longitudes, declinations, and cusps use degrees; speeds use
degrees per day. Load and parse the ephemeris once, then reuse it for later charts.

## Human Design design-date search

The design instant is the time before birth when the tropical Sun's longitude
was 88° less than its birth longitude. Calculate a tropical birth chart, subtract
88° from the Sun's longitude, wrap into [0°, 360°), and call `find_sun_crossing`
with `Direction::Backward`. Calculate another chart at the returned instant.

The [birth and design example](examples/birth_and_design.rs) demonstrates this
workflow:

```sh
cargo run --locked --example birth_and_design -- /absolute/path/to/cheb.bin
```

The search needs at least 89 days of dataset coverage before birth. Calculating
the design chart also requires speed-sampling support around the returned instant.

## Architecture and verification

[From JPL data to a natal chart](docs/architecture.md) explains the separation
between native dataset preparation and the immutable Rust/Wasm runtime.
[The public Rust API](src/lib.rs) exports chart calculation, ephemeris evaluation,
and solar-longitude search.

[Verification guidance](tools/README.md#run-focused-checks) distinguishes
structural checks, local Wasm execution, frozen regression parity, and dataset
regeneration. Recorded regression outputs preserve previous engine behavior;
these checks do not establish independent astronomical accuracy.

## License

This project is licensed under the [MIT License](LICENSE).

### Third-Party Dependencies

This project relies on the following build-time crates licensed under the Mozilla Public License 2.0 (MPL-2.0):

* **[hifitime](https://crates.io/crates/hifitime)** — High-precision duration and timekeeping ([MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/))
* **[ANISE](https://crates.io/crates/anise)** — Precise orbit and navigation library ([MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/))

Your use and modification of this repository's source code are governed by the MIT License. If you distribute compiled binaries or packaged builds that include these statically linked crates, the redistribution must satisfy the notice and source-availability requirements of the MPL-2.0 for those components.

Build-time attribution requires `cargo-about` 0.9.2. Generated bundles are ignored
and copied into browser/Worker outputs; see [attribution setup](tools/notices/README.md).
