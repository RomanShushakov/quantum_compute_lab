// Experiment 08, step 1: classical construction of a two-qubit Hamiltonian.
// Bathe Exercise 11.1: K phi = lambda M phi, with M positive definite.
// The qubit Hamiltonian is a classical matrix representation, not a QPU run.

use crate::bathe_iteration::STIFFNESS;

pub type Matrix4 = [[f64; 4]; 4];
pub type Matrix3 = [[f64; 3]; 3];

pub const PADDING_ENERGY: f64 = 12.0;

// For the lower 2x2 mass block B = [[2, 1], [1, 1]],
// B^(-1/2) = (1/sqrt(5)) * [[2, -1], [-1, 3]].
// The remaining mass entry is 2, so its inverse square root is 1/sqrt(2).
pub fn inverse_mass_sqrt() -> Matrix3 {
    let a = 1.0 / 2.0_f64.sqrt();
    let b = 1.0 / 5.0_f64.sqrt();
    [[a, 0.0, 0.0], [0.0, 2.0 * b, -b], [0.0, -b, 3.0 * b]]
}

pub fn transformed_hamiltonian() -> Matrix3 {
    let s = inverse_mass_sqrt();
    let mut h = [[0.0; 3]; 3];
    for (i, row) in h.iter_mut().enumerate() {
        for (j, value) in row.iter_mut().enumerate() {
            *value = (0..3)
                .flat_map(|a| (0..3).map(move |b| (a, b)))
                .map(|(a, b)| s[i][a] * STIFFNESS[a][b] * s[b][j])
                .sum();
        }
    }
    h
}

// |11> is a padding state with energy higher than the physical spectrum.
// Thus the minimum eigenvalue remains the Bathe ground eigenvalue.
pub fn padded_hamiltonian() -> Matrix4 {
    let h = transformed_hamiltonian();
    let mut result = [[0.0; 4]; 4];
    for i in 0..3 {
        result[i][..3].copy_from_slice(&h[i]);
    }
    result[3][3] = PADDING_ENERGY;
    result
}

// Real matrix representations of the 10 Pauli strings that can have
// nonzero coefficients in a real symmetric two-qubit Hamiltonian.
// YY is real because both factors contribute an imaginary unit.
pub const PAULI_LABELS: [&str; 10] = ["II", "IX", "IZ", "XI", "XX", "XZ", "YY", "ZI", "ZX", "ZZ"];

fn pauli_matrix(label: &str) -> Matrix4 {
    let identity = [[1.0, 0.0], [0.0, 1.0]];
    let x = [[0.0, 1.0], [1.0, 0.0]];
    let z = [[1.0, 0.0], [0.0, -1.0]];
    if label == "YY" {
        return [
            [0.0, 0.0, 0.0, -1.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0, 0.0],
        ];
    }
    let pauli = |symbol: u8| match symbol {
        b'I' => identity,
        b'X' => x,
        b'Z' => z,
        _ => panic!("unsupported real Pauli factor"),
    };
    let bytes = label.as_bytes();
    let a = pauli(bytes[0]);
    let b = pauli(bytes[1]);
    let mut result = [[0.0; 4]; 4];
    for (i, row) in result.iter_mut().enumerate() {
        for (j, value) in row.iter_mut().enumerate() {
            *value = a[i / 2][j / 2] * b[i % 2][j % 2];
        }
    }
    result
}

// c_P = Tr(P H) / 4 for the orthogonal two-qubit Pauli basis.
pub fn pauli_coefficients(hamiltonian: &Matrix4) -> [f64; 10] {
    std::array::from_fn(|index| {
        let p = pauli_matrix(PAULI_LABELS[index]);
        let mut trace = 0.0;
        for (i, row) in p.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                trace += value * hamiltonian[j][i];
            }
        }
        trace / 4.0
    })
}

pub fn reconstruct_from_paulis(coefficients: &[f64; 10]) -> Matrix4 {
    let mut result = [[0.0; 4]; 4];
    for (index, coefficient) in coefficients.iter().enumerate() {
        let p = pauli_matrix(PAULI_LABELS[index]);
        for (i, row) in result.iter_mut().enumerate() {
            for (j, value) in row.iter_mut().enumerate() {
                *value += coefficient * p[i][j];
            }
        }
    }
    result
}

#[cfg(test)]
#[path = "tests/bathe_hamiltonian.rs"]
mod tests;
