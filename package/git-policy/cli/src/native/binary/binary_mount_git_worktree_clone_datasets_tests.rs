//! What:
//!  The release's one executable as a person's machine meets it:
//!  it starts,
//!  the root modes answer their version,
//!  they refuse a caller that is not root,
//!  and the file links no OpenZFS CDDL library.
//! Why:
//!  The wrapper must start even when OpenZFS is absent
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility",
//!  and the acceptance test "Verify the release wrapper starts without OpenZFS and has no CDDL dependency"),
//!  and the binding joined the release for the ZFS route:
//!  a link to `libzfs` would make the wrapper unstartable on a machine without OpenZFS
//!  and would put a CDDL library into a release that is not under that licence.
//!  The root modes are modes of this one file now, so the arms start it under the
//!  installed ZED history event's name or through the words
//!  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`).
//!  The device-absent arm states its environment before it claims anything:
//!  on a host whose kernel has `/dev/zfs` it prints that it skipped,
//!  because a started program there proves nothing about a machine without the device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ldd(handler)).not.toMatch(/libzfs|libnvpair|libzpool/); expect(run(handler, ['--version'])).toBe(0);
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

/// The names of OpenZFS's CDDL libraries,
/// as a dynamic linker prints them with their `.so` suffix.
const CDDL_LIBRARY_STEMS: [&str; 7] = ["libzfs", "libzfs_core", "libzfsbootenv", "libnvpair", "libzpool", "libuutil", "libicp"];

/// The library every dynamically linked executable of this release carries,
/// which proves the linker answered rather than printing nothing.
const C_LIBRARY: &str = "libc.so";

/// What:
///  The CDDL libraries one dynamic-link listing names.
/// Why:
///   The listing is the linker's own answer,
///  so the check reads what the loader would load rather than the manifest's intent;
///  a name is matched with its `.so` suffix so a crate or symbol whose name merely contains one is not reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cddlLibraries = (listing) => listing.split('\n').filter((line) => stems.some((stem) => line.includes(`${stem}.so`)));
/// ```
fn cddl_libraries(listing: &str) -> Vec<String> {
    return listing
        .lines()
        .filter(|line| return CDDL_LIBRARY_STEMS.iter().any(|stem| return line.contains(&format!("{stem}.so"))))
        .map(String::from)
        .collect();
}

/// What:
///  What the dynamic linker reports for one executable.
/// Why:
///   `ldd` fails on a binary it cannot read,
///  so the caller checks the status before reading the listing as an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { status, stdout } = spawnSync('ldd', [binary]);
/// ```
fn dynamic_link_listing(binary: &str) -> (bool, String) {
    let output: std::process::Output = Command::new("ldd").arg(binary).output().expect("ldd runs: the dynamic linker's own answer is what this control reads");
    return (output.status.success(), format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())));
}

/// What:
///  What one executable prints on its standard output,
///  and the status it answered with.
/// Why:
///   A report a person pipes is read from standard output,
///  and the release workflow's smoke run pipes the handler's version through `grep`,
///  so a report on the other stream would leave that pipe empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { status, stdout } = spawnSync(binary, args, { stdio: ['ignore', 'pipe', 'ignore'] });
/// ```
fn reported(binary: &str, arguments: &[&str]) -> (Option<i32>, String) {
    let output: std::process::Output = Command::new(binary).args(arguments).output().expect("the release's executable runs");
    return (output.status.code(), String::from_utf8_lossy(output.stdout.as_slice()).into_owned());
}

/// What:
///  What one executable prints when it is started with the arguments given,
///  and the status it answered with.
/// Why:
///   The release ships programs a person starts,
///  so a start is judged by its status and its words rather than by the file's presence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { status, stdout, stderr } = spawnSync(binary, args);
/// ```
fn started(binary: &str, arguments: &[&str]) -> (Option<i32>, String) {
    let output: std::process::Output = Command::new(binary).args(arguments).output().expect("the release's executable runs");
    return (output.status.code(), format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())));
}

/// The one executable the release archive holds,
/// named as cargo built it for this test run.
const RELEASE_BINS: [&str; 1] = [env!("CARGO_BIN_EXE_monochromatic-git-policy-cli")];

/// Neither executable names a CDDL library,
/// and each listing is a real answer:
/// the linker succeeded and named the C library every one of them carries.
#[test]
fn neither_executable_links_an_openzfs_cddl_library() {
    for binary in RELEASE_BINS {
        let (succeeded, listing) = dynamic_link_listing(binary);
        assert!(succeeded, "ldd did not answer for {binary}: {listing}");
        assert!(listing.contains(C_LIBRARY), "the listing of {binary} names no {C_LIBRARY}, so it is no answer about its links: {listing}");
        let cddl: Vec<String> = cddl_libraries(listing.as_str());
        assert!(cddl.is_empty(), "{binary} links OpenZFS's CDDL libraries, which must never be part of this release: {cddl:#?}");
    }
}

/// The matcher reports a planted CDDL line and no line that merely mentions a name.
#[test]
fn the_matcher_names_a_planted_cddl_library_and_nothing_else() {
    let planted: &str = "linux-vdso.so.1 (0x00007ffe8b3fe000)\nlibzfs.so.4 => /lib/libzfs.so.4 (0x00007f2c4a200000)\nlibzpool.so.5 => /lib/libzpool.so.5 (0x00007f2c4a000000)\nlibc.so.6 => /lib/libc.so.6 (0x00007f2c49e00000)\n/lib64/ld-linux-x86-64.so.2 (0x00007f2c4a400000)\n";
    let found: Vec<String> = cddl_libraries(planted);
    assert_eq!(found.len(), 2, "both planted CDDL lines are reported: {found:#?}");
    assert!(found[0].contains("libzfs.so"), "{found:#?}");
    assert!(found[1].contains("libzpool.so"), "{found:#?}");
    let clean: &str = "linux-vdso.so.1 (0x00007ffe8b3fe000)\nlibc.so.6 => /lib/libc.so.6 (0x00007f2c49e00000)\nlibmonochromatic_zfs_request_to_openzfs_kernel_device.so => /app/lib.so (0x0)\n";
    assert_eq!(cddl_libraries(clean), Vec::<String>::new());
}

