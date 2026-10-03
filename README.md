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

```text
q0: |0> -> H -> * -> measure
                |
q1: |0> ------> X -> measure

               CNOT
```

The resulting Bell state is:

```text
(|00> + |11>) / sqrt(2)
```

In state-vector form:

```text
1/sqrt(2) * [1, 0, 0, 1]^T
```

The ideal measurement probabilities are:

```text
P(00) = 0.5
P(01) = 0.0
P(10) = 0.0
P(11) = 0.5
```

Typical 1000-shot results were:

```text
{'11': 475, '00': 525}
{'11': 517, '00': 483}
```

As a comparison, two independent Hadamard qubits produced all four
outcomes with approximately equal probability:

```text
00 ~ 25%
01 ~ 25%
10 ~ 25%
11 ~ 25%
```

The Bell state cannot be factored into a Kronecker product of two valid
individual qubit state vectors. This is the linear-algebra property that
makes the state entangled.

A `Z` gate was also used to change the Bell state from:

```text
(|00> + |11>) / sqrt(2)
```

to:

```text
(|00> - |11>) / sqrt(2)
```

Direct measurement still produced approximately `00` and `11` with
equal probability because the phase change does not change the squared
magnitudes of the amplitudes.

Decoding the Bell state with `CNOT` followed by `H` made the phase
observable through interference. With `Z`, the result was:

```text
{'01': 1000}
```

Without `Z`, the result was:

```text
{'00': 1000}
```

The simulator was also used to inspect the state vector directly. The
Bell state produced approximately:

```text
[0.70710678, 0, 0, 0.70710678]
```

and after the `Z` phase flip:

```text
[0.70710678, 0, 0, -0.70710678]
```

This connects the linear-algebra representation to measurement:

```text
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

## Linear algebra reference

A compact LaTeX reference is kept in:

```text
docs/quantum_linear_algebra_reference.tex
```

It covers column and row vectors, transpose and conjugate transpose,
inner and outer products, Kronecker/tensor products, matrix-vector
products, matrix-matrix products, and the Bell-state representation.

## Repository structure

```text
quantum_compute_lab/
  docs/
    quantum_linear_algebra_reference.tex
  experiments/
    01_first_qubit/
      main.py
    02_bell_state/
      main.py
  README.md
  commands.md
  requirements.txt
```

`commands.md` records the commands used while building and running the
experiments.
