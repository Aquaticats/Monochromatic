//! What:
//!  What a record of an interrupted ignored-state copy does to the next command
//!  when a crash left it empty or cut short:
//!  the journal,
//!  the stage claim,
//!  the record of an unfilled worktree,
//!  and the registration inside a private stage.
//! Why:
//!  Every command in a linked worktree reads these records before Git runs,
//!  so one record that a power cut published before its bytes stopped every command of the repository,
//!  in every worktree,
//!  until a person removed the file.
//! A record that cannot be read is the mark of an interrupted copy,
//!  never a reason to stop a command:
//!  the command moves it aside,
//!  says what became of it,
//!  and runs.
//! A cut that still leaves a record this release reads is not damage:
//!  the copy it describes is settled as it always was,
//!  and these controls say so per state instead of pretending every cut is unreadable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const damage of states(recordBytes)) { const ran = run(wrappedGit, ['status']); expect(ran.status).toBe(0); expect(movedNameOf(damage)).toContain(recordName); }
//! ```

/// The shared fixtures and the bounded process helpers.
use super::support::{Fixture, Observed, REAL_GIT, dead_copy_owner, fixture, remove, repository, run_wrapped, stderr_of};
use git_policy_cli::transaction_git::GitContext;
use git_policy_cli::worktree_add_binding::{bind_registration, path_identity};
use git_policy_cli::worktree_copy_ceiling::CopyCeiling;
use git_policy_cli::worktree_copy_journal::{create_journal_for, journal_root};
use git_policy_cli::worktree_copy_journal_encoding_and_validation::{JournalRefusal, parse_journal_reading};
use git_policy_cli::worktree_copy_owner::CopyOwner;
use git_policy_cli::worktree_copy_selection::CopySelection;
use git_policy_cli::worktree_copy_stage::{StagedSnapshot, new_stage_container, stage_ignored_snapshot, stage_ignored_snapshot_for};
use git_policy_cli::worktree_copy_stage_binding::{BindingFile, STAGE_BINDING_NAME, stage_binding_reading};
use git_policy_cli::worktree_copy_stage_claim::claim_stage_for_add_as;
use git_policy_cli::worktree_copy_stage_claim_reaping::{ClaimFile, claim_file_reading};
use git_policy_cli::worktree_copy_unfilled_record::{decode_unfilled, remember_unfilled, unfilled_path_of};
use git_policy_cli::worktree_copy_unreadable_records::{UNREADABLE_FOLDER, unreadable_record_names};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The line every settled record prints.
const SETTLED: &str = "could not be read";

/// One way a crash leaves a record:
/// empty,
/// cut at one field boundary,
/// cut inside one field,
/// or whole with stale bytes after it.
struct Damage {
    /// What the damage is called in a failure.
    what: String,
    /// The bytes a record holds after it.
    bytes: Vec<u8>,
}

/// Where a value that starts at `rest` ends: the next top-level separator or the object's closing brace.
fn value_end(rest: &[u8]) -> usize {
    let mut in_string: bool = false;
    let mut escaped: bool = false;
    let mut depth: usize = 0;
    for (index, byte) in rest.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' | b'[' => depth += 1,
            b'}' | b']' if depth == 0 => return index,
            b'}' | b']' => depth -= 1,
            b',' if depth == 0 => return index,
            _ => {}
        }
    }
    return rest.len();
}

/// The byte offsets a crash can cut one record at:
/// every field boundary of a JSON object and the middle of every field,
/// or every line boundary of a text record and the middle of every line.
fn cut_points(text: &[u8], shaped: char) -> Vec<(String, usize)> {
    let mut points: Vec<(String, usize)> = Vec::new();
    if shaped == 'j' {
        let mut in_string: bool = false;
        let mut escaped: bool = false;
        let mut depth: usize = 0;
        let mut field: usize = 0;
        for (index, byte) in text.iter().enumerate() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if *byte == b'\\' {
                    escaped = true;
                } else if *byte == b'"' {
                    in_string = false;
                }
                continue;
            }
            match byte {
                b'"' => in_string = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => depth -= 1,
                b':' if depth == 1 => {
                    let end: usize = value_end(&text[index + 1..]);
                    points.push((format!("inside field {}", field + 1), index + 1 + (end / 2).max(1)));
                }
                b',' if depth == 1 => {
                    field += 1;
                    points.push((format!("between fields {field} and {}", field + 1), index + 1));
                }
                _ => {}
            }
        }
        return points;
    }
    let mut start: usize = 0;
    let mut line: usize = 0;
    for (index, byte) in text.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        line += 1;
        points.push((format!("after line {line}"), index + 1));
        points.push((format!("inside line {line}"), start + ((index - start) / 2).max(1)));
        start = index + 1;
    }
    return points;
}

