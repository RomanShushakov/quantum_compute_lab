# Quantum Compute Lab

Small hands-on experiments for learning quantum computing from first
principles.

The lab focuses on understanding quantum-computing concepts through
executable examples rather than only theory. The initial experiments use
Python and Qiskit, including local simulation and later execution on IBM
Quantum hardware. After the initial Qiskit and IBM hardware experiments, the lab will
implement local quantum simulation in Rust, first on the CPU and then
using Vulkan compute through `ash` on the same development machine.

## Approach

The repository grows incrementally through small experiments.

Each experiment should have a concrete goal and remain small enough to
understand completely. Concepts are introduced when they are needed by
an experiment, and practical applications are connected to the theory
where useful.

The intended progression is approximately:

``` text
single qubit
    |
    v
superposition and measurement
    |
    v
multiple qubits and entanglement
    |
    v
IBM Quantum hardware
    |
    v
simulator vs real QPU
    |
    v
noise and hardware constraints
    |
    v
Bernstein-Vazirani algorithm on a QPU (complete)
    |
    v
Rust CPU state-vector simulation
    |
    v
Rust + ash + Vulkan compute simulation
```

## Experiments

### 01 - First qubit

Create a single-qubit circuit, apply a Hadamard gate, and measure the
result using the local Qiskit Aer simulator.

The qubit starts in the computational basis state:

``` text
|0>
```

The circuit is:

``` text
|0> -> H -> measure
```

The Hadamard gate transforms the initial state into an equal
superposition:

``` text
(|0> + |1>) / sqrt(2)
```

Therefore, the theoretical measurement probabilities are:

``` text
P(0) = 0.5
P(1) = 0.5
```

The circuit is executed with 1000 shots. A typical result is:

``` text
{'1': 483, '0': 517}
```

Individual runs differ because measurement outcomes are probabilistic,
while repeated measurements approach the theoretical probability
distribution.

During the experiment, two additional variants were tested.

Applying the Hadamard gate twice:

``` text
|0> -> H -> H -> measure
```

produced:

``` text
{'0': 1000}
```

because applying the Hadamard gate twice is equivalent to the identity
operation:

``` text
H * H = I
```

Adding a measurement between the two Hadamard gates:

``` text
|0> -> H -> measure -> H -> measure
```

produced four measurement histories with approximately equal
probability:

``` text
00 ~ 25%
01 ~ 25%
10 ~ 25%
11 ~ 25%
```

This demonstrates an important distinction between coherent quantum
evolution and measurement. While coherence is preserved, quantum
amplitudes can interfere. An intermediate measurement changes that
evolution.

### 02 - Bell state

Create a two-qubit Bell state and use it to explore multi-qubit state
spaces, correlation, entanglement, phase, interference, and state-vector
representation.

The canonical circuit is:

``` text
q0: |0> -> H -> * -> measure
                |
q1: |0> ------> X -> measure

               CNOT
```

The resulting Bell state is:

``` text
(|00> + |11>) / sqrt(2)
```

In state-vector form:

``` text
1/sqrt(2) * [1, 0, 0, 1]^T
```

The ideal measurement probabilities are:

``` text
P(00) = 0.5
P(01) = 0.0
P(10) = 0.0
P(11) = 0.5
```

Typical 1000-shot results were:

``` text
{'11': 475, '00': 525}
{'11': 517, '00': 483}
```

As a comparison, two independent Hadamard qubits produced all four
outcomes with approximately equal probability:

``` text
00 ~ 25%
01 ~ 25%
10 ~ 25%
11 ~ 25%
```

The Bell state cannot be factored into a Kronecker product of two valid
individual qubit state vectors. This is the linear-algebra property that
makes the state entangled.

A `Z` gate was also used to change the Bell state from:

``` text
(|00> + |11>) / sqrt(2)
```

to:

``` text
(|00> - |11>) / sqrt(2)
```

Direct measurement still produced approximately `00` and `11` with
equal probability because the phase change does not change the squared
magnitudes of the amplitudes.

Decoding the Bell state with `CNOT` followed by `H` made the phase
observable through interference. With `Z`, the result was:

``` text
{'01': 1000}
```

Without `Z`, the result was:

