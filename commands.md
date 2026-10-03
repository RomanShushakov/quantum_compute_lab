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
python3 -m venv .venv
```

Activate the virtual environment:

``` bash
source .venv/bin/activate
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
