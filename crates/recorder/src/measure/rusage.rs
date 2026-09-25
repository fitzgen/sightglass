//! Reading this process's hardware performance counters on macOS.
//!
//! User space cannot read the PMU counters directly, but the kernel maintains
//! per-process instruction and cycle counts and exposes them via
//! `proc_pid_rusage`, which requires neither sudo nor enabling Apple's
//! developer tools (which some corporate laptops disallow... on that
//! corporation's developers' machines... yeah.)
//!
//! These counts cover every thread in the process, not just the one measuring.

/// Read this process's resource-usage counters, including retired instructions
/// and elapsed cycles.
pub fn read() -> libc::rusage_info_v4 {
    // SAFETY: `rusage_info_v4` is a plain-old-data struct for which an all-zero
    // bit pattern is valid, and we hand `proc_pid_rusage` a pointer to storage
    // large enough for the `RUSAGE_INFO_V4` flavor we request.
    let mut info: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    let ret = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            &mut info as *mut libc::rusage_info_v4 as *mut libc::rusage_info_t,
        )
    };
    assert_eq!(
        ret, 0,
        "proc_pid_rusage failed to read this process's performance counters"
    );
    info
}