``` text
{'00': 1000}
```

The simulator was also used to inspect the state vector directly. The
Bell state produced approximately:

``` text
[0.70710678, 0, 0, 0.70710678]
```

and after the `Z` phase flip:

``` text
[0.70710678, 0, 0, -0.70710678]
```

This connects the linear-algebra representation to measurement:

``` text
state vector
    |
    v
complex probability amplitudes
    |
    | magnitude squared
    v
measurement probabilities
    |
    | repeated sampling
    v
counts
```

For `n` qubits there are `2^n` computational-basis states and therefore
`2^n` amplitudes in the state vector.


### 03 - First QPU

Run the Bell-state circuit from Experiment 02 on a physical IBM Quantum
processor.

The Runtime service reported `ibm_fez`, `ibm_kingston`, and
`ibm_marrakesh` as available backends. Selecting the least-busy
operational backend with at least two qubits selected:

``` text
Backend: ibm_fez
Qubits: 156
```

The logical Bell circuit uses `H` followed by `CX`. Before execution on
the physical QPU, Qiskit transpiles it for the selected backend.

For the observed transpilation, the logical operations:

``` text
measure: 2
h:       1
cx:      1
```

became an ISA circuit containing:

``` text
rz:      6
sx:      3
measure: 2
cz:      1
```

This demonstrates the hardware compilation step:

``` text
logical circuit
H + CX
    |
    v
transpilation for ibm_fez
    |
    v
QPU-native ISA circuit
Rz + SX + CZ
```

The transpiler mapped the two logical qubits to physical qubits 0 and 1
for the observed run.

The transpiled circuit was submitted with 1000 shots using the IBM
Runtime Sampler. One physical-QPU run returned:

``` text
{'00': 481, '10': 32, '11': 457, '01': 30}
```

A second run returned:

``` text
{'00': 483, '11': 450, '01': 26, '10': 41}
```

For the second run:

``` text
P(00) = 0.483
P(11) = 0.450
P(01) = 0.026
P(10) = 0.041
```

The ideal Bell outcomes accounted for `0.933` of the measurements, while
`01` and `10`, which are absent from the ideal Bell-state distribution,
accounted for `0.067`.

The ideal simulator predicts only `00` and `11`, while the physical QPU
also produced `01` and `10`. More shots improve the statistical estimate
of the physical output distribution, but do not by themselves remove
hardware noise or physical errors.

The complete execution path is:

``` text
Bell-state mathematics
    |
    v
logical Qiskit circuit
    |
    v
transpilation for ibm_fez
    |
    v
hardware-specific ISA circuit
    |
    v
IBM Runtime Sampler
    |
    v
physical QPU
    |
    v
1000 measurements
    |
    v
classical counts
```

This provides the physical-QPU baseline for the next simulator-versus-QPU
experiment.


### 04 - Simulator vs QPU

Compare the same Bell-state circuit using an ideal local simulator and a
physical IBM Quantum processor.

The exact theoretical Bell-state distribution is:

``` text
P(00) = 0.5
P(01) = 0.0
P(10) = 0.0
P(11) = 0.5
```

The experiment executes the circuit with 1000 shots on both Qiskit Aer
and the selected physical QPU, converts the resulting counts to
probabilities, and compares the distributions.

One run produced:

``` text
Ideal simulator:
{'11': 509, '00': 491}

Physical QPU:
{'00': 481, '11': 457, '01': 27, '10': 35}
```

The corresponding probabilities were:

``` text
State  Simulator  QPU
   00      0.491  0.481
   01      0.000  0.027
   10      0.000  0.035
   11      0.509  0.457
```

The ideal simulator differs slightly from the exact `0.5 / 0.5`
distribution because a finite number of shots samples the theoretical
probability distribution. It still produces no `01` or `10` outcomes.

The physical QPU produced `01` and `10` in 6.2% of the measurements:

``` text
P(01) + P(10) = 0.027 + 0.035 = 0.062
```

To compare complete distributions quantitatively, the experiment uses
total variation distance:

``` text
D_TV(P, Q) = 1/2 * sum_x |P(x) - Q(x)|
```

For the observed run:

