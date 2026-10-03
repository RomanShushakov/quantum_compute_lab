# Quantum Compute Lab

Small hands-on experiments for learning quantum computing from first
principles.

The lab focuses on understanding quantum-computing concepts through
executable examples rather than only theory. The initial experiments use
Python and Qiskit, including local simulation and later execution on IBM
Quantum hardware. Rust-based quantum-computing experiments will be
explored after the basic concepts and IBM workflow are understood.

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
Rust quantum experiments
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
  README.md
  commands.md
  requirements.txt
```

`commands.md` records the commands used while building and running the
experiments.
