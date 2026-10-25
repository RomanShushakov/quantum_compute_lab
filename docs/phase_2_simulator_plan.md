# Phase 2: Rust CPU to Vulkan GPU state-vector simulation

Status: planned. Phase 1 (Experiments 01-09) completed on 2026-10-25.

## Objective

Build a validated, measurable local state-vector simulator, using the
existing Rust CPU implementation as the reference before introducing Vulkan
compute through `ash`. IBM Quantum access is not required.

## Existing baseline

- `rust/quantum_core/src/lib.rs`: complex-amplitude state vectors and gates.
- `rust/quantum_cpu/src/`: Bell, Bernstein-Vazirani, VQE, Bathe Hamiltonian,
  classical Jacobi, and numerical validation.
- `rust/quantum_core/src/tests/` and `rust/quantum_cpu/src/tests/`:
  correctness tests to preserve while optimizing.
- `experiments/`: Qiskit/Aer reference circuits and historical QPU work.
- `results/`: existing measurements; do not overwrite historical JSON.

## Proposed sequence

1. Verify `cargo fmt --all -- --check`, `cargo check --workspace`,
   `cargo test --workspace`, and strict Clippy on the current CPU code.
2. Record CPU baseline timings for state preparation and gates at several
   qubit counts; use repeatable inputs and release builds for benchmarks.
3. Check normalization, amplitudes (up to global phase when appropriate),
   gate ordering, and numerical tolerance against known cases and Aer.
4. Profile CPU gate loops and memory allocation before changing kernels.
   Keep correctness tests passing after each change.
5. Implement a minimal Vulkan compute path for one gate with `ash`, then
   add multi-qubit operations only after validating output against CPU.
6. Compare CPU and GPU execution including allocation, transfer,
   synchronization, and kernel time. Report memory use and error bounds.

## Measurement rules

- Record CPU/GPU model, OS, Rust version, Vulkan driver, build profile,
  qubit count, gate sequence, number of repetitions, and elapsed time.
- Distinguish kernel-only timing from end-to-end timing.
- Compare complex amplitudes and probabilities with explicit tolerances;
  finite-shot distributions are not an exact numerical reference.
- Expect `2^n` complex amplitudes for an n-qubit pure state; document the
  practical memory limit before claiming scalability.
- No speedup or hardware-support claim without measured evidence.

## Scope

Use the local development machine for Vulkan (`ash`). Keep the Rust CPU
implementation usable independently. Jetson, Slurm, and WebGPU are not part
of this project's current GPU implementation plan. Return to IBM QPU only
for a new, clearly specified experiment, not routine validation.