``` text
Simulator vs theory: 0.009
QPU vs theory:       0.062
Simulator vs QPU:    0.062
```

The simulator-versus-theory distance represents finite-shot sampling
variation in an ideal simulation. The QPU-versus-theory result contains
both finite-shot sampling variation and physical hardware effects.

These distances cannot simply be subtracted to obtain a hardware error
rate. They compare complete probability distributions, and a single
finite-shot QPU sample does not isolate individual physical error
sources.

``` text
exact theoretical distribution
        |
        +--> ideal simulator sample
        |        |
        |        `--> sampling variation
        |
        `--> physical QPU sample
                 |
                 `--> sampling variation + physical effects
```

The next experiment investigates the physical effects collectively
described as quantum hardware noise.

### 05 - Noise and hardware properties (complete)

Inspect physical-qubit calibration data and use it to build an approximate
local noise model for the Bell circuit. The calibration and local noise-simulation scripts do not submit physical QPU jobs.
The corrected circuit-depth script performs a local-only dry run by default
and submits an IBM QPU job only with explicit `--submit`.

`main.py` queried `ibm_fez` (156 physical qubits) and reported the
following calibration values for physical qubits 0 and 1:

``` text
Property                 Qubit 0       Qubit 1
T1                       56.41 us      217.04 us
T2                       52.31 us      276.85 us
Readout error            0.0216        0.0258

CZ on (0, 1):
  Gate error:            0.009316654404477653
  Gate duration:         6.8e-08 s (68 ns)
```

These values are a calibration snapshot, not universal constants. They
may change over time and may not describe the hardware at the time of an
earlier QPU job. T1 describes energy relaxation; T2 describes loss of
phase coherence. Readout and gate errors are separately characterized
hardware properties, not quantities to simply add together.

`noise_simulation.py` builds an approximate Aer noise model from the IBM
backend and simulates the backend-transpiled Bell circuit on physical
qubits 0 and 1. It compares local ideal and noisy simulations with a
previously recorded QPU distribution. One run produced:

``` text
State  Ideal  Noisy  QPU (recorded)
   00  0.513  0.449  0.504
   01  0.000  0.024  0.033
   10  0.000  0.032  0.030
   11  0.487  0.495  0.433

Noisy vs ideal TV distance: 0.064
Noisy vs QPU TV distance:   0.064
```

The approximate noise model reproduces nonzero `01` and `10` outcomes,
but its full probability distribution differs from the recorded QPU run.
This comparison is exploratory: sampling variation, changing calibration
and noise-model limitations all matter. A single run cannot establish
individual hardware error contributions.

The circuit-depth experiment tests circuits with `2k` Hadamard gates,
which ideally implement the identity (`H^(2k) = I`) and always measure
`0`. Transpilation at optimization level 0 retained increasing physical
gate sequences on qubit 0. Three IBM QPU jobs were executed, each with
1000 shots per circuit:

| H pairs | ISA depth | Run 1 P(1) | Run 2 P(1) | Run 3 P(1) |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 1 | 0.006 | 0.007 | 0.007 |
| 1 | 7 | 0.007 | 0.005 | 0.013 |
| 5 | 31 | 0.025 | 0.039 | 0.034 |
| 10 | 61 | 0.051 | 0.050 | 0.062 |
| 25 | 151 | 0.128 | 0.114 | 0.120 |

Run 1: `dbedl27sstms73fbsam0`.
Run 2: `dbedldfpc3tc739lgb2g`.
Run 3: `dbedp7vpc3tc739lgg90`.

At 25 pairs, the ISA circuit contained 100 `rz` and 50 `sx` gates
plus barrier and measurement. Increasing physical circuit depth
correlated with larger observed deviations from the ideal output.
These observations do not isolate a particular noise mechanism;
readout errors, gate errors, decoherence, coherent effects and
finite-shot variation can all contribute. The 0-pair case still
includes readout and can produce incorrect outcomes.

The initial invocation without `--submit` also submitted a job due
to a misplaced dry-run guard; those first two datasets were retained.
The script was subsequently corrected and verified: running without
`--submit` printed "Dry run complete. No QPU job submitted." and saved
a local JSON record; running with `--submit` submitted one job containing
five circuits and saved a submission record followed by completed results.

Verified local JSON: `20261024T162752_655544Z_local_956e40f3.json`.
Verified third QPU JSON: `20261024T162815_562300Z_dbedp7vpc3tc739lgg90.json`.

### 06 - Bernstein-Vazirani algorithm (complete)

Recover a hidden four-bit string using a single quantum oracle query.
The oracle encodes `s = 1011` using three controlled-NOT gates
from the selected input qubits to an auxiliary qubit prepared in `|->`.
Hadamard gates before and after the oracle produce the hidden string
through phase kickback and interference. Qiskit bit strings are displayed
with the highest-index classical bit on the left.

The local ideal Qiskit Aer execution (1000 shots) returned:

```text
{'1011': 1000}
```

The explicit `--submit` execution used `ibm_fez` with 1000 shots.
The transpiled ISA circuit had depth 16 and operations:

```text
rz: 22
sx: 11
cz: 3
measure: 4
barrier: 2
```

IBM Runtime job ID: `dbedvi7pc3tc739lgpog`.
The physical QPU returned:

```text
{'1011': 879, '1111': 14, '1010': 27, '0011': 33,
 '1001': 25, '0001': 8, '0010': 2, '0000': 7,
 '1000': 3, '1110': 2}
