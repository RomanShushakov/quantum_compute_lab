"""Experiment 09: computational-basis calibration on an IBM QPU.

Dry run: python experiments/09_bathe_vqe_qpu/calibrate.py
Submit:  python experiments/09_bathe_vqe_qpu/calibrate.py --submit

No job is submitted unless --submit is provided. Results are saved to JSON.
"""

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_ibm_runtime import QiskitRuntimeService, SamplerV2

STATES = ("00", "01", "10", "11")


def prepare_state(bits: str) -> QuantumCircuit:
    """Prepare |q1 q0> and measure into classical bits c1 c0."""
    circuit = QuantumCircuit(2, 2)
    for qubit, bit in enumerate(reversed(bits)):
        if bit == "1":
            circuit.x(qubit)
    circuit.measure(0, 0)
    circuit.measure(1, 1)
    return circuit


def reduced_distribution(circuit: QuantumCircuit) -> tuple[np.ndarray, list[tuple[int, int]]]:
    """Compute ideal classical outcomes without simulating idle backend qubits."""
    used = sorted({circuit.find_bit(q).index for inst in circuit.data for q in inst.qubits})
    if len(used) > 12:
        raise RuntimeError("More than 12 active qubits; refusing local statevector simulation")
    remap = {physical: local for local, physical in enumerate(used)}
    reduced = QuantumCircuit(len(used))
    mapping = []
    for inst in circuit.data:
        physical = [circuit.find_bit(q).index for q in inst.qubits]
        if inst.operation.name == "measure":
            mapping.append((physical[0], circuit.find_bit(inst.clbits[0]).index))
        elif inst.operation.name not in ("barrier", "delay"):
            reduced.append(inst.operation, [remap[p] for p in physical], [])
    if len(mapping) != 2:
        raise RuntimeError(f"Expected two measurements, found {mapping}")
    probs = np.zeros(4)
    for index, probability in enumerate(Statevector.from_instruction(reduced).probabilities()):
        classical = 0
        for physical, cbit in mapping:
            classical |= ((index >> remap[physical]) & 1) << cbit
        probs[classical] += probability
    return probs, mapping


