from qiskit import QuantumCircuit
from qiskit.transpiler import generate_preset_pass_manager
from qiskit_aer import AerSimulator
from qiskit_ibm_runtime import QiskitRuntimeService
from qiskit_ibm_runtime.executor_sampler import Sampler


SHOTS = 1000
STATES = ["00", "01", "10", "11"]

THEORETICAL_PROBABILITIES = {
    "00": 0.5,
    "01": 0.0,
    "10": 0.0,
    "11": 0.5,
}


def total_variation_distance(p, q):
    return 0.5 * sum(
        abs(p[state] - q[state])
        for state in STATES
    )


def probabilities(counts):
    return {
        state: counts.get(state, 0) / SHOTS
        for state in STATES
    }


# Create the Bell-state circuit.
circuit = QuantumCircuit(2)
circuit.h(0)
circuit.cx(0, 1)
circuit.measure_all()

print("Bell circuit:")
print(circuit)


# Ideal simulation.
simulator = AerSimulator()

simulator_job = simulator.run(
    circuit,
    shots=SHOTS,
)

simulator_result = simulator_job.result()
simulator_counts = simulator_result.get_counts(circuit)
simulator_probabilities = probabilities(simulator_counts)

print("\nIdeal simulator counts:")
print(simulator_counts)


# Physical QPU.
service = QiskitRuntimeService()

backend = service.least_busy(
    operational=True,
    min_num_qubits=2,
)

print(f"\nQPU: {backend.name}")

pass_manager = generate_preset_pass_manager(
    backend=backend,
    optimization_level=1,
)

isa_circuit = pass_manager.run(circuit)

sampler = Sampler(mode=backend)

qpu_job = sampler.run(
    [isa_circuit],
    shots=SHOTS,
)

print(f"Job ID: {qpu_job.job_id()}")

qpu_result = qpu_job.result()
qpu_counts = qpu_result[0].data.meas.get_counts()
qpu_probabilities = probabilities(qpu_counts)

print("\nQPU counts:")
print(qpu_counts)


# Compare the measured probability distributions.
print("\nProbability comparison:")
print("State  Simulator  QPU")

for state in STATES:
    simulator_p = simulator_probabilities[state]
    qpu_p = qpu_probabilities[state]

    print(
        f"{state:>5}"
        f"  {simulator_p:>9.3f}"
        f"  {qpu_p:>5.3f}"
    )

simulator_vs_theory = total_variation_distance(
    simulator_probabilities,
    THEORETICAL_PROBABILITIES,
)

qpu_vs_theory = total_variation_distance(
    qpu_probabilities,
    THEORETICAL_PROBABILITIES,
)

simulator_vs_qpu = total_variation_distance(
    simulator_probabilities,
    qpu_probabilities,
)

print("\nTotal variation distance:")
print(f"Simulator vs theory: {simulator_vs_theory:.3f}")
print(f"QPU vs theory:       {qpu_vs_theory:.3f}")
print(f"Simulator vs QPU:    {simulator_vs_qpu:.3f}")