```

The measured success rate was `879 / 1000 = 0.879` (87.9%).
The remaining 121 measurements were different from the expected string.
This is an empirical hardware result, not a fixed error rate for the
algorithm or backend. The ideal circuit recovers the secret with certainty.

The experiment saved separate JSON records for local simulation,
submission, and completed QPU results:

```text
results/06_bernstein_vazirani/20261024T164113_833544Z_local_97cad56c.json
results/06_bernstein_vazirani/20261024T164144_897743Z_submitted_e53c276e.json
results/06_bernstein_vazirani/20261024T164151_358949Z_completed_db0d2c33.json
```

The completed JSON filename contains a generated suffix rather than the
IBM job ID; use the submission record and recorded job metadata to link
artifacts. The script runs locally by default and submits to the QPU
only when explicitly passed `--submit`.

The next phase is implementing quantum state-vector simulation in Rust,
starting with a CPU reference and later using Vulkan compute via `ash`.


## JSON experiment results

Experiment scripts now save machine-readable JSON files under `results/`.
Each execution creates a new timestamped file rather than overwriting
previous runs. This is the canonical way to collect new measurements;
manual transcription of console output is not required.

Observed output locations:

``` text
results/
  04_simulator_vs_qpu/
    <timestamp>_<unique_id>.json
  05_noise/
    calibration/
      <timestamp>_<unique_id>.json
    noise_simulation/
      <timestamp>_<unique_id>.json
    circuit_depth/
      <timestamp>_<local_or_job_id>.json
  06_bernstein_vazirani/
    <timestamp>_<local_submitted_or_completed_id>.json
