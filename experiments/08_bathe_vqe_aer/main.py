"""Experiment 08, step 3: reproduce Rust Bathe VQE in Qiskit Aer.

Run from the repository root after installing qiskit and qiskit-aer.
This is a local simulator experiment; it does not submit IBM Quantum jobs.
"""

import math
from collections import defaultdict

import numpy as np
from qiskit import QuantumCircuit, transpile
from qiskit.quantum_info import Operator, SparsePauliOp, Statevector
from qiskit_aer import AerSimulator

# Rust StateVector: q0 is least significant; Pauli labels are q1 q0.
ANGLES = (1.3978376255704157, 1.4124703904039295, 1.412470413122251)
RUST_ENERGY = 0.724456493728
REFERENCE_ENERGY = 0.724456493728  # Rounded Rust result, not an independent eigenvalue.
SHOTS_PER_BASIS = 16384
SEED = 20261025

PAULI_TERMS = {
    "II": 6.25,
    "IX": -1.0 / math.sqrt(10.0),
    "IZ": -1.95,
    "XI": 0.5 / math.sqrt(10.0),
    "XX": -2.1,
    "XZ": 0.5 / math.sqrt(10.0),
    "YY": -2.1,
    "ZI": -2.55,
    "ZX": -1.0 / math.sqrt(10.0),
    "ZZ": 1.25,
}


def circuit(angles: tuple[float, float, float]) -> QuantumCircuit:
    qc = QuantumCircuit(2)
    qc.ry(angles[0], 1)
    qc.ry(angles[1], 0)
    qc.cx(1, 0)
    qc.ry(angles[2], 0)
    qc.cx(1, 0)
    return qc


def reference_matrix() -> np.ndarray:
    return np.array(
        [
            [3.0, -2.0 / math.sqrt(10.0), 1.0 / math.sqrt(10.0), 0.0],
            [-2.0 / math.sqrt(10.0), 4.4, -4.2, 0.0],
            [1.0 / math.sqrt(10.0), -4.2, 5.6, 0.0],
            [0.0, 0.0, 0.0, 12.0],
        ],
        dtype=float,
    )


def measurement_basis(pauli: str) -> str:
    # I can be measured in any basis; choose Z to reduce settings.
    return pauli.replace("I", "Z")


def measured_circuit(base: QuantumCircuit, basis: str) -> QuantumCircuit:
    qc = base.copy()
    for qubit, axis in enumerate(reversed(basis)):
        if axis == "X":
            qc.h(qubit)
        elif axis == "Y":
            qc.sdg(qubit)
            qc.h(qubit)
    qc.measure_all()
    return qc


def expectation_from_counts(counts: dict[str, int], pauli: str) -> float:
    shots = sum(counts.values())
    total = 0
    for bits, count in counts.items():
        # Qiskit count strings are q1 q0; identities do not contribute parity.
        parity = sum(bit == "1" and axis != "I" for bit, axis in zip(bits.replace(" ", ""), pauli))
        total += (-1 if parity % 2 else 1) * count
    return total / shots


def main() -> None:
    qc = circuit(ANGLES)
    state = Statevector.from_instruction(qc)
    h = reference_matrix()
    pauli_h = SparsePauliOp.from_list(list(PAULI_TERMS.items()))
    assert np.allclose(Operator(pauli_h).data, h, atol=1e-12)

    exact_matrix = float(np.real(np.vdot(state.data, h @ state.data)))
    exact_pauli = float(np.real(state.expectation_value(pauli_h)))
    eigenvalues = np.linalg.eigvalsh(h)
    print("Bathe VQE / Qiskit Aer validation")
    print("Angles:", ANGLES)
    print("Statevector:", state.data)
    print(f"Exact matrix energy: {exact_matrix:.12f}")
    print(f"Exact Pauli energy:  {exact_pauli:.12f}")
    print(f"Rust energy:         {RUST_ENERGY:.12f}")
    print(f"Exact ground energy: {eigenvalues[0]:.12f}")
    print(f"Padding |11> probability: {abs(state.data[3]) ** 2:.12e}")
    assert abs(exact_matrix - exact_pauli) < 1e-11
    assert abs(exact_matrix - RUST_ENERGY) < 1e-8
    assert abs(exact_matrix - eigenvalues[0]) < 1e-8
    assert abs(state.data[3]) ** 2 < 1e-10

    # Share shot data between compatible terms by measuring in a full X/Y/Z basis.
    groups: dict[str, list[str]] = defaultdict(list)
    for pauli in PAULI_TERMS:
        if pauli != "II":
            groups[measurement_basis(pauli)].append(pauli)

    bases = sorted(groups)
    simulator = AerSimulator(seed_simulator=SEED)
    circuits = transpile([measured_circuit(qc, basis) for basis in bases], simulator)
    result = simulator.run(circuits, shots=SHOTS_PER_BASIS).result()

    sampled = PAULI_TERMS["II"]
    print(f"Measurement settings: {len(bases)}, shots per setting: {SHOTS_PER_BASIS}")
    for i, basis in enumerate(bases):
        counts = result.get_counts(i)
        for pauli in groups[basis]:
            expectation = expectation_from_counts(counts, pauli)
            sampled += PAULI_TERMS[pauli] * expectation
            print(f"  {pauli}: <P> = {expectation:+.6f} (basis {basis})")
    print(f"Sampled Aer energy:  {sampled:.12f}")
    print(f"Sampled minus exact: {sampled - exact_matrix:+.12f}")
    # Sampling is stochastic: do not assert machine-precision agreement.
    assert abs(sampled - exact_matrix) < 0.3, "Sampled result unexpectedly far from exact; investigate"


if __name__ == "__main__":
    main()
