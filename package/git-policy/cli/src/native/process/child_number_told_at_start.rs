//! What:
//!  The number of the child a caller is waiting for,
//!  published the moment the child starts.
//! Why:
//!   A caller that must name the child it waits for cannot read the number before it exists,
//! and the number is only trustworthy at the moment the start returned:
//! a number read later may belong to another process.
//!  The spot is shared rather than handed through a callback,
//! so the process code that starts the child owes nothing to the code that watches it,
//! and a watcher reads the number whenever it needs it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const told = new ChildNumber(); told.publish(child.pid); told.number();
//! ```

/// Where the number is kept until it is published,
/// and read after.
#[derive(Clone, Debug, Default)]
pub struct ChildNumber(std::sync::Arc<std::sync::OnceLock<i32>>);

impl ChildNumber {
    /// A child that has not started yet.
    pub fn not_started() -> Self {
        return Self::default();
    }

    /// What:
    ///  Keep `number` as the child's number.
    /// Why:
    ///   The first number published is the child:
    /// one caller starts one child into one of these,
    /// and a second start publishes into a fresh one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// told.publish(pid);
    /// ```
    pub fn publish(&self, number: i32) {
        let _kept = self.0.set(number);
    }

    /// The child's number,
    /// once it has started.
    pub fn number(&self) -> Option<i32> {
        return self.0.get().copied();
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "child_number_told_at_start_tests.rs"]
mod tests;
