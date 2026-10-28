# From classical eigenvalues to CPU, GPU, and IBM QPU

## Scientific question

Can the same Bathe Exercise 11.1 generalized eigenvalue problem be solved
consistently using a classical eigensolver, ideal VQE simulation on CPU and
GPU, sampled Aer simulation, and physical IBM QPU execution?

## Shared problem

Solve K phi = lambda M phi. The existing CPU implementation constructs
H = M^(-1/2) K M^(-1/2) and pads the 3x3 matrix to a 4x4 two-qubit
Hamiltonian with an elevated fourth-state energy. The common VQE ansatz is
RY(q1, theta0), RY(q0, theta1), CNOT(q1,q0), RY(q0, theta2), CNOT(q1,q0).
The classical eigenvalue is the numerical reference. CPU and GPU use ideal
state-vector expectation values, not shot sampling. IBM uses measurements.

## Historical results (do not overwrite)

| Method | Ground-energy estimate | Status |
| --- | ---: | --- |
| Classical / ideal Rust VQE | 0.724456493728 | Existing reference |
| Qiskit Aer sampled | 0.735229872905 | Historical |
| IBM QPU physical pair [0,1] | 6.672486800770 | Historical |
| IBM QPU physical pair [88,89] | 1.073026699181 | Historical |
| Vulkan GPU VQE | pending | Run bathe_gpu |

The two IBM physical mappings yielded very different energies. This is
consistent with hardware-dependent noise, native-gate transpilation,
readout error and qubit calibration, but the aggregate energies alone do
not establish a unique cause. Retain shot counts, circuit metadata,
calibration snapshots, individual Pauli estimates and uncertainty where
available. Do not infer quantum advantage from this small example.

## Accuracy analysis

Report absolute error against the classical reference, normalization,
complex-amplitude error between CPU f64 and GPU f32, and sampled uncertainty.
A noisy measured VQE estimate is not guaranteed to respect the ideal
variational bound. Keep algorithmic optimization error, finite-shot error
and hardware noise conceptually separate.

## Performance analysis

Measure per-gate and complete circuit performance for CPU and GPU with
matched qubit counts and explicit precision (CPU f64, GPU f32). Distinguish
host-to-device transfer, shader execution, pipeline setup, command recording,
queue wait and readback. On WSL2 Mesa Dozen, GPU timings characterize this
specific Vulkan-to-D3D12 stack, not native Linux GPU performance.
Physical QPU queue/network time and shot-based sampling are not directly
comparable to isolated CPU/GPU kernels.

## Reproducibility

Compile WGSL with Naga, run workspace tests and clippy, execute GPU smoke
test and bathe_gpu, save stdout and hardware information, and then update
the pending GPU row with measured results. Keep negative QPU outcomes.
