//! Reproducible, single-threaded CPU gate timings for Phase 2.
//!
//! Run in release mode. Each timed sample applies one gate to an already
//! allocated statevector; initialization and warm-up are excluded.
//! This remains an exploratory microbenchmark, not a statistical framework.

use quantum_core::StateVector;
use std::fs::{self, File};
use std::hint::black_box;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const QUBIT_COUNTS: [usize; 3] = [10, 15, 20];
const DEFAULT_REPETITIONS: usize = 50;
const DEFAULT_WARMUP: usize = 10;

struct Config {
    repetitions: usize,
    warmup: usize,
    output: Option<PathBuf>,
    gate: Option<String>,
    qubits: Option<usize>,
    target: Option<usize>,
    control: Option<usize>,
    implementation: String,
}

fn output_directory() -> PathBuf {
    // CARGO_MANIFEST_DIR points to rust/quantum_cpu, independent of the
    // shell's current working directory.
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Rust workspace directory")
        .parent()
        .expect("repository directory")
        .to_path_buf();
    repository_root.join("results/phase_2_cpu")
}

fn parse_args() -> Config {
    let mut config = Config {
        repetitions: DEFAULT_REPETITIONS,
        warmup: DEFAULT_WARMUP,
        output: None,
        gate: None,
        qubits: None,
        target: None,
        control: None,
        implementation: "baseline".into(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repetitions" => {
                config.repetitions = args
                    .next()
                    .expect("--repetitions requires a value")
                    .parse()
                    .expect("repetitions must be an integer");
            }
            "--warmup" => {
                config.warmup = args
                    .next()
                    .expect("--warmup requires a value")
                    .parse()
                    .expect("warmup must be an integer");
            }
            "--output" => {
                config.output = Some(PathBuf::from(
                    args.next().expect("--output requires a path"),
                ));
            }
            "--implementation" => {
                config.implementation = args.next().expect("--implementation requires a value")
            }
            "--gate" => config.gate = Some(args.next().expect("--gate requires a value")),
            "--qubits" => {
                config.qubits = Some(
                    args.next()
                        .expect("--qubits requires a value")
                        .parse()
                        .expect("qubits must be an integer"),
                )
            }
            "--target" => {
                config.target = Some(
                    args.next()
                        .expect("--target requires a value")
                        .parse()
                        .expect("target must be an integer"),
                )
            }
            "--control" => {
                config.control = Some(
                    args.next()
                        .expect("--control requires a value")
                        .parse()
                        .expect("control must be an integer"),
                )
            }
            _ => panic!(
                "usage: statevector_bench [--gate hadamard|rotation_y|cnot --qubits N --target N [--control N]] [--implementation baseline|chunks|both] [--repetitions N] [--warmup N] [--output PATH]"
            ),
        }
    }
    assert!(config.repetitions > 0, "repetitions must be positive");
    assert!(config.warmup > 0, "warmup must be positive");
    validate_config(&config);
    config
}

fn validate_config(config: &Config) {
    assert!(
        matches!(
            config.implementation.as_str(),
            "baseline" | "chunks" | "both"
        ),
        "implementation must be baseline, chunks, or both"
    );
    if config.implementation != "baseline" {
        assert!(
            config.gate.as_deref() == Some("hadamard"),
            "chunks and both require --gate hadamard"
        );
    }
    if let Some(gate) = &config.gate {
        assert!(
            matches!(gate.as_str(), "hadamard" | "rotation_y" | "cnot"),
            "unknown gate: {gate}"
        );
        assert!(
            config.qubits.is_some() && config.target.is_some(),
            "--gate requires --qubits and --target"
        );
    } else {
        assert!(
            config.qubits.is_none() && config.target.is_none() && config.control.is_none(),
            "--qubits, --target, and --control require --gate"
        );
    }
    if let (Some(gate), Some(qubits), Some(target)) = (&config.gate, config.qubits, config.target) {
        assert!(
            (1..=20).contains(&qubits),
            "qubits must be between 1 and 20 for this benchmark"
        );
        assert!(target < qubits, "target must be less than qubits");
        if gate == "cnot" {
            let control = config.control.unwrap_or(0);
            assert!(
                control < qubits && control != target,
                "CNOT control must be valid and different from target"
            );
        } else {
            assert!(config.control.is_none(), "--control is only valid for cnot");
        }
    }
}

// Automatically generated output paths use a high-resolution timestamp and an
// exclusive-create retry loop. Explicit --output paths never get overwritten.
fn open_output(explicit: Option<&Path>) -> io::Result<(PathBuf, File)> {
    if let Some(path) = explicit {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::options().write(true).create_new(true).open(path)?;
        return Ok((path.to_path_buf(), file));
    }
    let directory = output_directory();
    fs::create_dir_all(&directory)?;
    let epoch_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();
    for suffix in 0..1000 {
        let path = directory.join(format!("statevector_{epoch_nanos}_{suffix}.csv"));
        match File::options().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to allocate unique benchmark filename",
    ))
}

fn as_micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.0
}

fn percentile_nearest_rank(sorted: &[Duration], numerator: usize, denominator: usize) -> Duration {
    // Nearest-rank percentile, using a zero-based index.
    let rank = (sorted.len() * numerator).div_ceil(denominator);
    sorted[rank.saturating_sub(1)]
}

