//! What:
//!  The branch's remote-tracking reference,
//! brought up to date with what a successful push delivered,
//!  when Git could not write it.
//! Why:
//!  Git delivers a push and then updates the remote-tracking reference of the pushed branch.
//! When another Git holds that reference's lock,
//! it delivers the commits,
//! prints `cannot lock ref '...'`,
//! and still exits `0`,
//! which leaves the reference naming an older commit
//!  (`auto_push.rs`,
//! `REFERENCE_LOCK_REFUSAL`).
//! A repeat push repairs it only while the lock goes in time:
//! the retries the push already makes are a second and a half,
//! and a lock a killed Git
//!  left stands until its holder is proven gone (`stale_ref_locks.rs`),
//! so the reference can name an older commit for minutes.
//! Every question this repository answers about the branch's remote is answered from that reference,
//! so a stale one
//!  says a commit the remote already has is held and holds a branch back behind a verdict the remote already carries.
//!  The wrapper knows what its push delivered,
//! so it writes the reference itself,
//! with Git's own reflog words for such an update
//!  ("update by push"),
//! and with the retries the push itself gets when the lock still stands.
//! The write names the reference's full name and
//!  expects the value it read,
//! because a short name such as `origin/main` is accepted by `update-ref` and changes nothing
//!  (measured on 2026-11-11 with Git 2.56.0),
//! and because a value that moved meanwhile is another writer's to keep.
//! The reference written is the branch's remote-tracking one and no other,
//! and the value written is the commit the push delivered and never a name:
//! a short name is answered from `refs/heads` and `refs/tags` as readily as from `refs/remotes`,
//! and `update-ref` resolves the value it is given when it writes,
//! so either would move or publish something this push never carried.
//! A reference that already names the delivered commit or a commit above it is left alone.
//! A reference that is absent is created,
//! as Git itself creates the branch's remote-tracking reference when the remote's fetch refspec maps the branch
//!  (`transport_update_tracking_ref` in Git's `transport.c`,
//! measured with Git 2.56.0 on 2026-11-11),
//! and the creation expects the old value to be absent,
//! so a reference another writer created meanwhile is that writer's.
//! A write the repair could not make is said instead of left silent,
//! and the sentence names what stopped it:
//! a lock another Git holds,
//! another writer that wrote the reference first,
//! a reference Git could not read,
//! or a name that is no remote-tracking reference or no commit name.
//! Nothing a person asked for waits on this:
//! it runs after the push answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repaired = bringUpToDate(git, 'refs/remotes/origin/main', delivered); // true when the reference names the commit
//! ```

use crate::auto_push::{PUSH_LOCK_PAUSE, PUSH_LOCK_RETRIES};
use crate::auto_push_reconcile_basics_and_limits::{is_ancestor, is_ancestor_awaiting};
use crate::auto_push_single_flight::{PushGit, run_query, run_query_awaiting};
use crate::diagnostic_log::warn;
use crate::service_fault::{TRACKING_REF_BEFORE_THE_READ, reach};
use crate::service_protocol::is_object_name;
use std::ffi::OsString;

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// The reflog message Git writes for an update a push made.
pub const UPDATE_BY_PUSH: &str = "update by push";

/// What a query of this module answers,
/// and whether it ran at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { code: number | null; stdout: string };
/// ```
struct Answer {
    /// The exit status,
    /// or nothing when a signal ended the process.
    code: Option<i32>,
    /// Standard output as text.
    stdout: String,
    /// Standard error as text,
    /// which is what says why a write was refused.
    stderr: String,
}

impl Answer {
    /// Whether this query said yes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ok = (answer) => answer.code === 0;
    /// ```
    fn ok(&self) -> bool {
        return self.code == Some(0);
    }

    /// The answer's text,
    /// trimmed,
    /// or empty.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const text = (answer) => answer.stdout.trim();
    /// ```
    fn text(&self) -> String {
        return self.stdout.trim().to_owned();
    }

