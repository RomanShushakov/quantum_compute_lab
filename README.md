# Quantum Compute Lab

A personal learning and engineering project exploring quantum algorithms, numerical linear algebra, IBM quantum hardware, and CPU/GPU state-vector simulation.

The goal was to understand the mathematics and the implementation details through small, reproducible experiments - not to demonstrate quantum advantage.

## What is included

The project progressed from simple Qiskit circuits to a small Rust quantum simulator, a direct Vulkan compute backend, and a comparison with measurements from an IBM QPU.

| Area | Implementation |
| --- | --- |
| Quantum fundamentals | Single-qubit gates, superposition, Bell states, measurement |
| Algorithms and hardware | Bernstein–Vazirani, Qiskit Aer, IBM Quantum experiments |
| Noise investigation | Simulator/QPU comparisons, circuit depth, calibration and qubit selection |
| Numerical methods | Classical Jacobi eigensolver and Bathe Exercise 11.1 |
| CPU simulation | Rust `f64` statevectors, gates, Pauli expectations and VQE |
| GPU simulation | Rust `ash`, Vulkan compute, WGSL shaders compiled to SPIR-V with Naga |
| GPU benchmarking | Prepared GPU-resident circuits, timestamp queries and repeated multi-gate benchmarks |

The Rust workspace contains three crates:

- **`quantum_core`** - statevector representation, quantum gates and measurements.
- **`quantum_cpu`** - quantum algorithms, VQE, numerical reference solvers and CPU benchmarks.
- **`quantum_gpu`** - Vulkan device management, buffers, compute pipelines, reusable circuits and GPU benchmarks.

The GPU backend uses Vulkan directly, rather than a higher-level GPU framework.

## Main experiment: Bathe Exercise 11.1

The numerical comparison uses the generalized eigenvalue problem

\[
K\phi = \lambda M\phi
\]

from Bathe Exercise 11.1. It is transformed into a symmetric eigenproblem and embedded in a two-qubit Hamiltonian for VQE experiments.

The classical Jacobi solution provides the reference ground-state energy. The same problem is then evaluated with ideal CPU/GPU statevector simulation, sampled Qiskit Aer measurements and IBM quantum hardware.

| Method | Ground-state energy estimate |
| --- | ---: |
| Classical Jacobi | 0.724456494 |
| Rust CPU VQE (`f64`, ideal) | 0.724456493728 |
| Vulkan GPU VQE (`f32`, ideal) | 0.7244564897 |
| Qiskit Aer (sampled, 16,384 shots per basis) | 0.735229872905 |
| IBM Kingston, physical qubits `[0, 1]` | 6.672486800770 |
| IBM Kingston, physical qubits `[88, 89]` | 1.073026699181 |

The CPU and GPU ideal simulations closely agree with the classical reference. The sampled and hardware measurements have additional uncertainty and noise. The two IBM mappings produced substantially different results, but the observations do not isolate every source of hardware error.

**Full discussion:** [Final comparison report](docs/final_comparison_report.md)

## Vulkan GPU experiments

The GPU implementation runs statevector gates through WGSL compute shaders, compiled to SPIR-V using Naga. A reusable `PreparedCircuit` records multiple gate dispatches and keeps the state resident in GPU-accessible memory across gates.

The development setup was an AMD Radeon integrated GPU under Ubuntu WSL2, using Vulkan through Mesa Dozen / Direct3D12. Dozen reports itself as non-conformant, so the measurements should be understood in that environment.

A repeated **12-gate circuit benchmark** (Hadamard, Ry and CNOT) compared CPU `f64` execution with GPU `f32` execution at 10, 15 and 20 qubits. For 20 qubits, across three runs:

| Measurement | Observed median range |
| --- | ---: |
| CPU circuit execution | 8.904–9.612 ms |
| GPU device execution (Vulkan timestamps) | 3.436–3.603 ms |
| GPU end-to-end, excluding one-time setup | 5.979–6.477 ms |

The maximum observed CPU/GPU amplitude difference was `6.575e-8`. These are measurements of this circuit and hardware/software stack, not a general performance claim. CPU and GPU also use different numerical precision.

For methodology and caveats, see [GPU implementation](docs/gpu_implementation.md) and [GPU benchmarks](docs/gpu_benchmark.md).

## Repository layout

```text
quantum_compute_lab/
|--- README.md
|--- commands.md
|--- requirements.txt
|--- result_writer.py
|--- docs/
│   |--- final_comparison_report.md
│   |--- final_comparison_plan.md
│   |--- gpu_implementation.md
│   |--- gpu_benchmark.md
│   |--- gpu_prepared_circuit_benchmark.md
│   |--- ... other experiment notes and references
|--- experiments/
│   |--- 01_first_qubit/
│   |--- 02_bell_state/
│   |--- 03_first_qpu/
│   |--- 04_simulator_vs_qpu/
│   |--- 05_noise/
│   |--- 06_bernstein_vazirani/
│   |--- 07_interacting_vqe/
│   |--- 08_bathe_vqe_aer/
│   |--- 09_bathe_vqe_qpu/
|--- results/                 # Historical experiment JSON
|--- rust/
    |--- Cargo.toml
    |--- quantum_core/
    |--- quantum_cpu/
    |--- quantum_gpu/
        |--- shaders/         # WGSL sources and compiled SPIR-V
        |--- src/
```

## Running the Rust experiments

The Rust workspace can be tested without an IBM Quantum account:

```bash
cd rust

cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Run the CPU demonstrations:

```bash
cargo run --release -p quantum_cpu
```

On a machine with a compatible Vulkan compute setup and the required shaders:

```bash
cargo run --release -p quantum_gpu
cargo run --release -p quantum_gpu --bin bathe_gpu
cargo run --release -p quantum_gpu --bin circuit_bench
```

For shader recompilation, Python/Qiskit setup, individual experiments and additional commands, see [commands.md](commands.md). IBM hardware experiments require appropriate credentials and access.

## Documentation and results

- [Final scientific comparison](docs/final_comparison_report.md) — numerical methods, CPU/GPU simulation, sampled measurements, IBM QPU observations and limitations.
- [Bathe Exercise 11.1](docs/bathe_exercise_11_1.md) — mathematical reference problem.
- [IBM QPU investigation](docs/09_bathe_vqe_qpu.md) — physical-qubit selection, calibration and Bell-state diagnostics.
- [Vulkan implementation](docs/gpu_implementation.md) — backend design and prepared circuits.
- [GPU benchmarking](docs/gpu_benchmark.md) — timestamp methodology and repeatability.
- [Commands](commands.md) — practical command reference.

The `results/` directory preserves historical JSON outputs from the quantum experiments. Generated CPU benchmark measurements are ignored by Git.

## Scope

This is a small personal study, not a production quantum computing framework. The CPU and GPU implementations are classical simulations; running them on a GPU does not provide quantum advantage. The hardware observations are tied to the recorded IBM device conditions, shot counts and physical-qubit mappings.

The main outcome is a better understanding of quantum algorithms, numerical verification, GPU compute programming, and the practical differences between ideal simulation and real quantum hardware.
