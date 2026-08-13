//! Synthetic measurements that sum a phase's counts across all benchmarks.

use sightglass_data::{Engine, Measurement, Phase};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// The benchmark name given to the synthetic measurements produced by
/// [`calculate`].
pub const SUM_TOTAL: &str = "Sum Total";

/// The `process` given to the synthetic measurements produced by [`calculate`].
///
/// A total sums samples recorded in many different processes, so it doesn't
/// belong to any one of them.
const SUM_TOTAL_PROCESS: u32 = 0;

/// The fields that measurements must agree on to be summed together: they are
/// only ever summed across benchmarks, never across architectures, engines,
/// phases, or events.
type GroupKey<'a> = (Cow<'a, str>, Engine<'a>, Phase, Cow<'a, str>);

/// One benchmark's `(process, iteration, count)` samples within a group, which
/// sort into a deterministic order.
type Samples = Vec<(u32, u32, u64)>;

/// Sum measurement `count`s across all benchmarks, producing new "Sum Total"
/// measurements.
///
/// That is, if the given measurements have 50 samples for each benchmark, then
/// this produces 50 "Sum Total" measurements, each of which is the sum of, e.g.,
/// instructions retired when compiling all benchmarks on their `i`th sample. A
/// benchmark's `i`th sample is its `i`th measurement in order of the process and
/// iteration it was recorded in; which samples get summed with which is
/// arbitrary, but deterministic, as any one sample is as good as any other.
///
/// This is useful because (a) it gives us a single "top line" number for the
/// whole benchmark run, and (b) on noisy machines with high variance it can
/// (often) be the case that any individual benchmark doesn't show statistically
/// significant differences between two engines but the sum totals *do* show
/// statistically significant differences due to effetively having a greater
/// number of samples.
///
/// Only the samples that *every* benchmark in a group has are summed. If one
/// benchmark has fewer samples than the others -- because it was measured with
/// different `--processes`/`--iterations-per-process`, for example -- then those
/// extra samples are left out, so that every total remains a total across all of
/// the benchmarks.
///
/// Any [`SUM_TOTAL`] measurements already present in `measurements` are ignored,
/// so that previously-computed totals are never counted a second time.
pub fn calculate<'a>(measurements: &[Measurement<'a>]) -> Vec<Measurement<'a>> {
    // Bucket every benchmark's samples by the fields we sum within: (arch,
    // engine, phase, event).
    //
    // Note that we cannot simply group the measurements by their `process` and
    // `iteration` and sum each group: benchmarking with `--processes N` gives
    // each (engine, benchmark) pair its own subprocesses, so no two benchmarks
    // ever share a process id and such groups would only ever hold a single
    // benchmark's sample.
    let mut groups: BTreeMap<GroupKey<'a>, BTreeMap<Cow<'a, str>, Samples>> = BTreeMap::new();
    for m in measurements.iter().filter(|m| m.wasm != SUM_TOTAL) {
        groups
            .entry((m.arch.clone(), m.engine.clone(), m.phase, m.event.clone()))
            .or_default()
            .entry(m.wasm.clone())
            .or_default()
            .push((m.process, m.iteration, m.count));
    }

    let mut totals = Vec::new();

    for ((arch, engine, phase, event), benchmarks) in groups {
        // Line each benchmark's samples up in a deterministic order, so that we
        // can sum the `i`th sample of each of them.
        let counts: Vec<Vec<u64>> = benchmarks
            .into_values()
            .map(|mut samples| {
                samples.sort_unstable();
                samples.into_iter().map(|(_, _, count)| count).collect()
            })
            .collect();

        // Only produce totals for the samples that every benchmark has.
        let samples = counts.iter().map(|c| c.len()).min().unwrap_or(0);

        totals.extend((0..samples).map(|i| Measurement {
            arch: arch.clone(),
            engine: engine.clone(),
            wasm: SUM_TOTAL.into(),
            process: SUM_TOTAL_PROCESS,
            iteration: i as u32,
            phase,
            event: event.clone(),
            count: counts.iter().map(|c| c[i]).sum(),
        }));
    }

    totals
}

