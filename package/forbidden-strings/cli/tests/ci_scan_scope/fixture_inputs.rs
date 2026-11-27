//! What:
//!  The text and the shell the scan-scope fixtures install:
//!  the system Git to run,
//!  the fixture deny-list and the literal it forbids,
//!  the workflow revision from before the scope marker,
//!  and the fake `gh run list` the workflow's own body calls.
//! Why:
//!  Every case needs the same plumbing,
//!  and each constant here exists because a near-miss shape must leak into
//!  the candidate list when the workflow body drops the filter that excludes
//!  it.
//!  The stub therefore rejects any argument or output shape it does not
//!  model,
//!  so a body change surfaces as a failure instead of a silent pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const SENTINEL = 'zzq-scope-violation-sentinel'; const GH_RUN_LIST_STUB = '#!/bin/sh ...'
//! ```

// What:     `const GIT: &str = "/usr/bin/git"` names the system Git.
// Why:      The workflow runs on ubuntu-latest with the runner's stock Git.
//           The developer's `git` on PATH may be the cli-git policy wrapper,
//           which would add policy hooks to fixture commits that CI never
//           runs.
pub(super) const GIT: &str = "/usr/bin/git";

// What:     `const FIXTURE_RULE` is the rule name in the fixture deny-list.
// Why:      Findings report `rule=<name>`, so asserting on the name proves
//           the detecting rule is the fixture one, not the embedded baseline.
pub(super) const FIXTURE_RULE: &str = "ci-scope-fixture-sentinel";

// What:     `const SENTINEL` is the literal the fixture deny-list forbids.
// Why:      A test file content marker that the repository's real rule set
//           does not match keeps the fixture violation out of CI findings on
//           this test source itself.
pub(super) const SENTINEL: &str = "zzq-scope-violation-sentinel";

// What:     `const MARKER_FREE_WORKFLOW` is a pre-marker workflow revision.
// Why:      Runs that executed a revision without the watermark marker
//           scanned only their own push range, so their head commit is not a
//           validated tree; the fixture needs a committed revision that the
//           marker check must reject.
pub(super) const MARKER_FREE_WORKFLOW: &str = "name: forbidden-strings\n\
                                   # Pre-marker revision: push runs here scan only their own push range.\n";

// What:     `const FIXTURE_RULES` is one named tail-format rule forbidding
//           `SENTINEL`.
// Why:      The scan step reads `$RUNNER_TEMP/forbidden-strings.rules.txt`,
//           produced by the preceding workflow step in CI; the fixture writes
//           the same path with a rule set whose only finding is the planted
//           violation.
pub(super) const FIXTURE_RULES: &str = "# Fixture deny-list for the CI scan-scope regression tests.\n\
                             ==> ci-scope-fixture-sentinel <==\n\
                             zzq-scope-violation-sentinel\n";

// What:     `const GH_RUN_LIST_STUB` is a fake `gh run list` built from
//           shell, installed on the fixture PATH ahead of any real `gh`.
// Why:      It implements the documented server-side filter semantics
//           honestly: a filter the caller omits is not applied, so dropping
//           `--status`, `--event`, `--branch`, or `--workflow` from the
//           workflow body leaks the extra fixture rows into the candidate
//           list and the tests fail. The stub also rejects any argument or

pub(super) const GH_RUN_LIST_STUB: &str = r#"#!/bin/sh
set -eu

status_filter=''
event_filter=''
branch_filter=''
workflow_filter=''
json_fields=''
jq_expr=''
while [ "$#" -gt 0 ]; do
  case "$1" in
    run|list)
      shift
      ;;
    --repo|--status|--event|--branch|--workflow|--json|--jq|--limit)
      flag=$1
      value=$2
      if [ "$flag" = '--status' ]; then status_filter=$value; fi
      if [ "$flag" = '--event' ]; then event_filter=$value; fi
      if [ "$flag" = '--branch' ]; then branch_filter=$value; fi
      if [ "$flag" = '--workflow' ]; then workflow_filter=$value; fi
      if [ "$flag" = '--json' ]; then json_fields=$value; fi
      if [ "$flag" = '--jq' ]; then jq_expr=$value; fi
      shift 2
      ;;
    *)
      echo "gh run list stub: unsupported argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ "$json_fields" != 'headSha' ]; then
  echo "gh run list stub: unsupported --json value: $json_fields" >&2
  exit 2
fi
if [ "$jq_expr" != '.[].headSha' ]; then
  echo "gh run list stub: unsupported --jq expression: $jq_expr" >&2
  exit 2
fi

tab=$(printf '\t')
while IFS=$tab read -r conclusion event head_branch workflow head_sha; do
  if [ -n "$status_filter" ] && [ "$conclusion" != "$status_filter" ]; then continue; fi
  if [ -n "$event_filter" ] && [ "$event" != "$event_filter" ]; then continue; fi
  if [ -n "$branch_filter" ] && [ "$head_branch" != "$branch_filter" ]; then continue; fi
  if [ -n "$workflow_filter" ]; then
    matched=''
    if [ "$workflow_filter" = "$workflow" ]; then matched=yes; fi
    if [ "$workflow_filter" = "${workflow%.yml}" ]; then matched=yes; fi
    if [ -z "$matched" ]; then continue; fi
  fi
  printf '%s\n' "$head_sha"
done < "$GH_RUN_FIXTURE"
"#;

// What:     `struct RunRow` is one workflow run listed by the stub.
// Why:      The candidate baseline must come from a run that proved its head
//           tree: a successful push run of this workflow. Each field exists so
//           a test can plant a near-miss row that only leaks into the
