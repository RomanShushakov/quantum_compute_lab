# Quantum Compute Lab commands

Run commands from the repository root:

``` text
quantum_compute_lab
```

## Python environment

Check the installed Python version:

``` bash
python3 --version
```

Create the virtual environment:

``` bash
python3 -m venv venv
```

Activate the virtual environment:

``` bash
source venv/bin/activate
```

Upgrade pip:

``` bash
python -m pip install --upgrade pip
```

Install Qiskit:

``` bash
pip install "qiskit~=2.5.2"
```

Save the installed Python dependencies:

``` bash
pip freeze > requirements.txt
```

Verify the Qiskit installation:

``` bash
python -c "import qiskit; print(qiskit.__version__)"
```

Install the Qiskit Aer simulator:

``` bash
pip install qiskit-aer
```

Update the saved Python dependencies:

``` bash
pip freeze > requirements.txt
```

For an existing checkout, install the saved dependencies with:

``` bash
pip install -r requirements.txt
```

## Experiment 01 - First qubit

Run the first-qubit experiment:

``` bash
python experiments/01_first_qubit/main.py
```

## Experiment 02 - Bell state

Run the Bell-state experiment:

``` bash
python experiments/02_bell_state/main.py
```

The same command was used while testing the experiment variants,
including the independent-Hadamard comparison, the `Z` phase flip,
Bell-state decoding, and state-vector inspection.

After returning `main.py` to the canonical Bell-state circuit, run it
again with:

``` bash
python experiments/02_bell_state/main.py
```

## Documentation

The linear-algebra reference is stored at:

``` text
docs/quantum_linear_algebra_reference.tex
```

## Experiment 03 - First QPU

Install the IBM Qiskit Runtime client:

``` bash
pip install qiskit-ibm-runtime
pip freeze > requirements.txt
```

Save the IBM Quantum credentials locally from an interactive Python
session:

``` bash
python
```

```python
from qiskit_ibm_runtime import QiskitRuntimeService

QiskitRuntimeService.save_account(
    token="YOUR_API_KEY",
    instance="YOUR_INSTANCE_CRN",
    set_as_default=True,
    overwrite=True,
)
```

Do not store the real API key or instance CRN in the repository.

Run the physical-QPU experiment:

``` bash
python experiments/03_first_qpu/main.py
```

The same command was used while inspecting the selected backend,
transpiling the Bell circuit for its ISA, and submitting the final
1000-shot physical-QPU runs.

## Experiment 04 - Simulator vs QPU

Run the simulator-versus-QPU experiment:

``` bash
python experiments/04_simulator_vs_qpu/main.py
```

The same command was used while first establishing the ideal Aer
baseline, then adding physical-QPU execution, and finally comparing the
sampled distributions with the exact Bell-state distribution using total
variation distance.

## Dependency upgrade (October 2026)

Upgrade the principal Qiskit packages and their required dependencies:

``` bash
python -m pip install --upgrade pip
python -m pip install --upgrade qiskit qiskit-aer qiskit-ibm-runtime
```

Check package consistency and refresh the installed dependency snapshot:

``` bash
python -m pip show qiskit qiskit-aer qiskit-ibm-runtime
python -m pip check
python -m pip freeze > requirements.txt
```

Local verification without submitting QPU jobs:

``` bash
python experiments/01_first_qubit/main.py
python experiments/02_bell_state/main.py
```

## Experiment 05 - Noise and hardware properties

Inspect current backend calibration data (no Sampler job):

``` bash
python experiments/05_noise/main.py
```

Run the local calibration-derived Aer noise simulation, comparing with
previously recorded QPU counts (no Sampler job):

``` bash
python experiments/05_noise/noise_simulation.py
```

The updated scripts automatically save JSON under:

``` text
results/05_noise/calibration/
results/05_noise/noise_simulation/
```

## Automatic JSON collection and verification

The updated Experiment 04 script saves a new timestamped JSON file after
submitting its simulator and physical-QPU executions. **This command
submits a new IBM QPU job**:

``` bash
python experiments/04_simulator_vs_qpu/main.py
```

Inspect the generated result files:

``` bash
ls -lh results/04_simulator_vs_qpu/
jq . "$(ls -t results/04_simulator_vs_qpu/*.json | head -n 1)"
```

The verified saved run used backend `ibm_fez`, job ID
`dbe7cpfsstms73fbjo0g`, and 1000 shots. Its JSON includes counts,
probabilities, logical and ISA circuits, and total variation distances.

Experiment 03 also submits a physical QPU job when run; do not rerun it
just to generate JSON. Future Rust CPU/Vulkan comparisons should load
existing JSON artifacts offline.
