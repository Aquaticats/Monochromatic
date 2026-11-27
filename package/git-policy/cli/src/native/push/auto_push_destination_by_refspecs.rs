//! What:
//!  Where the configured `remote.<name>.push` refspecs send one branch,
//!  judged from one `for-each-ref` answer about the local refs.
//! Why:
//!  The local refs the refspecs can name are asked for in one query;
//!  everything else is decided here,
//!  and a refspec form this service does not interpret,
//!  or one that sends the branch nowhere,
//!  becomes a skip that names the refspec,
//!  never a push;
//!  both faces of the push judge one answer here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await namedByRefspecs({ git, branchRef, settings }): Promise<NamedPush>;
//! ```

use super::auto_push_destination_by_push_default::skip;
/// Import the settings one branch is pushed by,
/// the push or skip they name,
/// the skip of a branch no refspec sends,
/// and the builder of a query's arguments.
use super::auto_push_destination_settings::{
    NamedPush,
    PushSettings,
    argv,
};
/// Import the judgement of the configured refspecs,
/// and the reading of one refspec.
use crate::auto_push_refspec_judging::{
    Judged,
    judge,
    parse_refspec,
};
/// Import the probe's Git runner and its two faces of one query.
use crate::auto_push_single_flight::{
    PushGit,
    SingleFlightError,
    run_query,
    run_query_awaiting,
};
/// Import the finished answer of a bounded query.
use crate::bounded_query::QueryOutput;

///  The one `for-each-ref` question the configured refspecs ask,
/// for the local refs they can name.
/// Why:
///   Both faces of `named_by_refspecs` ask Git the same question,
/// and only about the patterns the refspecs spell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['for-each-ref', '--format=%(refname)', ...patterns];
/// ```
fn refspec_query(settings: &PushSettings) -> Vec<String> {
    let mut patterns: Vec<String> = Vec::new();
    for text in &settings.refspecs {
        if let Some(item) = parse_refspec(text)
            && !item.negative
        {
            patterns.push(item.source);
        }
    }
    patterns.sort();
    patterns.dedup();
    let mut query: Vec<String> = vec![
        String::from("for-each-ref"),
        String::from("--format=%(refname)"),
    ];
    query.extend(patterns);
    return query;
}

///  The destinations of the configured push refspecs for the branch,
/// or the note for why there are none.
/// Why:
///   The local refs the refspecs can name are asked for in one `for-each-ref`;
/// everything else is decided here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namedByRefspecs({ git, branchRef, settings }): Promise<NamedPush>;
/// ```
pub(super) fn named_by_refspecs(
    git: &PushGit,
    branch_ref: &str,
    settings: &PushSettings,
) -> Result<NamedPush, SingleFlightError> {
    let query: Vec<String> = refspec_query(settings);
    let asked = run_query(
        git,
        &argv(&query.iter().map(String::as_str).collect::<Vec<&str>>()),
    )?;
    return named_by_refspecs_of(branch_ref, settings, &asked, &query);
}

///  The same push,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The landing round's push obeys the configured refspecs as a task,
/// and the synchronous face enters the runtime and is refused from there.
/// The question and the reading of its answer are the synchronous face's,
/// so the two cannot name different targets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await namedByRefspecsOnTheRuntime({ git, branchRef, settings });
/// ```
pub(super) async fn named_by_refspecs_awaiting(
    git: &PushGit,
    branch_ref: &str,
    settings: &PushSettings,
) -> Result<NamedPush, SingleFlightError> {
    let query: Vec<String> = refspec_query(settings);
    let asked = run_query_awaiting(
        git,
        &argv(&query.iter().map(String::as_str).collect::<Vec<&str>>()),
    )
    .await?;
    return named_by_refspecs_of(branch_ref, settings, &asked, &query);
}

///  The push the configured refspecs name,
/// judged from one finished answer about the local refs.
/// Why:
///   Both faces of `named_by_refspecs` judge one answer here,
/// so a refspec form either face does not interpret is skipped by both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// judge(branchRef, remote, settings.refspecs, localRefs); // push | skip
/// ```
fn named_by_refspecs_of(
    branch_ref: &str,
    settings: &PushSettings,
    asked: &QueryOutput,
    query: &[String],
) -> Result<NamedPush, SingleFlightError> {
    let branch = settings.branch.as_str();
    let remote = settings.push_remote.as_str();
    if asked.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: query.join(" "),
            code: asked.code,
            stderr: String::from_utf8_lossy(&asked.stderr).into_owned(),
        });
    }
    let local_refs: Vec<String> = String::from_utf8_lossy(&asked.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    return Ok(
        match judge(branch_ref, remote, &settings.refspecs, &local_refs) {
            Judged::Targets(targets) if !targets.is_empty() => NamedPush::Push {
                remote: remote.to_owned(),
                targets,
            },
            Judged::Targets(_) => skip(
                branch,
                &format!(
                    "no `remote.{remote}.push` refspec sends it anywhere, so Git pushes other refs or nothing for it. Add a refspec for it, or push it \
             with `git push {remote} {branch}`."
                ),
            ),
            Judged::Refused(reason) => skip(
                branch,
                &format!(
                    "{reason} Fix `remote.{remote}.push`, or push it with `git push {remote} {branch}`."
                ),
            ),
            Judged::Unjudged(text) => skip(
                branch,
                &format!(
                    "`remote.{remote}.push` has `{text}`, a form this service does not interpret (it reads full ref names and one `*` per side). \
             Push it yourself with `git push`."
                ),
            ),
        },
    );
}
