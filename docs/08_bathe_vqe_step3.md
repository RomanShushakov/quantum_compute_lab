# Experiment 08 - Step 3: Qiskit Aer validation

Goal: reproduce the Rust two-qubit Bathe VQE state and energy using the same
optimized angles and gate sequence, without submitting a hardware job.

Run from the repository root:

```bash
python -m pip install 'qiskit>=2,<3' 'qiskit-aer>=0.15' numpy
python python/experiments/08_bathe_vqe_aer.py
```

The experiment validates the 4x4 matrix against the Pauli operator, computes
exact statevector energy and compares it to the Rust result, then estimates the
energy with finite shots in a local AerSimulator.

Qubit conventions: Rust q0 = Qiskit qubit 0 = least significant bit. Pauli
labels and measurement bit strings are ordered q1 q0.

The circuit is Ry(a) on q1, Ry(b) on q0, CX(1,0), Ry(c) on q0,
CX(1,0). This is the exact Rust gate sequence; it is not a generic CRY.

Finite-shot energies fluctuate. The test uses 16384 shots per measurement basis
and a deliberately loose stochastic guardrail, not a precision guarantee.
For QPU use, validate the transpiled circuit and the measurement bases on an
ideal simulator first; consider calibration, mitigation and uncertainty bars.

No Rust files are changed in this step.
