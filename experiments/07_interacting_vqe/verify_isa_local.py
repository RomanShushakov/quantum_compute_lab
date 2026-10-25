#!/usr/bin/env python3
"""Offline ideal-state verification of Experiment 07 saved IBM ISA circuits.

Reads the saved completed JSON. Does not contact IBM Quantum or submit jobs.
"""
import argparse
import json
import math
import re
from pathlib import Path

from qiskit import QuantumCircuit
from qiskit.quantum_info import Statevector


def parse_saved_isa(qasm: str) -> QuantumCircuit:
    """Parse the small, known ISA instruction subset saved by Experiment 07."""
    circuit = QuantumCircuit(2)
    for raw in qasm.splitlines():
        line = raw.strip()
        if not line or line.startswith(('OPENQASM', 'include', 'bit[')):
            continue
        if line.startswith('c[') and 'measure' in line:
            continue
        match = re.fullmatch(r'sx \$(\d+);', line)
        if match:
            circuit.sx(int(match.group(1)))
            continue
        match = re.fullmatch(r'rz\(([^)]+)\) \$(\d+);', line)
        if match:
            expression = match.group(1)
            if expression == 'pi':
                angle = math.pi
            elif expression == '-pi':
                angle = -math.pi
            elif expression == 'pi/2':
                angle = math.pi / 2
            elif expression == '-pi/2':
                angle = -math.pi / 2
            else:
                angle = float(expression)
            circuit.rz(angle, int(match.group(2)))
            continue
        match = re.fullmatch(r'cz \$(\d+), \$(\d+);', line)
        if match:
            circuit.cz(int(match.group(1)), int(match.group(2)))
            continue
        raise ValueError(f'Unsupported saved ISA instruction: {line}')
    return circuit


def expectations(probabilities: dict[str, float]) -> tuple[float, float]:
    z0 = sum((1 if bits[-1] == '0' else -1) * p for bits, p in probabilities.items())
    z1 = sum((1 if bits[-2] == '0' else -1) * p for bits, p in probabilities.items())
    return z0, z1


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('record', type=Path, help='Experiment 07 completed JSON')
    args = parser.parse_args()
    record = json.loads(args.record.read_text(encoding='utf-8'))
    qasm = record['isa_qasm']
    z_probs = Statevector.from_instruction(parse_saved_isa(qasm['z'])).probabilities_dict()
    xx_probs = Statevector.from_instruction(parse_saved_isa(qasm['xx'])).probabilities_dict()
    z0, z1 = expectations(z_probs)
    xx = sum((1 if bits[-1] == bits[-2] else -1) * p for bits, p in xx_probs.items())
    energy = z0 + z1 + 0.5 * xx
    theta = record['theta']
    target_z = math.cos(theta)
    target_xx = math.sin(theta)
    target_energy = 2 * target_z + 0.5 * target_xx
    print('Offline ISA verification (no IBM connection)')
    print('Z probabilities:', z_probs)
    print('XX probabilities:', xx_probs)
    print(f'ISA <Z0> = {z0:+.12f}; target = {target_z:+.12f}')
    print(f'ISA <Z1> = {z1:+.12f}; target = {target_z:+.12f}')
    print(f'ISA <XX> = {xx:+.12f}; target = {target_xx:+.12f}')
    print(f'ISA energy = {energy:+.12f}; target = {target_energy:+.12f}')
    print(f'IBM measured energy = {record["expectations"]["energy"]:+.12f}')
    if not all(math.isclose(a, b, rel_tol=0, abs_tol=1e-9) for a, b in (
        (z0, target_z), (z1, target_z), (xx, target_xx), (energy, target_energy)
    )):
        raise AssertionError('Saved ISA circuits do not match intended measurement expectations')
    print('PASS: saved ISA circuits reproduce the intended ideal expectations')


if __name__ == '__main__':
    main()
