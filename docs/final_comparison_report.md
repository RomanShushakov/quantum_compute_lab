# From Classical Eigenvalue Solvers to Quantum Hardware

## A CPU, GPU, Qiskit Aer, and IBM QPU study

**Project:** Quantum Compute Lab  
**Experiment:** Bathe, *Finite Element Procedures*, Exercise 11.1  
**Status:** Final experimental comparison, 28 October 2026

## Abstract

We solve one generalized symmetric eigenvalue problem with a classical Jacobi eigensolver, an ideal variational quantum eigensolver (VQE) evaluated on a Rust CPU statevector simulator, an ideal VQE evaluated on a Rust/Vulkan GPU statevector simulator, shot-based Qiskit Aer measurements, and physical IBM quantum hardware. The lowest classical eigenvalue is approximately **0.724456494**. The ideal CPU and GPU evaluations reproduce it to the reported precision; sampled Aer yields **0.735229872905**, while two physical qubit mappings on `ibm_kingston` yield **6.672486800770** and **1.073026699181**. The physical results expose the importance of mapping and device noise rather than demonstrating a different eigenvalue or quantum speedup. Separately, a reusable Vulkan circuit simulator achieves lower measured execution time than the CPU implementation on the tested 20-qubit, 12-gate workload, subject to precision and WSL2/Dozen limitations.

## 1. Shared mathematical problem

The generalized eigenproblem is

\[
K\phi = \lambda M\phi.
\]

With a positive-definite mass matrix, define the symmetric transformed problem

\[
H = M^{-1/2} K M^{-1/2}, \qquad H\psi = \lambda\psi,
\]

where \(\psi=M^{1/2}\phi\). The three classical eigenvalues are approximately **0.724456494**, **2.965179863**, and **9.310363643**. To use a two-qubit statevector, the transformed 3×3 matrix is padded to a 4×4 Hamiltonian with a fourth diagonal energy of 12:

\[
H_4=\begin{bmatrix}
3 & -0.632455532 & 0.316227766 & 0\\
-0.632455532 & 4.4 & -4.2 & 0\\
0.316227766 & -4.2 & 5.6 & 0\\
0 & 0 & 0 & 12
\end{bmatrix}.
\]

The fourth level does not change the ground-state eigenvalue. The VQE circuit uses the same fixed gate topology across platforms:

`Ry(q1, θ0) → Ry(q0, θ1) → CNOT(q1,q0) → Ry(q0, θ2) → CNOT(q1,q0)`.

The Rust CPU optimizer finds angles `[1.3978376255704157, 1.4124703904039295, 1.412470413122251]`. Subsequent GPU, Aer, and hardware evaluations use this optimized state preparation rather than conducting independent hardware-side optimization. The energy is the expectation \(\langle\psi(\theta)|H_4|\psi(\theta)\rangle\), evaluated exactly from statevectors or estimated from measured Pauli terms depending on the backend.

## 2. Numerical comparison

The absolute errors below use the more precise ideal CPU result **0.724456493728** as the common numerical reference. The Jacobi reference is displayed rounded, so its last digits must not be treated as an independent high-precision error estimate.

| Method | Energy estimate | Absolute deviation from CPU reference | Evaluation model |
|---|---:|---:|---|
| Classical Jacobi | ≈ 0.724456494 | Rounded reference | Direct eigensolver |
| Rust CPU VQE (`f64`) | 0.724456493728 | 0 | Ideal statevector expectation |
| Rust Vulkan GPU VQE (`f32`) | 0.7244564897 | ≈ 0.00000000403 | Ideal statevector expectation at CPU angles |
| Qiskit Aer | 0.735229872905 | ≈ 0.010773379177 | 16,384 shots per measurement basis |
| IBM `ibm_kingston`, qubits `[0,1]` | 6.672486800770 | ≈ 5.948030307042 | 4,096 shots per basis; shot-only SE ≈ 0.0724781 |
| IBM `ibm_kingston`, qubits `[88,89]` | 1.073026699181 | ≈ 0.348570205453 | 4,096 shots per basis; shot-only SE ≈ 0.0257754 |

**Interpretation.** The ideal CPU and GPU evaluations agree within expected floating-point tolerance. Aer's finite-shot estimate differs from the ideal expectation; a single sampled outcome is not sufficient to attribute the full difference exclusively to statistical fluctuation without its variance and repeat runs. The IBM estimates are affected by systematic hardware effects in addition to finite-shot uncertainty. A noisy energy estimate need not obey the ideal variational bound, and it is not an eigenvalue of a new Hamiltonian.

## 3. IBM hardware investigation

The original physical mapping `[0,1]` had severe basis-state calibration errors: observed correct-outcome rates of **19.19%, 91.60%, 17.99%, 89.43%** for prepared `00`, `01`, `10`, `11`. On mapping `[88,89]`, the corresponding rates improved to **99.68%, 99.17%, 99.37%, 98.85%**. Moving the VQE circuit to `[88,89]` reduced its absolute energy error by approximately **94.1%** relative to the original mapping.

The improved mapping still did not reproduce the ideal VQE energy. For the VQE state, measured `XX ≈ 0.899414` and `YY ≈ 0.889160`, compared with ideal values around `0.972759`; both Pauli coefficients have magnitude 2.1, making these deviations energetically important. A separate Bell-state diagnostic on `[88,89]` returned `XX=+0.978516`, `YY=-0.952637`, and `ZZ=+0.959961` versus ideal `+1`, `-1`, and `+1`. These results support the presence of circuit- and measurement-dependent imperfections, but they **do not uniquely isolate** gate errors, initialization, readout, decoherence, transpilation, or drift. The reported shot-only standard errors exclude systematic bias.

