from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from result_writer import save_result

from qiskit import QuantumCircuit
from qiskit.transpiler import generate_preset_pass_manager
from qiskit_ibm_runtime import QiskitRuntimeService
from qiskit_ibm_runtime.executor_sampler import Sampler

SHOTS = 1000
service = QiskitRuntimeService()
backend = service.least_busy(operational=True, min_num_qubits=2)
print(f"Backend: {backend.name}")
print(f"Qubits: {backend.num_qubits}")
circuit = QuantumCircuit(2)
circuit.h(0)
circuit.cx(0, 1)
circuit.measure_all()
print("\nLogical circuit:")
print(circuit)
pass_manager = generate_preset_pass_manager(backend=backend, optimization_level=1)
isa_circuit = pass_manager.run(circuit)
sampler = Sampler(mode=backend)
job = sampler.run([isa_circuit], shots=SHOTS)
print(f"\nJob ID: {job.job_id()}")
print(f"Job status: {job.status()}")
result = job.result()
counts = result[0].data.meas.get_counts()
print("\nQPU result:")
print(counts)
save_result("03_first_qpu", {
    "backend": backend.name, "job_id": job.job_id(), "shots": SHOTS,
    "logical_circuit_qasm": circuit.qasm() if hasattr(circuit, "qasm") else None,
    "logical_circuit_text": str(circuit),
    "isa_circuit_text": str(isa_circuit),
    "counts": counts,
    "probabilities": {s: counts.get(s, 0) / SHOTS for s in ["00", "01", "10", "11"]},
})
