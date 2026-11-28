//! What:
//! The controls of the check that a declared bin name fits the longest name the exposure folder's file
//! system holds a link's temporary name under.
//! Why:
//! The check exists because the link write failed on a name no plan wrote,
//! so one control builds that failure on a real file system
//! and the others hold what the refusal says and when it is quiet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-bin-names-the-exposure-folder-can-hold-tests.ts
//! test('a plan whose bins fit the folder is accepted');
//! test('a bin the folder cannot hold is refused, naming the row, the bin and the limit');
//! test('the question is put to the nearest directory that exists');
//! test('a name at the limit is written, and one byte more fails with the system\'s own error');
//! ```

/// Imports `fs` from `std`,
/// for the link a write leaves and for the scratch directory's own facts.
use std::fs;
/// Imports the owned path type these controls build their plans and scopes from.
use std::path::PathBuf;

/// Imports `statvfs` from `rustix`,
/// the same call the check asks,
/// so a control reads the limit from the file system rather than from a number written here.
use rustix::fs::statvfs;

/// Imports what a hidden name records about the process writing it,
/// which is what the temporary name of a link carries.
use super::archive_staging::Owner;
/// Imports the link write every installer shares,
/// whose failure is the one the check comes before.
use super::install_support::atomic_symlink;
/// Imports the name a link is written under,
/// built for a process of the control's choosing.
use super::install_support::the_temporary_link_of;
/// Imports the plan reader and the scope builder.
use super::plan::parse_plan;
use super::scope::{
    EnvMap,
    repo_scope,
};
/// Imports the check under test,
/// and the reserve it subtracts from the limit,
/// which a control derives its expected numbers from rather than writing them down.
use super::the_bin_names_the_exposure_folder_can_hold::{
    refuse_bin_names_the_exposure_folder_cannot_hold, the_bytes_a_temporary_name_adds,
};
/// Imports the disposable directory a control works in.
use crate::test_scratch::scratch;

/// Imports the plant that makes the file system's answer a number of the control's own.
use super::planting_the_folders_name_limit;

/// What:
/// A plan naming one tool whose single bin is `bin`.
/// Why:
/// The check reads declared bin names and nothing else,
/// so the smallest plan a person could write holds all it asks about,
/// and a name of any length is written the way a plan would carry it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aPlanDeclaringTheBin(bin: string): string {
///   return `{ "schemaVersion": 1, "tools": { "fixtool": { "request": "github:example/fixtool@1.0.0", "route": "archive", "bins": ["${bin}"] } } }`;
/// }
/// ```
fn a_plan_declaring_the_bin(bin: &str) -> String {
    return format!(
        "{{ \"schemaVersion\": 1, \"tools\": {{ \"fixtool\": {{ \"request\": \"github:example/fixtool@1.0.0\", \"route\": \"archive\", \"bins\": [\"{bin}\"] }} }} }}"
    );
}

/// A plan and a scope whose exposure folder sits in a directory of this control's own.
fn a_plan_and_a_scope(
    bin: &str,
    directory: &std::path::Path,
) -> (super::plan::Plan, super::scope::Scope) {
    let plan_text: String = a_plan_declaring_the_bin(bin);
    let plan = parse_plan(&plan_text, "the plan a control wrote").expect("a plan the reader takes");
    let env = EnvMap::from_pairs(&[
        ("HOME", "/a/throwaway/home"),
        ("PATH", "/usr/bin:/bin"),
    ]);
    let scope = repo_scope(directory, &env).expect("a scope in the scratch directory");
    return (plan, scope);
}

/// The longest name the file system holding `directory` holds,
/// as the check asks it.
fn the_longest_name_there(directory: &std::path::Path) -> u64 {
    return statvfs(directory)
        .expect("the file system answers for the scratch directory")
        .f_namemax;
}

/// A bin name of `bytes` bytes,
/// made of one letter so it is a plain file name.
fn a_bin_name_of(bytes: usize) -> String {
    return "a".repeat(bytes);
}

