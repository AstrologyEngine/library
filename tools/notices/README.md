# Build-time attribution

Generated `notices/` files are ignored and never committed. Browser and Worker
build wrappers generate attribution from their selected locked dependency graph
and copy it alongside build outputs. They fail if attribution cannot be resolved.
Install `cargo-about` 0.9.2 (`cargo install cargo-about --version 0.9.2 --features cli --locked`),
or set `CARGO_ABOUT` to that executable. Cargo fetches the selected locked graph;
license collection is offline after dependencies are available.

Rust standard-library attribution is read from the pinned installed toolchain,
or downloaded from its versioned documentation URL, checked against the recorded
SHA-256 and cached under ignored `notices/`. A complete verified cache works offline.
The small supplemental license files here are reviewed source inputs for upstream
packages that omit material or emit JavaScript. They are not generated bundles.

For native distributions, run `bun tools/notices/generate.ts builder` (or `runtime`)
and include `target/distribution/builder/` (or `runtime/`) with your artifact.
The generator runs anew each build, so no committed snapshot or freshness gate
is needed. Preserve emitted attribution and MPL source links when redistributing;
modified covered dependencies need their actual corresponding modified source.
No first-party source archive is generated or required by this MIT project.
