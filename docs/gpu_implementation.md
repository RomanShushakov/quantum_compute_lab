# Vulkan GPU implementation

The `quantum_gpu` crate implements direct Vulkan compute using Rust `ash`
(0.38), WGSL gate shaders compiled to SPIR-V with the Naga CLI, and an
interleaved complex `f32` statevector. The CPU reference uses `f64`. This is
a classical GPU statevector simulator, **not** quantum hardware.

## Components and execution

- `src/context.rs`: Vulkan instance, physical-device selection, logical
  compute device, queue, and command resources.
- `src/buffer.rs`: storage buffers and mapped host-visible memory.
- `src/compute.rs`: compute pipelines, single-gate execution helpers,
  `CircuitGate`, and reusable `PreparedCircuit`.
- `shaders/{transform,hadamard,ry,cnot}.wgsl`: shader sources, with matching
  `.spv` binaries used by the executable.
- `src/main.rs`: Vulkan smoke tests for buffers, transform, and Hadamard.
- `src/bin/bathe_gpu.rs`: optimized-angle Bathe VQE CPU/GPU comparison and
  exploratory single-Hadamard benchmark.
- `src/bin/circuit_bench.rs`: repeatable 12-gate benchmark with optional
  Vulkan timestamp queries and CPU reference amplitudes.

`PreparedCircuit` prepares resources and records the entire gate sequence
into one command buffer. Gates operate on a GPU-resident statevector, with
compute-to-compute barriers between dependent dispatches. A prepared circuit
can be submitted repeatedly, with state reset before independent benchmark
samples. The older per-gate helpers remain available for smoke tests.

## Build and verify

Run from `rust/`:

```bash
for gate in transform hadamard ry cnot; do
  naga "quantum_gpu/shaders/${gate}.wgsl" "quantum_gpu/shaders/${gate}.spv"
done
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p quantum_gpu
cargo run --release -p quantum_gpu --bin bathe_gpu
cargo run --release -p quantum_gpu --bin circuit_bench
```

The last verified workspace run passed **92 tests** (39 core, 47 CPU
algorithms, 5 CPU benchmark tests, 1 GPU benchmark reference test) with
strict Clippy clean. GPU runtime checks are executed by the binaries,
not by the ordinary `cargo test` suite.

## Numerical results

The Bathe Hamiltonian and VQE optimizer come from `quantum_cpu`; the GPU
evaluates the CPU-optimized angles rather than running a separate optimizer.

| Calculation | Energy |
| --- | ---: |
| CPU ideal VQE (`f64`) | 0.724456493728 |
| GPU ideal VQE (`f32`) | 0.7244564897 |

The observed absolute energy difference is approximately `4.071e-9`.
The GPU's single-qubit Hadamard, inverse Hadamard, complex amplitudes,
and Bathe ansatz were verified at runtime.

## Hardware and interpretation

Development system: AMD Radeon integrated graphics through Mesa Dozen
(Vulkan over Direct3D12) on WSL2; Vulkan API 1.2.354, compute queue family
1. The driver warns that `dzn` is non-conformant. `shaderFloat64` is
reported as supported, but the current WGSL gate shaders use `f32`.

For performance methodology and timestamp limitations, see
`docs/gpu_benchmark.md` (path relative to repository root). Historical
prepared-circuit development notes are in
`docs/gpu_prepared_circuit_benchmark.md`.
