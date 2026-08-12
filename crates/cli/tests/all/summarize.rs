use super::util::sightglass_cli;
use assert_cmd::prelude::*;
use predicates::prelude::*;

fn results_json() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/results.json")
}

/// Results for three benchmarks, each measured in its own two processes.
fn multi_engine_v38_json() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/multi_engine_v38.json")
}

/// summarize reads raw JSON and prints a human-readable table by default.
#[test]
fn summarize_human_readable() {
    sightglass_cli()
        .args(["summarize", "-f", results_json()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("compilation")
                .or(predicate::str::contains("Compilation"))
                .and(predicate::str::contains("cycles")),
        );
}

/// summarize --output-format json produces parseable JSON.
#[test]
fn summarize_output_format_json() {
    let assert = sightglass_cli()
        .args(["summarize", "-f", results_json(), "--output-format", "json"])
        .assert()
        .success();

    let stdout = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    assert!(
        serde_json::from_str::<serde_json::Value>(stdout).is_ok(),
        "stdout was not valid JSON: {stdout}"
    );
}

/// summarize --output-format csv produces a CSV header row.
#[test]
fn summarize_output_format_csv() {
    sightglass_cli()
        .args(["summarize", "-f", results_json(), "--output-format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mean"));
}

/// Just like `sightglass-cli benchmark`, `summarize` reports a synthetic "Sum
/// Total" benchmark that sums each sample's counts across all of the benchmarks
/// in its input.
#[test]
fn summarize_sum_total() {
    sightglass_cli()
        .args(["summarize", "-f", multi_engine_v38_json()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Sum Total"));
}

/// The "Sum Total" summary sums each sample's counts across the benchmarks, even
/// though every benchmark in our test data was measured in its own processes.
///
/// Every benchmark has the same number of samples here, so the total's mean must
/// be the sum of the benchmarks' means. Pooling the benchmarks' samples together
/// instead of summing them would give roughly their average, i.e. a third of
/// that.
#[test]
fn summarize_sum_total_sums_across_benchmarks() {
    let assert = sightglass_cli()
        .args([
            "summarize",
            "-f",
            multi_engine_v38_json(),
            "--output-format",
            "json",
        ])
        .assert()
        .success();

    let stdout = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    let summaries: Vec<serde_json::Value> =
        serde_json::from_str(stdout).unwrap_or_else(|e| panic!("stdout was not valid JSON: {e}"));

    for phase in ["Compilation", "Instantiation", "Execution"] {
        let summaries: Vec<_> = summaries.iter().filter(|s| s["phase"] == phase).collect();
        assert_eq!(summaries.len(), 4, "expected three benchmarks and a total");

        let total = summaries.iter().find(|s| s["wasm"] == "Sum Total").unwrap();
        let expected: f64 = summaries
            .iter()
            .filter(|s| s["wasm"] != "Sum Total")
            .map(|s| s["mean"].as_f64().unwrap())
            .sum();
        let actual = total["mean"].as_f64().unwrap();
        assert!(
            (actual - expected).abs() < expected * 1e-9,
            "{phase}: the total's mean is {actual}, but the benchmarks' means sum to {expected}"
        );
    }
}

/// summarize with a nonexistent input file fails.
#[test]
fn summarize_missing_file_fails() {
    sightglass_cli()
        .args(["summarize", "-f", "nonexistent_xyz.json"])
        .assert()
        .failure();
}
