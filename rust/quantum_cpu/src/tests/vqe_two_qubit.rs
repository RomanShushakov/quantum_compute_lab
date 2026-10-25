use super::*;

const EPSILON: f64 = 1e-12;

#[test]
fn zero_angles_give_energy_three() {
    assert!((energy(0.0, 0.0) - 3.0).abs() < EPSILON);
}

#[test]
fn pi_rotation_gives_ground_state_energy() {
    assert!((energy(PI, 0.0) + 1.0).abs() < EPSILON);
}

#[test]
fn both_pi_rotations_give_ground_state_energy() {
    assert!((energy(PI, PI) + 1.0).abs() < EPSILON);
}

#[test]
fn optimizer_finds_ground_state_energy() {
    let result = optimize(20);

    assert!((result.energy + 1.0).abs() < EPSILON);
}
