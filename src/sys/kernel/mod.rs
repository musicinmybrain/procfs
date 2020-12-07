//! Global kernel info / tuning miscellaneous stuff
//!
//! The files in this directory can be used to tune and monitor miscellaneous
//! and general things in the operation of the Linux kernel.

use std::cmp;
use std::str::FromStr;

use crate::{read_value, ProcResult};

pub mod keys;
pub mod random;

/// Represents a kernel version, in major.minor.release version.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u8) -> Version {
        Version { major, minor, patch }
    }

    /// Returns the kernel version of the currently running kernel.
    ///
    /// This is taken from `/proc/sys/kernel/osrelease`;
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/osrelease")
    }

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs::KernelVersion;
    /// let a = KernelVersion::from_str("3.16.0-6-amd64").unwrap();
    /// let b = KernelVersion::new(3, 16, 0);
    /// assert_eq!(a, b);
    ///
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let pos = s.find(|c: char| c != '.' && !c.is_ascii_digit());
        let kernel = if let Some(pos) = pos {
            let (s, _) = s.split_at(pos);
            s
        } else {
            s
        };
        let mut kernel_split = kernel.split('.');

        let major = kernel_split.next().ok_or("Missing major version component")?;
        let minor = kernel_split.next().ok_or("Missing minor version component")?;
        let patch = kernel_split.next().ok_or("Missing patch version component")?;

        let major = major.parse().map_err(|_| "Failed to parse major version")?;
        let minor = minor.parse().map_err(|_| "Failed to parse minor version")?;
        let patch = patch.parse().map_err(|_| "Failed to parse patch version")?;

        Ok(Version { major, minor, patch })
    }
}

impl FromStr for Version {
    type Err = &'static str;

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs::KernelVersion;
    /// let a: KernelVersion = "3.16.0-6-amd64".parse().unwrap();
    /// let b = KernelVersion::new(3, 16, 0);
    /// assert_eq!(a, b);
    ///
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::from_str(s)
    }
}

impl cmp::Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.major.cmp(&other.major) {
            cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                x => x,
            },
            x => x,
        }
    }
}

impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

/// Returns the maximum process ID number.
///
/// This is taken from `/proc/sys/kernel/pid_max`.
///
/// # Example
///
/// ```
/// let pid_max = procfs::sys::kernel::pid_max().unwrap();
///
/// let pid = 42; // e.g. from user input, CLI args, etc.
///
/// if pid > pid_max {
///     eprintln!("bad process ID: {}", pid)
/// } else {
///     println!("good process ID: {}", pid);
/// }
/// ```
pub fn pid_max() -> ProcResult<i32> {
    read_value("/proc/sys/kernel/pid_max")
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Represents the data from `/proc/sys/kernel/sem`
pub struct SemaphoreLimits {
    /// The maximum semaphores per semaphore set
    pub semmsl: u64,
    /// A system-wide limit on the number of semaphores in all semaphore sets
    pub semmns: u64,
    /// The maximum number of operations that may be specified in a semop(2) call
    pub semopm: u64,
    /// A system-wide limit on the maximum number of semaphore identifiers
    pub semmni: u64,
}

impl SemaphoreLimits {
    pub fn new() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/sem")
    }

    fn from_str(s: &str) -> Result<Self, &'static str> {
        let mut s = s.split_ascii_whitespace();

        let semmsl = s.next().ok_or("Missing SEMMSL")?;
        let semmns = s.next().ok_or("Missing SEMMNS")?;
        let semopm = s.next().ok_or("Missing SEMOPM")?;
        let semmni = s.next().ok_or("Missing SEMMNI")?;

        let semmsl = semmsl.parse().map_err(|_| "Failed to parse SEMMSL")?;
        let semmns = semmns.parse().map_err(|_| "Failed to parse SEMMNS")?;
        let semopm = semopm.parse().map_err(|_| "Failed to parse SEMOPM")?;
        let semmni = semmni.parse().map_err(|_| "Failed to parse SEMMNI")?;

        Ok(SemaphoreLimits {
            semmsl,
            semmns,
            semopm,
            semmni,
        })
    }
}

impl FromStr for SemaphoreLimits {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SemaphoreLimits::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let a = Version::from_str("3.16.0-6-amd64").unwrap();
        let b = Version::new(3, 16, 0);
        assert_eq!(a, b);

        let a = Version::from_str("3.16.0").unwrap();
        let b = Version::new(3, 16, 0);
        assert_eq!(a, b);

        let a = Version::from_str("3.16.0_1").unwrap();
        let b = Version::new(3, 16, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_current() {
        let _ = Version::current().unwrap();
    }

    #[test]
    fn test_pid_max() {
        assert!(pid_max().is_ok());
    }

    #[test]
    fn test_semaphore_limits() {
        // Note that the below string has tab characters in it. Make sure to not remove them.
        let a = SemaphoreLimits::from_str("32000	1024000000	500	32000").unwrap();
        let b = SemaphoreLimits {
            semmsl: 32_000,
            semmns: 1_024_000_000,
            semopm: 500,
            semmni: 32_000,
        };
        assert_eq!(a, b);

        let a = SemaphoreLimits::from_str("1");
        assert!(a.is_err() && a.err().unwrap() == "Missing SEMMNS");

        let a = SemaphoreLimits::from_str("1 string 500 3200");
        assert!(a.is_err() && a.err().unwrap() == "Failed to parse SEMMNS");
    }

    #[test]
    fn test_sem() {
        let _ = SemaphoreLimits::new().unwrap();
    }
}
