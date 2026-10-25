"""Experiment 09: Bathe VQE on IBM Quantum hardware.

Dry run (no job submission): python experiments/09_bathe_vqe_qpu/main.py
Submit: python experiments/09_bathe_vqe_qpu/main.py --submit
Requires qiskit, qiskit-ibm-runtime, numpy and saved IBM Quantum credentials.
"""

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
import math
from collections import defaultdict

import numpy as np
from qiskit import QuantumCircuit
from qiskit.quantum_info import SparsePauliOp, Statevector
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_ibm_runtime import QiskitRuntimeService, SamplerV2

ANGLES = (1.3978376255704157, 1.4124703904039295, 1.412470413122251)
RUST_ENERGY = 0.724456493728
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


def ansatz() -> QuantumCircuit:
    qc = QuantumCircuit(2)
    qc.ry(ANGLES[0], 1)
    qc.ry(ANGLES[1], 0)
    qc.cx(1, 0)
    qc.ry(ANGLES[2], 0)
    qc.cx(1, 0)
    return qc


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
    if shots == 0:
        raise ValueError("No shots returned")
    total = 0
    for bits, count in counts.items():
        bits = bits.replace(" ", "")
        parity = sum(bit == "1" and axis != "I" for bit, axis in zip(bits, pauli))
        total += (-1 if parity % 2 else 1) * count
    return total / shots


def grouped_bases() -> dict[str, list[str]]:
    groups: dict[str, list[str]] = defaultdict(list)
    for pauli in PAULI_TERMS:
        if pauli != "II":
            groups[pauli.replace("I", "Z")].append(pauli)
    return dict(sorted(groups.items()))


def energy_and_standard_error(counts_by_basis: dict[str, dict[str, int]], groups: dict[str, list[str]]) -> tuple[float, float]:
    """Sample covariance is retained by computing energy per shot per basis."""
    energy = PAULI_TERMS["II"]
    variance = 0.0
    for basis, terms in groups.items():
        counts = counts_by_basis[basis]
        shots = sum(counts.values())
        mean = 0.0
        mean_square = 0.0
        for bits, count in counts.items():
            bits = bits.replace(" ", "")
            value = 0.0
            for pauli in terms:
                parity = sum(bit == "1" and axis != "I" for bit, axis in zip(bits, pauli))
                value += PAULI_TERMS[pauli] * (-1 if parity % 2 else 1)
            weight = count / shots
            mean += weight * value
            mean_square += weight * value * value
        energy += mean
        if shots > 1:
            variance += max(0.0, mean_square - mean * mean) / (shots - 1)
    return energy, math.sqrt(variance)