/// Augment `measurements` with the [`calculate`]d "Sum Total" measurements, so
/// that analyses of these measurements report totals in addition to
/// per-benchmark results.
///
/// Any "Sum Total" measurements already in `measurements` are replaced, so this
/// is idempotent.
pub fn add<'a>(measurements: &mut Vec<Measurement<'a>>) {
    measurements.retain(|m| m.wasm != SUM_TOTAL);
    let totals = calculate(measurements);
    measurements.extend(totals);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measurement(
        wasm: &'static str,
        process: u32,
        iteration: u32,
        count: u64,
    ) -> Measurement<'static> {
        Measurement {
            arch: "x86_64".into(),
            engine: Engine {
                name: "e".into(),
                flags: None,
            },
            wasm: wasm.into(),
            process,
            iteration,
            phase: Phase::Execution,
            event: "cycles".into(),
            count,
        }
    }

    /// The `count`s of the given measurements, sorted.
    fn counts(measurements: &[Measurement<'_>]) -> Vec<u64> {
        let mut counts: Vec<_> = measurements.iter().map(|m| m.count).collect();
        counts.sort();
        counts
    }

    #[test]
    fn sum_totals_sums_counts_across_benchmarks() {
        // Two benchmarks measured over two iterations of one process, sharing
        // every other field.
        let measurements = vec![
            measurement("a.wasm", 7, 0, 10),
            measurement("b.wasm", 7, 0, 100),
            measurement("a.wasm", 7, 1, 20),
            measurement("b.wasm", 7, 1, 200),
        ];

        let totals = calculate(&measurements);
        assert_eq!(totals.len(), 2);
        for t in &totals {
            assert_eq!(t.wasm, SUM_TOTAL);
            assert_eq!(t.phase, Phase::Execution);
            assert_eq!(t.event, "cycles");
        }

        let mut by_iteration: Vec<_> = totals.iter().map(|t| (t.iteration, t.count)).collect();
        by_iteration.sort();
        assert_eq!(by_iteration, vec![(0, 110), (1, 220)]);
    }

    #[test]
    fn sums_across_benchmarks_measured_in_different_processes() {
        // Benchmarking with `--processes N` runs each benchmark in its own
        // subprocesses, so no two benchmarks share a process id. The totals must
        // still sum across the benchmarks.
        let measurements = vec![
            measurement("a.wasm", 1, 0, 10),
            measurement("a.wasm", 1, 1, 20),
            measurement("a.wasm", 2, 0, 30),
            measurement("a.wasm", 2, 1, 40),
            measurement("b.wasm", 3, 0, 100),
            measurement("b.wasm", 3, 1, 200),
            measurement("b.wasm", 4, 0, 300),
            measurement("b.wasm", 4, 1, 400),
        ];

        // One total per sample, rather than one per benchmark per sample...
        let totals = calculate(&measurements);
        assert_eq!(totals.len(), 4);

        // ...each of which sums one sample from each benchmark...
        assert_eq!(counts(&totals), vec![110, 220, 330, 440]);

        // ...and so, with every benchmark measured the same number of times,
        // every sample is accounted for exactly once.
        assert_eq!(
            totals.iter().map(|t| t.count).sum::<u64>(),
            measurements.iter().map(|m| m.count).sum::<u64>()
        );
    }

    #[test]
    fn only_sums_the_samples_that_every_benchmark_has() {
        // `b.wasm` was only measured once, so there is only one sample for which
        // we can total up every benchmark. Summing `a.wasm`'s extra samples on
        // their own would report totals that leave `b.wasm` out.
        let measurements = vec![
            measurement("a.wasm", 1, 0, 10),
            measurement("a.wasm", 1, 1, 20),
            measurement("a.wasm", 1, 2, 30),
            measurement("b.wasm", 2, 0, 100),
        ];

        let totals = calculate(&measurements);
        assert_eq!(counts(&totals), vec![110]);
    }

    #[test]
    fn measurements_are_only_summed_within_a_phase_and_event() {
        let compilation = |m: Measurement<'static>| Measurement {
            phase: Phase::Compilation,
            ..m
        };
        let nanoseconds = |m: Measurement<'static>| Measurement {
            event: "nanoseconds".into(),
            ..m
        };

        let measurements = vec![
            measurement("a.wasm", 1, 0, 10),
            measurement("b.wasm", 2, 0, 100),
            compilation(measurement("a.wasm", 1, 0, 1_000)),
            compilation(measurement("b.wasm", 2, 0, 10_000)),
            nanoseconds(measurement("a.wasm", 1, 0, 100_000)),
            nanoseconds(measurement("b.wasm", 2, 0, 1_000_000)),
        ];

        let totals = calculate(&measurements);
        assert_eq!(counts(&totals), vec![110, 11_000, 1_100_000]);
    }

    #[test]
    fn add_appends_totals_to_the_measurements() {
        let mut measurements = vec![
            measurement("a.wasm", 1, 0, 10),
            measurement("b.wasm", 2, 0, 100),
        ];
        add(&mut measurements);

        assert_eq!(measurements.len(), 3);
        let totals: Vec<_> = measurements
            .iter()
            .filter(|m| m.wasm == SUM_TOTAL)
            .cloned()
            .collect();
        assert_eq!(counts(&totals), vec![110]);
    }

    #[test]
    fn add_is_idempotent() {
        // Totals that are already present are recomputed rather than summed into
        // the new totals, so analyzing already-augmented measurements doesn't
        // double count.
        let mut measurements = vec![
            measurement("a.wasm", 1, 0, 10),
            measurement("b.wasm", 2, 0, 100),
        ];
        add(&mut measurements);
        let once = measurements.clone();
        add(&mut measurements);

        assert_eq!(measurements.len(), once.len());
        assert_eq!(counts(&measurements), counts(&once));
    }
}
