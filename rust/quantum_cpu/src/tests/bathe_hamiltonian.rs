use super::*;
use crate::bathe_iteration::MASS;

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "{a} != {b}");
}

#[test]
fn inverse_sqrt_whitens_mass() {
    let s = inverse_mass_sqrt();
    for i in 0..3 {
        for j in 0..3 {
            let value: f64 = (0..3)
                .flat_map(|a| (0..3).map(move |b| (a, b)))
                .map(|(a, b)| s[i][a] * MASS[a][b] * s[b][j])
                .sum();
            close(value, if i == j { 1.0 } else { 0.0 });
        }
    }
}

#[test]
fn transformed_hamiltonian_matches_independent_reference() {
    let h = transformed_hamiltonian();
    let expected = [
        [3.0, -2.0 / 10.0_f64.sqrt(), 1.0 / 10.0_f64.sqrt()],
        [-2.0 / 10.0_f64.sqrt(), 4.4, -4.2],
        [1.0 / 10.0_f64.sqrt(), -4.2, 5.6],
    ];
    for i in 0..3 {
        for j in 0..3 {
            close(h[i][j], expected[i][j]);
        }
    }
}

#[test]
fn pauli_decomposition_reconstructs_padded_hamiltonian() {
    let h = padded_hamiltonian();
    let coefficients = pauli_coefficients(&h);
    let reconstructed = reconstruct_from_paulis(&coefficients);
    for i in 0..4 {
        for j in 0..4 {
            close(h[i][j], reconstructed[i][j]);
        }
    }
}

#[test]
fn pauli_coefficients_match_reference() {
    let c = pauli_coefficients(&padded_hamiltonian());
    let expected = [
        6.25,
        -1.0 / 10.0_f64.sqrt(),
        -1.95,
        0.5 / 10.0_f64.sqrt(),
        -2.1,
        0.5 / 10.0_f64.sqrt(),
        -2.1,
        -2.55,
        -1.0 / 10.0_f64.sqrt(),
        1.25,
    ];
    for (actual, reference) in c.iter().zip(expected) {
        close(*actual, reference);
    }
}

#[test]
fn transformed_spectrum_matches_generalized_jacobi() {
    let h = transformed_hamiltonian();
    let identity = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
    ];
    let result = crate::jacobi::generalized_jacobi(
        &h.iter().map(|row| row.to_vec()).collect::<Vec<_>>(),
        &identity,
        1e-12,
        100,
    )
    .expect("standard eigenproblem should converge");
    assert!(result.converged);
    for (actual, expected) in result
        .eigenvalues
        .iter()
        .zip([0.724456494, 2.965179863, 9.310363643])
    {
        assert!((actual - expected).abs() < 1e-8);
    }
    assert!(PADDING_ENERGY > result.eigenvalues[2]);
}