The fixed angles, five measurement bases, shot counts, physical mappings, and historical JSON results should remain archived under `results/09_bathe_vqe_qpu/`.

## 4. GPU implementation and correctness

The GPU implementation uses **Rust + `ash` + Vulkan compute**, with **WGSL compiled to SPIR-V using Naga**. It does not use `wgpu`. `PreparedCircuit` records multiple gates into a reusable command buffer, retains the state in GPU-visible storage between gates, and synchronizes only after circuit submission. The benchmark resets the state between independent samples. A 12-gate H/Ry/CNOT circuit was compared against a separate CPU `f64` reference, yielding maximum observed amplitude difference **6.575×10⁻⁸** for the tested sizes.

The hardware stack is an AMD integrated Radeon GPU accessed from Ubuntu WSL2 via Mesa **Dozen (`dzn`)**, a Vulkan-over-D3D12 implementation that warns it is not conformant. The selected compute queue reported `timestampValidBits=64` and `timestampPeriod=10 ns`. Timestamp measurements are informative but not independently validated native Vulkan timings.

## 5. Performance: repeated 12-gate circuit

Three separate benchmark invocations each used **3 warm-ups and 9 measured samples**, reporting medians. Times are in milliseconds. The GPU device interval covers the recorded compute work, not necessarily one shader kernel; GPU end-to-end includes reset, submission, synchronization, query retrieval and mapped-memory readback, **but excludes one-time setup**.

| Qubits | CPU `f64` median, run 1 / 2 / 3 | GPU device median, run 1 / 2 / 3 | GPU end-to-end median, run 1 / 2 / 3 |
|---|---|---|---|
| 10 | 0.01453 / 0.00802 / 0.01390 | 0.11908 / 0.12808 / 0.11396 | 0.76327 / 0.74980 / 0.72830 |
| 15 | 0.43864 / 0.24624 / 0.42239 | 0.14272 / 0.14968 / 0.15116 | 0.85064 / 0.69800 / 0.79252 |
| 20 | 9.61218 / 9.38962 / 8.90438 | 3.43608 / 3.44204 / 3.60284 | 6.01531 / 5.97852 / 6.47745 |

For **20 qubits**, the CPU/GPU device-time ratio ranges approximately **2.47–2.80×**, while CPU/GPU end-to-end ratio ranges approximately **1.37–1.61×**, depending on the invocation. This is evidence of a benefit for this **particular implementation and workload**, not a general claim that GPUs are faster for quantum simulation. At 10 qubits the CPU is clearly faster; at 15 qubits the GPU device interval is lower but GPU end-to-end remains slower.

**Caveats:** CPU is `f64`, GPU is `f32`; neither is a precision-matched performance baseline. Setup took approximately 18–61 ms across the recorded invocations and is excluded from end-to-end medians. Readback uses mapped host-visible memory and is not necessarily a discrete GPU-to-host copy. CPU-side variability is noticeable, especially at 10–15 qubits. This experiment measures classical statevector simulation, **not quantum advantage**. IBM QPU queue/network latency and shot collection are different operations and should not be plotted as equivalent to GPU kernel timing.

## 6. What this study establishes

1. **Mathematical consistency:** The Bathe generalized eigenproblem, symmetric transformation, two-qubit padding, and ideal VQE state preparation produce consistent lowest-energy estimates.
2. **Precision behavior:** The GPU `f32` simulator reproduces the CPU `f64` reference closely for the tested circuits.
3. **Measurement effects:** Finite-shot Aer and noisy IBM QPU estimates differ from exact statevector expectations, for different reasons that must not be conflated.
4. **Hardware mapping matters:** The IBM Kingston physical-qubit choice strongly affected the measured result; calibration and Bell-state diagnostics helped characterize, but not fully explain, the discrepancy.
5. **Classical acceleration:** Reusable Vulkan execution and GPU-resident states improved measured performance on the 20-qubit, 12-gate benchmark in this WSL2 environment, while setup and readback remained significant.

## 7. Limitations and future work

This is a small, deliberately controlled case study rather than evidence of asymptotic advantage. A more rigorous hardware-performance study would require precision-matched CPU/GPU implementations, more circuit depths and qubit counts, confidence intervals across independent benchmark sessions, native Linux Vulkan validation, and explicit state-initialization/transfer policies. A stronger quantum-hardware error analysis would require additional calibration-aware and noise-model experiments, repeated QPU jobs over time, and error-mitigation comparisons. None is required to preserve the conclusions already supported by this study.

## 8. Reproduction and provenance

From `rust/`:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run --release -p quantum_gpu
cargo run --release -p quantum_gpu --bin bathe_gpu
cargo run --release -p quantum_gpu --bin circuit_bench
```

The verified workspace suite contains **92 tests**. Recompile modified WGSL shaders with `naga quantum_gpu/shaders/<name>.wgsl quantum_gpu/shaders/<name>.spv` before running GPU binaries. Preserve the IBM result JSON files, `docs/09_bathe_vqe_qpu.md`, `docs/gpu_benchmark.md`, and experiment scripts for auditability. No new QPU jobs are required to reproduce the archived analysis.

---

**Conclusion.** A single well-defined finite-element eigenproblem provides a useful bridge between classical numerical linear algebra, ideal quantum-circuit simulation on CPU/GPU, sampled simulation, and real quantum hardware. The closest numerical agreement occurs in ideal statevector simulations; physical QPU results reveal substantial hardware sensitivity. GPU acceleration is demonstrated for one classical simulation workload, but no quantum computational advantage is claimed.
