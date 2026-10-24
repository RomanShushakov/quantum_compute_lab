"""Experiment 06: Bernstein-Vazirani, local by default, QPU via --submit."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
from uuid import uuid4

from qiskit import QuantumCircuit
from qiskit.transpiler import generate_preset_pass_manager
from qiskit_aer import AerSimulator

SECRET = "1011"
SHOTS = 1000
BACKEND_NAME = "ibm_fez"


def circuit_for_secret(secret: str) -> QuantumCircuit:
    n = len(secret)
    qc = QuantumCircuit(n + 1, n)
    auxiliary = n
    qc.x(auxiliary)
    qc.h(auxiliary)
    for qubit in range(n):
        qc.h(qubit)
    qc.barrier()
    # Qiskit prints higher classical bit indices on the left.
    # secret[-1] maps to q0, secret[0] to q(n-1).
    for qubit, bit in enumerate(reversed(secret)):
        if bit == "1":
            qc.cx(qubit, auxiliary)
    qc.barrier()
    for qubit in range(n):
        qc.h(qubit)
        qc.measure(qubit, qubit)
    return qc


def save_json(payload, label):
    directory = Path(__file__).resolve().parents[2] / "results" / "06_bernstein_vazirani"
    directory.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S_%fZ")
    filename = f"{stamp}_{label}_{uuid4().hex[:8]}.json"
    path = directory / filename
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print("Saved JSON:", path)
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--submit", action="store_true", help="Submit a real IBM QPU job")
    parser.add_argument("--secret", default=SECRET, help="Hidden binary string")
    parser.add_argument("--shots", type=int, default=SHOTS)
    parser.add_argument("--backend", default=BACKEND_NAME)
    args = parser.parse_args()
    if not args.secret or any(bit not in "01" for bit in args.secret):
        parser.error("--secret must be a nonempty binary string")
    if args.shots < 1:
        parser.error("--shots must be positive")

    circuit = circuit_for_secret(args.secret)
    print("Logical circuit:\n", circuit)
    local_counts = AerSimulator().run(circuit, shots=args.shots).result().get_counts()
    print("Ideal Aer counts:", local_counts)
    assert local_counts == {args.secret: args.shots}, "Unexpected ideal result; check bit ordering"

    payload = {
        "schema_version": 1,
        "experiment": "06_bernstein_vazirani",
        "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
        "secret": args.secret,
        "shots": args.shots,
        "logical_circuit_qasm3": None,
        "logical_circuit_text": str(circuit),
        "ideal_counts": local_counts,
        "ideal_success_probability": 1.0,
        "mode": "local",
        "qpu": None,
    }
    # QASM 3 provides a machine-readable circuit for future Rust comparison.
    from qiskit import qasm3
    payload["logical_circuit_qasm3"] = qasm3.dumps(circuit)

    if not args.submit:
        print("Local-only run. No IBM QPU job submitted.")
        save_json(payload, "local")
        return

    from qiskit_ibm_runtime import QiskitRuntimeService
    from qiskit_ibm_runtime.executor_sampler import Sampler
    service = QiskitRuntimeService()
    backend = service.backend(args.backend)
    pm = generate_preset_pass_manager(backend=backend, optimization_level=1)
    isa = pm.run(circuit)
    print("QPU backend:", backend.name)
    print("ISA operations:", dict(isa.count_ops()))
    print("ISA depth:", isa.depth())
    payload.update({
        "mode": "qpu_submitted",
        "backend": backend.name,
        "isa_depth": isa.depth(),
        "isa_operations": dict(isa.count_ops()),
        "isa_circuit_text": str(isa),
        "isa_circuit_qasm3": qasm3.dumps(isa),
    })
    sampler = Sampler(mode=backend)
    print("Submitting one IBM QPU job...")
    job = sampler.run([isa], shots=args.shots)
    payload["job_id"] = job.job_id()
    print("Job ID:", job.job_id())
    save_json(payload, "submitted")

    counts = job.result()[0].data.c.get_counts()
    successes = counts.get(args.secret, 0)
    payload["mode"] = "qpu_completed"
    payload["qpu"] = {
        "counts": counts,
        "successes": successes,
        "success_probability": successes / args.shots,
    }
    print("QPU counts:", counts)
    print(f"Success probability: {successes / args.shots:.3f}")
    save_json(payload, "completed")


if __name__ == "__main__":
    main()