/// Every damage this control applies to one record's bytes.
fn damages_of(bytes: &[u8], shaped: char) -> Vec<Damage> {
    let mut damages: Vec<Damage> = vec![Damage { what: String::from("empty"), bytes: Vec::new() }];
    for (what, at) in cut_points(bytes, shaped) {
        if at == 0 || at >= bytes.len() {
            continue;
        }
        damages.push(Damage { what: format!("cut {what}"), bytes: bytes[..at].to_vec() });
    }
    let mut stale: Vec<u8> = bytes.to_vec();
    stale.extend_from_slice(b"STALE\n");
    damages.push(Damage { what: String::from("whole, then stale bytes"), bytes: stale });
    damages.push(Damage { what: String::from("whole"), bytes: bytes.to_vec() });
    return damages;
}

/// A repository with one linked worktree the wrapper did not fill,
/// so that every record this control writes is the only one there.
fn repository_with_a_linked_worktree(name: &str) -> (Fixture, PathBuf, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let added: Observed = run_wrapped(
        &fixture,
        repo.as_path(),
        &["worktree", "add", "--no-worktree-copy", "--quiet", "-b", "one", "../one"],
    );
    assert_eq!(added.code, Some(0), "{}", stderr_of(&added));
    let one: PathBuf = std::fs::canonicalize(fixture.root.join("one")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    return (fixture, repo, one, common);
}

/// The Git the package's own tests start.
fn context() -> GitContext {
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// The bytes a record held before it was damaged,
/// written at its final name with the private mode a writer gives it.
fn write_record(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).expect("the record's bytes");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).expect("a private record");
}