#[test]
fn a_plan_whose_bins_fit_the_exposure_folder_is_accepted() {
    let directory = scratch();
    let (plan, scope) = a_plan_and_a_scope("fixtool", directory.path());
    refuse_bin_names_the_exposure_folder_cannot_hold(&plan, &scope)
        .expect("a short bin name fits the folder's file system");
}

/// What:
/// A bin the folder cannot hold a link's temporary name for is refused,
/// and the refusal names the row,
/// the bin,
/// the limit the file system answered and the longest bin that fits.
/// Why:
/// The failure the check replaces named the temporary link,
/// which is a name no person wrote and no plan holds,
/// so the refusal has to name the things a person can change.
#[test]
fn a_bin_the_folder_cannot_hold_is_refused_naming_the_row_the_bin_and_the_limit() {
    let directory = scratch();
    let longest: u64 = the_longest_name_there(directory.path());
    // The exposure folder does not exist yet, as in a scope that has never run `install`.
    let refusal_length: usize = usize::try_from(longest).expect("a name limit this machine holds");
    let bin: String = a_bin_name_of(refusal_length);
    let (plan, scope) = a_plan_and_a_scope(&bin, directory.path());
    assert!(
        !scope.exposure_dir.exists(),
        "the question is asked before the folder exists: {}",
        scope.exposure_dir.display()
    );
    let error = refuse_bin_names_the_exposure_folder_cannot_hold(&plan, &scope)
        .expect_err("a bin the file system cannot hold a link's temporary name for is refused");
    assert!(
        error.message.contains(&format!("declares the bin {bin:?} of fixtool")),
        "the refusal names the row and the bin: {}",
        error.message
    );
    assert!(
        error
            .message
            .contains(&format!("names of at most {longest} bytes")),
        "the refusal names the limit the file system answered: {}",
        error.message
    );
    // What: the longest bin name that fits, which is the limit minus the bytes a temporary name adds.
    // Why: the reader has to be told the number to shorten the name to, and this is that number.
    let the_most_that_fits: u64 = longest
        .saturating_sub(u64::try_from(the_bytes_a_temporary_name_adds()).expect("a small reserve"));
    assert!(
        error
            .message
            .contains(&format!("is at most {the_most_that_fits} bytes")),
        "the refusal says the longest bin name that would fit: {}",
        error.message
    );
    assert!(
        error.message.contains(&scope.plan_file.display().to_string()),
        "the refusal names the plan file a person edits: {}",
        error.message
    );
    assert!(
        error.message.contains("options.binAliases"),
        "the refusal says how a shorter exposed name is declared: {}",
        error.message
    );
}

/// The question is asked of the nearest directory above the exposure folder that exists,
/// which is the same file system unless a mount sits exactly there.
#[test]
fn the_question_is_put_to_the_nearest_directory_that_exists() {
    let directory = scratch();
    let deeper: PathBuf = directory.path().join("a-scope-that-was-never-installed");
    let (plan, scope) = a_plan_and_a_scope(&a_bin_name_of(usize::try_from(the_longest_name_there(
        directory.path(),
    ))
    .expect("a name limit this machine holds")), &deeper);
    let error = refuse_bin_names_the_exposure_folder_cannot_hold(&plan, &scope)
        .expect_err("the same file system answers through the directory above the folder");
    assert!(
        error.message.contains(&format!(
            "names of at most {} bytes",
            the_longest_name_there(directory.path())
        )),
        "the nearest existing directory answers for the folder: {}",
        error.message
    );
}

