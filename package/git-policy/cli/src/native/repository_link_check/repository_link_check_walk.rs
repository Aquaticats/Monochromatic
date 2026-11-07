//! What:
//!  The repository link check:
//! phase `after` checks the link to the 860,
//! phase `before` checks the real folder on the NVMe,
//! and a deadline gives up on a drive that does not answer.
//! Why:
//!  The answers and texts equal the TypeScript check's
//! (`package/repository-link/check/src/check-link.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await checkRepositoryLink({ read, probe, deadlineSeconds });
//! ```

use crate::repository_link_answer::{AnswerKind, LinkAnswer};
use crate::repository_link_identity::{LinkIdentity, LinkPhase};
use crate::repository_link_probe::{EntryKind, LinkProbe};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// The file inside `.git` that holds the first commit hash of `main`,
/// written at cut-over:
/// a folder named `.git` alone is not an identity.
pub const ROOT_COMMIT_MARKER: &str = "monochromatic-root-commit";

/// How long the check waits for the filesystem before it says the drive did not answer.
/// The plan asks for "a few seconds" (`doc/planning/repository-on-the-860-drive.md`,
/// W1),
/// and the owner's rule is "nothing should ever wait forever or indefinitely";
/// the TypeScript check waits the same five seconds.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(5);

/// How every refusal ends:
/// a person reads that the check changed nothing.
const NOTHING_WRITTEN: &str = "nothing was written";

/// What:
///  The answer when no identity file was read.
/// Why:
///  Nothing can be checked without the recorded identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusal({ kind: 'identity-missing', message })
/// ```
pub fn answer_when_identity_is_missing() -> LinkAnswer {
    return LinkAnswer::refusal(
        AnswerKind::IdentityMissing,
        String::from("the repository link identity file is missing; the cut-over step writes it with repository-link-record"),
    );
}

/// Phase `before`:
/// a real folder on the NVMe with a `.git` folder.
fn check_before(identity: &LinkIdentity, probe: &dyn LinkProbe) -> LinkAnswer {
    let link: &Path = Path::new(identity.link_path.as_str());
    let entry: EntryKind = probe.lstat(link);
    if entry != EntryKind::Directory {
        return LinkAnswer::refusal(
            AnswerKind::NotARepository,
            format!(
                "{} is not a real folder (it is {}), but phase before expects the repository there; {NOTHING_WRITTEN}",
                identity.link_path,
                entry.name()
            ),
        );
    }
    let uuid: String = probe.uuid_of(link);
    if uuid != identity.nvme_uuid || identity.nvme_uuid.is_empty() {
        return LinkAnswer::refusal(
            AnswerKind::WrongFilesystem,
            format!(
                "{} is on filesystem {}, the recorded NVMe is {}; {NOTHING_WRITTEN}",
                identity.link_path,
                or_word(uuid.as_str(), "unknown"),
                or_word(identity.nvme_uuid.as_str(), "unrecorded")
            ),
        );
    }
    if probe.lstat(link.join(".git").as_path()) != EntryKind::Directory {
        return LinkAnswer::refusal(AnswerKind::NotARepository, format!("{}/.git is not a folder; {NOTHING_WRITTEN}", identity.link_path));
    }
    return LinkAnswer::ready(AnswerKind::ReadyBefore, format!("the repository at {} is ready (before the move)", identity.link_path));
}

/// The text,
/// or a word in its place when it is empty.
fn or_word<'text>(text: &'text str, word: &'text str) -> &'text str {
    return if text.is_empty() { word } else { text };
}

/// The link's target is not there:
/// the drive is not mounted,
/// or the folder is missing on a mounted drive.
fn explain_missing_target(identity: &LinkIdentity, probe: &dyn LinkProbe) -> LinkAnswer {
    if !probe.is_mount_point(Path::new(identity.mount_point.as_str())) {
        return LinkAnswer::refusal(
            AnswerKind::DriveNotMounted,
            format!(
                "the 860 drive is not mounted yet ({} is empty), so the repository at {} is not there; unlock it (the mount-860 service does that at login) and run this again",
                identity.mount_point, identity.link_path
            ),
        );
    }
    return LinkAnswer::refusal(
        AnswerKind::MountedFolderMissing,
        format!("the 860 drive is mounted but {} does not exist; {NOTHING_WRITTEN}", identity.canonical_root),
    );
}

