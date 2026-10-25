"""Read-only diagnosis of Experiment 09's completed IBM Runtime job.

Run from the repository root:
    python experiments/09_bathe_vqe_qpu/diagnose.py
    python experiments/09_bathe_vqe_qpu/diagnose.py --job-id JOB_ID
"""
import argparse
from collections import Counter

import numpy as np
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector
from qiskit_ibm_runtime import QiskitRuntimeService

from main import (
    PAULI_TERMS,
    ansatz,
    energy_and_standard_error,
    expectation_from_counts,
    grouped_bases,
    measured_circuit,
    validate_transpiled_measurements,
)

DEFAULT_JOB_ID = "dbf31c7sstms73fd1hgg"


def submitted_circuits(job):
    """Extract decoded circuits when IBM Runtime exposes them through job.inputs."""
    inputs = job.inputs
    pubs = inputs.get("pubs", []) if isinstance(inputs, dict) else []
    circuits = []
    for pub in pubs:
        circuit = pub[0] if isinstance(pub, (tuple, list)) else getattr(pub, "circuit", None)
        if not isinstance(circuit, QuantumCircuit):
            raise TypeError(f"Submitted PUB circuit not decoded: {type(circuit).__name__}")
        circuits.append(circuit)
    return circuits


def measurement_map(circuit):
    pairs = []
    for inst in circuit.data:
        if inst.operation.name == "measure":
            pairs.append((circuit.find_bit(inst.qubits[0]).index,
                          circuit.find_bit(inst.clbits[0]).index))
    return pairs


def ideal_counts_probabilities(circuit):
    """Statevector probabilities indexed by classical measurement bitstring."""
    pairs = measurement_map(circuit)
    active = sorted({circuit.find_bit(q).index for inst in circuit.data for q in inst.qubits})
    if len(active) > 12:
        raise ValueError("Too many active physical qubits for local statevector")
    index = {physical: local for local, physical in enumerate(active)}
    reduced = QuantumCircuit(len(active))
    for inst in circuit.data:
        if inst.operation.name in ("measure", "barrier", "delay"):
            continue
        qubits = [index[circuit.find_bit(q).index] for q in inst.qubits]
        reduced.append(inst.operation, qubits, [])
    probs = np.zeros(4)
    for state_index, p in enumerate(Statevector.from_instruction(reduced).probabilities()):
        classical = 0
        for physical, cbit in pairs:
            classical |= ((state_index >> index[physical]) & 1) << cbit
        probs[classical] += p
    return probs


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--job-id", default=DEFAULT_JOB_ID)
    args = parser.parse_args()
    job = QiskitRuntimeService().job(args.job_id)
    print(f"Job: {args.job_id}; backend: {job.backend().name}; status: {job.status()}")
    groups = grouped_bases()
    bases = list(groups)
    result = job.result()
    counts_by_basis = {basis: result[i].data.meas.get_counts() for i, basis in enumerate(bases)}
    energy, stderr = energy_and_standard_error(counts_by_basis, groups)
    print(f"Reconstructed energy: {energy:.12f}; shot-only SE: {stderr:.12f}")

    circuits = None
    try:
        circuits = submitted_circuits(job)
        if len(circuits) != len(bases):
            raise ValueError(f"Expected {len(bases)} submitted circuits, got {len(circuits)}")
        logical = [measured_circuit(ansatz(), basis) for basis in bases]
        validate_transpiled_measurements(logical, circuits)
        print("Original submitted ISA circuits: ideal distributions verified")
    except (AttributeError, KeyError, TypeError, ValueError) as exc:
        print(f"Original submitted circuits unavailable/undecodable: {exc}")
        circuits = None

    for i, basis in enumerate(bases):
        counts = counts_by_basis[basis]
        shots = sum(counts.values())
        print(f"\nBasis {basis}; shots={shots}")
        if circuits is not None:
            circuit = circuits[i]
            print(f"  physical->classical measurements: {measurement_map(circuit)}")
            print(f"  depth={circuit.depth()}, size={circuit.size()}")
            expected = ideal_counts_probabilities(circuit)
            print("  outcome:   00       01       10       11")
            print("  ideal:   " + " ".join(f"{p:.4f}" for p in expected))
        print("  QPU:     " + " ".join(f"{counts.get(format(j, '02b'), 0)/shots:.4f}" for j in range(4)))
        for bit in range(2):
            p1 = sum(n for b, n in counts.items() if b.replace(' ', '')[-1-bit] == '1') / shots
            print(f"  P(classical bit {bit}=1): {p1:.5f}")
        for pauli in groups[basis]:
            measured = expectation_from_counts(counts, pauli)
            ideal = float(np.real(Statevector.from_instruction(ansatz()).expectation_value(
                __import__('qiskit').quantum_info.SparsePauliOp.from_list([(pauli, 1.0)]))))
            c = PAULI_TERMS[pauli]
            print(f"  {pauli}: ideal={ideal:+.6f} QPU={measured:+.6f} "
                  f"energy contribution ideal={c*ideal:+.6f} QPU={c*measured:+.6f}")

    print("\nRead-only analysis completed. No QPU job submitted.")


if __name__ == "__main__":
    main()
