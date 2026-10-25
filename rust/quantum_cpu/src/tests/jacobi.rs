use super::*;

fn residual(k: &[Vec<f64>], m: &[Vec<f64>], result: &JacobiResult) -> f64 {
    let n = k.len();
    let mut largest: f64 = 0.0;
    for col in 0..n {
        for row in 0..n {
            let left: f64 = (0..n)
                .map(|j| k[row][j] * result.eigenvectors[j][col])
                .sum();
            let right: f64 = (0..n)
                .map(|j| m[row][j] * result.eigenvectors[j][col])
                .sum();
            largest = largest.max((left - result.eigenvalues[col] * right).abs());
        }
    }
    largest
}

#[test]
fn shifted_interacting_hamiltonian() {
    let k = vec![
        vec![5.0, 0.0, 0.0, 0.5],
        vec![0.0, 3.0, 0.5, 0.0],
        vec![0.0, 0.5, 3.0, 0.0],
        vec![0.5, 0.0, 0.0, 1.0],
    ];
    let m: Vec<Vec<f64>> = (0..4)
        .map(|i| (0..4).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    let result = generalized_jacobi(&k, &m, 1e-12, 30).unwrap();
    let expected = [
        3.0 - 17.0_f64.sqrt() / 2.0,
        2.5,
        3.5,
        3.0 + 17.0_f64.sqrt() / 2.0,
    ];
    for (got, want) in result.eigenvalues.iter().zip(expected) {
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
    }
    assert!(residual(&k, &m, &result) < 1e-9);
}

#[test]
fn bathe_exercise_11_1() {
    let k = vec![
        vec![6.0, -1.0, 0.0],
        vec![-1.0, 4.0, -1.0],
        vec![0.0, -1.0, 2.0],
    ];
    let m = vec![
        vec![2.0, 0.0, 0.0],
        vec![0.0, 2.0, 1.0],
        vec![0.0, 1.0, 1.0],
    ];
    let result = generalized_jacobi(&k, &m, 1e-12, 30).unwrap();
    let expected = [0.72445649, 2.96517986, 9.31036364];
    for (got, want) in result.eigenvalues.iter().zip(expected) {
        assert!((got - want).abs() < 1e-8, "{got} vs {want}");
    }
    assert!(residual(&k, &m, &result) < 1e-9);
}

#[test]
fn rejects_nonsymmetric_input() {
    let k = vec![vec![2.0, 1.0], vec![0.0, 2.0]];
    let m = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
    assert!(generalized_jacobi(&k, &m, 1e-12, 20).is_err());
}

#[test]
fn rejects_invalid_tolerance() {
    let k = vec![vec![1.0]];
    assert!(generalized_jacobi(&k, &k, 0.0, 10).is_err());
}

#[test]
fn bathe_eigenpairs_have_small_residuals() {
    let stiffness = vec![
        vec![6.0, -1.0, 0.0],
        vec![-1.0, 4.0, -1.0],
        vec![0.0, -1.0, 2.0],
    ];

    let mass = vec![
        vec![2.0, 0.0, 0.0],
        vec![0.0, 2.0, 1.0],
        vec![0.0, 1.0, 1.0],
    ];

    let result =
        super::generalized_jacobi(&stiffness, &mass, 1e-12, 30).expect("Jacobi solver failed");

    assert!(result.converged);

    let residuals = super::eigenpair_residuals(&stiffness, &mass, &result);

    for residual in residuals {
        assert!(residual < 1e-8, "Eigenpair residual too large: {residual}");
    }
}

#[test]
fn bathe_eigenvectors_are_mass_orthonormal() {
    let stiffness = vec![
        vec![6.0, -1.0, 0.0],
        vec![-1.0, 4.0, -1.0],
        vec![0.0, -1.0, 2.0],
    ];

    let mass = vec![
        vec![2.0, 0.0, 0.0],
        vec![0.0, 2.0, 1.0],
        vec![0.0, 1.0, 1.0],
    ];

    let result =
        super::generalized_jacobi(&stiffness, &mass, 1e-12, 30).expect("Jacobi solver failed");

    assert!(result.converged);

    let gram = super::mass_gram_matrix(&mass, &result);

    for (i, row) in gram.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            let expected = if i == j { 1.0 } else { 0.0 };

            assert!(
                (value - expected).abs() < 1e-8,
                "Mass orthogonality failed at ({i}, {j}): {value}"
            );
        }
    }
}
