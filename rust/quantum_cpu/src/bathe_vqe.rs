// Experiment 08, step 2: noiseless two-qubit VQE for Bathe Exercise 11.1.
// Qubit 0 is the least significant bit in StateVector's basis ordering.
// A three-angle real-amplitude ansatz spans all real two-qubit pure states.

use crate::bathe_hamiltonian::{self, Matrix4};
use quantum_core::StateVector;
use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct VqeResult {
    pub angles: [f64; 3],
    pub energy: f64,
    pub sweeps: usize,
    pub padding_probability: f64,
}

pub fn prepare_state(angles: [f64; 3]) -> StateVector {
    let mut state = StateVector::zero(2);
    state.rotation_y(1, angles[0]);
    state.rotation_y(0, angles[1]);
    // Controlled RY(2*angles[2]) on q0, controlled by q1,
    // up to the sign convention of the CNOT sandwich.
    state.cnot(1, 0);
    state.rotation_y(0, angles[2]);
    state.cnot(1, 0);
    state
}

pub fn energy_from_matrix(state: &StateVector, h: &Matrix4) -> f64 {
    assert_eq!(state.num_qubits, 2);
    (0..4)
        .map(|i| {
            (0..4)
                .map(|j| (state.amplitudes[i].conj() * state.amplitudes[j]).re * h[i][j])
                .sum::<f64>()
        })
        .sum()
}

pub fn energy(angles: [f64; 3]) -> f64 {
    let h = bathe_hamiltonian::padded_hamiltonian();
    energy_from_matrix(&prepare_state(angles), &h)
}

// Deterministic periodic one-coordinate minimization: coarse global scan,
// then golden-section refinement of the best local interval.
fn minimize_coordinate(mut angles: [f64; 3], coordinate: usize, h: &Matrix4) -> f64 {
    const SAMPLES: usize = 32;
    let step = TAU / SAMPLES as f64;
    let mut best_angle = 0.0;
    let mut best_energy = f64::INFINITY;
    for sample in 0..SAMPLES {
        angles[coordinate] = sample as f64 * step;
        let candidate = energy_from_matrix(&prepare_state(angles), h);
        if candidate < best_energy {
            best_energy = candidate;
            best_angle = angles[coordinate];
        }
    }
    let mut left = best_angle - step;
    let mut right = best_angle + step;
    let golden = (5.0_f64.sqrt() - 1.0) / 2.0;
    let mut x1 = right - golden * (right - left);
    let mut x2 = left + golden * (right - left);
    angles[coordinate] = x1;
    let mut f1 = energy_from_matrix(&prepare_state(angles), h);
    angles[coordinate] = x2;
    let mut f2 = energy_from_matrix(&prepare_state(angles), h);
    for _ in 0..50 {
        if f1 > f2 {
            left = x1;
            x1 = x2;
            f1 = f2;
            x2 = left + golden * (right - left);
            angles[coordinate] = x2;
            f2 = energy_from_matrix(&prepare_state(angles), h);
        } else {
            right = x2;
            x2 = x1;
            f2 = f1;
            x1 = right - golden * (right - left);
            angles[coordinate] = x1;
            f1 = energy_from_matrix(&prepare_state(angles), h);
        }
    }
    ((left + right) / 2.0).rem_euclid(TAU)
}

pub fn optimize() -> VqeResult {
    let h = bathe_hamiltonian::padded_hamiltonian();
    let seeds = [
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [2.0, 4.0, 1.0],
        [4.0, 2.0, 5.0],
        [5.0, 5.0, 3.0],
    ];
    let mut best = VqeResult {
        angles: [0.0; 3],
        energy: f64::INFINITY,
        sweeps: 0,
        padding_probability: 1.0,
    };
    for mut angles in seeds {
        let mut sweeps = 0;
        for _ in 0..100 {
            let before = energy_from_matrix(&prepare_state(angles), &h);
            for coordinate in 0..3 {
                angles[coordinate] = minimize_coordinate(angles, coordinate, &h);
            }
            sweeps += 1;
            let after = energy_from_matrix(&prepare_state(angles), &h);
            if (before - after).abs() < 1e-13 {
                break;
            }
        }
        let state = prepare_state(angles);
        let result = VqeResult {
            angles,
            energy: energy_from_matrix(&state, &h),
            sweeps,
            padding_probability: state.probabilities()[3],
        };
        if result.energy < best.energy {
            best = result;
        }
    }
    best
}

#[cfg(test)]
#[path = "tests/bathe_vqe.rs"]
mod tests;
