from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from result_writer import save_result
from qiskit_ibm_runtime import QiskitRuntimeService

service = QiskitRuntimeService()
backend = service.backend("ibm_fez")
properties = backend.properties()
print(f"Backend: {backend.name}")
print(f"Physical qubits: {backend.num_qubits}")
print("\nQubit calibration properties:")
qubits = {}
for qubit in [0, 1]:
    t1 = properties.t1(qubit)
    t2 = properties.t2(qubit)
    readout_error = properties.readout_error(qubit)
    qubits[str(qubit)] = {"t1_seconds": t1, "t2_seconds": t2, "readout_error": readout_error}
    print(f"\nQubit {qubit}:")
    print(f"  T1:            {t1 * 1e6:.2f} us")
    print(f"  T2:            {t2 * 1e6:.2f} us")
    print(f"  Readout error: {readout_error:.4f}")
print("\nTwo-qubit gate properties:")
gates = {}
for gate in backend.target.operation_names:
    if gate in ["cz", "cx", "ecr"]:
        try:
            instruction = backend.target[gate][(0, 1)]
            gates[gate] = {"qubits": [0, 1], "error": instruction.error, "duration_seconds": instruction.duration}
            print(f"\nGate: {gate}")
            print("  Qubits: (0, 1)")
            print(f"  Error: {instruction.error}")
            print(f"  Duration: {instruction.duration}")
        except KeyError:
            print(f"  {gate}: not available on physical qubits (0, 1)")
save_result("05_noise", {
    "backend": backend.name, "physical_qubits": backend.num_qubits,
    "qubit_calibrations": qubits, "two_qubit_gates": gates,
    "calibration_last_update": str(properties.last_update_date) if properties else None,
}, category="calibration")
