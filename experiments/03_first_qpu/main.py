from qiskit import QuantumCircuit
from qiskit.transpiler import generate_preset_pass_manager
from qiskit_ibm_runtime import QiskitRuntimeService
from qiskit_ibm_runtime.executor_sampler import Sampler


# Connect using the IBM Quantum credentials stored locally by Qiskit.
service = QiskitRuntimeService()

# Select an operational QPU that can execute our two-qubit circuit.
backend = service.least_busy(
    operational=True,
    min_num_qubits=2,
)

print(f"Backend: {backend.name}")
print(f"Qubits: {backend.num_qubits}")

# Create the same Bell-state circuit used in Experiment 02.
circuit = QuantumCircuit(2)
circuit.h(0)
circuit.cx(0, 1)
circuit.measure_all()

print("\nLogical circuit:")
print(circuit)

# Compile the logical circuit into the instruction set and physical
# qubit topology supported by the selected QPU.
pass_manager = generate_preset_pass_manager(
    backend=backend,
    optimization_level=1,
)

isa_circuit = pass_manager.run(circuit)

# Submit the transpiled circuit to the physical QPU.
sampler = Sampler(mode=backend)

job = sampler.run(
    [isa_circuit],
    shots=1000,
)

print(f"\nJob ID: {job.job_id()}")
print(f"Job status: {job.status()}")

# Wait for the job to complete and retrieve the physical measurements.
result = job.result()
counts = result[0].data.meas.get_counts()

print("\nQPU result:")
print(counts)
