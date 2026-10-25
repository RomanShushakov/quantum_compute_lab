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

## Experiment 05 - Circuit depth vs hardware noise

The circuit-depth script prepares five identity-equivalent circuits
with 0, 1, 5, 10 and 25 Hadamard pairs. It prints transpiled ISA
depths and runs local ideal Aer simulation before QPU submission.

**Historical note:** The original script accidentally submitted a QPU job
without `--submit`. The corrected script has now been tested: the default
command performs only local simulation, saves JSON, and does not submit
a job. QPU submission requires the explicit `--submit` flag.

Inspect circuits and save a local JSON result without a QPU job:

```bash
python experiments/05_noise/circuit_depth.py
```

Submit one IBM job containing five 1000-shot circuits only when intended:

```bash
python experiments/05_noise/circuit_depth.py --submit
```

Three jobs were completed on `ibm_fez` (1000 shots per circuit):

```text
dbedl27sstms73fbsam0
dbedldfpc3tc739lgb2g
dbedp7vpc3tc739lgg90
```

A verified local dry run also saved:

```text
results/05_noise/circuit_depth/20261024T162752_655544Z_local_956e40f3.json
```

The third QPU run saved its submission and completed results to:

```text
results/05_noise/circuit_depth/20261024T162815_562300Z_dbedp7vpc3tc739lgg90.json
```

All circuit-depth machine-readable records are stored under:

```text
results/05_noise/circuit_depth/
```

Inspect the saved JSON without submitting another job:

```bash
ls -lh results/05_noise/circuit_depth/
jq . results/05_noise/circuit_depth/*.json
```

## Experiment 06 - Bernstein-Vazirani (complete)

The experiment recovers the secret `1011` using one oracle call.
The local run uses Qiskit Aer only and saves a JSON result:

```bash
python experiments/06_bernstein_vazirani/main.py
```

To deliberately submit one IBM QPU job (1000 shots):

```bash
python experiments/06_bernstein_vazirani/main.py --submit
```

Verified local result: `{'1011': 1000}`.
Verified QPU result: `879 / 1000` correct (87.9%).
Backend: `ibm_fez`; ISA depth: 16; IBM job ID:
`dbedvi7pc3tc739lgpog`.

Saved JSON artifacts:

```text
results/06_bernstein_vazirani/20261024T164113_833544Z_local_97cad56c.json
results/06_bernstein_vazirani/20261024T164144_897743Z_submitted_e53c276e.json
results/06_bernstein_vazirani/20261024T164151_358949Z_completed_db0d2c33.json
```

Inspect results without rerunning or submitting a job:

```bash
ls -lh results/06_bernstein_vazirani/
jq . results/06_bernstein_vazirani/*.json
```

## Rust CPU simulator - completed milestones

The Rust workspace was initially created from the repository root:

```bash
mkdir -p rust
cd rust
cargo new quantum_core --lib
cargo new quantum_cpu --bin
```

The workspace contains `quantum_core` (library) and `quantum_cpu`
(executable). The library supports state vectors, H/X/Z/CNOT/Ry gates,
sampling, and Z/ZZ/XX expectations. The executable runs Bell-state,
Bernstein-Vazirani, and VQE demonstrations.

The `rand` dependency is centralized in `rust/Cargo.toml` under
`[workspace.dependencies]` and referenced by both crates with
`rand = { workspace = true }`. `quantum_core` also uses `num-complex`.

### Build and verification

From the Rust workspace directory:

```bash
cd ~/workdir/quantum_compute_lab/rust
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p quantum_cpu
```

Verified: **64 tests passed** (36 in `quantum_core`, 28 in
`quantum_cpu`), zero failures, and no Clippy warnings.

`quantum_core/src/tests.rs` contains the core unit tests.
`quantum_cpu/src/bernstein_vazirani.rs` contains the Bernstein-Vazirani
implementation and its tests. `vqe.rs`, `vqe_two_qubit.rs`, and `vqe_interacting.rs` contain the
VQE experiments and tests. `jacobi.rs` contains the generalized Jacobi
eigensolver and numerical validation tests.

