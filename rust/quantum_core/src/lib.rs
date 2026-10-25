// external imports
use num_complex::Complex64;
use rand::{Rng, RngExt};

#[cfg(test)]
mod tests;

pub struct Qubit {
    pub amplitudes: [Complex64; 2],
}

impl Qubit {
    pub fn zero() -> Self {
        Self {
            amplitudes: [Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)],
        }
    }

    pub fn hadamard(&mut self) {
        let a = self.amplitudes[0];
        let b = self.amplitudes[1];

        let factor = 1.0 / 2.0_f64.sqrt();

        self.amplitudes[0] = (a + b) * factor;
        self.amplitudes[1] = (a - b) * factor;
    }

    pub fn probabilities(&self) -> [f64; 2] {
        [self.amplitudes[0].norm_sqr(), self.amplitudes[1].norm_sqr()]
    }
}

pub struct StateVector {
    pub num_qubits: usize,
    pub amplitudes: Vec<Complex64>,
}

impl StateVector {
    pub fn zero(num_qubits: usize) -> Self {
        assert!(num_qubits > 0, "num_qubits must be positive");

        let num_states = 1usize
            .checked_shl(num_qubits as u32)
            .expect("too many qubits");

        let mut amplitudes = vec![Complex64::new(0.0, 0.0); num_states];

        amplitudes[0] = Complex64::new(1.0, 0.0);

        Self {
            num_qubits,
            amplitudes,
        }
    }

    pub fn probabilities(&self) -> Vec<f64> {
        self.amplitudes
            .iter()
            .map(|amplitude| amplitude.norm_sqr())
            .collect()
    }

    pub fn hadamard(&mut self, target: usize) {
        assert!(target < self.num_qubits, "invalid target qubit");

        let stride = 1usize << target;
        let block_size = stride * 2;
        let factor = 1.0 / 2.0_f64.sqrt();

        for block_start in (0..self.amplitudes.len()).step_by(block_size) {
            for offset in 0..stride {
                let i0 = block_start + offset;
                let i1 = i0 + stride;

                let a = self.amplitudes[i0];
                let b = self.amplitudes[i1];

                self.amplitudes[i0] = (a + b) * factor;
                self.amplitudes[i1] = (a - b) * factor;
            }
        }
    }

    pub fn cnot(&mut self, control: usize, target: usize) {
        assert!(control < self.num_qubits, "invalid control qubit");
        assert!(target < self.num_qubits, "invalid target qubit");
        assert_ne!(control, target, "control and target must differ");

        let control_mask = 1usize << control;
        let target_mask = 1usize << target;

        for index in 0..self.amplitudes.len() {
            let control_is_one = (index & control_mask) != 0;
            let target_is_zero = (index & target_mask) == 0;

            if control_is_one && target_is_zero {
                let flipped_index = index | target_mask;
                self.amplitudes.swap(index, flipped_index);
            }
        }
    }

    pub fn pauli_x(&mut self, target: usize) {
        assert!(target < self.num_qubits, "invalid target qubit");

        let target_mask = 1usize << target;

        for index in 0..self.amplitudes.len() {
            if index & target_mask == 0 {
                let flipped_index = index | target_mask;
                self.amplitudes.swap(index, flipped_index);
            }
        }
    }

    pub fn pauli_z(&mut self, target: usize) {
        assert!(target < self.num_qubits, "invalid target qubit");

        let target_mask = 1usize << target;

        for (index, amplitude) in self.amplitudes.iter_mut().enumerate() {
            if index & target_mask != 0 {
                *amplitude = -*amplitude;
            }
        }
    }

    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let threshold: f64 = rng.random();
        let mut cumulative_probability = 0.0;

        for (index, amplitude) in self.amplitudes.iter().enumerate() {
            cumulative_probability += amplitude.norm_sqr();

            if threshold < cumulative_probability {
                return index;
            }
        }

        // Handle small floating-point normalization errors.
        self.amplitudes.len() - 1
    }

    pub fn rotation_y(&mut self, target: usize, theta: f64) {
        assert!(target < self.num_qubits, "invalid target qubit");

        let cosine = (theta / 2.0).cos();
        let sine = (theta / 2.0).sin();

        let stride = 1usize << target;
        let block_size = stride * 2;

        for block_start in (0..self.amplitudes.len()).step_by(block_size) {
            for offset in 0..stride {
                let i0 = block_start + offset;
                let i1 = i0 + stride;

                let a = self.amplitudes[i0];
                let b = self.amplitudes[i1];

                self.amplitudes[i0] = a * cosine - b * sine;
                self.amplitudes[i1] = a * sine + b * cosine;
            }
        }
    }

    pub fn expectation_z(&self, target: usize) -> f64 {
        assert!(target < self.num_qubits, "invalid target qubit");

        let target_mask = 1usize << target;

        self.amplitudes
            .iter()
            .enumerate()
            .map(|(index, amplitude)| {
                let probability = amplitude.norm_sqr();

                if index & target_mask == 0 {
                    probability
                } else {
                    -probability
                }
            })
            .sum()
    }

    pub fn expectation_zz(&self, qubit_a: usize, qubit_b: usize) -> f64 {
        assert!(qubit_a < self.num_qubits, "invalid first qubit");
        assert!(qubit_b < self.num_qubits, "invalid second qubit");
        assert_ne!(qubit_a, qubit_b, "qubits must differ");

        let mask_a = 1usize << qubit_a;
        let mask_b = 1usize << qubit_b;

        self.amplitudes
            .iter()
            .enumerate()
            .map(|(index, amplitude)| {
                let bit_a = index & mask_a != 0;
                let bit_b = index & mask_b != 0;

                let probability = amplitude.norm_sqr();

                if bit_a == bit_b {
                    probability
                } else {
                    -probability
                }
            })
            .sum()
    }

    pub fn expectation_xx(&self, qubit_a: usize, qubit_b: usize) -> f64 {
        assert!(qubit_a < self.num_qubits, "invalid first qubit");
        assert!(qubit_b < self.num_qubits, "invalid second qubit");
        assert_ne!(qubit_a, qubit_b, "qubits must differ");

        let flip_mask = (1usize << qubit_a) | (1usize << qubit_b);

        self.amplitudes
            .iter()
            .enumerate()
            .map(|(index, amplitude)| (amplitude.conj() * self.amplitudes[index ^ flip_mask]).re)
            .sum()
    }
}