/// What:
/// A bin whose link's temporary name is exactly the longest name the file system holds is written,
/// and one byte more fails with the system's own error.
/// Why:
/// This is the failure the check comes before,
/// built on a real file system:
/// it shows the reserve the check uses covers a name that fits,
/// and that the write really does fail one byte past the limit with the temporary name in its words.
#[test]
fn a_name_at_the_limit_is_written_and_one_byte_more_fails_with_the_systems_own_error() {
    let directory = scratch();
    let longest: u64 = the_longest_name_there(directory.path());
    let exposure: PathBuf = directory.path().join("toolchain-bin");
    fs::create_dir_all(&exposure).expect("make the folder the links go in");
    // What: how many bytes the temporary name adds to the bin name for the process writing this control.
    // Why: the concrete suffix is what decides which bin name is the longest one that still fits,
    //  so it is read from the name builder rather than written down here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const suffix = theTemporaryLinkOf(join(exposure, ''), Owner.here()).length - basename(exposure).length;
    // ```
    let probe: PathBuf = exposure.join("a-bin");
    let suffix: usize = the_temporary_link_of(&probe, Owner::here())
        .file_name()
        .expect("a name")
        .len()
        - "a-bin".len();
    let fits: usize = usize::try_from(longest)
        .expect("a name limit this machine holds")
        .saturating_sub(suffix);
    let at_the_limit: String = a_bin_name_of(fits);
    atomic_symlink(&exposure.join(&at_the_limit), &probe)
        .expect("the longest temporary name the file system holds is written");
    assert!(
        fs::symlink_metadata(exposure.join(&at_the_limit)).is_ok(),
        "the link a write at the limit leaves is there"
    );
    let one_more: String = a_bin_name_of(fits + 1);
    let error = atomic_symlink(&exposure.join(&one_more), &probe)
        .expect_err("one byte past the limit is refused by the file system");
    assert!(
        error.message.contains("File name too long"),
        "the write fails with the system's own words: {}",
        error.message
    );
    assert!(
        error
            .message
            .contains(&the_temporary_link_of(&exposure.join(&one_more), Owner::here())
                .display()
                .to_string()),
        "the failure names the temporary name rather than the plan, which is what the check replaces: {}",
        error.message
    );
}
/// What:
/// The longest bin that fits the folder's file system is accepted,
/// one byte more is refused,
/// the reserve covers the temporary name this machine writes,
/// and the length counted is bytes rather than characters.
/// Why:
/// The edge is the whole rule:
/// a check that refused the longest name that fits,
/// or accepted one byte past it,
/// or measured a multibyte name in characters,
/// would refuse a plan that links today or link a name the file system cannot hold.
/// Every length here is derived from the file system's own answer and the reserve the check
/// subtracts;
/// the only numbers written down are the two bytes a two-byte character takes and the one byte
/// the comparison is off by.
#[test]
fn a_bin_of_the_longest_fitting_length_is_accepted_and_one_byte_more_is_refused() {
    let directory = scratch();
    let longest: u64 = the_longest_name_there(directory.path());
    // What: the bytes a temporary name adds, as the check subtracts them.
    // Why: the edge this control holds is `longest - reserve`, and the other answers below are read from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reserve = theBytesATemporaryNameAdds();
    // ```
    let reserve: u64 = u64::try_from(the_bytes_a_temporary_name_adds()).expect("a small reserve");
    // What: the bytes this machine's own temporary name adds to a bin name for the process running this control.
    // Why: the reserve has to cover a name a run really writes, and this is that name's suffix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const exposure = join(scratch, 'toolchain-bin');
    // const suffix = theTemporaryLinkOf(join(exposure, 'a-bin'), Owner.here()).basename.length - 'a-bin'.length;
    // ```
    let exposure: PathBuf = directory.path().join("toolchain-bin");
    fs::create_dir_all(&exposure).expect("make the folder the links go in");
    let probe: PathBuf = exposure.join("a-bin");
    let suffix_here: usize = the_temporary_link_of(&probe, Owner::here())
        .file_name()
        .expect("a name")
        .len()
        - "a-bin".len();
    assert!(
        usize::try_from(reserve).expect("a small reserve") >= suffix_here,
        "the reserve covers the temporary name this machine writes ({suffix_here} bytes)"
    );
    let most: usize = usize::try_from(longest)
        .expect("a name limit this machine holds")
        .saturating_sub(usize::try_from(reserve).expect("a small reserve"));
    let (plan, scope) = a_plan_and_a_scope(&a_bin_name_of(most), directory.path());
    refuse_bin_names_the_exposure_folder_cannot_hold(&plan, &scope)
        .expect("the longest bin name that fits is accepted");
    // What: the same plan and scope pair for the length one byte past the edge, under names of their own.
    // Why: a name reused for a second answer of the same kind would shadow the first in the compiler's eyes,
    //  and these are two different plans.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [oneMorePlan, oneMoreScope] = aPlanAndAScope(aBinNameOf(most + 1), directory.path());
    // ```
    let (one_more_plan, one_more_scope) =
        a_plan_and_a_scope(&a_bin_name_of(most + 1), directory.path());
    let error = refuse_bin_names_the_exposure_folder_cannot_hold(&one_more_plan, &one_more_scope)
        .expect_err("one byte past the longest name that fits is refused");
    assert!(
        error.message.contains(&format!("is at most {most} bytes")),
        "the refusal names the longest bin name that fits: {}",
        error.message
    );
    // What: a bin name of two-byte characters whose characters fit and whose bytes do not.
    // Why: the file system counts bytes, so a name read as characters would be accepted here and fail at the
    //  link write, which is the failure the check exists to come before.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const twoByteCharacters = Math.floor(most / 2) + 1;
    // const multibyte = 'é'.repeat(twoByteCharacters);
    // ```
    let two_byte_characters: usize = most / 2 + 1;
    let multibyte: String = "é".repeat(two_byte_characters);
    assert!(
        multibyte.chars().count() <= most && multibyte.len() > most,
        "the name fits by characters and not by bytes: {} characters, {} bytes, {most} fit",
        multibyte.chars().count(),
        multibyte.len()
    );
    // What: the third plan and scope pair, for the multibyte name.
    // Why: a distinct pair per plan keeps each answer's plan identifiable in a failing assertion.
    let (multibyte_plan, multibyte_scope) = a_plan_and_a_scope(&multibyte, directory.path());
    refuse_bin_names_the_exposure_folder_cannot_hold(&multibyte_plan, &multibyte_scope)
        .expect_err("a bin the file system cannot hold is refused however few characters it has");
}

