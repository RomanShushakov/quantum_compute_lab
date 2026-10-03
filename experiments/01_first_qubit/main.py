from qiskit import QuantumCircuit
from qiskit_aer import AerSimulator


# Create a circuit with one qubit and one classical bit.
# Qiskit initializes the qubit in the |0> state.
circuit = QuantumCircuit(1, 1)

# Apply the Hadamard gate to create an equal superposition:
#
# |0> -> (|0> + |1>) / sqrt(2)
#
# Measuring the resulting state therefore gives 0 or 1
# with equal theoretical probability.
circuit.h(0)

# Measure qubit 0 and store the result in classical bit 0.
circuit.measure(0, 0)

print(circuit)

# Execute the circuit locally using the Qiskit Aer simulator.
simulator = AerSimulator()

# Repeat the complete circuit 1000 times.
# Each repetition is called a shot.
job = simulator.run(circuit, shots=1000)
result = job.result()

# Count how often each classical measurement result occurred.
counts = result.get_counts(circuit)

print(counts)
