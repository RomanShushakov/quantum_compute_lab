"""Experiment 05: identity circuits of increasing depth on Aer and IBM QPU.

Default: local-only dry run. Add --submit to execute one IBM Runtime job.
Run from the quantum_compute_lab repository root.
"""

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
from uuid import uuid4

from qiskit import QuantumCircuit
from qiskit.transpiler import generate_preset_pass_manager
from qiskit_aer import AerSimulator
from qiskit_ibm_runtime import QiskitRuntimeService
from qiskit_ibm_runtime.executor_sampler import Sampler


PAIRS = [0, 1, 5, 10, 25]
SHOTS = 1000
BACKEND_NAME = "ibm_fez"
PHYSICAL_QUBIT = 0
OUTPUT_DIR = Path("results/05_noise/circuit_depth")


def make_circuit(pairs: int) -> QuantumCircuit:
    circuit = QuantumCircuit(1)
    for _ in range(pairs):
        circuit.h(0)
        circuit.h(0)
    circuit.measure_all()
    return circuit


def probability_one(counts: dict[str, int]) -> float:
    total = sum(counts.values())
    if total <= 0:
        raise ValueError("Counts must contain at least one shot")
    return counts.get("1", 0) / total


def save_result(payload: dict, job_id: str | None = None) -> Path:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S_%fZ")
    suffix = job_id if job_id else f"local_{uuid4().hex[:8]}"
    output_path = OUTPUT_DIR / f"{timestamp}_{suffix}.json"
    with output_path.open("x", encoding="utf-8") as file:
        json.dump(payload, file, indent=2)
        file.write("\n")
    return output_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--submit",
        action="store_true",
        help="Submit one job containing all circuits to the IBM QPU",
    )
    args = parser.parse_args()

    service = QiskitRuntimeService()
    backend = service.backend(BACKEND_NAME)
    circuits = [make_circuit(pairs) for pairs in PAIRS]

    pass_manager = generate_preset_pass_manager(
        backend=backend,
        optimization_level=0,
        initial_layout=[PHYSICAL_QUBIT],
    )
    isa_circuits = pass_manager.run(circuits)

    print("Transpiled circuit statistics:")
    records = []
    for pairs, logical, isa in zip(PAIRS, circuits, isa_circuits):
        operations = {str(name): int(count) for name, count in isa.count_ops().items()}
        print(f"Pairs={pairs:2d}, depth={isa.depth():3d}, gates={operations}")
        records.append({
            "pairs": pairs,
            "logical_h_gates": 2 * pairs,
            "logical_circuit_qasm3": None,  # QASM export is optional; see circuit text below.
            "logical_circuit_text": str(logical.draw(output="text")),
            "isa_circuit_text": str(isa.draw(output="text")),
            "isa_depth": isa.depth(),
            "isa_operations": operations,
            "theoretical_p1": 0.0,
        })

    # Verify that the transpiler did not collapse the identity sequences.
    depths = [record["isa_depth"] for record in records]
    sx_counts = [record["isa_operations"].get("sx", 0) for record in records]
    if not all(a < b for a, b in zip(depths, depths[1:])):
        raise RuntimeError("ISA depths do not strictly increase; do not submit")
    if sx_counts != [2 * pairs for pairs in PAIRS]:
        raise RuntimeError(f"Unexpected SX counts {sx_counts}; do not submit")

    simulator = AerSimulator()
    ideal_result = simulator.run(circuits, shots=SHOTS).result()
    print("\nIdeal simulation:")
    for index, (pairs, record) in enumerate(zip(PAIRS, records)):
        counts = ideal_result.get_counts(index)
        record["ideal_counts"] = counts
        record["ideal_p1"] = probability_one(counts)
        print(f"Pairs={pairs:2d}: {counts}")
        if record["ideal_p1"] != 0.0:
            raise RuntimeError("Ideal simulator produced an unexpected 1 outcome")

    payload = {
        "schema_version": 1,
        "experiment": "05_noise_circuit_depth",
        "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
        "mode": "qpu" if args.submit else "local",
        "backend": backend.name,
        "job_id": None,
        "shots_per_circuit": SHOTS,
        "physical_qubit": PHYSICAL_QUBIT,
        "optimization_level": 0,
        "records": records,
    }

    if not args.submit:
        output_path = save_result(payload)
        print("\nDry run complete. No QPU job submitted.")
        print("Saved JSON:", output_path)
        return

    print("\nSubmitting one QPU job containing five circuits...")
    sampler = Sampler(mode=backend)
    job = sampler.run(isa_circuits, shots=SHOTS)
    payload["job_id"] = job.job_id()
    print("Job ID:", job.job_id())

    # Preserve the job ID even if retrieving results fails, so the job can be recovered.
    pending_payload = {**payload, "status": "submitted"}
    pending_path = save_result(pending_payload, job_id=job.job_id())
    print("Saved submission record:", pending_path)

    result = job.result()
    print("\nCircuit depth comparison:")
    print("Pairs  Depth  Ideal P(1)  QPU P(1)")
    for index, record in enumerate(records):
        counts = result[index].data.meas.get_counts()
        record["qpu_counts"] = counts
        record["qpu_p1"] = probability_one(counts)
        print(
            f"{record['pairs']:5d}  {record['isa_depth']:5d}  "
            f"{record['ideal_p1']:10.3f}  {record['qpu_p1']:9.3f}"
        )

    payload["status"] = "completed"
    payload["recorded_at_utc"] = datetime.now(timezone.utc).isoformat()
    # Update the same per-job record, preserving the unique job identifier.
    pending_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print("\nSaved JSON:", pending_path)


if __name__ == "__main__":
    main()
