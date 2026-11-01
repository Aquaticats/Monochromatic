//! What:
//!  Target-independent Windows prefix forms driven through production normalization,
//!  counting and scanning.
//! Why:
//!  Windows's native prefix parser runs only on Windows,
//!  so these controls supply the prefix bytes it returns
//! and check that the scan skips exactly that prefix before treating every following component as a name.
//!
//! Each fixture prefix is the raw `Component::Prefix` byte prefix the standard library produces for its path:
//! `parse_prefix` in `library/std/src/sys/path/windows_prefix.rs` selects the prefix kind,
//! and `Prefix::len` in `library/std/src/path.rs` fixes how many leading path bytes it spans.
//! Both were read in the installed `nightly-2026-09-22` standard library source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const records = scanNormalized(normalize(path, "windows"), countPrefixParts(nativePrefix), rules);
//! // expect(records.display).toBe(expectedDisplay);
//! ```

/// Import the private canonical scan and the structured record it returns.
use super::{scan_normalized_records, PathScanRecords};
/// Import the production Windows normalization and prefix counting rather than a test-local copy.
use crate::path_name_bytes::{count_prefix_parts, normalize_bytes};
/// Import loaded matcher sets,
///  the type the scan borrows.
use crate::frx_load::LoadedRules;
/// Import the structured finding compared by every assertion.
use crate::ScanFinding;

/// What:
///  One native Windows prefix with a forbidden-name candidate and a clean control under the same prefix.
/// Why:
///  Every prefix form needs both a positive and a negative observation from one shared assertion.
///
/// `&'static [u8]` is a borrowed byte slice baked into the test binary.
/// Siblings are `Vec<u8>` (owned,
///  growable) and `[u8; N]` (fixed length);
/// fixture literals never change,
///  so a borrowed static slice needs no allocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrefixForm = { prefix: Uint8Array; hit: Uint8Array; hitDisplay: string; clean: Uint8Array; cleanDisplay: string };
/// ```
struct PrefixForm {
    /// Raw native prefix bytes that Windows's parser returns for both pathnames.
    prefix: &'static [u8],
    /// Native pathname whose first name after the prefix is forbidden.
    hit: &'static [u8],
    /// Expected display with that first name masked.
    hit_display: &'static str,
    /// Native pathname under the same prefix whose names are all clean.
    clean: &'static [u8],
    /// Expected display of the clean control,
    ///  with nothing masked.
    clean_display: &'static str,
}

/// What:
///  Load the single forbidden fixture name used by every form.
/// Why:
///  Rules compile in memory,
///  so no user-owned cache is read or written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadRules(): LoadedRules { return compileRules("VAULTTOKEN_LONG\n"); }
/// ```
fn load_rules() -> LoadedRules {
    return crate::frx_load::test_rules("VAULTTOKEN_LONG\n");
}

/// What:
///  Scan one native Windows pathname with its separately supplied native prefix bytes.
/// Why:
///  This is the production Windows composition,
///  `normalized_path` plus `prefix_parts`,
/// minus only the native `Component::Prefix` detection that Linux cannot run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanWindowsForm(path: Uint8Array, prefix: Uint8Array, loaded: LoadedRules): PathScanRecords {
///   if (!startsWith(path, prefix)) throw new Error("fixture prefix must be a byte prefix of its path");
///   return scanNormalized(normalize(path, true), countPrefixParts(prefix), loaded);
/// }
/// ```
fn scan_windows_form(path: &[u8], prefix: &[u8], loaded: &LoadedRules) -> PathScanRecords {
    // A native prefix always spans the leading bytes of its path; reject a fixture that violates that.
    assert!(path.starts_with(prefix), "fixture prefix must be a byte prefix of its path");
    // `true` selects Windows separator semantics; the Vec owns the normalized copy.
    let normalized: Vec<u8> = normalize_bytes(path, true);
    // usize counts prefix parts, matching the scan parameter; u32 or u64 would need a cast.
    let parts: usize = count_prefix_parts(prefix);
    // `&normalized` lends the bytes to the scan without giving up ownership.
    return scan_normalized_records(&normalized, parts, loaded);
}

