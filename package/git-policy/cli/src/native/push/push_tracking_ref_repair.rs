//! What:
//!  The branch's remote-tracking reference, brought up to date with what a successful push delivered,
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
//! A write the lock refuses through every retry is said instead of left silent.
//! Nothing a person asked for waits on this:
//! it runs after the push answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repaired = bringUpToDate(git, 'refs/remotes/origin/main', delivered); // true when the reference names the commit
//! ```

use crate::auto_push::{PUSH_LOCK_PAUSE, PUSH_LOCK_RETRIES};
use crate::auto_push_reconcile_basics_and_limits::is_ancestor;
use crate::auto_push_single_flight::{PushGit, run_query, run_query_awaiting};
use crate::diagnostic_log::warn;
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
///  and nothing when the name is not a ref or Git could not say.
/// Why:
///   A write takes the full name:
/// `update-ref origin/main <commit>` is accepted and changes nothing,
/// as Git 2.56.0 was measured to do on
///  2026-11-11,
/// so a repair written from the short name would report success and leave the reference behind.
/// A name that is not under `refs/` is refused
///  here rather than written somewhere nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullRef = (git, name) => { const answer = ask(git, ['rev-parse', '--symbolic-full-name', '--verify', name]); const full = answer?.text(); return answer?.ok() && full.startsWith('refs/') ? full : undefined; };
/// ```
pub fn resolve(git: &PushGit, name: &str) -> Option<String> {
    return full_ref_from(git, name, ask);
}

/// What:
///  The full reference a name or a remote-tracking shorthand means,
///  asked with the query given.
/// Why:
///   A branch's upstream is answered as `origin/main`,
/// which is no ref name,
///  and `rev-parse --symbolic-full-name` answers a shorthand with the shorthand itself,
/// so the name is asked as it
///  stands and then as the remote-tracking ref under `refs/remotes`;
/// a name neither answers for is no reference and is
///  never written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const candidate of [name, `refs/remotes/${name}`]) { const said = ask(candidate); if (said?.ok() && said.text().startsWith('refs/')) return said.text() } return undefined;
/// ```
fn full_ref_from(git: &PushGit, name: &str, query: fn(&PushGit, &[&str]) -> Option<Answer>) -> Option<String> {
    let under_remote: String = format!("refs/remotes/{name}");
    for candidate in [name, under_remote.as_str()] {
        let Some(answer) = query(git, &["rev-parse", "--symbolic-full-name", "--verify", candidate]) else { continue };
        let full: String = answer.text();
        if answer.ok() && full.starts_with("refs/") {
            return Some(full);
        }
    }
    return None;
}

/// The same,
/// awaited by a caller that is itself a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fullRefAwaiting = async (git, name) => { const answer = await askAwaiting(git, ['rev-parse', '--symbolic-full-name', '--verify', name]); const full = answer?.text(); return answer?.ok() && full.startsWith('refs/') ? full : undefined; };
/// ```
pub async fn resolve_awaiting(git: &PushGit, name: &str) -> Option<String> {
    let under_remote: String = format!("refs/remotes/{name}");
    for candidate in [name, under_remote.as_str()] {
        let Some(answer) = ask_awaiting(git, &["rev-parse", "--symbolic-full-name", "--verify", candidate]).await else { continue };
        let full: String = answer.text();
        if answer.ok() && full.starts_with("refs/") {
            return Some(full);
        }
    }
    return None;
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
    return answer.ok().then(|| { return answer.text(); });
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
    return answer.ok().then(|| { return answer.text(); });
}

/// What:
///  Whether `full` already names `delivered` or a commit above it,
///  and written so that a reference naming an unrelated commit is not read as one that holds it.
/// Why:
///   A reference that holds the delivered commit is what the push would have written,
/// so it is left alone;
/// a question Git cannot answer is not a reason to leave a stale reference stale,
///  and the write's expected value answers it exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alreadyThere = (git, full, delivered) => isAncestor(git, delivered, full) === true;
/// ```
fn already_there(git: &PushGit, full: &str, delivered: &str) -> bool {
    return matches!(is_ancestor(git, delivered, full), Ok(Some(true)));
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
/// A write can meet the same held lock the push met,
///  so it is tried as often and as patiently as the push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const full = fullRef(git, upstream); if (!full) return false;
/// if (alreadyThere(git, full, delivered)) return true;
/// for (let attempt = 0; attempt <= RETRIES; attempt += 1) { const old = valueOf(git, full); if (old && write(git, full, delivered, old)) return true; if (attempt < RETRIES) sleep(PAUSE * (attempt + 1)); }
/// warn(stillBehind(full, delivered)); return false;
/// ```
pub fn bring_up_to_date(git: &PushGit, full: &str, delivered: &str) -> bool {
    if !full.starts_with("refs/") || delivered.is_empty() {
        return false;
    }
    if already_there(git, full, delivered) {
        return true;
    }
    for attempt in 0..=PUSH_LOCK_RETRIES {
        let Some(old) = value_of(git, full) else { return false };
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await bringUpToDateAwaiting(git, upstream, delivered)
/// ```
pub async fn bring_up_to_date_awaiting(git: &PushGit, full: &str, delivered: &str) -> bool {
    if !full.starts_with("refs/") || delivered.is_empty() {
        return false;
    }
    for attempt in 0..=PUSH_LOCK_RETRIES {
        let Some(old) = value_of_awaiting(git, full).await else { return false };
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