    /// What:
    ///  Whether the answer is `rev-parse --verify --quiet` saying the reference is not there.
    /// Why:
    ///   `--quiet` makes a reference that is not there the only refusal that says nothing at all,
    /// so an absent reference is told apart from one Git could not read,
    /// and only the absent one is created
    ///  (measured with Git 2.56.0 on 2026-11-11).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const missing = (answer) => answer.code === 1 && answer.stdout.trim() === '' && answer.stderr.trim() === '';
    /// ```
    fn said_the_reference_is_missing(&self) -> bool {
        return self.code == Some(1) && self.text().is_empty() && self.stderr.trim().is_empty();
    }
}

/// The arguments of one query from string literals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argv = (...parts) => parts.map(String);
/// ```
fn argv(parts: &[&str]) -> Vec<OsString> {
    return parts.iter().map(OsString::from).collect();
}

/// One query,
/// and nothing when Git could not be started at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ask = (git, parts) => { try { const out = runQuery(git, argv(parts)); return { code: out.code, stdout: text(out.stdout) } } catch { return undefined } };
/// ```
fn ask(git: &PushGit, parts: &[&str]) -> Option<Answer> {
    let output = run_query(git, argv(parts).as_slice()).ok()?;
    return Some(Answer {
        code: output.code,
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into_owned(),
        stderr: String::from_utf8_lossy(output.stderr.as_slice()).into_owned(),
    });
}

/// The same query as a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const askAwaiting = async (git, parts) => { try { const out = await runQueryAwaiting(git, argv(parts)); return { code: out.code, stdout: text(out.stdout) } } catch { return undefined } };
/// ```
async fn ask_awaiting(git: &PushGit, parts: &[&str]) -> Option<Answer> {
    let output = run_query_awaiting(git, argv(parts).as_slice()).await.ok()?;
    return Some(Answer {
        code: output.code,
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into_owned(),
        stderr: String::from_utf8_lossy(output.stderr.as_slice()).into_owned(),
    });
}

/// What:
///  The full ref name of the reference a branch's upstream is,
///  and nothing when the name is no remote-tracking reference.
/// Why:
///   A write takes the full name:
/// `update-ref origin/main <commit>` is accepted and changes nothing,
/// as Git 2.56.0 was measured to do on
///  2026-11-11,
/// so a repair written from the short name would report success and leave the reference behind.
/// The name is derived rather than taken from Git's answer,
/// because `rev-parse` reads a short name from `refs/heads` and `refs/tags` as readily as from `refs/remotes`:
/// a repository that holds a local branch named `origin/main` would have that branch written,
/// and the remote-tracking reference this repair exists for would be left behind.
///   Whether Git holds the reference is not asked here,
/// because a reference that is absent is one this repair creates:
/// Git itself creates the branch's remote-tracking reference on a push whose fetch refspec maps the branch,
/// and the name of that reference is known from the branch's upstream whether the reference is there or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullRef = (name) => trackingRef(name);
/// ```
pub fn resolve(_git: &PushGit, name: &str) -> Option<String> {
    return remote_tracking_ref(name);
}

/// What:
///  The full remote-tracking reference name a name a branch's upstream is answered with means,
///  and nothing for a name that is no remote-tracking reference.
/// Why:
///   The reference a push updates is the branch's upstream,
/// which Git abbreviates as `origin/main`;
/// the repair writes the reference under `refs/remotes` and nothing else,
///  and a name that already spells a ref out is read only where it is one of those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trackingRef = (name) => name.startsWith('refs/remotes/') ? name : name.startsWith('refs/') ? undefined : `refs/remotes/${name}`;
/// ```
fn remote_tracking_ref(name: &str) -> Option<String> {
    if name.starts_with("refs/remotes/") {
        return Some(name.to_owned());
    }
    if name.starts_with("refs/") {
        return None;
    }
    return Some(format!("refs/remotes/{name}"));
}