fn measure(
    gate: (&str, &str),
    num_qubits: usize,
    target: usize,
    control: Option<usize>,
    config: &Config,
    apply_gate: impl Fn(&mut StateVector, usize),
    writer: &mut impl Write,
) -> io::Result<()> {
    let (name, implementation) = gate;
    let mut state = StateVector::zero(num_qubits);
    for _ in 0..config.warmup {
        apply_gate(black_box(&mut state), black_box(target));
    }

    let mut samples = Vec::with_capacity(config.repetitions);
    for _ in 0..config.repetitions {
        let start = Instant::now();
        apply_gate(black_box(&mut state), black_box(target));
        samples.push(start.elapsed());
    }
    black_box(&state);

    samples.sort_unstable();
    let min = samples[0];
    let median = samples[samples.len() / 2];
    let p95 = percentile_nearest_rank(&samples, 95, 100);
    let max = samples[samples.len() - 1];
    let control_str = control.map_or_else(String::new, |value| value.to_string());
    let row = format!(
        "{name},{num_qubits},{target},{control_str},{},{},{:.3},{:.3},{:.3},{:.3},{implementation}",
        config.repetitions,
        config.warmup,
        as_micros(min),
        as_micros(median),
        as_micros(p95),
        as_micros(max),
    );
    println!("{row}");
    writeln!(writer, "{row}")
}

fn run(config: &Config, writer: &mut impl Write) -> io::Result<()> {
    let header = "gate,qubits,target,control,repetitions,warmup,min_us,median_us,p95_us,max_us,implementation";
    println!("{header}");
    writeln!(writer, "{header}")?;

    if let (Some(gate), Some(qubits), Some(target)) = (&config.gate, config.qubits, config.target) {
        match gate.as_str() {
            "hadamard" => {
                if config.implementation != "chunks" {
                    measure(
                        ("hadamard", "baseline"),
                        qubits,
                        target,
                        None,
                        config,
                        |state, q| state.hadamard(q),
                        writer,
                    )?;
                }
                if config.implementation != "baseline" {
                    measure(
                        ("hadamard", "chunks"),
                        qubits,
                        target,
                        None,
                        config,
                        |state, q| state.hadamard_chunks(q),
                        writer,
                    )?;
                }
            }
            "rotation_y" => measure(
                ("rotation_y", "baseline"),
                qubits,
                target,
                None,
                config,
                |state, q| state.rotation_y(q, 0.37),
                writer,
            )?,
            "cnot" => {
                let control = config.control.unwrap_or(0);
                measure(
                    ("cnot", "baseline"),
                    qubits,
                    target,
                    Some(control),
                    config,
                    |state, q| state.cnot(control, q),
                    writer,
                )?;
            }
            _ => unreachable!("validated gate"),
        }
        return Ok(());
    }

    for num_qubits in QUBIT_COUNTS {
        // Three distinct positions expose different amplitude-pair strides.
        for target in [0, num_qubits / 2, num_qubits - 1] {
            measure(
                ("hadamard", "baseline"),
                num_qubits,
                target,
                None,
                config,
                |state, q| {
                    state.hadamard(q);
                },
                writer,
            )?;
            measure(
                ("rotation_y", "baseline"),
                num_qubits,
                target,
                None,
                config,
                |state, q| {
                    // Fixed angle: the same amount of work is performed per call.
                    state.rotation_y(q, 0.37);
                },
                writer,
            )?;
        }
        measure(
            ("cnot", "baseline"),
            num_qubits,
            num_qubits - 1,
            Some(0),
            config,
            |state, q| {
                state.cnot(0, q);
            },
            writer,
        )?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let config = parse_args();
    let (output, mut file) = open_output(config.output.as_deref())?;
    eprintln!(
        "CPU baseline: release mode recommended; single-threaded; allocation excluded; warmup={} repetitions={}",
        config.warmup, config.repetitions
    );
    run(&config, &mut file)?;
    eprintln!("CSV saved: {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_nearest_rank_works_for_small_samples() {
        let values = [1, 2, 3, 4, 5].map(Duration::from_micros);
        assert_eq!(
            percentile_nearest_rank(&values, 95, 100),
            Duration::from_micros(5)
        );
        assert_eq!(
            percentile_nearest_rank(&values, 50, 100),
            Duration::from_micros(3)
        );
    }

    #[test]
    fn targeted_config_validation() {
        let mut config = Config {
            repetitions: 10,
            warmup: 2,
            output: None,
            gate: Some("hadamard".into()),
            qubits: Some(20),
            target: Some(19),
            control: None,
            implementation: "baseline".into(),
        };
        validate_config(&config);
        config.gate = Some("cnot".into());
        config.control = Some(0);
        validate_config(&config);
    }

    #[test]
    #[should_panic(expected = "different from target")]
    fn cnot_rejects_same_control_and_target() {
        let config = Config {
            repetitions: 10,
            warmup: 2,
            output: None,
            gate: Some("cnot".into()),
            qubits: Some(20),
            target: Some(0),
            control: Some(0),
            implementation: "baseline".into(),
        };
        validate_config(&config);
    }

    #[test]
    fn alternative_requires_hadamard() {
        let mut config = Config {
            repetitions: 10,
            warmup: 2,
            output: None,
            gate: Some("hadamard".into()),
            qubits: Some(20),
            target: Some(0),
            control: None,
            implementation: "both".into(),
        };
        validate_config(&config);
        config.implementation = "chunks".into();
        validate_config(&config);
    }

    #[test]
    fn target_positions_are_distinct() {
        for qubits in QUBIT_COUNTS {
            let positions = [0, qubits / 2, qubits - 1];
            assert!(positions[0] < positions[1] && positions[1] < positions[2]);
        }
    }
}
