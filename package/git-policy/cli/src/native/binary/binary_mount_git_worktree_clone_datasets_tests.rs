//! What:
//!  The release's two executables as a person's machine meets them:
//!  both start,
//!  the handler refuses every mode whose step has not landed,
//!  and neither links one of OpenZFS's CDDL libraries.
//! Why:
//!  The wrapper must start even when OpenZFS is absent
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility",
//!  and the acceptance test "Verify the release wrapper starts without OpenZFS and has no CDDL dependency"),
//!  and the second executable is the reason the binding joined the release:
//!  a link to `libzfs` would make the wrapper unstartable on a machine without OpenZFS
//!  and would put a CDDL library into a release that is not under that licence.
//!  The device-absent arm states its environment before it claims anything:
//!  on a host whose kernel has `/dev/zfs` it prints that it skipped,
//!  because a started program there proves nothing about a machine without the device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ldd(handler)).not.toMatch(/libzfs|libnvpair|libzpool/); expect(run(handler, ['--version'])).toBe(0);
//! ```

use std::path::Path;
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

/// The two executables the release archive holds,
/// named as cargo built them for this test run.
const RELEASE_BINS: [&str; 2] = [
    env!("CARGO_BIN_EXE_monochromatic-git-policy-cli"),
    env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets"),
];

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

/// The handler answers its version and refuses every mode whose step has not landed,
/// naming the step and that nothing was touched.
#[test]
fn the_handler_answers_its_version_and_refuses_its_unbuilt_modes() {
    let (version_status, version) = started(RELEASE_BINS[1], &["--version"]);
    assert_eq!(version_status, Some(0), "{version}");
    assert!(version.contains("mount-git-worktree-clone-datasets"), "{version}");
    assert!(version.contains(env!("CARGO_PKG_VERSION")), "{version}");
    for (arguments, step) in [([].as_slice(), "plan step 8"), (["boot-reconcile"].as_slice(), "plan step 6"), (["shutdown-unmount"].as_slice(), "plan step 9")] {
        let (status, said) = started(RELEASE_BINS[1], arguments);
        assert_eq!(status, Some(2), "{arguments:?} must refuse: {said}");
        assert!(said.contains(step), "{arguments:?} names the step that lands it: {said}");
        assert!(said.contains("Nothing was opened"), "{arguments:?} says nothing was touched: {said}");
    }
}

/// The wrapper answers its own help namespace,
/// which is what the release workflow's smoke run checks.
#[test]
fn the_wrapper_answers_its_own_help() {
    let (help_status, help) = started(RELEASE_BINS[0], &["monochromatic-git-policy-cli", "--help"]);
    assert_eq!(help_status, Some(0), "{help}");
    assert!(help.contains("Usage: git monochromatic-git-policy-cli"), "{help}");
}

/// Both executables start on a machine whose kernel has no ZFS device.
///
/// The device is stated absent first:
/// on a host whose kernel loaded OpenZFS this arm prints that it skipped,
/// because a start there says nothing about a machine without the device.
#[test]
fn both_executables_start_without_the_zfs_device() {
    if Path::new("/dev/zfs").exists() {
        eprintln!("SKIPPED: this machine's kernel has /dev/zfs, so a start here does not show a start without the device; the gate image has none and runs this arm");
        return;
    }
    let (wrapper_status, wrapper_said) = started(RELEASE_BINS[0], &["monochromatic-git-policy-cli", "--help"]);
    assert_eq!(wrapper_status, Some(0), "the wrapper starts without OpenZFS: {wrapper_said}");
    let (handler_status, handler_said) = started(RELEASE_BINS[1], &["--version"]);
    assert_eq!(handler_status, Some(0), "the handler starts without OpenZFS: {handler_said}");
}