/// The root modes answer their version,
/// and on a machine whose kernel has no ZFS device every mode that will make a
/// request refuses at the start-time gate before the mode is entered.
///
/// The device is stated absent first:
/// on a host whose kernel loaded OpenZFS the gate may pass there and the mode
/// answers its own step instead,
/// which the dispatch controls read with a gate that answers without a kernel
/// (`handler_mode_dispatch_tests`).
#[test]
fn the_root_modes_answer_their_version_and_refuse_their_request_modes_behind_the_gate() {
    let link: PathBuf = crate::support_installed_name::installed_name("modes");
    let words: [&str; 2] = crate::support_installed_name::root_mode_words();
    let (version_status, version) = started(link.display().to_string().as_str(), &["--version"]);
    assert_eq!(version_status, Some(0), "{version}");
    let (report_status, report) = reported(link.display().to_string().as_str(), &["--version"]);
    assert_eq!(report_status, Some(0), "{report}");
    assert_eq!(report, format!("{} {} {}\n", words[0], words[1], env!("CARGO_PKG_VERSION")));
    assert!(version.contains(env!("CARGO_PKG_VERSION")), "{version}");
    if Path::new("/dev/zfs").exists() {
        eprintln!("SKIPPED: this machine's kernel has /dev/zfs, so a start here says nothing about a machine without the device; the gate image has none and runs this arm");
        return;
    }
    for mode in [words[0], "boot-reconcile", "shutdown-unmount"] {
        let arguments: Vec<&str> = if mode == words[0] { vec![words[0], words[1], "boot-reconcile"] } else { vec![words[0], words[1], mode] };
        let (status, said) = started(link.display().to_string().as_str(), arguments.as_slice());
        assert_eq!(status, Some(2), "{arguments:?} must refuse: {said}");
        assert!(said.contains("the start-time ZFS compatibility gate refused"), "{arguments:?} names the gate: {said}");
        assert!(said.contains("reads included"), "{arguments:?} names what stays disabled: {said}");
        assert!(said.contains("registrations and durable recovery state"), "{arguments:?} says nothing was changed: {said}");
        assert!(!said.contains("plan step"), "{arguments:?} never enters the mode: {said}");
    }
}

/// What:
///  A caller that is not root is refused by `start-probe` before any write,
///  with the refusal status and the sentence the adoption answers with.
/// Why:
///  Every write the probe makes is root's,
///  and before this refusal the command answered zero for a probe that reached
///  no step and told an account to make `/run` writable by root.
///  A host control is what shows it:
///  the arm states its environment before it claims anything,
///  so a run as root prints that it skipped and why.
#[test]
fn a_start_probe_that_is_not_root_is_refused_before_any_write() {
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let effective_user: u32 = unsafe { libc::geteuid() };
    if effective_user == 0 {
        eprintln!("SKIPPED: this run is root, so a caller that is not root cannot be shown here; the dispatch control drives the same refusal in process");
        return;
    }
    let link: PathBuf = crate::support_installed_name::installed_name("modes");
    let words: [&str; 2] = crate::support_installed_name::root_mode_words();
    let (status, said) = started(link.display().to_string().as_str(), &[words[0], words[1], "start-probe"]);
    assert_eq!(status, Some(2), "a caller that is not root must be refused: {said}");
    assert!(said.contains("the start-probe mode runs as root"), "the refusal names the mode and the rule: {said}");
    assert!(said.contains(&format!("effective user is {effective_user}")), "the refusal names the user the kernel reported: {said}");
    assert!(!said.contains("compatibility gate"), "the gate is not asked for a caller that is not root: {said}");
    assert!(!said.contains("make /run writable by root"), "the way forward is the caller's own: {said}");
}

/// The wrapper answers its own help namespace,
/// which is what the release workflow's smoke run checks.
#[test]
fn the_wrapper_answers_its_own_help() {
    let (help_status, help) = reported(RELEASE_BINS[0], &["monochromatic-git-policy-cli", "--help"]);
    assert_eq!(help_status, Some(0), "{help}");
    assert!(help.contains("Usage: git monochromatic-git-policy-cli"), "{help}");
}

/// The one executable starts on a machine whose kernel has no ZFS device,
/// and so do the root modes under the installed name.
///
/// The device is stated absent first:
/// on a host whose kernel loaded OpenZFS this arm prints that it skipped,
/// because a start there says nothing about a machine without the device.
#[test]
fn the_executable_starts_without_the_zfs_device() {
    if Path::new("/dev/zfs").exists() {
        eprintln!("SKIPPED: this machine's kernel has /dev/zfs, so a start here does not show a start without the device; the gate image has none and runs this arm");
        return;
    }
    let (wrapper_status, wrapper_said) = started(RELEASE_BINS[0], &["monochromatic-git-policy-cli", "--help"]);
    assert_eq!(wrapper_status, Some(0), "the wrapper starts without OpenZFS: {wrapper_said}");
    let link: PathBuf = crate::support_installed_name::installed_name("modes");
    let (modes_status, modes_said) = started(link.display().to_string().as_str(), &["--version"]);
    assert_eq!(modes_status, Some(0), "the root modes start without OpenZFS: {modes_said}");
}
