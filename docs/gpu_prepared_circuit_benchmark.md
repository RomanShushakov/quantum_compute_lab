# Prepared Vulkan circuit benchmark

`PreparedCircuit` retains the state buffer, per-gate uniforms, pipelines, descriptors, and a recorded command buffer. All gates in a circuit execute on the same GPU-resident state, with compute-to-compute barriers between gates. The Bathe circuit is now submitted as one command buffer, with a single synchronization before readback.

`bathe_gpu` reports: CPU `f64` gate time; GPU setup (pipeline and command-buffer creation); GPU repeated submission + queue idle (not pure shader execution); and GPU host-visible memory readback. The CPU and GPU use different precision and the GPU timings are measured on WSL2 Dozen, a non-conformant Vulkan implementation. These are exploratory timings, not a claim of acceleration or a fair kernel-only comparison.

Run from `rust/`:

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p quantum_gpu --bin bathe_gpu
```

Vulkan timestamp queries and a 12-gate benchmark were subsequently added; see `docs/gpu_benchmark.md` for measured results and remaining precision/driver limitations.
