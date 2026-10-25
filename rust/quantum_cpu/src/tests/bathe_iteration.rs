use super::*;

const EPSILON: f64 = 1e-9;

fn check_vector(actual: [f64; 3], expected: [f64; 3]) {
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < EPSILON, "{a} vs {e}");
    }
}

#[test]
fn inverse_iterations_match_exercise_11_1() {
    let result = inverse_two_iterations();
    check_vector(result.first, [1.0 / 3.0, 26.0 / 33.0, 1.0]);
    check_vector(result.second, [0.202752779248, 0.750661725781, 1.0]);
    assert!((result.rayleigh_quotient - 0.7262858816960368).abs() < EPSILON);
}

#[test]
fn forward_iterations_match_exercise_11_1() {
    let result = forward_two_iterations();
    check_vector(result.first, [1.0, 0.4, 0.0]);
    check_vector(result.second, [1.0, 5.0 / 14.0, -0.5]);
    assert!((result.rayleigh_quotient - 3.0973871733966747).abs() < EPSILON);
}

#[test]
fn rayleigh_quotient_is_scale_invariant() {
    let x = [0.2, 0.75, 1.0];
    let scaled = x.map(|value| value * 7.0);
    assert!((rayleigh_quotient(x) - rayleigh_quotient(scaled)).abs() < 1e-12);
}

#[test]
fn seven_forward_iterations_approach_largest_eigenvalue() {
    let history = forward_iterations(7);
    assert_eq!(history.len(), 7);
    assert!((history[1].1 - 3.0973871733966747).abs() < 1e-10);
    assert!((history[6].1 - 9.30786143327272).abs() < 1e-9);
    let exact = 9.310363643;
    assert!((history[6].1 - exact).abs() < 0.003);
}