/// What:
///  The single expected finding for a forbidden first name after a prefix.
/// Why:
///  Component numbering starts at 1 after the prefix,
///  and the fixture rule is unnamed rule 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstName = [{ kind: "name", component: 1, rule: "0" }];
/// ```
fn first_name() -> Vec<ScanFinding> {
    // String::from copies the literal into the owned String the finding stores.
    return vec![ScanFinding::Name { component: 1, rule: String::from("0") }];
}

/// What:
///  Assert one form's positive candidate is reported and masked and its clean control stays visible.
/// Why:
///  Every prefix form shares the same contract,
///  so one assertion keeps the forms comparable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertPrefixForm(form: PrefixForm): void { ... }
/// ```
fn assert_prefix_form(form: &PrefixForm) {
    let loaded: LoadedRules = load_rules();
    let hit: PathScanRecords = scan_windows_form(form.hit, form.prefix, &loaded);
    assert_eq!(hit.display, form.hit_display);
    assert_eq!(hit.findings, first_name());
    let clean: PathScanRecords = scan_windows_form(form.clean, form.prefix, &loaded);
    assert_eq!(clean.display, form.clean_display);
    assert!(clean.findings.is_empty());
}

/// A drive prefix `C:` is one part;
///  the first directory after it is name 1.
#[test]
fn drive_prefix_form() {
    // br"..." is a raw byte-string literal: backslashes are literal bytes, never escapes.
    assert_prefix_form(&PrefixForm {
        prefix: br"C:",
        hit: br"C:\VAULTTOKEN_LONG\clean.txt",
        hit_display: r"C\x3a/[REDACTED]/clean.txt",
        clean: br"C:\clean\clean.txt",
        clean_display: r"C\x3a/clean/clean.txt",
    });
}

/// A drive-relative name directly after `C:` is still name 1.
#[test]
fn drive_relative_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"C:",
        hit: br"C:VAULTTOKEN_LONG.txt",
        hit_display: r"C\x3a/[REDACTED]",
        clean: br"C:clean.txt",
        clean_display: r"C\x3a/clean.txt",
    });
}

/// A backslash UNC prefix skips its server and share.
#[test]
fn unc_backslash_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\server\share",
        hit: br"\\server\share\VAULTTOKEN_LONG\clean.txt",
        hit_display: "//server/share/[REDACTED]/clean.txt",
        clean: br"\\server\share\clean\clean.txt",
        clean_display: "//server/share/clean/clean.txt",
    });
}

/// A forward-slash UNC prefix skips its server and share.
#[test]
fn unc_slash_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: b"//server/share",
        hit: b"//server/share/VAULTTOKEN_LONG/clean.txt",
        hit_display: "//server/share/[REDACTED]/clean.txt",
        clean: b"//server/share/clean/clean.txt",
        clean_display: "//server/share/clean/clean.txt",
    });
}

/// A verbatim drive prefix skips `?` and the drive.
#[test]
fn verbatim_drive_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\?\C:",
        hit: br"\\?\C:\VAULTTOKEN_LONG\clean.txt",
        hit_display: r"//?/C\x3a/[REDACTED]/clean.txt",
        clean: br"\\?\C:\clean\clean.txt",
        clean_display: r"//?/C\x3a/clean/clean.txt",
    });
}

/// A verbatim UNC prefix skips `?`,
///  `UNC`,
///  server and share.
#[test]
fn verbatim_unc_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\?\UNC\server\share",
        hit: br"\\?\UNC\server\share\VAULTTOKEN_LONG\clean.txt",
        hit_display: "//?/UNC/server/share/[REDACTED]/clean.txt",
        clean: br"\\?\UNC\server\share\clean\clean.txt",
        clean_display: "//?/UNC/server/share/clean/clean.txt",
    });
}

/// A verbatim named prefix skips `?` and the name.
#[test]
fn verbatim_name_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\?\name",
        hit: br"\\?\name\VAULTTOKEN_LONG\clean.txt",
        hit_display: "//?/name/[REDACTED]/clean.txt",
        clean: br"\\?\name\clean\clean.txt",
        clean_display: "//?/name/clean/clean.txt",
    });
}

