# Experiment 08 - Step 2: noiseless Rust VQE

Use the real two-qubit ansatz:

    RY(q1, a); RY(q0, b); CX(q1, q0); RY(q0, c); CX(q1, q0)

Qubit 0 is the least-significant bit in the Rust statevector indexing.
This three-parameter ansatz can prepare any real normalized 4-vector,
including the ground eigenvector of Bathe's padded real Hamiltonian.

The deterministic optimizer performs periodic one-coordinate scans and
local golden-section refinement, with several initial seeds. All energy
values are exact statevector expectation values, not sampled estimates.

Reference ground energy from generalized Jacobi: 0.724456494.
The padded |11> basis state has energy 12 and should have negligible
probability at the optimum.

Next: reproduce this exact circuit and basis convention in Qiskit Aer;
only then consider an IBM QPU run. No QPU job is submitted here.
