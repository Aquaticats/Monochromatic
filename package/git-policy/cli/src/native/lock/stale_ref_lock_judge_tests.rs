//! Controls for the margins of the reference-lock judge:
//! each must clear the measurement it was set from.

use super::*;

/// A lock younger than the longest live hold measured may still be live,
/// so the margin must exceed that hold.
#[test]
fn the_margin_exceeds_the_longest_live_reference_step_measured() {
    assert!(
        MARGIN > LONGEST_LIVE_REF_STEP,
        "{MARGIN:?} must exceed {LONGEST_LIVE_REF_STEP:?}"
    );
}

/// The margin is the multiple of the measurement,
/// rounded up to whole seconds and no further.
#[test]
fn the_margin_is_the_multiple_of_the_measurement_rounded_up_to_a_whole_second() {
    let wanted: Duration =
        LONGEST_LIVE_REF_STEP * u32::try_from(MARGIN_MULTIPLE).expect("a small multiple");
    assert_eq!(MARGIN.subsec_nanos(), 0);
    assert!(MARGIN >= wanted, "{MARGIN:?} must cover {wanted:?}");
    assert!(
        MARGIN - wanted < Duration::from_secs(1),
        "{MARGIN:?} is a second or more over {wanted:?}"
    );
}

/// What:
///  The name another program chose for the process that holds the judged file:
///  a closing parenthesis that would end the parentheses around it,
///  an escape that would erase the line,
///  a line break that would start a line this program never wrote,
///  and a right-to-left override that would reorder it.
/// Why:
///  The name comes from `/proc/<pid>/comm`, which the program sets to any bytes it likes,
///  and the verdict is the line a person reads to find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NAME_A_PROGRAM_CHOSE = 'x) FORGED\u001b[2K\n\u202e';
/// ```
const NAME_A_PROGRAM_CHOSE: &str = "x) FORGED\u{1b}[2K\n\u{202e}";

/// What:
///  The name shown in the sentence that says a live process holds the judged file.
/// Why:
///  The four shapes are written out,
///  so the sentence keeps one line and the parentheses around the name stay the program's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SHOWN_NAME = 'x\\) FORGED\\u{1b}[2K\\n\\u{202e}';
/// ```
const SHOWN_NAME: &str = "x\\) FORGED\\u{1b}[2K\\n\\u{202e}";

/// What:
///  A process table with one process of the given number whose one open descriptor is `held`,
///  named as the kernel would name it.
/// Why:
///  The scan looks for the file's device and inode among every process's descriptors,
///  so a control writes the link the kernel writes instead of starting such a program,
///  and states the name the program chose in `comm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcessHolding(held, 4242);
/// ```
fn a_table_with_one_process_holding(
    held: &Path,
    pid: i64,
) -> PathBuf {
    let root: PathBuf = crate::test_support::fixture("stale-ref-lock-judge-name");
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    let descriptors: PathBuf = entry.join("fd");
    std::fs::create_dir_all(descriptors.as_path()).expect("a descriptor directory");
    std::os::unix::fs::symlink(held, descriptors.join("3")).expect("the file the process holds open");
    std::fs::write(entry.join("comm"), NAME_A_PROGRAM_CHOSE.as_bytes())
        .expect("the name the program chose");
    return table;
}

/// What:
///  The judge of a fixture process table that looks at no other process.
/// Why:
///  Every process of the machine is evidence about a lock,
///  so a control that plants one names the process of its table and no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judge = { ...Judge.here(), procRoot: table, margin: 0, scanScope: [4242] };
/// ```
fn a_judge_of(table: &Path, pid: i64) -> Judge {
    return Judge {
        proc_root: table.to_path_buf(),
        margin: Duration::ZERO,
        scan_scope: Some(vec![pid]),
        ..Judge::here()
    };
}

/// The verdict of a file a live process holds open names that process,
/// and the name it got back cannot end the parentheses around it or start a line.
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_verdict_of_a_lock() {
    let root: PathBuf = crate::test_support::fixture("stale-ref-lock-held");
    let lock: PathBuf = root.join("index.lock");
    std::fs::write(lock.as_path(), b"lock\n").expect("a lock file");
    let pid: i64 = 4242;
    let table: PathBuf = a_table_with_one_process_holding(lock.as_path(), pid);
    let judge: Judge = a_judge_of(table.as_path(), pid);
    let examined: Examined = judge.examine(lock.as_path()).expect("the lock is a file");
    assert_eq!(
        examined.verdict,
        Verdict::Held(format!("process {pid} ({SHOWN_NAME}) has the file open"))
    );
    crate::test_support::remove(root.as_path());
}

/// The verdict of a staging file a live process holds open names that process,
/// and the name it got back cannot end the parentheses around it or start a line.
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_verdict_of_a_staging_file() {
    let root: PathBuf = crate::test_support::fixture("stale-ref-lock-staging-held");
    let staging: PathBuf = root.join("packed-refs.new");
    std::fs::write(staging.as_path(), b"refs\n").expect("a staging file");
    let pid: i64 = 4242;
    let table: PathBuf = a_table_with_one_process_holding(staging.as_path(), pid);
    let judge: Judge = a_judge_of(table.as_path(), pid);
    let examined: Examined = judge
        .examine_staging(staging.as_path())
        .expect("the staging file is a file");
    assert_eq!(
        examined.verdict,
        Verdict::Held(format!("process {pid} ({SHOWN_NAME}) has the file open"))
    );
    crate::test_support::remove(root.as_path());
}