/// A device-namespace port prefix skips its `.` marker and device name,
///  and nothing after them.
#[test]
fn device_namespace_port_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\COM1",
        hit: br"\\.\COM1\VAULTTOKEN_LONG\clean.txt",
        hit_display: "//./COM1/[REDACTED]/clean.txt",
        clean: br"\\.\COM1\clean\clean.txt",
        clean_display: "//./COM1/clean/clean.txt",
    });
}

/// A device-namespace volume prefix skips its `.` marker and drive,
///  and nothing after them.
#[test]
fn device_namespace_drive_prefix_form() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\C:",
        hit: br"\\.\C:\VAULTTOKEN_LONG\clean.txt",
        hit_display: r"//./C\x3a/[REDACTED]/clean.txt",
        clean: br"\\.\C:\clean\clean.txt",
        clean_display: r"//./C\x3a/clean/clean.txt",
    });
}

/// A pipe name containing dots follows the `\\.\pipe` prefix and is name 1.
#[test]
fn device_namespace_pipe_name_with_dots() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\pipe",
        hit: br"\\.\pipe\VAULTTOKEN_LONG.with.dots",
        hit_display: "//./pipe/[REDACTED]",
        clean: br"\\.\pipe\name.with.dots",
        clean_display: "//./pipe/name.with.dots",
    });
}

/// A device name that itself contains dots is one prefix part.
#[test]
fn device_name_containing_dots() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\device.with.dots",
        hit: br"\\.\device.with.dots\VAULTTOKEN_LONG",
        hit_display: "//./device.with.dots/[REDACTED]",
        clean: br"\\.\device.with.dots\clean.txt",
        clean_display: "//./device.with.dots/clean.txt",
    });
}

/// Navigation markers after a prefix are neither names nor prefix parts.
#[test]
fn navigation_markers_after_prefix_are_not_names() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\COM1",
        hit: br"\\.\COM1\.\..\VAULTTOKEN_LONG\clean.txt",
        hit_display: "//./COM1/./../[REDACTED]/clean.txt",
        clean: br"\\.\COM1\.\..\clean.txt",
        clean_display: "//./COM1/./../clean.txt",
    });
    assert_prefix_form(&PrefixForm {
        prefix: br"\\server\share",
        hit: br"\\server\share\..\.\VAULTTOKEN_LONG",
        hit_display: "//server/share/.././[REDACTED]",
        clean: br"\\server\share\..\.\clean.txt",
        clean_display: "//server/share/.././clean.txt",
    });
}

/// Empty components after a prefix are neither names nor prefix parts.
#[test]
fn empty_components_after_prefix_are_not_names() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\COM1",
        hit: br"\\.\COM1\\\VAULTTOKEN_LONG\\clean.txt",
        hit_display: "//./COM1///[REDACTED]//clean.txt",
        clean: br"\\.\COM1\\\clean.txt",
        clean_display: "//./COM1///clean.txt",
    });
}

/// Mixed separators inside and after a prefix keep its boundary.
#[test]
fn mixed_separators_keep_prefix_boundary() {
    assert_prefix_form(&PrefixForm {
        prefix: br"//.\COM1",
        hit: br"//.\COM1/VAULTTOKEN_LONG\clean.txt",
        hit_display: "//./COM1/[REDACTED]/clean.txt",
        clean: br"//.\COM1/clean\clean.txt",
        clean_display: "//./COM1/clean/clean.txt",
    });
    // A slash inside `\\?/` cancels verbatim parsing, so std parses a UNC server `?` and share `C:`.
    assert_prefix_form(&PrefixForm {
        prefix: br"\\?/C:",
        hit: br"\\?/C:\VAULTTOKEN_LONG",
        hit_display: r"//?/C\x3a/[REDACTED]",
        clean: br"\\?/C:\clean.txt",
        clean_display: r"//?/C\x3a/clean.txt",
    });
}

