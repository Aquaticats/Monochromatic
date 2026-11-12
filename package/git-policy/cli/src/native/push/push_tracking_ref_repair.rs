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
//! A write the lock refuses through every retry is said instead of left silent.
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

    /// The answer's first line of text,
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
    return Some(Answer { code: output.code, stdout: String::from_utf8_lossy(output.stdout.as_slice()).into_owned() });
}

/// The same query as a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const askAwaiting = async (git, parts) => { try { const out = await runQueryAwaiting(git, argv(parts)); return { code: out.code, stdout: text(out.stdout) } } catch { return undefined } };
/// ```
async fn ask_awaiting(git: &PushGit, parts: &[&str]) -> Option<Answer> {
    let output = run_query_awaiting(git, argv(parts).as_slice()).await.ok()?;
    return Some(Answer { code: output.code, stdout: String::from_utf8_lossy(output.stdout.as_slice()).into_owned() });
}

/// What:
///  The full ref name of the reference a branch's upstream is,
///  and nothing when the name is not the remote-tracking reference of one,
///  or Git could not say.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullRef = (git, name) => { const full = trackingRef(name); const answer = full === undefined ? undefined : ask(git, ['rev-parse', '--symbolic-full-name', '--verify', full]); return answer?.ok() && answer.text() === full ? full : undefined; };
/// ```
pub fn resolve(git: &PushGit, name: &str) -> Option<String> {
    return full_ref_from(git, name, ask);
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

/// What:
///  Whether an answer names exactly the reference the repair asked about.
/// Why:
///   `rev-parse --symbolic-full-name --verify` answers an ambiguous name with no output at all and still exits `0`
/// (measured on 2026-11-11 with Git 2.56.0),
/// so the text is compared with the expected name and never taken on trust.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesIt = (answer, full) => answer.ok() && answer.text() === full;
/// ```
fn names_exactly(answer: &Answer, full: &str) -> bool {
    return answer.ok() && answer.text() == full;
}

/// What:
///  The full reference a name a branch's upstream is answered with means,
///  asked with the query given.
/// Why:
///   `rev-parse --symbolic-full-name` answers a shorthand with the shorthand itself,
/// so the derived name is asked for and accepted only when Git answers that very name;
/// a name Git does not hold is no reference and is never written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = ask(git, ['rev-parse', '--symbolic-full-name', '--verify', full]); return said && namesIt(said, full) ? full : undefined;
/// ```
fn full_ref_from(git: &PushGit, name: &str, query: fn(&PushGit, &[&str]) -> Option<Answer>) -> Option<String> {
    let full: String = remote_tracking_ref(name)?;
    let answer: Answer = query(git, &["rev-parse", "--symbolic-full-name", "--verify", full.as_str()])?;
    return names_exactly(&answer, full.as_str()).then_some(full);
}

/// The same,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullRefAwaiting = async (git, name) => { const full = trackingRef(name); const answer = full === undefined ? undefined : await askAwaiting(git, ['rev-parse', '--symbolic-full-name', '--verify', full]); return answer && namesIt(answer, full) ? full : undefined; };
/// ```
pub async fn resolve_awaiting(git: &PushGit, name: &str) -> Option<String> {
    let full: String = remote_tracking_ref(name)?;
    let answer: Answer = ask_awaiting(git, &["rev-parse", "--symbolic-full-name", "--verify", full.as_str()]).await?;
    return names_exactly(&answer, full.as_str()).then_some(full);
}

/// The commit a full reference names,
/// and nothing when it names none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const valueOf = (git, full) => { const answer = ask(git, ['rev-parse', '--verify', '--quiet', full + '^{commit}']); return answer?.ok() ? answer.text() : undefined; };
/// ```
fn value_of(git: &PushGit, full: &str) -> Option<String> {
    let peeled: String = format!("{full}^{{commit}}");
    let answer: Answer = ask(git, &["rev-parse", "--verify", "--quiet", peeled.as_str()])?;
    return match answer.ok() {
        true => Some(answer.text()),
        false => None,
    };
}

/// The same,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const valueAwaiting = async (git, full) => { const answer = await askAwaiting(git, ['rev-parse', '--verify', '--quiet', full + '^{commit}']); return answer?.ok() ? answer.text() : undefined; };
/// ```
async fn value_of_awaiting(git: &PushGit, full: &str) -> Option<String> {
    let peeled: String = format!("{full}^{{commit}}");
    let answer: Answer = ask_awaiting(git, &["rev-parse", "--verify", "--quiet", peeled.as_str()]).await?;
    return match answer.ok() {
        true => Some(answer.text()),
        false => None,
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

/// One `git update-ref` of the full reference,
/// expecting the value that was read.
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
/// const wrote = ask(git, ['update-ref', '-m', 'update by push', full, delivered, old])?.ok() === true;
/// ```
fn write_the_reference(git: &PushGit, full: &str, delivered: &str, old: &str) -> bool {
    let answer: Option<Answer> = ask(git, &["update-ref", "-m", UPDATE_BY_PUSH, full, delivered, old]);
    return matches!(answer, Some(said) if said.ok());
}

/// The same write,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wroteAwaiting = async (git, full, delivered, old) => (await askAwaiting(git, ['update-ref', '-m', 'update by push', full, delivered, old]))?.ok() === true;
/// ```
async fn write_the_reference_awaiting(git: &PushGit, full: &str, delivered: &str, old: &str) -> bool {
    let answer: Option<Answer> = ask_awaiting(git, &["update-ref", "-m", UPDATE_BY_PUSH, full, delivered, old]).await;
    return matches!(answer, Some(said) if said.ok());
}

/// What:
///  Makes the reference `upstream` names hold `delivered`,
///  and answers whether it does.
/// Why:
///   The reference a push updates is what every later question about the branch's remote is answered from,
/// so it must name what the
///  remote holds as soon as the push says it delivered,
/// and not wait for a later push or a fetch.
///   Only a remote-tracking reference is written,
/// and only with a commit name:
/// a branch name given to `update-ref` is resolved when it is written,
/// so a commit that landed after the push and never reached the remote would be published by this repair.
/// A write can meet the same held lock the push met,
///  so it is tried as often and as patiently as the push.
///   The value read is the one the ancestry question is asked about and the one the write expects,
/// so a reference another writer advanced is left to that writer rather than moved to an older commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!full.startsWith('refs/remotes/') || !isObjectName(delivered)) return false;
/// for (let attempt = 0; attempt <= RETRIES; attempt += 1) { const old = valueOf(git, full); if (!old) return false; if (holdsTheDelivery(git, old, delivered)) return true; if (write(git, full, delivered, old)) return true; if (attempt < RETRIES) sleep(PAUSE * (attempt + 1)); }
/// warn(stillBehind(full, delivered)); return false;
/// ```
pub fn bring_up_to_date(git: &PushGit, full: &str, delivered: &str) -> bool {
    if !full.starts_with("refs/remotes/") || !is_object_name(delivered) {
        return false;
    }
    for attempt in 0..=PUSH_LOCK_RETRIES {
        let Some(old) = value_of(git, full) else { return false };
        if holds_the_delivery(git, old.as_str(), delivered) {
            return true;
        }
        if write_the_reference(git, full, delivered, old.as_str()) {
            return true;
        }
        // A lock another Git holds ends; nothing else is waited for, and the last try says what it found.
        if attempt < PUSH_LOCK_RETRIES {
            std::thread::sleep(PUSH_LOCK_PAUSE * (attempt + 1));
        }
    }
    warn(TAG, still_behind(full, delivered).as_str());
    return false;
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
/// if (!full.startsWith('refs/remotes/') || !isObjectName(delivered)) return false;
/// for (let attempt = 0; attempt <= RETRIES; attempt += 1) { const old = await valueOf(git, full); if (!old) return false; if (await holdsTheDelivery(git, old, delivered)) return true; if (await write(git, full, delivered, old)) return true; if (attempt < RETRIES) await sleep(PAUSE * (attempt + 1)); }
/// warn(stillBehind(full, delivered)); return false;
/// ```
pub async fn bring_up_to_date_awaiting(git: &PushGit, full: &str, delivered: &str) -> bool {
    if !full.starts_with("refs/remotes/") || !is_object_name(delivered) {
        return false;
    }
    for attempt in 0..=PUSH_LOCK_RETRIES {
        let Some(old) = value_of_awaiting(git, full).await else { return false };
        if holds_the_delivery_awaiting(git, old.as_str(), delivered).await {
            return true;
        }
        if write_the_reference_awaiting(git, full, delivered, old.as_str()).await {
            return true;
        }
        if attempt < PUSH_LOCK_RETRIES {
            tokio::time::sleep(PUSH_LOCK_PAUSE * (attempt + 1)).await;
        }
    }
    warn(TAG, still_behind(full, delivered).as_str());
    return false;
}

/// What:
///  What is said when the tracking reference could not be written,
///  so the state is named instead of silent.
/// Why:
///  Git can refuse this write for as long as another Git holds the reference's lock,
/// and nothing may clear that lock without a proof that
///  its holder is gone (`stale_ref_locks.rs`);
/// the reference then names an older commit until a push or a fetch writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillBehind = (full, delivered) => `the push delivered ${delivered}, but Git could not update ${full} ...`;
/// ```
pub fn still_behind(full: &str, delivered: &str) -> String {
    return format!(
        "the push delivered {delivered}, but Git could not update the tracking reference {full} while another Git holds its lock, so {full} names an \
         older commit until a push or a fetch writes it"
    );
}

/// Controls for the repair stay out of the release executable.
#[cfg(test)]
#[path = "push_tracking_ref_repair_tests.rs"]
mod tests;