```

The updated Experiment 03 script is also configured to save results on
future runs. Avoid running physical-QPU scripts merely to produce a
file; they submit real jobs.

The Experiment 04 JSON includes a schema version, UTC recording time,
backend, IBM job ID, shot count, logical and ISA circuit text, theory,
ideal simulator and QPU counts/probabilities, and total variation
distances. The calibration and noise scripts save their own experiment
results. Keep the generated JSON files with the repository as a
historical dataset, excluding credentials and other secrets.

A confirmed saved physical reference is:

``` text
Experiment: 04_simulator_vs_qpu
Backend:    ibm_fez
Job ID:     dbe7cpfsstms73fbjo0g
Shots:      1000
Aer:        00=489, 11=511
QPU:        00=481, 01=31, 10=37, 11=451
TV(Aer, theory): 0.011
TV(QPU, theory): 0.068
TV(Aer, QPU):    0.068
```

Future Rust simulators will be validated against exact mathematical
predictions and Qiskit Aer. Recorded IBM distributions provide a separate
physical-hardware reference, not an exact-output target for a noiseless
simulator. For reproducible comparisons, retain circuit definitions,
qubit/bit ordering, backend mapping, shots, noise-model configuration,
software versions and random seeds where available.

## Rust CPU simulator - milestone 01 (complete)

The first Rust milestone reproduces Experiment 01 without Qiskit or a
quantum simulation library. The workspace currently contains:

- `quantum_core`: a `Qubit` with two `num_complex::Complex64` amplitudes,
  initialization to `|0>`, an in-place Hadamard operation, and probabilities
  computed with `norm_sqr()`.
- `quantum_cpu`: a small executable that prints amplitudes and probabilities
  before and after one and two Hadamard gates.

For a qubit `|psi> = alpha|0> + beta|1>`, the Hadamard implementation updates
its amplitudes as:

```text
alpha' = (alpha + beta) / sqrt(2)
beta'  = (alpha - beta) / sqrt(2)
P(0)   = |alpha|^2
P(1)   = |beta|^2
```

Verified output on the laptop (WSL2 Ubuntu):

| Stage | P(0) | P(1) |
| --- | ---: | ---: |
| Initial `|0>` | 1.000000 | 0.000000 |
| After `H` | 0.500000 | 0.500000 |
| After `H` twice | 1.000000 | 0.000000 |

After two Hadamards, the first amplitude was
`0.9999999999999998 + 0i`; the small deviation from 1 is ordinary
floating-point rounding. This milestone computes exact state-vector
probabilities (subject to floating-point precision), not random measurement
samples.

Four unit tests in `quantum_core` passed:

1. `initial_state_is_zero`
2. `hadamard_creates_equal_probabilities`
3. `hadamard_twice_is_identity`
4. `state_remains_normalized` (100 successive Hadamard applications)

The verified command `cargo test --workspace` reported four passing tests,
zero failures. The next Rust step is a general `StateVector` for multiple
qubits, followed by Bell-state preparation and controlled gates. This will
establish the amplitude-pair indexing needed for the later Vulkan backend.

## Rust and Vulkan simulator roadmap

With experiments 01-06 complete, develop a local simulator in
stages:

1. Rust CPU state-vector simulator: complex amplitudes, initialization,
   single-qubit gates, controlled gates, measurement and sampling.
2. Verify state vectors, probabilities and circuits against theory and
   Qiskit Aer, including Bell states and phase-sensitive circuits.
3. Rust Vulkan compute backend using `ash`, storage buffers, descriptors,
   command buffers, compute pipelines and SPIR-V shaders.
4. Implement parallel amplitude-pair updates without constructing full
   `2^n x 2^n` matrices for elementary gates.
5. Compare CPU and Vulkan correctness, precision, runtime, memory use,
   transfer overhead and scaling with qubit count.

All GPU work is planned for the local development machine using Vulkan
through `ash`; Jetson, Slurm and WebGPU are not part of this plan.
Classical state-vector simulation still requires exponentially growing
storage in the general case (`2^n` complex amplitudes). GPU acceleration
does not remove that fundamental constraint.

## Linear algebra reference

A compact LaTeX reference is kept in:

``` text
docs/quantum_linear_algebra_reference.tex
```

It covers column and row vectors, transpose and conjugate transpose,
inner and outer products, Kronecker/tensor products, matrix-vector
products, matrix-matrix products, and the Bell-state representation.

## Repository structure

``` text
quantum_compute_lab/
  docs/
    quantum_linear_algebra_reference.tex
  experiments/
    01_first_qubit/
      main.py
    02_bell_state/
      main.py
    03_first_qpu/
      main.py
    04_simulator_vs_qpu/
      main.py
    05_noise/
      main.py
      noise_simulation.py
      circuit_depth.py
    06_bernstein_vazirani/
      main.py
  results/
    04_simulator_vs_qpu/
      *.json
    05_noise/
      calibration/
        *.json
      noise_simulation/
        *.json
      circuit_depth/
        *.json
    06_bernstein_vazirani/
      *.json
  rust/
    Cargo.toml
    Cargo.lock
    quantum_core/
      Cargo.toml
      src/
        lib.rs
    quantum_cpu/
      Cargo.toml
      src/
        main.rs
  README.md
  commands.md
  requirements.txt
```

`commands.md` records the commands used while building and running the
experiments. The JSON results are intended for future offline analysis
and Rust CPU/Vulkan simulator comparisons.