def candidate_pairs(backend):
    """Rank connected pairs using available readout and two-qubit errors."""
    target = backend.target
    readout = {}
    if "measure" in target.operation_names:
        for qargs, properties in target["measure"].items():
            if properties is not None and properties.error is not None:
                readout[qargs[0]] = float(properties.error)
    edges = {}
    for name in target.operation_names:
        if name in ("measure", "reset", "delay", "barrier"):
            continue
        for qargs, properties in target[name].items():
            if qargs is None or len(qargs) != 2:
                continue
            if properties is None or properties.error is None:
                continue
            pair = tuple(qargs)
            error = float(properties.error)
            if pair not in edges or error < edges[pair][0]:
                edges[pair] = (error, name)
    rows = []
    for (a, b), (gate_error, gate_name) in edges.items():
        if a not in readout or b not in readout:
            continue
        rows.append((readout[a] + readout[b] + gate_error, a, b,
                     readout[a], readout[b], gate_error, gate_name))
    rows.sort()
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--submit", action="store_true", help="Submit an IBM hardware job")
    parser.add_argument("--backend", default="ibm_kingston")
    parser.add_argument("--physical-qubits", nargs=2, type=int, metavar=("Q0", "Q1"))
    parser.add_argument("--list-pairs", action="store_true", help="Print ranked connected pairs and exit")
    parser.add_argument("--auto-pair", action="store_true", help="Use highest-ranked available connected pair")
    parser.add_argument("--shots", type=int, default=4096)
    parser.add_argument("--output-dir", default="results/09_bathe_vqe_qpu/calibration")
    args = parser.parse_args()
    if args.shots < 2:
        parser.error("Use at least 2 shots")
    if args.physical_qubits is not None and (len(set(args.physical_qubits)) != 2 or min(args.physical_qubits) < 0):
        parser.error("Use two distinct nonnegative physical qubits")
    if args.auto_pair and args.physical_qubits is not None:
        parser.error("Choose either --auto-pair or --physical-qubits")

    service = QiskitRuntimeService()
    backend = service.backend(args.backend)
    if backend.configuration().simulator:
        raise RuntimeError("A real IBM QPU backend is required")
    ranked = candidate_pairs(backend)
    if args.list_pairs:
        print(f"Backend: {backend.name}; ranked pairs (heuristic, not predicted circuit fidelity):")
        if not ranked:
            print("No pairs with complete readout and two-qubit error data")
        for score, a, b, ra, rb, ge, gate in ranked[:20]:
            print(f"  [{a}, {b}]: readout=({ra:.5f}, {rb:.5f}), {gate}={ge:.5f}, score={score:.5f}")
        return
    if args.auto_pair:
        if not ranked:
            raise RuntimeError("No ranked connected pairs; specify --physical-qubits explicitly")
        args.physical_qubits = [ranked[0][1], ranked[0][2]]
        print("Selected top-ranked pair from current backend calibration data")
    elif args.physical_qubits is None:
        args.physical_qubits = [0, 1]
    if max(args.physical_qubits) >= backend.num_qubits:
        raise RuntimeError("Physical qubit index exceeds backend size")

    logical = [prepare_state(state) for state in STATES]
    pm = generate_preset_pass_manager(
        backend=backend, optimization_level=1, initial_layout=args.physical_qubits
    )
    isa = pm.run(logical)
    print(f"Backend: {backend.name}")
    print(f"Logical q0,q1 -> physical qubits: {args.physical_qubits}")
    print(f"Shots per prepared state: {args.shots}")
    mappings = []
    for state, circuit in zip(STATES, isa):
        probabilities, mapping = reduced_distribution(circuit)
        expected = np.zeros(4)
        expected[int(state, 2)] = 1.0
        if not np.allclose(probabilities, expected, atol=1e-8):
            raise RuntimeError(f"ISA validation failed for {state}: {probabilities}")
        expected_mapping = [(args.physical_qubits[0], 0), (args.physical_qubits[1], 1)]
        if sorted(mapping) != expected_mapping:
            raise RuntimeError(f"Unexpected physical/classical mapping: {mapping} != {expected_mapping}")
        mappings.append(mapping)
        print(f"  Prepare {state}: depth={circuit.depth()}, size={circuit.size()}, mapping={mapping}, ideal OK")

    if not args.submit:
        print("Dry run complete; no job submitted. Use --submit to run calibration.")
        return

    sampler = SamplerV2(mode=backend)
    job = sampler.run(isa, shots=args.shots)
    job_id = job.job_id()
    print(f"Job ID: {job_id}", flush=True)
    result = job.result()
    rows = []
    for index, state in enumerate(STATES):
        counts = result[index].data.c.get_counts()
        counts = {str(k).replace(" ", ""): int(v) for k, v in counts.items()}
        shots = sum(counts.values())
        correct = counts.get(state, 0) / shots
        bit0_one = sum(v for k, v in counts.items() if k[-1] == "1") / shots
        bit1_one = sum(v for k, v in counts.items() if k[-2] == "1") / shots
        rows.append({"prepared": state, "counts": counts, "shots": shots,
                     "correct_probability": correct, "p_c0_one": bit0_one,
                     "p_c1_one": bit1_one, "physical_to_classical": mappings[index]})
        print(f"  {state}: counts={counts}, correct={correct:.4%}, P(c0=1)={bit0_one:.4%}, P(c1=1)={bit1_one:.4%}")

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(timezone.utc)
    stamp = timestamp.strftime("%Y%m%dT%H%M%S_%fZ")
    path = output_dir / f"{stamp}_{job_id}.json"
    payload = {"experiment": "09_bathe_vqe_qpu_computational_basis_calibration",
               "timestamp_utc": timestamp.isoformat(),
               "backend": backend.name, "job_id": job_id,
               "physical_qubits": args.physical_qubits,
               "shots_requested_per_state": args.shots, "results": rows}
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Saved results: {path}")


if __name__ == "__main__":
    main()
