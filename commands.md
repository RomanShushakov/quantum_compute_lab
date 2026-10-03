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

Create the experiment directory:

``` bash
mkdir -p experiments/02_bell_state
touch experiments/02_bell_state/main.py
```

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

Create the documentation directory:

``` bash
mkdir -p docs
```

The linear-algebra reference is stored at:

``` text
docs/quantum_linear_algebra_reference.tex
```
