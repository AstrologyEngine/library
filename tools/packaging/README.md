# Browser packaging

Normal builds and tests do not generate attribution. For distribution install
cargo-about 0.9.2 and rustup's rust-docs component for the pinned toolchain, then
run `bun run package` in examples/configurable-chart. Dependencies must already
be installed. The command builds the browser app into ignored
target/distribution/browser, with cargo-about and Vite license output.
Missing prerequisites or unresolved licenses fail packaging. Upstream license
retrieval may require network access. Ship the entire directory, including
licenses and ATTRIBUTION.txt. Supply ephemeris data separately at /ephemeris.bin.
No dependency license files or generated reports are committed.
