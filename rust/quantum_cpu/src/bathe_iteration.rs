// Bathe, Finite Element Procedures (2nd ed.), Exercise 11.1, p. 910.
// Two inverse and two forward iterations from x = [1, 1, 1].
// Each iterate is scaled by its largest absolute component.
// Scaling does not change the generalized Rayleigh quotient.

pub const STIFFNESS: [[f64; 3]; 3] = [[6.0, -1.0, 0.0], [-1.0, 4.0, -1.0], [0.0, -1.0, 2.0]];

pub const MASS: [[f64; 3]; 3] = [[2.0, 0.0, 0.0], [0.0, 2.0, 1.0], [0.0, 1.0, 1.0]];

#[derive(Debug, Clone, Copy)]
pub struct IterationResult {
    pub first: [f64; 3],
    pub second: [f64; 3],
    pub rayleigh_quotient: f64,
}

fn mat_vec(matrix: &[[f64; 3]; 3], x: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| (0..3).map(|j| matrix[i][j] * x[j]).sum())
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}

fn solve_3x3(matrix: &[[f64; 3]; 3], rhs: [f64; 3]) -> [f64; 3] {
    let mut a = [[0.0; 4]; 3];
    for row in 0..3 {
        a[row][..3].copy_from_slice(&matrix[row]);
        a[row][3] = rhs[row];
    }
    for col in 0..3 {
        let pivot = (col..3)
            .max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))
            .expect("pivot row");
        assert!(a[pivot][col].abs() > 1e-14, "singular iteration matrix");
        a.swap(col, pivot);
        let diagonal = a[col][col];
        for value in a[col].iter_mut().skip(col) {
            *value /= diagonal;
        }
        for row in 0..3 {
            if row == col {
                continue;
            }
            let factor = a[row][col];
            let pivot_row = a[col];
            for (value, pivot_value) in a[row].iter_mut().zip(pivot_row).skip(col) {
                *value -= factor * pivot_value;
            }
        }
    }
    std::array::from_fn(|i| a[i][3])
}

fn normalize_max(mut x: [f64; 3]) -> [f64; 3] {
    let scale = x.iter().fold(0.0_f64, |acc, value| acc.max(value.abs()));
    assert!(scale > 0.0 && scale.is_finite());
    for value in &mut x {
        *value /= scale;
    }
    x
}

pub fn rayleigh_quotient(x: [f64; 3]) -> f64 {
    dot(x, mat_vec(&STIFFNESS, x)) / dot(x, mat_vec(&MASS, x))
}

pub fn inverse_two_iterations() -> IterationResult {
    let mut x = [1.0; 3];
    // K x_(k+1) = M x_k
    x = normalize_max(solve_3x3(&STIFFNESS, mat_vec(&MASS, x)));
    let first = x;
    x = normalize_max(solve_3x3(&STIFFNESS, mat_vec(&MASS, x)));
    IterationResult {
        first,
        second: x,
        rayleigh_quotient: rayleigh_quotient(x),
    }
}

pub fn forward_two_iterations() -> IterationResult {
    let mut x = [1.0; 3];
    // M x_(k+1) = K x_k
    x = normalize_max(solve_3x3(&MASS, mat_vec(&STIFFNESS, x)));
    let first = x;
    x = normalize_max(solve_3x3(&MASS, mat_vec(&STIFFNESS, x)));
    IterationResult {
        first,
        second: x,
        rayleigh_quotient: rayleigh_quotient(x),
    }
}

// Run any number of forward iterations from the book's initial vector x1.
// The returned vector after n steps is x_(n+1).
pub fn forward_iterations(count: usize) -> Vec<([f64; 3], f64)> {
    let mut x = [1.0; 3];
    let mut history = Vec::with_capacity(count);
    for _ in 0..count {
        x = normalize_max(solve_3x3(&MASS, mat_vec(&STIFFNESS, x)));
        history.push((x, rayleigh_quotient(x)));
    }
    history
}

#[cfg(test)]
#[path = "tests/bathe_iteration.rs"]
mod tests;
