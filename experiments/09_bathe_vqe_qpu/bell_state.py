"""Bell-state correlation diagnostic for IBM Quantum hardware.

Run without --submit to validate the ISA circuits locally.
"""

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_ibm_runtime import QiskitRuntimeService, SamplerV2

BASES = ("XX", "YY", "ZZ")
IDEAL = {"XX": 1.0, "YY": -1.0, "ZZ": 1.0}


def bell_circuit(basis: str) -> QuantumCircuit:
    qc = QuantumCircuit(2)
    qc.h(0)
    qc.cx(0, 1)
    for qubit, axis in enumerate(reversed(basis)):
        if axis == "X":
            qc.h(qubit)
        elif axis == "Y":
            qc.sdg(qubit)
            qc.h(qubit)
    qc.measure_all()
    return qc


def verify_isa(logical: QuantumCircuit, compiled: QuantumCircuit, physical: list[int]) -> list[list[int]]:
    active = sorted({compiled.find_bit(q).index for ins in compiled.data for q in ins.qubits})
    if len(active) > 12:
        raise RuntimeError("Too many active qubits for local verification")
    indices = {q: i for i, q in enumerate(active)}
    reduced = QuantumCircuit(len(active))
    mapping = []
    for ins in compiled.data:
        qids = [compiled.find_bit(q).index for q in ins.qubits]
        if ins.operation.name == "measure":
            mapping.append([qids[0], compiled.find_bit(ins.clbits[0]).index])
        elif ins.operation.name not in ("barrier", "delay"):
            reduced.append(ins.operation, [indices[q] for q in qids], [])
    if sorted(mapping) != sorted([[physical[0], 0], [physical[1], 1]]):
        raise RuntimeError(f"Unexpected measurement mapping: {mapping}")
    state = Statevector.from_instruction(reduced)
    actual = np.zeros(4)
    for index, probability in enumerate(state.probabilities()):
        classical = 0
        for q, c in mapping:
            classical |= ((index >> indices[q]) & 1) << c
        actual[classical] += probability
    expected = Statevector.from_instruction(
        logical.remove_final_measurements(inplace=False)
    ).probabilities()
    if not np.allclose(actual, expected, atol=1e-7):
        raise RuntimeError(f"ISA distribution mismatch: {actual} != {expected}")
    return sorted(mapping)


def correlation(counts: dict[str, int]) -> tuple[float, float]:
    shots = sum(counts.values())
    if shots < 2:
        raise ValueError("At least two shots are required")
    mean = sum((1 if bits.count("1") % 2 == 0 else -1) * count
               for bits, count in counts.items()) / shots
    se = math.sqrt(max(0.0, 1.0 - mean * mean) / (shots - 1))
    return mean, se


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backend", default="ibm_kingston")
    parser.add_argument("--physical-qubits", nargs=2, type=int, default=[88, 89], metavar=("Q0", "Q1"))
    parser.add_argument("--shots", type=int, default=4096)
    parser.add_argument("--submit", action="store_true")
    parser.add_argument("--output-dir", default="results/09_bathe_vqe_qpu/bell_state")
    args = parser.parse_args()
    if args.shots < 2:
        parser.error("--shots must be at least 2")
    physical = args.physical_qubits
    if min(physical) < 0 or len(set(physical)) != 2:
        parser.error("--physical-qubits must contain two distinct nonnegative indices")

    service = QiskitRuntimeService()
    backend = service.backend(args.backend)
    if backend.configuration().simulator:
        raise RuntimeError("A physical QPU is required")
    if max(physical) >= backend.num_qubits:
        raise RuntimeError("Physical qubit index exceeds backend size")
    logical = [bell_circuit(basis) for basis in BASES]
    pm = generate_preset_pass_manager(
        backend=backend, optimization_level=1, initial_layout=physical
    )
    isa = pm.run(logical)
    print(f"Backend: {backend.name}; physical qubits: {physical}; shots per basis: {args.shots}")
    mappings = []
    for basis, original, compiled in zip(BASES, logical, isa):
        mapping = verify_isa(original, compiled, physical)
        mappings.append(mapping)
        print(f"  {basis}: depth={compiled.depth()}, size={compiled.size()}, mapping={mapping}, ideal OK")
    if not args.submit:
        print("Dry run complete; no QPU job submitted.")
        return

    job = SamplerV2(mode=backend).run(isa, shots=args.shots)
    job_id = job.job_id()
    print(f"Job ID: {job_id}", flush=True)
    result = job.result()
    rows = []
    for index, (basis, mapping) in enumerate(zip(BASES, mappings)):
        counts = {str(bits).replace(" ", ""): int(count)
                  for bits, count in result[index].data.meas.get_counts().items()}
        mean, se = correlation(counts)
        print(f"  {basis}: ideal={IDEAL[basis]:+.6f}, QPU={mean:+.6f}, shot-only SE={se:.6f}")
        rows.append({"basis": basis, "counts": counts, "shots": sum(counts.values()),
                     "physical_to_classical": mapping, "ideal_correlation": IDEAL[basis],
                     "qpu_correlation": mean, "shot_only_standard_error": se})

    timestamp = datetime.now(timezone.utc)
    path_dir = Path(args.output_dir)
    path_dir.mkdir(parents=True, exist_ok=True)
    path = path_dir / f"{timestamp.strftime('%Y%m%dT%H%M%S_%fZ')}_{job_id}.json"
    payload = {"experiment": "09_bathe_vqe_qpu_bell_state", "timestamp_utc": timestamp.isoformat(),
               "backend": backend.name, "job_id": job_id, "physical_qubits": physical,
               "shots_requested_per_basis": args.shots, "results": rows}
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Saved results: {path}")


if __name__ == "__main__":
    main()
