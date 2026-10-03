from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator


# Create a circuit with two qubits and two classical bits.
# Both qubits start in the |0> state.
circuit = QuantumCircuit(2, 2)

# Put qubit 0 into an equal superposition.
circuit.h(0)

# Entangle qubit 0 and qubit 1.
#
# The resulting Bell state is:
#
#     (|00> + |11>) / sqrt(2)
#
circuit.cx(0, 1)

# Measure both qubits.
circuit.measure([0, 1], [0, 1])

print(circuit)

simulator = AerSimulator()

job = simulator.run(circuit, shots=1000)
result = job.result()

counts = result.get_counts(circuit)

print(counts)