def validate_transpiled_measurements(
    original: list[QuantumCircuit], isa: list[QuantumCircuit], physical_qubits: list[int] | None
) -> list[list[tuple[int, int]]]:
    """Verify ideal measurement distributions including logical-to-physical mapping.

    Reduce each transpiled circuit to its active qubits before statevector
    simulation; IBM backends may have hundreds of otherwise idle qubits.
    Abort if routing uses more than 12 active qubits.
    """
    all_mappings = []
    for logical, compiled in zip(original, isa):
        active = set()
        for instruction in compiled.data:
            if instruction.operation.name != "measure":
                active.update(compiled.find_bit(qubit).index for qubit in instruction.qubits)
            else:
                active.update(compiled.find_bit(qubit).index for qubit in instruction.qubits)
        ordered = sorted(active)
        if len(ordered) > 12:
            raise RuntimeError("ISA circuit uses more than 12 active qubits; cannot validate locally")
        remap = {physical: local for local, physical in enumerate(ordered)}
        reduced = QuantumCircuit(len(ordered))
        measurements = []
        for instruction in compiled.data:
            name = instruction.operation.name
            qids = [compiled.find_bit(q).index for q in instruction.qubits]
            if name == "measure":
                measurements.append((remap[qids[0]], compiled.find_bit(instruction.clbits[0]).index))
            elif name not in ("barrier", "delay"):
                reduced.append(instruction.operation, [remap[q] for q in qids], [])
        if len(measurements) != 2:
            raise RuntimeError("Expected exactly two measurement operations")
        physical_mapping = sorted((ordered[q], bit) for q, bit in measurements)
        if physical_qubits is not None:
            expected_mapping = sorted([(physical_qubits[0], 0), (physical_qubits[1], 1)])
            if physical_mapping != expected_mapping:
                raise RuntimeError(f"Unexpected physical measurement mapping: {physical_mapping} != {expected_mapping}")
        all_mappings.append(physical_mapping)
        state = Statevector.from_instruction(reduced)
        probabilities = np.zeros(4)
        for state_index, prob in enumerate(state.probabilities()):
            classical = 0
            for qubit, bit in measurements:
                classical |= ((state_index >> qubit) & 1) << bit
            probabilities[classical] += prob
        expected = Statevector.from_instruction(logical.remove_final_measurements(inplace=False)).probabilities()
        # Original logical circuit has basis rotations, so measurement probabilities
        # are its computational-basis probabilities.
        if not np.allclose(probabilities, expected, atol=1e-7):
            raise RuntimeError(f"ISA validation failed: {probabilities} != {expected}")
    return all_mappings


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--submit", action="store_true", help="Submit paid/limited QPU job")
    parser.add_argument("--backend", help="IBM backend name (default: least busy operational QPU)")
    parser.add_argument("--shots", type=int, default=4096)
    parser.add_argument("--physical-qubits", nargs=2, type=int, metavar=("Q0", "Q1"),
                        help="Pin logical q0,q1 to these physical qubits")
    parser.add_argument("--output-dir", default="results/09_bathe_vqe_qpu/vqe")
    args = parser.parse_args()
    if args.shots < 2:
        parser.error("--shots must be at least 2")
    if args.physical_qubits is not None and (
        len(set(args.physical_qubits)) != 2 or min(args.physical_qubits) < 0
    ):
        parser.error("--physical-qubits requires two distinct nonnegative indices")

    base = ansatz()
    hamiltonian = SparsePauliOp.from_list(list(PAULI_TERMS.items()))
    exact = float(np.real(Statevector.from_instruction(base).expectation_value(hamiltonian)))
    if abs(exact - RUST_ENERGY) > 1e-8:
        raise RuntimeError("Rust/Qiskit exact energy mismatch")
    groups = grouped_bases()
    bases = list(groups)
    logical = [measured_circuit(base, basis) for basis in bases]
    print(f"Rust / Qiskit ideal energy: {exact:.12f}")
    print(f"Measurement bases: {bases}; shots per basis: {args.shots}")

    service = QiskitRuntimeService()
    if args.backend:
        backend = service.backend(args.backend)
    else:
        backend = service.least_busy(operational=True, simulator=False, min_num_qubits=2)
    if backend.configuration().simulator:
        raise RuntimeError("Selected backend is a simulator, not a QPU")
    print(f"Backend: {backend.name}")
    if args.physical_qubits is not None and max(args.physical_qubits) >= backend.num_qubits:
        raise RuntimeError("Physical qubit index exceeds backend size")
    pm = generate_preset_pass_manager(
        backend=backend, optimization_level=1, initial_layout=args.physical_qubits
    )
    isa = pm.run(logical)
    for basis, circuit in zip(bases, isa):
        print(f"  {basis}: depth={circuit.depth()}, size={circuit.size()}")
    mappings = validate_transpiled_measurements(logical, isa, args.physical_qubits)
    for basis, mapping in zip(bases, mappings):
        print(f"  {basis}: physical->classical measurements={mapping}")
    print("Ideal ISA measurement distributions verified")

    if not args.submit:
        print("Dry run complete. Use --submit to run on IBM Quantum hardware.")
        return

    sampler = SamplerV2(mode=backend)
    job = sampler.run(isa, shots=args.shots)
    job_id = job.job_id()
    print(f"Job ID: {job_id}", flush=True)
    result = job.result()
    counts_by_basis = {}
    for i, basis in enumerate(bases):
        counts = result[i].data.meas.get_counts()
        counts_by_basis[basis] = {str(k).replace(" ", ""): int(v) for k, v in counts.items()}
        print(f"  {basis}: {sum(counts.values())} shots")
    energy, standard_error = energy_and_standard_error(counts_by_basis, groups)
    print(f"QPU energy: {energy:.12f}")
    print(f"Estimated standard error (shot noise only): {standard_error:.12f}")
    print(f"QPU minus ideal: {energy - exact:+.12f}")
    print("Note: standard error excludes hardware systematic errors and drift.")
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(timezone.utc)
    stamp = timestamp.strftime("%Y%m%dT%H%M%S_%fZ")
    path = output_dir / f"{stamp}_{job_id}.json"
    payload = {
        "experiment": "09_bathe_vqe_qpu",
        "timestamp_utc": timestamp.isoformat(),
        "backend": backend.name,
        "job_id": job_id,
        "physical_qubits": args.physical_qubits,
        "shots_requested_per_basis": args.shots,
        "angles": list(ANGLES),
        "ideal_energy": exact,
        "qpu_energy": energy,
        "shot_only_standard_error": standard_error,
        "qpu_minus_ideal": energy - exact,
        "pauli_terms": PAULI_TERMS,
        "basis_groups": groups,
        "results": [
            {"basis": basis, "counts": counts_by_basis[basis],
             "shots": sum(counts_by_basis[basis].values()),
             "physical_to_classical": mapping,
             "expectations": {term: expectation_from_counts(counts_by_basis[basis], term)
                              for term in groups[basis]}}
            for basis, mapping in zip(bases, mappings)
        ],
    }
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Saved results: {path}")


if __name__ == "__main__":
    main()
