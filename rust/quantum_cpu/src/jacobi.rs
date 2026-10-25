// Generalized Jacobi eigensolver based on the rotation scheme in
// K. J. Bathe, Finite Element Procedures, Section 11.3, JACOBI.FOR.
// Dense symmetric storage is used instead of Fortran triangular indexing.

#[derive(Debug, Clone)]
pub struct JacobiResult {
    pub eigenvalues: Vec<f64>,
    // eigenvectors[row][column]; eigenvectors are M-orthonormal.
    pub eigenvectors: Vec<Vec<f64>>,
    pub sweeps: usize,
    pub converged: bool,
}

fn validate_matrix(matrix: &[Vec<f64>], n: usize, name: &str) -> Result<(), String> {
    if matrix.len() != n || matrix.iter().any(|row| row.len() != n) {
        return Err(format!("{name} must be a square {n}x{n} matrix"));
    }
    for (i, row) in matrix.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            if !value.is_finite() {
                return Err(format!("{name} contains a non-finite entry"));
            }
            let scale = value.abs().max(matrix[j][i].abs()).max(1.0);
            if (value - matrix[j][i]).abs() > 1e-12 * scale {
                return Err(format!("{name} must be symmetric"));
            }
        }
    }
    Ok(())
}

// Apply C <- T^T C T, where columns j,k of T are respectively
// e_j + cg*e_k and e_k + ca*e_j.
fn congruence_rotation(matrix: &mut [Vec<f64>], j: usize, k: usize, ca: f64, cg: f64) {
    let ajj = matrix[j][j];
    let akk = matrix[k][k];
    let ajk = matrix[j][k];

    // Update columns j and k, excluding the rotation indices.
    for (i, row) in matrix.iter_mut().enumerate() {
        if i == j || i == k {
            continue;
        }

        let aij = row[j];
        let aik = row[k];

        row[j] = aij + cg * aik;
        row[k] = aik + ca * aij;
    }

    // Restore symmetry using the updated columns.
    // Collect only the affected values to avoid borrowing conflicts.
    let column_j: Vec<f64> = matrix.iter().map(|row| row[j]).collect();
    let column_k: Vec<f64> = matrix.iter().map(|row| row[k]).collect();

    for (i, row) in matrix.iter_mut().enumerate() {
        if i == j {
            for (index, &value) in column_j.iter().enumerate() {
                if index != j && index != k {
                    row[index] = value;
                }
            }
        } else if i == k {
            for (index, &value) in column_k.iter().enumerate() {
                if index != j && index != k {
                    row[index] = value;
                }
            }
        }
    }

    // Update the diagonal entries.
    matrix[j][j] = ajj + 2.0 * cg * ajk + cg * cg * akk;
    matrix[k][k] = akk + 2.0 * ca * ajk + ca * ca * ajj;

    // Both off-diagonal elements vanish in exact arithmetic.
    matrix[j][k] = 0.0;
    matrix[k][j] = 0.0;
}

fn small_off_diagonals(a: &[Vec<f64>], b: &[Vec<f64>], threshold: f64) -> bool {
    let n = a.len();
    for j in 0..n {
        for k in j + 1..n {
            let aa = (a[j][k] / a[j][j]) * (a[j][k] / a[k][k]);
            let bb = (b[j][k] / b[j][j]) * (b[j][k] / b[k][k]);
            if aa >= threshold || bb >= threshold {
                return false;
            }
        }
    }
    true
}

