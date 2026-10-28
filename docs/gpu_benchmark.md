# Prepared multi-gate circuit benchmark

Run from `rust/`:

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p quantum_gpu --bin circuit_bench
cargo run --release -p quantum_gpu --bin bathe_gpu
```

The benchmark prepares a 12-gate H/Ry/CNOT circuit, uses one recorded Vulkan command buffer and a GPU-resident state, and resets the state to `|0...0>` before each independent sample. Three warmups precede nine measured runs. Each result is compared with a separate CPU f64 statevector calculation; reported errors are absolute amplitude differences. CPU and GPU precisions differ (`f64` vs `f32`).

- `GPU_device_median_us`: timestamp interval around compute dispatches, only when selected queue supports timestamps; not a direct single-shader kernel duration.
- `GPU_f32_submit_wait_median_us`: CPU-observed queue submission + wait, excludes setup and readback.
- `GPU_readback_median_us`: mapped-memory read and allocation of the output vector; **not** necessarily a device-to-host transfer (storage uses host-visible memory).
- `GPU_end_to_end_median_us`: reset + submit/wait + query retrieval + readback; setup excluded.
- `GPU_setup_us`: one-off resource creation and command recording.

The Vulkan `dzn` implementation in WSL2 is non-conformant and may not provide reliable timestamp support. Treat GPU timestamp results as provisional until checked against a native Vulkan implementation. This is a small exploratory workload, not a claim of quantum advantage.