/// The resolved root is on the recorded filesystem,
/// is a repository,
/// and carries the recorded marker.
fn check_resolved_root(identity: &LinkIdentity, probe: &dyn LinkProbe, root: &Path) -> LinkAnswer {
    let uuid: String = probe.uuid_of(root);
    if uuid != identity.drive_uuid || identity.drive_uuid.is_empty() {
        return LinkAnswer::refusal(
            AnswerKind::WrongFilesystem,
            format!(
                "{} resolves to {} on filesystem {}, but the recorded 860 is {}; {NOTHING_WRITTEN}",
                identity.link_path,
                root.display(),
                or_word(uuid.as_str(), "unknown"),
                or_word(identity.drive_uuid.as_str(), "unrecorded")
            ),
        );
    }
    let git: PathBuf = root.join(".git");
    if probe.lstat(root) != EntryKind::Directory || probe.lstat(git.as_path()) != EntryKind::Directory {
        return LinkAnswer::refusal(
            AnswerKind::NotARepository,
            format!("{} is not a repository folder with a .git folder; {NOTHING_WRITTEN}", root.display()),
        );
    }
    let git_uuid: String = probe.uuid_of(git.as_path());
    if git_uuid != uuid {
        return LinkAnswer::refusal(
            AnswerKind::WrongFilesystem,
            format!("{}/.git is on filesystem {} but {} is on {uuid}; {NOTHING_WRITTEN}", root.display(), or_word(git_uuid.as_str(), "unknown"), root.display()),
        );
    }
    let marker: String = probe.read_text(git.join(ROOT_COMMIT_MARKER).as_path()).trim().to_string();
    if marker != identity.root_commit || identity.root_commit.is_empty() {
        return LinkAnswer::refusal(
            AnswerKind::WrongRepository,
            format!(
                "{}/.git/{ROOT_COMMIT_MARKER} holds {}, the recorded root commit is {}; {NOTHING_WRITTEN}",
                root.display(),
                or_word(marker.as_str(), "nothing"),
                or_word(identity.root_commit.as_str(), "unrecorded")
            ),
        );
    }
    return LinkAnswer::ready(AnswerKind::ReadyAfter, format!("the repository at {} is ready (on the 860)", identity.link_path));
}

/// Phase `after`:
/// a link to the recorded root on the recorded filesystem.
fn check_after(identity: &LinkIdentity, probe: &dyn LinkProbe) -> LinkAnswer {
    let link: &Path = Path::new(identity.link_path.as_str());
    let entry: EntryKind = probe.lstat(link);
    if entry == EntryKind::Directory {
        return LinkAnswer::refusal(
            AnswerKind::RealFolderReplacedLink,
            format!("{} is a real folder, not the link to the 860; {NOTHING_WRITTEN}", identity.link_path),
        );
    }
    if entry == EntryKind::Missing {
        return LinkAnswer::refusal(
            AnswerKind::LinkMissing,
            format!("{} does not exist; the link to the 860 is gone; {NOTHING_WRITTEN}", identity.link_path),
        );
    }
    let text: String = if entry == EntryKind::Link { probe.readlink(link) } else { String::new() };
    if entry != EntryKind::Link || text != identity.link_text {
        let what: String = if entry == EntryKind::Link { format!("a link to {text}") } else { format!("a {}", entry.name()) };
        return LinkAnswer::refusal(
            AnswerKind::WrongRepository,
            format!("{} is {what}, the recorded link text is {}; {NOTHING_WRITTEN}", identity.link_path, identity.link_text),
        );
    }
    let Some(root) = probe.realpath(link) else {
        return explain_missing_target(identity, probe);
    };
    if root != Path::new(identity.canonical_root.as_str()) {
        return LinkAnswer::refusal(
            AnswerKind::WrongRepository,
            format!("{} resolves to {}, the approved root is {}; {NOTHING_WRITTEN}", identity.link_path, root.display(), identity.canonical_root),
        );
    }
    return check_resolved_root(identity, probe, root.as_path());
}

/// What:
///  Checks the repository link without a deadline.
/// Why:
///  The table of fixture states runs this directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await answerWithoutDeadline({ read, probe })
/// ```
pub fn check_repository_link(identity: &LinkIdentity, probe: &dyn LinkProbe) -> LinkAnswer {
    return match identity.phase {
        LinkPhase::Before => check_before(identity, probe),
        LinkPhase::After => check_after(identity, probe),
    };
}

/// What:
///  Checks the repository link and gives up after a deadline.
/// Why:
///  A stalled `stat` on a hung drive must not hold a Git command forever:
/// the check runs on a worker thread that is abandoned at the deadline,
/// and the answer is that the drive did not answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await checkRepositoryLink({ read, probe, deadlineSeconds })
/// ```
pub fn check_repository_link_within(identity: &LinkIdentity, probe: Arc<dyn LinkProbe + Send + Sync>, deadline: Duration) -> LinkAnswer {
    let (sender, receiver) = std::sync::mpsc::channel::<LinkAnswer>();
    let owned: LinkIdentity = identity.clone();
    let worker_probe: Arc<dyn LinkProbe + Send + Sync> = Arc::clone(&probe);
    let spawned = std::thread::Builder::new().name(String::from("repository-link-check")).spawn(move || {
        let answer: LinkAnswer = check_repository_link(&owned, worker_probe.as_ref());
        let _sent = sender.send(answer);
    });
    if spawned.is_err() {
        // No worker could be started: the check runs here, without a deadline.
        return check_repository_link(identity, probe.as_ref());
    }
    return match receiver.recv_timeout(deadline) {
        Ok(answer) => answer,
        Err(_timeout) => LinkAnswer::refusal(
            AnswerKind::DriveDidNotAnswer,
            format!(
                "the drive did not answer within {} s, so the state of {} is unknown; {NOTHING_WRITTEN}",
                deadline.as_secs_f64(),
                identity.link_path
            ),
        ),
    };
}

/// Check controls stay out of the release executable.
#[cfg(test)]
#[path = "repository_link_check_walk_tests.rs"]
mod tests;
