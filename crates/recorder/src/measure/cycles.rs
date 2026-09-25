//! Measure the number of CPU cycles elapsed.

use super::{Measure, Measurements};
use sightglass_data::Phase;

/// Read a free-running counter from user space (e.g. `RDTSC`).
///
/// These generally tick at a fixed rate rather than at the core's clock speed,
/// so they are cheap and precise but measure elapsed time, not work done.
#[cfg(not(target_os = "macos"))]
mod imp {
    use lazy_static::lazy_static;
    use precision::{Config, Precision, Timestamp};

    lazy_static! {
        static ref PRECISION: Precision = {
            // NB: Disable wall-time measurement, as that requires calibrating the
            // CPU frequency, which adds ~5 seconds on start up time per
            // benchmarking process, and we only care about ticks anyways.
            let config = Config::default().wall_time(false);
            Precision::new(config).unwrap()
        };
    }

    pub type State = Option<Timestamp>;

    pub fn new() -> State {
        None
    }

    pub fn start(state: &mut State) {
        *state = Some(PRECISION.now());
    }

    pub fn end(state: &mut State) -> u64 {
        // Deref the lazy-static just once.
        let precision = &*PRECISION;

        let end = precision.now();
        let start = state.take().expect("must call start before end");
        (end - start).ticks()
    }
}

/// Ask the kernel for this process's cycle count from the hardware performance
/// monitor.
///
/// On aarch64, `cntvct_el0`, is a 24 MHz system timer on Apple silicon, so it
/// reports wall time rather than cycles.
///
/// The kernel's count is process-wide, so it includes parallel compilation
/// threads, and each read costs a syscall (~2 microseconds), which coarsens
/// very short phases.
#[cfg(target_os = "macos")]
mod imp {
    use super::super::rusage;

    pub type State = u64;

    pub fn new() -> State {
        0
    }

    pub fn start(state: &mut State) {
        *state = rusage::read().ri_cycles;
    }

    pub fn end(state: &mut State) -> u64 {
        rusage::read().ri_cycles.wrapping_sub(*state)
    }
}

pub struct CycleMeasure(imp::State);

impl Default for CycleMeasure {
    fn default() -> Self {
        Self::new()
    }
}

impl CycleMeasure {
    pub fn new() -> Self {
        Self(imp::new())
    }
}

impl Measure for CycleMeasure {
    fn start(&mut self, _phase: Phase) {
        imp::start(&mut self.0);
    }

    fn end(&mut self, phase: Phase, measurements: &mut Measurements) {
        let elapsed = imp::end(&mut self.0);
        measurements.add(phase, "cycles".into(), elapsed);
    }
}
