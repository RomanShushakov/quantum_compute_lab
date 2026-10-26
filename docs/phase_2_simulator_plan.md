# Phase 2: Rust CPU to Vulkan GPU state-vector simulation

Status: in progress. Phase 1 (Experiments 01-09) completed on 2026-10-25.
CPU benchmarking and the Hadamard loop comparison are complete as
exploratory experiments; Vulkan implementation has not started.

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

## Milestones and next steps

1. Completed: verify `cargo fmt --all -- --check`, `cargo check --workspace`,
   `cargo test --workspace`, and strict Clippy on the CPU code.
2. Completed for gate microbenchmarks: record CPU timings at several
   qubit counts with warmups and release builds. State preparation and
   end-to-end circuit timing remain separate future measurements.
3. Check normalization, amplitudes (up to global phase when appropriate),
   gate ordering, and numerical tolerance against known cases and Aer.
4. Completed for Hadamard: profile the index-based loop and compare it
   with a slice-based implementation. Keep both for CPU/GPU validation.
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

## CPU benchmark baseline (Phase 2, step 1)

`rust/quantum_cpu/src/bin/statevector_bench.rs` is a sequential CPU-only
microbenchmark. It measures Hadamard and Ry at first/middle/last qubit targets
and CNOT (control 0, last target) for 10, 15, and 20 qubits. Initialization
and configurable warm-up iterations are outside the timed region. CSV output
contains gate, qubit count, target, control, repetitions, warm-up, minimum,
median, p95, and maximum gate time in microseconds. Results are saved in
`results/phase_2_cpu/` without overwriting previous runs.

The first exploratory 50-repetition run (before middle targets and CSV files
were introduced) yielded the following medians at 20 qubits:

| Gate | Target 0 | Target 19 |
| --- | ---: | ---: |
| Hadamard | 1400.393 us | 457.477 us |
| Ry | 1806.621 us | 756.923 us |

These are observations, not proof of a specific cache or vectorization cause.
The original run had noticeable variability. Follow-up standard/native release runs compared both Hadamard
implementations; they remain exploratory rather than definitive.

The benchmark reuses a statevector across calls. This measures per-gate
execution rather than circuit-level end-to-end time. State-dependent numerical
results are not benchmarked here; correctness is covered by the Rust test
suite. See `commands.md` for execution and recording instructions.

## Hadamard comparison (completed CPU experiment)

At 20 qubits (1,048,576 `Complex64` amplitudes, 16 MiB), the
single-threaded benchmark ran 100 warmups and 1,000 timed repetitions.
Median latency in microseconds:

| Target | Standard baseline | Standard chunks | Native baseline | Native chunks |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 1374.660 | 532.622 | 1297.177 | 679.361 |
| 10 | 704.412 | 675.393 | 718.905 | 617.032 |
| 19 | 474.542 | 441.176 | 393.392 | 462.541 |

The `chunks_exact_mut` implementation achieved about 2.58x speedup
at target 0 in the standard build. It was not consistently faster
at other targets or across build configurations. Each comparison
ran baseline before chunks; cache, scheduling, frequency, and
temperature can bias these exploratory measurements. Keep both
implementations; do not assume one is universally optimal.

## Next Vulkan milestone

On the local WSL2 laptop, use Rust `ash` for Vulkan device discovery,
queue-family enumeration, and a minimal compute dispatch before porting
the Hadamard gate. Confirm that the selected Vulkan implementation
supports the needed compute features; WSL Vulkan support can differ
from native Windows/Linux drivers. Compare GPU output to CPU complex
amplitudes before measuring speed. A future `f64` GPU kernel must
explicitly check `shaderFloat64` support; a `f32` prototype needs
a different numerical tolerance from the CPU `Complex64` reference.
