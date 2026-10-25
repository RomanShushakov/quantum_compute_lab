# Experiment 09: Bathe VQE on IBM Quantum QPU

This is the real-hardware continuation of Experiment 08 (`experiments/08_bathe_vqe_aer/main.py`).

The optimized Rust angles and 10 Pauli coefficients are fixed. No hardware optimization is performed. Five compatible measurement bases are measured independently. The program uses IBM Runtime SamplerV2 and validates the ideal measurement distributions of the transpiled circuits before submission.

## Requirements

In the existing Python virtual environment:

```bash
python -m pip install 'qiskit>=2,<3' 'qiskit-ibm-runtime>=0.40' numpy
```

Use existing saved IBM Quantum credentials. Do not place tokens in source control.

## Run

From the repository root:

```bash
python experiments/09_bathe_vqe_qpu/main.py
```

This is a dry run. It contacts IBM to select a backend but submits no job. Confirm that `Ideal ISA measurement distributions verified` is printed. The dry run does not check noise or calibration accuracy.

Then explicitly submit:

```bash
python experiments/09_bathe_vqe_qpu/main.py --submit --shots 4096
```

Optionally specify a backend with `--backend NAME`. Record backend name, job ID, shots, QPU energy and standard error in `results/` after running.

## Interpretation

- Exact Rust/Aer reference: 0.724456493728
- Aer sampled reference (prior run): 0.735229872905 with 16384 shots per basis
- QPU results and calibration investigation: see below

The reported standard error accounts for finite-shot fluctuations and correlations of Pauli terms measured from the same bitstrings. It does not account for readout bias, gate noise or device drift. Hardware results may be far from the exact value.


## Hardware investigation (2026-10-25)

Backend: `ibm_kingston`. All VQE runs used the same fixed angles and
five measurement bases with 4096 shots per basis. These are evaluations
of an already optimized circuit, not a hardware-side VQE optimization.

| Measurement | Physical pair | Job ID | Result |
| --- | --- | --- | --- |
| Original VQE | [0, 1] | dbf31c7sstms73fd1hgg | Energy 6.672486800770; shot-only SE 0.072478085914 |
| Basis calibration | [0, 1] | dbf36afpc3tc739mluh0 | Correct outcomes: 19.19%, 91.60%, 17.99%, 89.43% for 00, 01, 10, 11 |
| Basis calibration | [88, 89] | dbf3aungdj9s73fmp73g | Correct outcomes: 99.68%, 99.17%, 99.37%, 98.85% |
| Repeated VQE | [88, 89] | dbf3cs7gdj9s73fmpa10 | Energy 1.073026699181; shot-only SE 0.025775407159 |

The ideal energy is 0.724456493728. Changing the physical mapping
reduced absolute energy error from 5.948030307042 to 0.348570205452,
a reduction of about 94.1%. This is strong evidence that the original
large error was associated with the physical-qubit selection. It does
not isolate initialization, readout, gate noise, or drift individually.

For the new pair, the measured XX and YY correlations were 0.899414 and
0.889160 versus ideal values of about 0.972759 each. These two terms,
weighted by -2.1 each, explain most of the remaining energy discrepancy.
The shot-only SE does not cover systematic hardware errors.

### Results layout

```text
results/09_bathe_vqe_qpu/
  calibration/
    *.json
  vqe/
    *.json
  bell_state/
    *.json
```

The results files preserve job IDs, physical mappings, raw counts, and
computed observables for offline analysis. Keep results outside the
`experiments/` source directory.

### Bell-state correlation diagnostic

`experiments/09_bathe_vqe_qpu/bell_state.py` prepares
`(|00> + |11>)/sqrt(2)` and measures XX, YY, and ZZ on the same physical
pair. Expected ideal correlations are XX=+1, YY=-1, ZZ=+1.

Dry run (no QPU submission):

```bash
python experiments/09_bathe_vqe_qpu/bell_state.py \
  --backend ibm_kingston --physical-qubits 88 89
```

Submit only after the ideal ISA checks pass:

```bash
python experiments/09_bathe_vqe_qpu/bell_state.py \
  --backend ibm_kingston --physical-qubits 88 89 --submit --shots 4096
```

### Bell-state QPU result (2026-10-25)

The dry run verified the ideal ISA distributions and mappings for all
three bases. The successful IBM Kingston job was
`dbf3g6om94ac738sps30`, using physical qubits `[88, 89]` and 4096
shots per basis.

| Basis | Ideal | QPU | Shot-only SE | Counts (00, 01, 10, 11) |
| --- | ---: | ---: | ---: | --- |
| XX | +1.000000 | +0.978516 | 0.003222 | 2047, 19, 25, 2005 |
| YY | -1.000000 | -0.952637 | 0.004752 | 73, 2073, 1926, 24 |
| ZZ | +1.000000 | +0.959961 | 0.004378 | 2105, 38, 44, 1909 |

The measured signs match the Bell-state predictions and the correlations
are close to their ideal magnitudes. Deviations exceed the reported
shot-only uncertainty, so finite-shot sampling is not the whole story.

For comparison, the Bathe VQE run on the same pair measured
`XX=+0.899414` and `YY=+0.889160` for a different target state and
longer/different circuits (ideal values approximately +0.972759).
The Bell test therefore does not provide a direct error correction for
VQE or isolate a specific gate, but it supports further investigation
of circuit-dependent noise and basis measurements.

The first Bell submission attempt failed before job submission during
IBM Cloud IAM connectivity (`Connection reset by peer`). Retrying
without changing credentials succeeded. This was a transient
connectivity/authentication-service failure, not evidence of an
invalid API token.

The Bell JSON is stored under `results/09_bathe_vqe_qpu/bell_state/`.
These observations complete the planned Experiment 09 hardware tests.
Individual initialization, readout, gate, and drift contributions
remain unresolved.

The IBM Runtime SamplerV2 deprecation warning is not a job failure;
a future maintenance step should migrate to the supported client-side
Sampler API after verifying its result format.
