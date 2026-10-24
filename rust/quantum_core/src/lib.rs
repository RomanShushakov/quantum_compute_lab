use num_complex::Complex64;

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

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-12;

    fn approx_equal(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn initial_state_is_zero() {
        let qubit = Qubit::zero();

        assert_eq!(qubit.probabilities(), [1.0, 0.0]);
    }

    #[test]
    fn hadamard_creates_equal_probabilities() {
        let mut qubit = Qubit::zero();

        qubit.hadamard();

        let probabilities = qubit.probabilities();

        assert!(approx_equal(probabilities[0], 0.5));
        assert!(approx_equal(probabilities[1], 0.5));
    }

    #[test]
    fn hadamard_twice_is_identity() {
        let mut qubit = Qubit::zero();

        qubit.hadamard();
        qubit.hadamard();

        assert!(approx_equal(qubit.amplitudes[0].re, 1.0));
        assert!(approx_equal(qubit.amplitudes[0].im, 0.0));
        assert!(approx_equal(qubit.amplitudes[1].re, 0.0));
        assert!(approx_equal(qubit.amplitudes[1].im, 0.0));
    }

    #[test]
    fn state_remains_normalized() {
        let mut qubit = Qubit::zero();

        for _ in 0..100 {
            qubit.hadamard();

            let total: f64 = qubit.probabilities().iter().sum();

            assert!(approx_equal(total, 1.0));
        }
    }
}