/// The same,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullRefAwaiting = async (name) => trackingRef(name);
/// ```
pub async fn resolve_awaiting(_git: &PushGit, name: &str) -> Option<String> {
    return remote_tracking_ref(name);
}

/// What:
///  Where a full reference is:
///  naming a commit,
///  absent,
///  or beyond Git's answer.
/// Why:
///   An absent reference is created and a named one is compared with the delivery,
/// so the two are told apart before anything is written;
/// an answer Git could not give is never read as an absent reference,
/// because creating a reference that is there would replace the commit it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type At = { kind: 'commit'; value: string } | { kind: 'absent' } | { kind: 'unreadable' };
/// ```
enum At {
    /// The reference names this commit.
    Commit(String),
    /// The reference is not there.
    Absent,
    /// Git could not say.
    Unreadable,
}

/// What:
///  Where the reference is,
///  asked once and answered with what was read.
/// Why:
///   `rev-parse --verify --quiet` says a reference that is not there with an exit status and no text at all,
/// so absence is told apart from an answer Git could not give
/// (measured with Git 2.56.0 on 2026-11-11),
/// and the value this returns is the one the ancestry question is asked about and the one the write expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whereItIs = (git, full) => { const answer = ask(git, ['rev-parse', '--verify', '--quiet', full + '^{commit}']); return !answer ? unreadable : answer.ok() ? { kind: 'commit', value: answer.text() } : answer.saidTheReferenceIsMissing() ? absent : unreadable; };
/// ```
fn where_it_is(git: &PushGit, full: &str) -> At {
    let peeled: String = format!("{full}^{{commit}}");
    let Some(answer) = ask(git, &["rev-parse", "--verify", "--quiet", peeled.as_str()]) else { return At::Unreadable };
    if answer.ok() {
        return At::Commit(answer.text());
    }
    return match answer.said_the_reference_is_missing() {
        true => At::Absent,
        false => At::Unreadable,
    };
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whereItIsAwaiting = async (git, full) => { const answer = await askAwaiting(git, ['rev-parse', '--verify', '--quiet', full + '^{commit}']); return !answer ? unreadable : answer.ok() ? { kind: 'commit', value: answer.text() } : answer.saidTheReferenceIsMissing() ? absent : unreadable; };
/// ```
async fn where_it_is_awaiting(git: &PushGit, full: &str) -> At {
    let peeled: String = format!("{full}^{{commit}}");
    let Some(answer) = ask_awaiting(git, &["rev-parse", "--verify", "--quiet", peeled.as_str()]).await else { return At::Unreadable };
    if answer.ok() {
        return At::Commit(answer.text());
    }
    return match answer.said_the_reference_is_missing() {
        true => At::Absent,
        false => At::Unreadable,
    };
}

/// What:
///  Whether the value a reference holds is `delivered` or a commit above it,
///  asked about the value itself rather than about the reference.
/// Why:
///   A reference that holds the delivered commit is what the push would have written,
/// so it is left alone;
/// the value asked about is the one the write then expects,
/// so a reference another writer advanced between the two is never the reason an older commit replaces a newer one.
/// A question Git cannot answer is not a reason to leave a stale reference stale,
///  and the write's expected value answers it exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheDelivery = (git, at, delivered) => isAncestor(git, delivered, at) === true;
/// ```
fn holds_the_delivery(git: &PushGit, at: &str, delivered: &str) -> bool {
    return matches!(is_ancestor(git, delivered, at), Ok(Some(true)));
}

/// The same question,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsTheDeliveryAwaiting = async (git, at, delivered) => (await isAncestorAwaiting(git, delivered, at)) === true;
/// ```
async fn holds_the_delivery_awaiting(git: &PushGit, at: &str, delivered: &str) -> bool {
    return matches!(is_ancestor_awaiting(git, delivered, at).await, Ok(Some(true)));
}

/// What:
///  What stopped a repair,
///  so the sentence a person reads names the real cause.
/// Why:
///   The states are told apart by what Git said,
/// and each has one cause:
/// a lock another Git holds (`stale_ref_locks.rs`),
/// another writer that wrote the reference first,
/// a reference Git could not read,
/// a name that is no remote-tracking reference,
///  or a value that is no commit name.
/// A sentence that named the lock for all of them would blame a lock that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Impediment = 'locked' | 'another-writer' | 'unreadable' | 'not-a-tracking-reference' | 'not-a-commit';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Impediment {
    /// The reference's lock stands, and nothing may clear it without a proof its holder is gone.
    Locked,
    /// Another writer wrote the reference first, and its value is the one to keep.
    AnotherWriter,
    /// Git could not read the reference, so nothing could be decided about it.
    Unreadable,
    /// The name is not a reference under `refs/remotes/`, so this repair writes nothing there.
    NotATrackingReference,
    /// The value the push was said to deliver is no commit name.
    NotACommit,
}