### Bell-state output

`cargo run -p quantum_cpu` prepares `(|00> + |11>) / sqrt(2)` and
samples the resulting distribution using 1000 shots and seed 42:

```text
Measurement results (1000 shots):
|00>: 492
|01>: 0
|10>: 0
|11>: 508
```

These are ideal-simulator samples, not physical QPU measurements.

### Bernstein-Vazirani output

The same executable prepares a five-qubit Bernstein-Vazirani circuit:
four input qubits and one auxiliary qubit. The oracle represents the
secret `1011`; the auxiliary bit is discarded from the reported outcome.

```text
Bernstein-Vazirani experiment
Secret: 1011
1011: 1000
```

For historical comparisons, see Experiment 06 above: Qiskit Aer also
returned `1011` in all 1000 shots; the recorded IBM QPU returned it in
879 of 1000 shots.

## Rust VQE output checks

Run the Rust executable to inspect both VQE examples:

```bash
cd ~/workdir/quantum_compute_lab/rust
cargo run -p quantum_cpu
```

Verified energies: `H = Z` gives `-1.000000` using both grid search and
gradient descent; `H = Z0 + Z1 + Z0Z1` gives `-1.000000` using a two-angle
grid search.

## Rust interacting VQE and generalized Jacobi - completed

The Rust executable also runs an interacting two-qubit VQE experiment with
`H = Z0 + Z1 + 0.5 X0X1`, a classical generalized Jacobi reference, and
Bathe Exercise 11.1. The Jacobi implementation uses dense matrices; CSR
and PCG are not required for these small examples.

Run all Rust checks and the complete demonstration from `rust/`:

```bash
cd ~/workdir/quantum_compute_lab/rust
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p quantum_cpu
```

Verified interacting VQE output:

```text
Interacting two-qubit VQE experiment
Hamiltonian: Z0 + Z1 + 0.5 X0X1
Optimal theta: 3.386637
Minimum energy: -2.061552808
Exact ground energy: -2.061552813
```

The Jacobi reference solves `K*phi = lambda*M*phi` using `K = H + 3I`
and `M = I`. Subtracting 3 from the shifted eigenvalues recovers the
original Hamiltonian spectrum. It converged in two sweeps:

```text
Original H eigenvalues:
-2.061552813
-0.500000000
 0.500000000
 2.061552813
Ground-state eigenvector:
-0.122183264
 0.000000000
 0.000000000
 0.992507557
VQE/Jacobi ground-state fidelity: 1.000000000000
Grid-optimized VQE/Jacobi fidelity: 0.999999998925
```

The first fidelity uses the analytical optimal angle `pi + atan(0.25)`;
the second uses the actual grid-search angle. Fidelity is insensitive to
the global sign/phase of the eigenvector.

Bathe Exercise 11.1 provides a separate generalized eigenproblem test:

```text
Bathe Exercise 11.1 - Generalized eigenproblem
Converged: true
Sweeps: 6
Eigenpair 1: lambda = 0.724456494, residual = 4.041e-16
Eigenpair 2: lambda = 2.965179863, residual = 1.159e-15
Eigenpair 3: lambda = 9.310363643, residual = 2.878e-15
Mass Gram matrix (Phi^T M Phi):
   1.000000000   -0.000000000   -0.000000000
  -0.000000000    1.000000000    0.000000000
  -0.000000000    0.000000000    1.000000000
```

`jacobi::eigenpair_residuals()` checks
`||K*phi_i - lambda_i*M*phi_i||_2` using the original matrices, while
`jacobi::mass_gram_matrix()` checks `Phi^T*M*Phi` against the identity.
The full Rust test suite currently passes 64 tests (36 core, 28 CPU),
with zero Clippy warnings. These commands run locally and do not submit
IBM QPU jobs.
