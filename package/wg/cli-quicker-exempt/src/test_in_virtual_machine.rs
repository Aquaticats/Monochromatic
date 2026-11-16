//! Whether the running test was started inside a machine the runner prepared.
//!
//! Every ignored test of this crate changes state that belongs to the machine it
//! runs on: the root cgroup tree, the BPF filesystem, `/run`, the system
//! manager, or the user manager of the account that started it.
//! They therefore refuse unless the machine runner prepared the guest,
//! and the refusal names the mise task that does that.
//!
//! The marker is a directory the runner creates as root,
//! so a test started on a host,
//! by hand or by `cargo test -- --ignored`,
//! finds no marker and stops before it changes anything.

/// The guest file whose presence marks a prepared machine.
pub const MARKER_PATH: &str = "/run/wg-quicker-test-in-a-virtual-machine";

/// The words of the refusal,
/// which name the test and the task that runs it in a machine.
pub fn refusal(name: &str, mise_task: &str, marker: &std::path::Path) -> String {
    return format!(
        "The {name} test changes the cgroup tree, the BPF filesystem, /run or a service manager of the machine it runs on, so it runs only inside a throwaway virtual machine. Run `mise run {mise_task}`, which creates that machine, copies the checkout into it, runs this test there and destroys it. This process found no {}, so it is not in such a machine.",
        marker.display()
    );
}

/// Refuses when nothing is at `marker`,
/// and otherwise lets the test go on.
pub fn require_marker(marker: &std::path::Path, name: &str, mise_task: &str) -> Result<(), String> {
    if marker.exists() {
        return Ok(());
    }
    return Err(refusal(name, mise_task, marker));
}

/// Refuses unless this process runs inside a prepared machine.
///
/// # Panics
///
/// When the marker of a prepared machine is not there.
pub fn require_test_in_virtual_machine(name: &str, mise_task: &str) {
    if let Err(words) = require_marker(std::path::Path::new(MARKER_PATH), name, mise_task) {
        panic!("{words}");
    }
}

/// The unit test of the marker's two answers.
#[cfg(test)]
#[path = "test_in_virtual_machine_tests.rs"]
mod tests;