/// What:
///  What a refused write says it was refused by,
///  from Git's own words.
/// Why:
///   Git reports every refusal of `update-ref` as `cannot lock ref` and names the reason after it:
/// `Unable to create '[...].lock': File exists` for a lock another Git holds,
/// `reference already exists` for a creation whose reference another writer made first,
/// and `is at <commit> but expected <commit>` for a value that moved
/// (measured with Git 2.56.0 on 2026-11-11).
/// So the reason is read from the rest of the line and never from the phrase they share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const impedimentOf = (answer) => answer.stderr.includes('Unable to create') ? 'locked' : 'another-writer';
/// ```
fn impediment_of(answer: &Answer) -> Impediment {
    return match answer.stderr.contains("Unable to create") {
        true => Impediment::Locked,
        false => Impediment::AnotherWriter,
    };
}

/// One `git update-ref` of the full reference,
/// expecting the value that was read,
/// and what stopped it when it did not write.
/// Why:
///   The reference is written by Git itself,
/// with Git's own message,
/// so the reflog reads like every other push's row;
///  the expected value makes the write a compare-and-swap,
/// so a reference that moved between the read and the write
///  keeps the other writer's value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wrote = (git, full, delivered, old) => { const answer = ask(git, ['update-ref', '-m', 'update by push', full, delivered, old]); return !answer ? unreadable : answer.ok() ? undefined : impedimentOf(answer); };
/// ```
fn write_the_reference(git: &PushGit, full: &str, delivered: &str, old: &str) -> Result<(), Impediment> {
    let Some(answer) = ask(git, &["update-ref", "-m", UPDATE_BY_PUSH, full, delivered, old]) else { return Err(Impediment::Unreadable) };
    return match answer.ok() {
        true => Ok(()),
        false => Err(impediment_of(&answer)),
    };
}

/// The same write,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wroteAwaiting = async (git, full, delivered, old) => { const answer = await askAwaiting(git, ['update-ref', '-m', 'update by push', full, delivered, old]); return !answer ? unreadable : answer.ok() ? undefined : impedimentOf(answer); };
/// ```
async fn write_the_reference_awaiting(git: &PushGit, full: &str, delivered: &str, old: &str) -> Result<(), Impediment> {
    let Some(answer) = ask_awaiting(git, &["update-ref", "-m", UPDATE_BY_PUSH, full, delivered, old]).await else { return Err(Impediment::Unreadable) };
    return match answer.ok() {
        true => Ok(()),
        false => Err(impediment_of(&answer)),
    };
}

