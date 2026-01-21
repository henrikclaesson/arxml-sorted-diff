# Autosar sample generator

A small optional generator binary `generate_samples` is included to create large ARXML fixtures for testing and benchmarking. The generator uses the `autosar-data` crate and is enabled using the `autosar` cargo feature.

Run the generator:

  cargo run -q --bin generate_samples --features autosar -- --out-dir ./samples -p 20 -s 200

Notes
- Building with `--features autosar` pulls in `autosar-data` and its dependencies which may require a recent Rust toolchain. If you see compilation errors mentioning unstable let-patterns (E0658) or related diagnostics, update your Rust toolchain:

  rustup update stable
  # or build with a newer toolchain explicitly:
  rustup run nightly cargo build --features autosar

- To avoid building the optional generator, omit `--features autosar` when building the project.
