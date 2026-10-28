# Vulkan multi-gate circuit benchmark

The `circuit_bench` executable benchmarks a **12-gate H/Ry/CNOT circuit**
with a reusable Vulkan command buffer and GPU-resident statevector at 10,
15, and 20 qubits. It resets the state to `|0...0>` before each independent
sample. Three warmups precede nine measured samples. CPU `f64` amplitudes
are computed separately and compared against GPU `f32` amplitudes.

Run from `rust/`:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p quantum_gpu --bin circuit_bench
cargo run --release -p quantum_gpu --bin bathe_gpu
```

## Timing definitions

- `CPU_f64_median_us`: CPU-side execution of the equivalent circuit.
- `GPU_device_median_us`: Vulkan timestamp interval around compute
  dispatches when supported; it includes the recorded GPU work, not
  exclusively one shader kernel.
- `GPU_f32_submit_wait_median_us`: CPU-observed queue submission and wait,
  excluding setup and readback.
- `GPU_readback_median_us`: mapped-memory read and allocation of the output
  vector; **not** necessarily a discrete GPU-to-host transfer, since storage
  uses host-visible memory.
- `GPU_end_to_end_median_us`: reset, submission/wait, query retrieval, and
  readback, excluding one-time setup.
- `GPU_setup_us`: resource creation and command recording, reported
  separately.

Medians of individual stages need not sum to the reported end-to-end
median. The CPU and GPU have different precision (`f64` vs `f32`), so
these timings are not a precision-matched hardware comparison.

## Verified measurements (2026-10-28)

All times are microseconds. Each row is a nine-sample median from a
separate program run; values are not pooled across runs.

| Run | Qubits | CPU f64 | GPU device | GPU submit + wait | GPU readback | GPU end-to-end | GPU setup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial | 10 | 14.527 | 119.080 | 612.359 | 3.676 | 763.274 | 61192.999 |
| Initial | 15 | 438.641 | 142.720 | 632.584 | 21.385 | 850.641 | 21460.301 |
| Initial | 20 | 9612.184 | 3436.080 | 4515.185 | 748.470 | 6015.306 | 22298.716 |
| Repeat 1 | 10 | 8.016 | 128.080 | 586.742 | 2.521 | 749.796 | 22237.473 |
| Repeat 1 | 15 | 246.235 | 149.680 | 578.242 | 14.277 | 698.002 | 21011.570 |
| Repeat 1 | 20 | 9389.621 | 3442.040 | 4404.771 | 869.459 | 5978.519 | 18126.136 |
| Repeat 2 | 10 | 13.898 | 113.960 | 590.090 | 3.625 | 728.299 | 20312.806 |
| Repeat 2 | 15 | 422.392 | 151.160 | 603.958 | 18.813 | 792.524 | 19728.547 |
| Repeat 2 | 20 | 8904.384 | 3602.840 | 4945.492 | 700.855 | 6477.453 | 20894.074 |

All runs reported `max_amplitude_error=6.575e-8`. The selected queue
reported `timestamp_valid_bits=64` and `timestamp_period_ns=10`.

For this 20-qubit workload, the GPU device medians were approximately
2.5–2.8 times lower than CPU circuit medians; GPU end-to-end medians
(excluding setup) were approximately 1.4–1.6 times lower. At 10 qubits
CPU was substantially faster, and at 15 qubits the GPU device time was
lower but GPU end-to-end time remained higher.

## Limitations

The GPU is accessed through WSL2 Mesa Dozen (Vulkan-over-Direct3D12),
which warns that it is non-conformant. Timestamp support was reported,
but device timings have not been independently validated against native
Vulkan. Readback timings can reflect host-side memory access and copying,
not PCIe transfer. The 12-gate benchmark is an exploratory workload;
its performance does not establish a general GPU speedup or any quantum
advantage. The results should not be directly compared to IBM QPU
queue/network time or sampled hardware measurements.