/// What:
///  One `git update-ref` that creates the reference,
///  and what stopped it when it did not.
/// Why:
///   Git creates a branch's remote-tracking reference on a push whose fetch refspec maps the branch,
/// so a repair of a reference that is absent creates it the same way;
/// the empty expected value is Git's own rule that the reference must not exist,
///  which is what keeps a reference another writer made meanwhile
/// (measured with Git 2.56.0 on 2026-11-11:
/// `update-ref <ref> <commit> ''` refuses a reference that is there).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const created = (git, full, delivered) => { const answer = ask(git, ['update-ref', '-m', 'update by push', full, delivered, '']); return !answer ? unreadable : answer.ok() ? undefined : impedimentOf(answer); };
/// ```
fn create_the_reference(git: &PushGit, full: &str, delivered: &str) -> Result<(), Impediment> {
    let Some(answer) = ask(git, &["update-ref", "-m", UPDATE_BY_PUSH, full, delivered, ""]) else { return Err(Impediment::Unreadable) };
    return match answer.ok() {
        true => Ok(()),
        false => Err(impediment_of(&answer)),
    };
}

/// The same creation,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const createdAwaiting = async (git, full, delivered) => { const answer = await askAwaiting(git, ['update-ref', '-m', 'update by push', full, delivered, '']); return !answer ? unreadable : answer.ok() ? undefined : impedimentOf(answer); };
/// ```
async fn create_the_reference_awaiting(git: &PushGit, full: &str, delivered: &str) -> Result<(), Impediment> {
    let Some(answer) = ask_awaiting(git, &["update-ref", "-m", UPDATE_BY_PUSH, full, delivered, ""]).await else { return Err(Impediment::Unreadable) };
    return match answer.ok() {
        true => Ok(()),
        false => Err(impediment_of(&answer)),
    };
}

/// What:
///  Makes the reference `upstream` names hold `delivered`,
///  and answers what stopped it when it does not.
/// Why:
///   The reference a push updates is what every later question about the branch's remote is answered from,
/// so it must name what the
///  remote holds as soon as the push says it delivered,
/// and not wait for a later push or a fetch.
///   Only a remote-tracking reference is written,
/// and only with a commit name:
/// a branch name given to `update-ref` is resolved when it is written,
/// so a commit that landed after the push and never reached the remote would be published by this repair.
/// A reference that is absent is created,
/// as Git creates it,
/// and the creation expects it to be absent.
/// A write can meet the same held lock the push met,
///  so it is tried as often and as patiently as the push.
///   The value read is the one the ancestry question is asked about and the one the write expects,
/// so a reference another writer advanced is left to that writer rather than moved to an older commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!full.startsWith('refs/remotes/')) return 'not-a-tracking-reference';
/// if (!isObjectName(delivered)) return 'not-a-commit';
/// let why; for (let attempt = 0; attempt <= RETRIES; attempt += 1) { reach(BEFORE_THE_READ); const at = whereItIs(git, full); if (at.kind === 'unreadable') return 'unreadable'; if (at.kind === 'commit' && holdsTheDelivery(git, at.value, delivered)) return undefined; const wrote = at.kind === 'absent' ? create(git, full, delivered) : write(git, full, delivered, at.value); if (!wrote) return undefined; why = wrote; if (attempt < RETRIES) sleep(PAUSE * (attempt + 1)); }
/// warn(stillBehind(full, delivered, why)); return why;
/// ```
pub fn bring_up_to_date(git: &PushGit, full: &str, delivered: &str) -> Option<Impediment> {
    if !full.starts_with("refs/remotes/") {
        return Some(Impediment::NotATrackingReference);
    }
    if !is_object_name(delivered) {
        return Some(Impediment::NotACommit);
    }
    let mut why: Impediment = Impediment::Unreadable;
    for attempt in 0..=PUSH_LOCK_RETRIES {
        // A control stops here to move the reference between the repair's look and its write, as another push or a fetch does.
        reach(TRACKING_REF_BEFORE_THE_READ);
        let stopped: Impediment = match where_it_is(git, full) {
            At::Unreadable => return Some(Impediment::Unreadable),
            At::Commit(at) if holds_the_delivery(git, at.as_str(), delivered) => return None,
            At::Commit(at) => match write_the_reference(git, full, delivered, at.as_str()) {
                Ok(()) => return None,
                Err(impediment) => impediment,
            },
            At::Absent => match create_the_reference(git, full, delivered) {
                Ok(()) => return None,
                Err(impediment) => impediment,
            },
        };
        why = stopped;
        // A lock another Git holds ends, and a value another writer moved is read again; nothing else is waited for.
        if attempt < PUSH_LOCK_RETRIES {
            std::thread::sleep(PUSH_LOCK_PAUSE * (attempt + 1));
        }
    }
    warn(TAG, still_behind(full, delivered, why).as_str());
    return Some(why);
}

