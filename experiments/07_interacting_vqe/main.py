#!/usr/bin/env python3
"""Experiment 07: fixed-parameter interacting VQE on Aer or IBM Quantum."""

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path

from qiskit import QuantumCircuit, transpile
from qiskit.quantum_info import Statevector
from qiskit_aer import AerSimulator

THETA = 3.386571317
SEED = 42
EXACT_ENERGY = -math.sqrt(17.0) / 2.0
OUTPUT_DIR = Path(__file__).resolve().parents[2] / "results" / "07_interacting_vqe"


def ansatz(theta: float) -> QuantumCircuit:
    circuit = QuantumCircuit(2)
    circuit.ry(theta, 0)
    circuit.cx(0, 1)
    return circuit


def measurement_circuit(theta: float, basis: str) -> QuantumCircuit:
    circuit = QuantumCircuit(2, 2)
    circuit.ry(theta, 0)
    circuit.cx(0, 1)
    if basis == "xx":
        circuit.h(0)
        circuit.h(1)
    elif basis != "z":
        raise ValueError(f"Unsupported measurement basis: {basis}")
    circuit.measure([0, 1], [0, 1])
    return circuit


def estimate_expectations(z_counts: dict[str, int], xx_counts: dict[str, int]) -> dict[str, float]:
    z_total = sum(z_counts.values())
    xx_total = sum(xx_counts.values())
    if z_total == 0 or xx_total == 0:
        raise ValueError("Measurement counts must not be empty")
    z0 = sum((1 if bits[-1] == "0" else -1) * count for bits, count in z_counts.items()) / z_total
    z1 = sum((1 if bits[-2] == "0" else -1) * count for bits, count in z_counts.items()) / z_total
    xx = sum((1 if bits[-1] == bits[-2] else -1) * count for bits, count in xx_counts.items()) / xx_total
    return {"z0": z0, "z1": z1, "xx": xx, "energy": z0 + z1 + 0.5 * xx}


def timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S_%fZ")


def save_record(suffix: str, payload: dict) -> Path:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    path = OUTPUT_DIR / f"{timestamp()}_{suffix}.json"
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Saved: {path}")
    return path


def print_estimates(z_counts: dict[str, int], xx_counts: dict[str, int]) -> dict[str, float]:
    estimates = estimate_expectations(z_counts, xx_counts)
    print(f"Z counts: {z_counts}")
    print(f"XX counts: {xx_counts}")
    print(f"Estimated <Z0>: {estimates['z0']:.6f}")
    print(f"Estimated <Z1>: {estimates['z1']:.6f}")
    print(f"Estimated <XX>: {estimates['xx']:.6f}")
    print(f"Estimated energy: {estimates['energy']:.9f}")
    print(f"Difference from exact: {estimates['energy'] - EXACT_ENERGY:+.9f}")
    return estimates


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--submit", action="store_true", help="Submit one two-circuit IBM QPU job")
    parser.add_argument("--shots", type=int, default=10_000, help="Shots per measurement basis")
    parser.add_argument("--backend", help="Explicit IBM backend name (otherwise least busy)")
    args = parser.parse_args()
    if args.shots <= 0:
        parser.error("--shots must be positive")
    if args.backend and not args.submit:
        parser.error("--backend requires --submit")

    state = Statevector.from_instruction(ansatz(THETA))
    amplitudes = [complex(value) for value in state.data]
    exact_z = math.cos(THETA)
    exact_xx = math.sin(THETA)
    statevector_energy = 2.0 * exact_z + 0.5 * exact_xx
    if not math.isclose(statevector_energy, EXACT_ENERGY, rel_tol=0, abs_tol=1e-10):
        raise AssertionError("Statevector energy does not match analytical reference")

    circuits = [measurement_circuit(THETA, "z"), measurement_circuit(THETA, "xx")]
    common = {
        "experiment": "07_interacting_vqe",
        "theta": THETA,
        "hamiltonian": "Z0 + Z1 + 0.5 X0X1",
        "shots_per_basis": args.shots,
        "analytical_ground_energy": EXACT_ENERGY,
        "statevector_energy": statevector_energy,
        "statevector_amplitudes": [{"real": z.real, "imag": z.imag} for z in amplitudes],
        "logical_circuits": {"z": str(circuits[0]), "xx": str(circuits[1])},
        "basis_order": ["z", "xx"],
    }
    print("Experiment 07 - Interacting VQE")
    print(f"Theta: {THETA:.9f}")
    print(f"Statevector amplitudes: {[str(value) for value in amplitudes]}")
    print(f"Exact analytical ground energy: {EXACT_ENERGY:.12f}")
    print(f"Statevector energy: {statevector_energy:.12f}")

    if not args.submit:
        simulator = AerSimulator()
        result = simulator.run(circuits, shots=args.shots, seed_simulator=SEED).result()
        z_counts = result.get_counts(0)
        xx_counts = result.get_counts(1)
        estimates = print_estimates(z_counts, xx_counts)
        save_record("local", {
            **common, "execution": "local_aer", "seed_simulator": SEED,
            "counts": {"z": z_counts, "xx": xx_counts}, "expectations": estimates,
        })
        print("No IBM QPU job submitted.")
        return

    # IBM imports are intentionally lazy: local execution needs no credentials.
    from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
    from qiskit_ibm_runtime import QiskitRuntimeService, SamplerV2

    service = QiskitRuntimeService()
    backend = service.backend(args.backend) if args.backend else service.least_busy(
        operational=True, simulator=False, min_num_qubits=2
    )
    print(f"Selected backend: {backend.name}")
    pm = generate_preset_pass_manager(backend=backend, optimization_level=1)
    isa_circuits = pm.run(circuits)
    for basis, circuit in zip(("z", "xx"), isa_circuits):
        print(f"{basis.upper()} ISA circuit: depth={circuit.depth()}, operations={dict(circuit.count_ops())}")
        print(circuit.draw(output="text", idle_wires=False))

    # Save a pre-submission record for reproducibility even if the API call fails.
    metadata = {
        **common, "execution": "ibm_qpu", "backend": backend.name,
        "isa_circuits": {basis: str(circuit) for basis, circuit in zip(("z", "xx"), isa_circuits)},
        "isa_qasm": {basis: None for basis in ("z", "xx")},
    }
    try:
        from qiskit import qasm3
        metadata["isa_qasm"] = {basis: qasm3.dumps(circuit) for basis, circuit in zip(("z", "xx"), isa_circuits)}
    except (ImportError, ValueError):
        pass
    save_record("prepared", metadata)
    print(f"Submitting ONE IBM Runtime job with 2 circuits, {args.shots} shots each.")
    sampler = SamplerV2(mode=backend)
    job = sampler.run(isa_circuits, shots=args.shots)
    job_id = job.job_id()
    print(f"Submitted IBM job ID: {job_id}")
    save_record("submitted", {**metadata, "job_id": job_id, "status": "submitted"})

    print("Waiting for IBM job result (the submitted JSON can be used for later retrieval).")
    try:
        result = job.result()
        # The default classical register is named 'c' in QuantumCircuit(2, 2).
        z_counts = result[0].data.c.get_counts()
        xx_counts = result[1].data.c.get_counts()
        estimates = print_estimates(z_counts, xx_counts)
        save_record("completed", {
            **metadata, "job_id": job_id, "status": "completed",
            "counts": {"z": z_counts, "xx": xx_counts}, "expectations": estimates,
        })
    except Exception:
        print(f"Result collection failed or is incomplete. Job ID: {job_id}")
        print("The submission record was saved. Do not resubmit merely to retrieve this job.")
        raise


if __name__ == "__main__":
    main()