pub fn generalized_jacobi(
    stiffness: &[Vec<f64>],
    mass: &[Vec<f64>],
    tolerance: f64,
    max_sweeps: usize,
) -> Result<JacobiResult, String> {
    let n = stiffness.len();
    if n == 0 {
        return Err("matrices must not be empty".to_string());
    }
    validate_matrix(stiffness, n, "stiffness")?;
    validate_matrix(mass, n, "mass")?;
    if !tolerance.is_finite() || tolerance <= 0.0 || tolerance >= 1.0 {
        return Err("tolerance must be finite and between zero and one".to_string());
    }
    if max_sweeps == 0 {
        return Err("max_sweeps must be positive".to_string());
    }

    let mut a = stiffness.to_vec();
    let mut b = mass.to_vec();
    let mut x = vec![vec![0.0; n]; n];
    for (i, row) in x.iter_mut().enumerate() {
        if a[i][i] <= 0.0 || b[i][i] <= 0.0 {
            return Err("Bathe generalized Jacobi requires positive-definite matrices".to_string());
        }
        row[i] = 1.0;
    }
    let mut previous: Vec<f64> = (0..n).map(|i| a[i][i] / b[i][i]).collect();
    let mut sweeps = 0;
    let mut converged = n == 1;

    if n > 1 {
        for sweep in 1..=max_sweeps {
            sweeps = sweep;
            let threshold = 0.01_f64.powf(2.0 * sweep as f64);
            for j in 0..n - 1 {
                for k in j + 1..n {
                    let eptola = (a[j][k] / a[j][j]) * (a[j][k] / a[k][k]);
                    let eptolb = (b[j][k] / b[j][j]) * (b[j][k] / b[k][k]);
                    if eptola < threshold && eptolb < threshold {
                        continue;
                    }
                    let akk = a[k][k] * b[j][k] - b[k][k] * a[j][k];
                    let ajj = a[j][j] * b[j][k] - b[j][j] * a[j][k];
                    let ab = a[j][j] * b[k][k] - a[k][k] * b[j][j];
                    let scale = a[k][k] * b[k][k];
                    let check = ((ab / scale).powi(2) + 4.0 * (akk / scale) * (ajj / scale)) / 4.0;
                    if !check.is_finite() || check < 0.0 {
                        return Err("invalid generalized Jacobi rotation".to_string());
                    }
                    let root = scale * check.sqrt();
                    let d1 = ab / 2.0 + root;
                    let d2 = ab / 2.0 - root;
                    let den = if d2.abs() > d1.abs() { d2 } else { d1 };
                    let (ca, cg) = if den == 0.0 {
                        (0.0, -a[j][k] / a[k][k])
                    } else {
                        (akk / den, -ajj / den)
                    };
                    if !ca.is_finite() || !cg.is_finite() {
                        return Err("non-finite generalized Jacobi rotation".to_string());
                    }
                    congruence_rotation(&mut a, j, k, ca, cg);
                    congruence_rotation(&mut b, j, k, ca, cg);
                    for row in &mut x {
                        let xj = row[j];
                        let xk = row[k];
                        row[j] = xj + cg * xk;
                        row[k] = xk + ca * xj;
                    }
                }
            }
            for i in 0..n {
                if !a[i][i].is_finite() || !b[i][i].is_finite() || a[i][i] <= 0.0 || b[i][i] <= 0.0
                {
                    return Err("non-positive diagonal during Jacobi iteration".to_string());
                }
            }
            let current: Vec<f64> = (0..n).map(|i| a[i][i] / b[i][i]).collect();
            let stable = current
                .iter()
                .zip(&previous)
                .all(|(new, old)| (new - old).abs() <= tolerance * old.abs());
            if stable && small_off_diagonals(&a, &b, tolerance * tolerance) {
                converged = true;
                break;
            }
            previous = current;
        }
    }
    if !converged {
        return Err(format!(
            "generalized Jacobi did not converge in {max_sweeps} sweeps"
        ));
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| (a[i][i] / b[i][i]).total_cmp(&(a[j][j] / b[j][j])));
    let eigenvalues = order.iter().map(|&i| a[i][i] / b[i][i]).collect();
    let mut eigenvectors = vec![vec![0.0; n]; n];
    for (column, &source) in order.iter().enumerate() {
        let norm = b[source][source].sqrt();
        for row in 0..n {
            eigenvectors[row][column] = x[row][source] / norm;
        }
    }
    Ok(JacobiResult {
        eigenvalues,
        eigenvectors,
        sweeps,
        converged,
    })
}

// Compute ||K*phi_i - lambda_i*M*phi_i||_2 for every eigenpair.
// Eigenvectors are stored columnwise.
pub fn eigenpair_residuals(
    stiffness: &[Vec<f64>],
    mass: &[Vec<f64>],
    result: &JacobiResult,
) -> Vec<f64> {
    let n = result.eigenvalues.len();
    let mut residuals = Vec::with_capacity(n);

    for (eigen_index, &lambda) in result.eigenvalues.iter().enumerate() {
        let mut squared_norm = 0.0;

        for (row_k, row_m) in stiffness.iter().zip(mass.iter()) {
            let k_phi: f64 = row_k
                .iter()
                .zip(result.eigenvectors.iter())
                .map(|(&value, eigenvector_row)| value * eigenvector_row[eigen_index])
                .sum();

            let m_phi: f64 = row_m
                .iter()
                .zip(result.eigenvectors.iter())
                .map(|(&value, eigenvector_row)| value * eigenvector_row[eigen_index])
                .sum();

            let residual = k_phi - lambda * m_phi;
            squared_norm += residual * residual;
        }

        residuals.push(squared_norm.sqrt());
    }

    residuals
}

// Compute Phi^T*M*Phi.
// For mass-normalized eigenvectors, the result should be the identity.
pub fn mass_gram_matrix(mass: &[Vec<f64>], result: &JacobiResult) -> Vec<Vec<f64>> {
    let n = result.eigenvalues.len();
    let mut gram = vec![vec![0.0; n]; n];

    for (i, gram_row) in gram.iter_mut().enumerate() {
        for (j, entry) in gram_row.iter_mut().enumerate() {
            let mut value = 0.0;

            for (row_index, mass_row) in mass.iter().enumerate() {
                for (col_index, &mass_value) in mass_row.iter().enumerate() {
                    value += result.eigenvectors[row_index][i]
                        * mass_value
                        * result.eigenvectors[col_index][j];
                }
            }

            *entry = value;
        }
    }

    gram
}

#[cfg(test)]
#[path = "tests/jacobi.rs"]
mod tests;