/// What:
///  The same repair as a task of the runtime,
///  for the push the service makes.
/// Why:
///   The service's push runs on the runtime,
/// where a synchronous Git start is refused,
/// and the writing of the reference must be here for that face too.
/// It answers what the synchronous face answers, on the same reference and the same value read,
/// so two writers of one reference see one rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the synchronous face's steps, awaited one at a time
/// ```
pub async fn bring_up_to_date_awaiting(git: &PushGit, full: &str, delivered: &str) -> Option<Impediment> {
    if !full.starts_with("refs/remotes/") {
        return Some(Impediment::NotATrackingReference);
    }
    if !is_object_name(delivered) {
        return Some(Impediment::NotACommit);
    }
    let mut why: Impediment = Impediment::Unreadable;
    for attempt in 0..=PUSH_LOCK_RETRIES {
        reach(TRACKING_REF_BEFORE_THE_READ);
        let stopped: Impediment = match where_it_is_awaiting(git, full).await {
            At::Unreadable => return Some(Impediment::Unreadable),
            At::Commit(at) if holds_the_delivery_awaiting(git, at.as_str(), delivered).await => return None,
            At::Commit(at) => match write_the_reference_awaiting(git, full, delivered, at.as_str()).await {
                Ok(()) => return None,
                Err(impediment) => impediment,
            },
            At::Absent => match create_the_reference_awaiting(git, full, delivered).await {
                Ok(()) => return None,
                Err(impediment) => impediment,
            },
        };
        why = stopped;
        if attempt < PUSH_LOCK_RETRIES {
            tokio::time::sleep(PUSH_LOCK_PAUSE * (attempt + 1)).await;
        }
    }
    warn(TAG, still_behind(full, delivered, why).as_str());
    return Some(why);
}

/// What:
///  What is said when the tracking reference could not be brought up to date,
///  naming the one thing that stopped it,
///  so the state is named instead of silent and the cause is the real one.
/// Why:
///  Git can refuse this write for as long as another Git holds the reference's lock,
/// and nothing may clear that lock without a proof that
///  its holder is gone (`stale_ref_locks.rs`);
/// another writer can write the reference first, and its value is the one to keep;
/// and Git can be unable to read it at all.
/// A sentence naming a lock for all of them would blame a lock that is not there,
///  which is what a person was told before this repair asked Git why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillBehind = (full, delivered, why) => `the push delivered ${delivered}, but ${cause(why)}`;
/// ```
pub fn still_behind(full: &str, delivered: &str, impediment: Impediment) -> String {
    let cause: String = match impediment {
        Impediment::Locked => format!(
            "another Git held the lock of {full} through every retry, so it still names what it named before and every question about the branch's \
             remote is answered from there until a push or a fetch writes it"
        ),
        Impediment::AnotherWriter => format!(
            "another process wrote {full} first, so this repair left that writer's value there rather than replace it with an older commit"
        ),
        Impediment::Unreadable => format!("Git could not read {full}, so nothing could be written there"),
        Impediment::NotATrackingReference => format!("{full} is not a remote-tracking reference, and this repair writes no other"),
        Impediment::NotACommit => format!("{delivered} is not a commit name, so {full} was not written"),
    };
    return format!("the push delivered {delivered}, but {cause}");
}

/// Controls for the repair stay out of the release executable.
#[cfg(test)]
#[path = "push_tracking_ref_repair_tests.rs"]
mod tests;