/// Run one read-only command in the linked worktree and require that it ran:
/// exit 0 and Git's own answer on standard output.
fn require_the_command_ran(fixture: &Fixture, worktree: &Path, damage: &Damage) -> String {
    let ran: Observed = run_wrapped(fixture, worktree, &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(ran.code, Some(0), "{}\n{}", damage.what, stderr_of(&ran));
    assert_eq!(ran.stdout, b"true\n", "{}", damage.what);
    return stderr_of(&ran);
}

/// Require that the one line about a record this release cannot read stands on standard error,
/// and that it says where the record went.
fn require_the_settlement_line(stderr: &str, damage: &Damage) {
    assert!(stderr.contains(SETTLED), "{}: {stderr}", damage.what);
    assert!(stderr.contains(UNREADABLE_FOLDER), "{}: the line says where the record went: {stderr}", damage.what);
}

/// The names the folder of records this release cannot read keeps.
fn kept_names(common: &Path) -> Vec<String> {
    return unreadable_record_names(common).expect("kept records");
}

/// The records one command kept that were not kept before it ran.
fn newly_kept(before: &[String], after: &[String]) -> Vec<String> {
    return after.iter().filter(|name| { return !before.contains(name); }).cloned().collect();
}

/// Require that one record of `name` joined the folder when it could not be read,
/// and none did when it could.
fn require_what_became_of_the_record(before: &[String], common: &Path, name: &str, refused: bool, damage: &Damage, stderr: &str) {
    let added: Vec<String> = newly_kept(before, kept_names(common).as_slice());
    assert_eq!(added.len(), usize::from(refused), "{}: {added:?}\n{stderr}", damage.what);
    if !refused {
        return;
    }
    assert!(
        added[0] == name || added[0].starts_with(format!("{name}.").as_str()),
        "{}: the record is kept under its own name, {added:?}",
        damage.what
    );
}

/// An unreadable journal of a writer that is gone is moved aside in every state a crash can leave it,
/// the folder keeps it,
/// a journal this release still reads is recovered as before,
/// and the command that met it runs either way.
#[test]
fn an_unreadable_journal_never_stops_a_command() {
    let (fixture, repo, one, common) = repository_with_a_linked_worktree("unreadable-journal");
    let source: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical source");
    let owner: CopyOwner = dead_copy_owner();
    let root: PathBuf = journal_root(common.as_path());
    let whole: Vec<u8> = {
        let snapshot: StagedSnapshot = stage_ignored_snapshot(&context(), source.as_path(), new_stage_container(one.as_path()).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
        let pending = create_journal_for(common.as_path(), one.as_path(), &snapshot, &owner).expect("journal");
        let bytes: Vec<u8> = std::fs::read(pending.path.as_path()).expect("the journal's bytes");
        std::fs::remove_file(pending.path.as_path()).expect("the whole record is only the control's");
        bytes
    };
    for damage in damages_of(whole.as_slice(), 'j') {
        let snapshot: StagedSnapshot = stage_ignored_snapshot(&context(), source.as_path(), new_stage_container(one.as_path()).expect("container").as_path(), &[], &CopySelection::Ignored).expect("staged");
        let pending = create_journal_for(common.as_path(), one.as_path(), &snapshot, &owner).expect("journal");
        let name: String = pending.path.file_name().expect("a name").to_string_lossy().into_owned();
        write_record(pending.path.as_path(), damage.bytes.as_slice());
        let refused: bool = match parse_journal_reading(String::from_utf8_lossy(damage.bytes.as_slice()).as_ref(), pending.path.as_path(), root.as_path()) {
            Ok(_readable) => false,
            Err(JournalRefusal::Unsafe(_fault)) => false,
            Err(_damage) => true,
        };
        let before: Vec<String> = kept_names(common.as_path());
        let stderr: String = require_the_command_ran(&fixture, one.as_path(), &damage);
        assert!(!pending.path.exists(), "{}: no journal this release cannot read stands where every command reads", damage.what);
        require_what_became_of_the_record(before.as_slice(), common.as_path(), name.as_str(), refused, &damage, stderr.as_str());
        if refused {
            require_the_settlement_line(stderr.as_str(), &damage);
            assert!(stderr.contains(name.as_str()), "{}: the line names the record: {stderr}", damage.what);
        }
    }
    remove(&fixture);
}

/// An unreadable stage claim of a writer that is gone is moved aside in every state a crash can leave it,
/// written again when the record beside it names its folder,
/// a claim this release still reads is settled as before,
/// and the command that met it runs either way.
#[test]
fn an_unreadable_stage_claim_never_stops_a_command() {
    let (fixture, _repo, one, common) = repository_with_a_linked_worktree("unreadable-claim");
    let root: PathBuf = journal_root(common.as_path());
    let owner: CopyOwner = dead_copy_owner();
    let whole: Vec<u8> = {
        let claim = claim_stage_for_add_as(common.as_path(), one.as_path(), &dead_copy_owner()).expect("an add's claim");
        let path: PathBuf = root.join(format!("{}.stage", claim.identifier()));
        let bytes: Vec<u8> = std::fs::read(path.as_path()).expect("the claim's bytes");
        std::fs::remove_file(path.as_path()).expect("the whole record is only the control's");
        bytes
    };
    for damage in damages_of(whole.as_slice(), 'l') {
        let claim = claim_stage_for_add_as(common.as_path(), one.as_path(), &owner).expect("an add's claim");
        let name: String = format!("{}.stage", claim.identifier());
        let path: PathBuf = root.join(name.as_str());
        write_record(path.as_path(), damage.bytes.as_slice());
        let refused: bool = matches!(claim_file_reading(path.as_path()).expect("reading"), Some(ClaimFile::Unreadable { .. }));
        let before: Vec<String> = kept_names(common.as_path());
        let stderr: String = require_the_command_ran(&fixture, one.as_path(), &damage);
        require_what_became_of_the_record(before.as_slice(), common.as_path(), name.as_str(), refused, &damage, stderr.as_str());
        if refused {
            require_the_settlement_line(stderr.as_str(), &damage);
            assert!(stderr.contains("written again beside the claim name"), "{}: the claim is written again from the records beside it: {stderr}", damage.what);
            assert!(stderr.contains(claim.container().to_string_lossy().as_ref()), "{}: the line names the folder: {stderr}", damage.what);
        }
    }
    remove(&fixture);
}

/// An unreadable record of a worktree whose copy did not finish is moved aside in every state a crash can leave it,
/// a record this release still reads keeps naming its worktree,
/// and the command that met it runs either way.
#[test]
fn an_unreadable_unfilled_record_never_stops_a_command() {
    let (fixture, _repo, one, common) = repository_with_a_linked_worktree("unreadable-unfilled");
    let identity = path_identity(one.as_path()).expect("the worktree's identity");
    let owner: CopyOwner = dead_copy_owner();
    let _root: PathBuf = git_policy_cli::worktree_copy_journal::ensure_journal_root(common.as_path()).expect("the records folder of this repository");
    let journal: PathBuf = journal_root(common.as_path()).join("8f14e45f.unread.json");
    let path: PathBuf = unfilled_path_of(journal.as_path());
    let whole: Vec<u8> = {
        remember_unfilled(journal.as_path(), one.as_path(), identity, fixture.root.as_path(), Some(owner.token().as_str())).expect("the record");
        let bytes: Vec<u8> = std::fs::read(path.as_path()).expect("the record's bytes");
        std::fs::remove_file(path.as_path()).expect("the whole record is only the control's");
        bytes
    };
    for damage in damages_of(whole.as_slice(), 'l') {
        write_record(path.as_path(), damage.bytes.as_slice());
        let refused: bool = decode_unfilled(String::from_utf8_lossy(damage.bytes.as_slice()).as_ref()).is_err();
        let name: String = path.file_name().expect("a name").to_string_lossy().into_owned();
        let before: Vec<String> = kept_names(common.as_path());
        let stderr: String = require_the_command_ran(&fixture, one.as_path(), &damage);
        require_what_became_of_the_record(before.as_slice(), common.as_path(), name.as_str(), refused, &damage, stderr.as_str());
        if refused {
            assert!(!path.exists(), "{}: the record no longer stands where every command reads", damage.what);
            require_the_settlement_line(stderr.as_str(), &damage);
        } else {
            assert!(stderr.contains("holds less than"), "{}: a record this release reads keeps naming its worktree: {stderr}", damage.what);
        }
    }
    remove(&fixture);
}

/// A registration inside a private stage that cannot be read is moved aside in every state a crash can leave it,
/// the copy it belongs to finishes on the destination's own registration,
/// a registration this release still reads is checked as before,
/// and the command that met it runs either way.
#[test]
fn an_unreadable_stage_binding_never_stops_a_command() {
    let (fixture, repo, one, common) = repository_with_a_linked_worktree("unreadable-binding");
    let source: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical source");
    let owner: CopyOwner = dead_copy_owner();
    let bound = bind_registration(common.as_path(), one.as_path()).expect("the registration of the linked worktree");
    let whole: Vec<u8> = {
        let container: PathBuf = new_stage_container(one.as_path()).expect("container");
        let snapshot: StagedSnapshot = stage_ignored_snapshot_for(&context(), source.as_path(), container.as_path(), &[], &CopySelection::Ignored, Some(&bound.binding), &CopyCeiling::unbounded()).expect("staged");
        let bytes: Vec<u8> = std::fs::read(snapshot.stage_container.join(STAGE_BINDING_NAME)).expect("the binding's bytes");
        let _journal = create_journal_for(common.as_path(), one.as_path(), &snapshot, &owner).expect("journal");
        std::fs::remove_dir_all(snapshot.stage_container.as_path()).expect("the stage is only the control's");
        bytes
    };
    for damage in damages_of(whole.as_slice(), 'l') {
        let container: PathBuf = new_stage_container(one.as_path()).expect("container");
        let snapshot: StagedSnapshot = stage_ignored_snapshot_for(&context(), source.as_path(), container.as_path(), &[], &CopySelection::Ignored, Some(&bound.binding), &CopyCeiling::unbounded()).expect("staged");
        let _journal = create_journal_for(common.as_path(), one.as_path(), &snapshot, &owner).expect("journal");
        let record: PathBuf = container.join(STAGE_BINDING_NAME);
        write_record(record.as_path(), damage.bytes.as_slice());
        let refused: bool = matches!(stage_binding_reading(container.as_path()).expect("reading"), Some(BindingFile::Unreadable { .. }));
        let name: String = format!("{}.{STAGE_BINDING_NAME}", container.file_name().expect("a container name").to_string_lossy());
        let before: Vec<String> = kept_names(common.as_path());
        let stderr: String = require_the_command_ran(&fixture, one.as_path(), &damage);
        require_what_became_of_the_record(before.as_slice(), common.as_path(), name.as_str(), refused, &damage, stderr.as_str());
        if refused {
            require_the_settlement_line(stderr.as_str(), &damage);
        }
        let _cleaned: Result<(), std::io::Error> = std::fs::remove_dir_all(container.as_path());
    }
    remove(&fixture);
}

/// An unreadable claim whose folder nothing beside it names is moved aside,
/// and the worktree it named is named by the settlement of the destination record instead.
#[test]
fn an_unreadable_claim_without_its_folder_is_moved_aside_and_named() {
    let (fixture, _repo, one, common) = repository_with_a_linked_worktree("unreadable-claim-no-folder");
    let root: PathBuf = journal_root(common.as_path());
    let claim = claim_stage_for_add_as(common.as_path(), fixture.root.join("elsewhere").as_path(), &dead_copy_owner()).expect("an add's claim");
    let name: String = format!("{}.stage", claim.identifier());
    write_record(root.join(name.as_str()).as_path(), b"");
    let damage: Damage = Damage { what: String::from("empty claim"), bytes: Vec::new() };

    let stderr: String = require_the_command_ran(&fixture, one.as_path(), &damage);

    require_the_settlement_line(stderr.as_str(), &damage);
    assert!(kept_names(common.as_path()).iter().any(|found| { return found == &name; }), "the claim is kept aside: {stderr}");
    assert!(stderr.contains("destination record"), "the line says what else is settled: {stderr}");
    remove(&fixture);
}
