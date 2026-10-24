from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from result_writer import save_result
from qiskit import QuantumCircuit, transpile
from qiskit_aer import AerSimulator
from qiskit_aer.noise import NoiseModel
from qiskit_ibm_runtime import QiskitRuntimeService

SHOTS = 1000
STATES = ["00", "01", "10", "11"]
# Historical IBM QPU result; this script does NOT submit a new QPU job.
QPU_JOB_ID = "dbe71f7gdj9s73flac30"
QPU_COUNTS = {"00": 504, "01": 33, "10": 30, "11": 433}

def probabilities(counts):
    total = sum(counts.values())
    return {state: counts.get(state, 0) / total for state in STATES}

def total_variation_distance(p, q):
    return 0.5 * sum(abs(p[state] - q[state]) for state in STATES)

service = QiskitRuntimeService()
backend = service.backend("ibm_fez")
noise_model = NoiseModel.from_backend(backend)
circuit = QuantumCircuit(2)
circuit.h(0)
circuit.cx(0, 1)
circuit.measure_all()
isa_circuit = transpile(circuit, backend=backend, initial_layout=[0, 1], optimization_level=1)
print("Transpiled circuit:")
print(isa_circuit)
ideal_counts = AerSimulator().run(circuit, shots=SHOTS).result().get_counts()
noisy_counts = AerSimulator(noise_model=noise_model, basis_gates=noise_model.basis_gates).run(
    isa_circuit, shots=SHOTS
).result().get_counts()
ideal_p = probabilities(ideal_counts)
noisy_p = probabilities(noisy_counts)
qpu_p = probabilities(QPU_COUNTS)
print("\nProbability comparison:")
print("State  Ideal  Noisy  QPU")
for state in STATES:
    print(f"{state:>5}  {ideal_p[state]:>5.3f}  {noisy_p[state]:>5.3f}  {qpu_p[state]:>5.3f}")
noisy_vs_ideal = total_variation_distance(noisy_p, ideal_p)
noisy_vs_qpu = total_variation_distance(noisy_p, qpu_p)
print("\nTotal variation distance:")
print("Noisy vs ideal:", f"{noisy_vs_ideal:.3f}")
print("Noisy vs QPU:  ", f"{noisy_vs_qpu:.3f}")
save_result("05_noise", {
    "backend": backend.name, "shots": SHOTS,
    "historical_qpu_job_id": QPU_JOB_ID,
    "logical_circuit_text": str(circuit), "isa_circuit_text": str(isa_circuit),
    "ideal": {"counts": ideal_counts, "probabilities": ideal_p},
    "noisy": {"counts": noisy_counts, "probabilities": noisy_p},
    "historical_qpu": {"counts": QPU_COUNTS, "probabilities": qpu_p},
    "total_variation_distance": {"noisy_vs_ideal": noisy_vs_ideal, "noisy_vs_qpu": noisy_vs_qpu},
}, category="noise_simulation")