/// What:
/// The check runs where the plan is read,
/// so `load` itself refuses such a plan rather than its caller.
/// Why:
/// The refusal has to come before any download or link write,
/// and a control of the check alone would hold nothing about whether any command asks it.
#[test]
fn reading_the_plan_refuses_a_bin_the_exposure_folder_cannot_hold() {
    let directory = scratch();
    let longest: u64 = the_longest_name_there(directory.path());
    let bin: String = a_bin_name_of(usize::try_from(longest).expect("a name limit this machine holds"));
    let plan_text: String = a_plan_declaring_the_bin(&bin);
    fs::write(directory.path().join(super::scope::PLAN_FILE_NAME), &plan_text)
        .expect("write the plan the control reads");
    let env = EnvMap::from_pairs(&[
        ("HOME", "/a/throwaway/home"),
        ("PATH", "/usr/bin:/bin"),
    ]);
    let scope = repo_scope(directory.path(), &env).expect("a scope in the scratch directory");
    let error = super::load::load(&scope)
        .expect_err("reading the plan refuses a bin the folder cannot hold a link's temporary name for");
    assert!(
        error.message.contains(&format!("declares the bin {bin:?} of fixtool")),
        "the refusal is the check's own, reached through the plan reader: {}",
        error.message
    );
}

