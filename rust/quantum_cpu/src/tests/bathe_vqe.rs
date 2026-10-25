use super::*;

#[test]
fn initial_state_energy_is_three() {
    assert!((energy([0.0; 3]) - 3.0).abs() < 1e-12);
}

#[test]
fn ansatz_preserves_normalization() {
    for angles in [[0.0; 3], [0.3, 1.2, 2.1], [5.0, 4.0, 3.0]] {
        let norm: f64 = prepare_state(angles).probabilities().iter().sum();
        assert!((norm - 1.0).abs() < 1e-12);
    }
}

#[test]
fn pauli_reconstruction_gives_same_energy() {
    let h = bathe_hamiltonian::padded_hamiltonian();
    let coefficients = bathe_hamiltonian::pauli_coefficients(&h);
    let reconstructed = bathe_hamiltonian::reconstruct_from_paulis(&coefficients);
    for angles in [[0.3, 1.2, 2.1], [1.7, 0.9, 4.3]] {
        let state = prepare_state(angles);
        assert!(
            (energy_from_matrix(&state, &h) - energy_from_matrix(&state, &reconstructed)).abs()
                < 1e-12
        );
    }
}

#[test]
fn optimized_energy_matches_bathe_ground_eigenvalue() {
    let result = optimize();
    assert!((result.energy - 0.724456494).abs() < 1e-6, "{result:?}");
    assert!(result.padding_probability < 1e-6, "{result:?}");
}

#[test]
fn optimized_energy_respects_variational_bound() {
    let result = optimize();
    assert!(result.energy >= 0.724456494 - 1e-8);
}
