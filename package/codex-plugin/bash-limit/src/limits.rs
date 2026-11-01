//! Parent cgroup verification and once-per-session child creation.

/// What:
/// `use` imports standard-library modules;
/// `::` separates module names.
/// Why:
/// Read Linux's actual resource controls rather than trusting hook input.
/// TypeScript equivalent:
/// `import * as fs from 'node:fs';`
use std::fs;
/// Import the standard I/O error and result types.
use std::io;
/// Import borrowed paths,
/// unlike the owning `PathBuf` type.
use std::path::Path;
/// Import the subprocess builder used only for session initialization.
use std::process::Command;

/// What:
/// `pub` exposes a function to sibling modules;
/// `&Path` borrows an existing path.
/// Why:
/// Every command must observe the aggregate limits that actually apply.
/// `Result<()>` carries success without data or an I/O error.
/// TypeScript equivalent:
/// `export function verify(parent: string): void { /* may throw */ }`
pub fn verify(parent: &Path) -> io::Result<()> {
    // Iterate fixed memory, swap, and task controls without changing them.
    for (file, expected) in [
        ("memory.max", "4294967296"),
        ("memory.swap.max", "0"),
        ("pids.max", "2048"),
    ] {
        // What: `?` propagates a read error to the caller rather than supplying a default.
        // Why: Unreadable controls cannot establish the worker budget.
        // TypeScript equivalent: `const observed = readFileSync(join(parent, file), 'utf8');`
        let observed = fs::read_to_string(parent.join(file))?;
        if observed.trim() != expected {
            // What: `String` owns mutable text; `&str` is its read-only borrowed sibling.
            // Why: Include the affected control and required value in the diagnostic.
            // TypeScript equivalent: `let message = 'Worker control ';`
            let mut message = String::from("Worker control ");
            message.push_str(file);
            message.push_str(" must be ");
            message.push_str(expected);
            message.push_str("; restore the systemd slice configuration.");
            // What: `Err` returns the failure variant, unlike successful `Ok`.
            // Why: Changed limits must prevent the pending shell from executing.
            // TypeScript equivalent: `throw new Error(message);`
            return Err(io::Error::other(message));
        }
    }
    // Read both quota and scheduling period from the kernel's CPU control.
    let cpu = fs::read_to_string(parent.join("cpu.max"))?;
    // Advance through the two kernel fields without allocating a list.
    let mut fields = cpu.split_whitespace();
    // Require the quota field before parsing it.
    let quota = fields.next().ok_or_else(|| {
        // Return the missing-field error from this zero-argument closure (`||`).
        return io::Error::other("Worker cpu.max has no quota.");
    })?;
    // Require the scheduling period independently of the quota.
    let period = fields.next().ok_or_else(|| {
        // Preserve which CPU field is absent.
        return io::Error::other("Worker cpu.max has no period.");
    })?;
    // What: `u64` is an unsigned 64-bit integer; siblings include u32, i32, i64 and usize.
    // Why: Kernel quota values are nonnegative numbers, not memory indexes or signed values.
    // TypeScript equivalent: `const quotaValue = parsePositiveInteger(quota);`
    let quota_value = quota.parse::<u64>().map_err(io::Error::other)?;
    // Parse the period using the same checked integer conversion.
    let period_value = period.parse::<u64>().map_err(io::Error::other)?;
    if quota_value == 0 || period_value == 0 || fields.next().is_some() {
        // Keep explicit CPU increases possible, but reject unbounded or malformed controls.
        return Err(io::Error::other("Worker cpu.max must contain a finite positive quota and period."));
    }
    // What: `Ok(())` returns successful completion without a payload.
    // Why: Verification supplies no authority to change the limits.
    // TypeScript equivalent: `return;`
    return Ok(());
}

/// Initialize the user-owned slice and child before per-command hooks run.
/// TypeScript equivalent:
/// `export function initialize(parent: string): void { /* may throw */ }`
pub fn initialize(parent: &Path) -> io::Result<()> {
    if !parent.exists() {
        // Extract the slice name from the trusted configuration path.
        let slice = parent.file_name().ok_or_else(|| {
            // Refuse a path with no final component.
            return io::Error::other("The worker slice path has no unit name.");
        })?;
        // Borrow the unit name as text; refuse a path systemd cannot name.
        let slice_text = slice.to_str().ok_or_else(|| {
            // Report invalid path encoding without silently replacing its bytes.
            return io::Error::other("The worker slice name must be UTF-8.");
        })?;
        if !slice_text.ends_with(".slice") {
            // Restrict initialization to slice units, not arbitrary services.
            return Err(io::Error::other("The worker unit name must end in .slice."));
        }
        // Start only the configured slice; arguments are separate process values, not shell text.
        let status = Command::new("/usr/bin/systemctl")
            .args(["--user", "start", "--", slice_text])
            .status()?;
        if !status.success() {
            // Preserve the failed startup as a setup failure.
            return Err(io::Error::other("systemctl could not start the worker slice."));
        }
    }
    // Reuse verification before creating a writable child under this parent.
    verify(parent)?;
    // Repeated initialization leaves the shared cgroup and running workers intact.
    fs::create_dir_all(parent.join("codex"))?;
    // Complete setup without an output payload.
    return Ok(());
}
