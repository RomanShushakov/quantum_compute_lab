# Apply test refactor

Extract this ZIP at the quantum_compute_lab repository root.
Overwrite matching files. The ZIP does not contain Cargo manifests
or unmodified Python experiment files.

IMPORTANT: delete rust/quantum_core/src/tests.rs after extraction.
It has been replaced by rust/quantum_core/src/tests/mod.rs and siblings.

Then run from rust/:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p quantum_cpu
```

Expected: 36 quantum_core tests + 33 quantum_cpu tests.
No QPU jobs are submitted.
