//! What:
//!  The child a caller is waiting for,
//!  published the moment it starts:
//!  its number,
//!  and,
//! when the caller asks for one,
//! the descriptor that names it for good.
//! Why:
//!   A caller that must name the child it waits for cannot read the number before it exists,
//! and a number read later may belong to another process.
//!  The spot is shared rather than handed through a callback,
//! so the process code that starts the child owes nothing to the code that watches it,
//! and a watcher reads what it needs whenever it needs it.
//!  The descriptor is opened at the moment of the start,
//! while this process is the child's parent and its number cannot have been reused,
//! so a later stop reaches that child and never a process that took its number
//! (finding 1 of the second review).
//!  A caller that starts a child nobody must stop asks for no descriptor,
//! and every caller that does not get one keeps the answer that nothing is proven.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const told = ChildNumber.pinnedAtStart(); told.publish(child.pid); told.number(); told.descriptor();
//! ```

/// The child as the start found it.
#[derive(Debug)]
struct Started {
    /// The number `spawn` returned.
    number: i32,
    /// The descriptor the kernel made for it,
    /// or nothing when the caller wanted none or the kernel refused one.
    descriptor: Option<std::os::fd::OwnedFd>,
}

/// Where the child is kept until it is published,
/// and read after.
#[derive(Clone, Debug)]
pub struct ChildNumber {
    /// The child,
    /// once it has started.
    started: std::sync::Arc<std::sync::OnceLock<Started>>,
    /// Whether the descriptor is asked for when the child starts.
    pinned: bool,
}

impl ChildNumber {
    /// A child that has not started yet,
    /// kept by its number alone.
    pub fn not_started() -> Self {
        return Self { started: std::sync::Arc::default(), pinned: false };
    }

    /// A child that has not started yet,
    /// kept by its number and by the descriptor the kernel makes for it the moment it starts.
    pub fn not_started_pinned() -> Self {
        return Self { started: std::sync::Arc::default(), pinned: true };
    }

    /// What:
    ///  Keep `number` as the child's number,
    /// and the descriptor the kernel makes for it when one is asked for.
    /// Why:
    ///   The first number published is the child:
    /// one caller starts one child into one of these,
    /// and a second start publishes into a fresh one.
    ///  The descriptor is made here,
    /// at the start,
    /// because a number read at a stop is a number a reuse can be behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// told.publish(pid);
    /// ```
    pub fn publish(&self, number: i32) {
        let descriptor: Option<std::os::fd::OwnedFd> = if self.pinned {
            crate::member_signalled_through_its_pidfd::pidfd_of(number)
        } else {
            None
        };
        let _kept = self.started.set(Started { number, descriptor });
    }

    /// The child's number,
    /// once it has started.
    pub fn number(&self) -> Option<i32> {
        let started: &Started = self.started.get()?;
        return Some(started.number);
    }

    /// The descriptor that names the child for good,
    /// once it has started and the kernel made one.
    pub fn descriptor(&self) -> Option<&std::os::fd::OwnedFd> {
        let started: &Started = self.started.get()?;
        return started.descriptor.as_ref();
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "child_number_told_at_start_tests.rs"]
mod tests;
