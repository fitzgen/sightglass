use super::util::sightglass_cli;
use assert_cmd::prelude::*;
use predicates::prelude::*;

fn multi_engine_v38_json() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/multi_engine_v38.json")
}

fn multi_engine_v38_epoch_json() -> &'static str {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/multi_engine_v38_epoch.json"
    )
}

/// effect-size reads two-engine JSON and prints a human-readable comparison by default.
#[test]
fn effect_size_human_readable() {
    sightglass_cli()
        .args([
            "effect-size",
            "-f",
            multi_engine_v38_json(),
            "-f",
            multi_engine_v38_epoch_json(),
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("cycles").and(
                predicate::str::contains("Δ = ")
                    .or(predicate::str::contains("No difference in performance.")),
            ),
        );
}

/// Just like `sightglass-cli benchmark`, `effect-size` compares a synthetic
/// "Sum Total" benchmark that sums each sample's counts across all of the
/// benchmarks in its input.
#[test]
fn effect_size_sum_total() {
    sightglass_cli()
        .args([
            "effect-size",
            "-f",
            multi_engine_v38_json(),
            "-f",
            multi_engine_v38_epoch_json(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sum Total"));
}

/// Compare the two engines in our test data, returning the parsed effect sizes.
fn effect_sizes_json() -> Vec<serde_json::Value> {
    let assert = sightglass_cli()
        .args([
            "effect-size",
            "-f",
            multi_engine_v38_json(),
            "-f",
            multi_engine_v38_epoch_json(),
            "--output-format",
            "json",
        ])
        .assert()
        .success();

    let stdout = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    serde_json::from_str(stdout).unwrap_or_else(|e| panic!("stdout was not valid JSON: {e}"))
}

/// effect-size with --output-format json produces parseable JSON.
#[test]
fn effect_size_output_format_json() {
    let effects = effect_sizes_json();

    // Each of the three benchmarks in the input data is compared, as is our
    // synthetic "Sum Total", and none of them are compared more than once.
    let mut wasms: Vec<&str> = effects
        .iter()
        .filter(|e| e["phase"] == "Compilation")
        .map(|e| e["wasm"].as_str().unwrap())
        .collect();
    wasms.sort();
    assert_eq!(
        wasms,
        [
            "Sum Total",
            "benchmarks/bz2/benchmark.wasm",
            "benchmarks/pulldown-cmark/benchmark.wasm",
            "benchmarks/spidermonkey/benchmark.wasm",
        ]
    );
}

/// The "Sum Total" comparison sums each sample's counts across the benchmarks,
/// even though every benchmark in our test data was measured in its own
/// processes.
///
/// Every benchmark has the same number of samples here, so each engine's mean
/// total must be the sum of that engine's per-benchmark means. Pooling the
/// benchmarks' samples together instead of summing them would give roughly their
/// average, i.e. a third of that.
#[test]
fn effect_size_sum_total_sums_across_benchmarks() {
    let effects = effect_sizes_json();

    for phase in ["Compilation", "Instantiation", "Execution"] {
        let effects: Vec<_> = effects.iter().filter(|e| e["phase"] == phase).collect();
        assert_eq!(effects.len(), 4, "expected three benchmarks and a total");

        let total = effects.iter().find(|e| e["wasm"] == "Sum Total").unwrap();
        for mean in ["a_mean", "b_mean"] {
            let expected: f64 = effects
                .iter()
                .filter(|e| e["wasm"] != "Sum Total")
                .map(|e| e[mean].as_f64().unwrap())
                .sum();
            let actual = total[mean].as_f64().unwrap();
            assert!(
                (actual - expected).abs() < expected * 1e-9,
                "{phase} {mean}: the total is {actual}, but the benchmarks sum to {expected}"
            );
        }
    }
}

/// effect-size with --output-format csv produces a CSV header row.
#[test]
fn effect_size_output_format_csv() {
    sightglass_cli()
        .args([
            "effect-size",
            "-f",
            multi_engine_v38_json(),
            "-f",
            multi_engine_v38_epoch_json(),
            "--output-format",
            "csv",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("mean"));
}

/// effect-size with a nonexistent input file fails with an error.
#[test]
fn effect_size_missing_file_fails() {
    sightglass_cli()
        .args(["effect-size", "-f", "nonexistent_file_xyz.json"])
        .assert()
        .failure();
}

/// effect-size with a single-engine file (no comparison possible) exits non-zero.
#[test]
fn effect_size_single_engine_fails() {
    sightglass_cli()
        .args([
            "effect-size",
            "-f",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/results.json"),
        ])
        .assert()
        .failure();
}