/// Non-UTF-8 native bytes after a prefix are matched and masked as a whole name.
#[test]
fn non_utf8_name_after_prefix() {
    // `\xed\xa0\x80` is the WTF-8 spelling of an unpaired surrogate, which a Windows OsStr can hold.
    assert_prefix_form(&PrefixForm {
        prefix: b"\\\\.\\COM1",
        hit: b"\\\\.\\COM1\\\xed\xa0\x80VAULTTOKEN_LONG\\clean.txt",
        hit_display: "//./COM1/[REDACTED]/clean.txt",
        clean: b"\\\\.\\COM1\\\xed\xa0\x80\\clean.txt",
        clean_display: r"//./COM1/\xed\xa0\x80/clean.txt",
    });
}

/// A device name spelled `..` (`DeviceNS("..")`) is consumed as a prefix part.
#[test]
fn device_name_spelled_parent_marker_is_prefix() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\..",
        hit: br"\\.\..\VAULTTOKEN_LONG",
        hit_display: "//./../[REDACTED]",
        clean: br"\\.\..\clean.txt",
        clean_display: "//./../clean.txt",
    });
}

/// An empty device name (`DeviceNS("")`) leaves the four-byte prefix `\\.\` with the single part `.`.
#[test]
fn empty_device_name_prefix_is_one_part() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\.\",
        hit: br"\\.\\VAULTTOKEN_LONG",
        hit_display: "//.//[REDACTED]",
        clean: br"\\.\\clean.txt",
        clean_display: "//.//clean.txt",
    });
}

/// A UNC share spelled `..` (`UNC("server", "..")`) is consumed as a prefix part.
#[test]
fn unc_share_spelled_parent_marker_is_prefix() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\server\..",
        hit: br"\\server\..\VAULTTOKEN_LONG",
        hit_display: "//server/../[REDACTED]",
        clean: br"\\server\..\clean.txt",
        clean_display: "//server/../clean.txt",
    });
}

/// A verbatim prefix containing `/./` (`Verbatim("a/./b")`) counts the `.` run as a part,
///  because verbatim parsing
/// splits only at backslashes while the scan splits at both separators.
#[test]
fn verbatim_prefix_with_current_marker_run_is_prefix() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\?\a/./b",
        hit: br"\\?\a/./b\VAULTTOKEN_LONG",
        hit_display: "//?/a/./b/[REDACTED]",
        clean: br"\\?\a/./b\clean.txt",
        clean_display: "//?/a/./b/clean.txt",
    });
}

/// A verbatim UNC server spelled `..` (`VerbatimUNC("..", "share")`) is consumed as a prefix part.
#[test]
fn verbatim_unc_server_spelled_parent_marker_is_prefix() {
    assert_prefix_form(&PrefixForm {
        prefix: br"\\?\UNC\..\share",
        hit: br"\\?\UNC\..\share\VAULTTOKEN_LONG",
        hit_display: "//?/UNC/../share/[REDACTED]",
        clean: br"\\?\UNC\..\share\clean.txt",
        clean_display: "//?/UNC/../share/clean.txt",
    });
}

/// A prefix with no following component yields no name and an unmasked display.
#[test]
fn prefix_without_following_component() {
    let loaded: LoadedRules = load_rules();
    // Each pair is a whole native path and the prefix std returns for it.
    let cases: [(&[u8], &[u8], &str); 5] = [
        (br"\\.\COM1", br"\\.\COM1", "//./COM1"),
        (br"\\.\COM1\", br"\\.\COM1", "//./COM1/"),
        (br"\\?\UNC\server", br"\\?\UNC\server", "//?/UNC/server"),
        (br"\\server\share", br"\\server\share", "//server/share"),
        (br"C:", br"C:", r"C\x3a"),
    ];
    // `&cases` lends the fixed array to the loop; each item is a borrowed tuple.
    for (path, prefix, display) in &cases {
        let records: PathScanRecords = scan_windows_form(path, prefix, &loaded);
        assert_eq!(records.display, *display);
        assert!(records.findings.is_empty());
    }
}