/// What:
/// The reserve is the widest temporary name a run can write,
/// not merely a number above this machine's own
/// suffix.
/// Why:
/// `README.md` promises 95 bytes,
/// a file system of 255 bytes 160 bytes of bin name and 161 as the first refused
/// length,
/// and the control above holds none of those numbers:
/// any reserve from this machine's 70 bytes up to more than 95 passed it.
/// This one derives the widest name from the fields it carries:
/// a `u32` process number,
/// a `u64` start time and a `u64` namespace,
/// the purpose text the link is held under,
/// the separators between them,
/// and the boot identifier's length as the kernel itself writes it,
/// so a reserve narrowed to this machine's suffix,
/// a digit count taken from a narrower type,
/// and a boot identifier's length raised past the kernel's all fail it.
#[test]
fn the_reserve_is_the_widest_temporary_name_a_run_can_write() {
    // What: the boot identifier this machine is running, as the kernel writes it.
    // Why: the name records that identifier's text, so its length is the kernel's answer and not a number
    //  written here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const boot = readFileSync('/proc/sys/kernel/random/boot_id', 'utf8').trim();
    // ```
    let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .expect("the kernel tells a program which boot it is running")
        .trim()
        .to_string();
    // What: the widest temporary name's surplus over the bin name, derived one field at a time.
    // Why: each term is one field's widest write: the leading dot, the purpose text and its dots, the
    //  decimal digits each number can hold, and the boot identifier.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const widest = 1 + 1 + 'tmp'.length + 1 + String(2 ** 32 - 1).length + 1 + String(2 ** 64 - 1).length + 1 + String(2 ** 64 - 1).length + 1 + boot.length;
    // ```
    let widest: usize = 1
        + 1
        + "tmp".len()
        + 1
        + format!("{}", u32::MAX).len()
        + 1
        + format!("{}", u64::MAX).len()
        + 1
        + format!("{}", u64::MAX).len()
        + 1
        + boot.len();
    assert_eq!(
        the_bytes_a_temporary_name_adds(),
        widest,
        "the reserve is the widest name a run can write, and the widest this machine's fields allow is {widest} bytes over the bin name"
    );
}

/// What:
/// The name limit the check refuses against is the file system's answer,
/// not a number written in the code.
/// Why:
/// A folder on eCryptfs holds 143 bytes where this machine's holds 255,
/// so a hard-coded 255 would accept a bin that file system cannot hold and would name the wrong limit in its
/// refusal;
/// no mount-free file system here holds anything but 255,
/// so the control plants the answer the check asks for.
/// The planted number is the 143 bytes `README.md` names for eCryptfs;
/// it stands in for a file system this control cannot mount,
/// and no claim about a real eCryptfs rests on it.
#[test]
fn the_name_limit_comes_from_the_file_system_and_not_from_a_number_written_here() {
    // What: the limit a control plants, which is the one `README.md` names for eCryptfs.
    // Why: it has to differ from this machine's 255 bytes, or the control could not tell the file system's
    //  answer from a constant.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const planted = 143;
    // ```
    let planted: u64 = 143;
    let directory = scratch();
    // What: the longest bin name that fits the planted limit.
    // Why: the arm that has to be accepted, and the one byte past it has to be refused.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const most = planted - theBytesATemporaryNameAdds();
    // ```
    let reserve: u64 = u64::try_from(the_bytes_a_temporary_name_adds()).expect("a small reserve");
    let most: usize = usize::try_from(planted.saturating_sub(reserve))
        .expect("a name limit this machine holds");
    let _plant = planting_the_folders_name_limit(planted);
    let (plan, scope) = a_plan_and_a_scope(&a_bin_name_of(most), directory.path());
    refuse_bin_names_the_exposure_folder_cannot_hold(&plan, &scope)
        .expect("the longest bin name the planted limit holds is accepted");
    let (one_more_plan, one_more_scope) =
        a_plan_and_a_scope(&a_bin_name_of(most + 1), directory.path());
    let error = refuse_bin_names_the_exposure_folder_cannot_hold(&one_more_plan, &one_more_scope)
        .expect_err("one byte past the planted limit is refused");
    assert!(
        error.message.contains(&format!("names of at most {planted} bytes")),
        "the refusal names the limit the file system answered, not a number written here: {}",
        error.message
    );
}
