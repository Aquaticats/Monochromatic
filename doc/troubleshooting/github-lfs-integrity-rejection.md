# GitHub rejects a Git 2.56.0 push with GH009 while retaining the local commit

## Symptom

On 2026-10-15,
auto-pushing commit `634b29b15e99197040ad12529cce848258f5a57c` failed with:

```text
remote: error: GH009: Git LFS object integrity could not be checked. Please contact GitHub support.
 ! [remote rejected]       main -> main (pre-receive hook declined)
cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.
```

The commit command completed successfully because the commit was saved locally.
A direct remote query still returned its parent,
`6d81bcc32121e4fc8c14ba6975b88c222d5ee0c4`.
This was an actual remote rejection,
separate from the advisory [push-wait notice](cli-git-push-wait-notice.md).

## Root cause boundary

The remote reported failure of its LFS integrity check.
The underlying GitHub server condition is unknown;
later success does not identify that cause.
A Git LFS maintainer attributes this diagnostic to GitHub in
[git-lfs/git-lfs issue 4787](https://github.com/git-lfs/git-lfs/issues/4787#issuecomment-995811913).
The deciding GitHub service implementation was not available for inspection.
No claim about its internal validation algorithm or a local LFS defect is established.

The local commit-preservation behavior is explicit in
`package/git-policy/cli/src/native/auto_push.rs:265`:

```text
// package/git-policy/cli/src/native/auto_push.rs:265
///   The commit has happened by the time this runs and Git ignores a post-commit hook's status,
///  so a failed push or a failed coordination is printed and returned as `Failed`, and never fails the commit.
```

The failure note is declared separately in
`package/git-policy/cli/src/native/auto_push.rs:45`:

```rust
// package/git-policy/cli/src/native/auto_push.rs:45
pub const PUSH_FAILED_NOTE: &str =
    "cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.";
```

The repository-owned wrapper source was inspected at the failed commit.
No third-party client patch was implicated by this evidence.

## Verification

The forwarded Git version was `2.56.0`.
The command entry was `.cache/cli-git-bin/git`.
GitHub's server version was not exposed.
The rejected commit changed only a TypeScript test and its Markdown handover.
That observation does not establish which historical LFS references the remote attempted to validate.

The following commands were run from the repository root:

```sh
# Repository root: read the actual remote instead of relying on a cached tracking ref.
git --version
git show --stat --oneline 634b29b15e
git ls-remote origin refs/heads/main
git rev-parse HEAD
```

Observed failure:

- The commit's automatic push returned `GH009` and a rejected remote update.
- The direct remote query still named the parent while local `HEAD` named the saved commit.

Observed success:

- One subsequent ordinary `git push origin main` exited zero.
- A fresh remote query and local `HEAD` both named
  `634b29b15e99197040ad12529cce848258f5a57c`.

No deliberate reproduction of the remote failure was available.
These are incident observations,
not a deterministic reproducer or a guarantee that retry always succeeds.

## Verified recovery

After verifying that the local commit was absent remotely,
the ordinary push succeeded without changing commits,
LFS configuration,
or repository history:

```sh
# Repository root: retry only the intended branch and verify the resulting ref.
git push origin main
git ls-remote origin refs/heads/main
git rev-parse HEAD
```

The retry can transfer the pending branch updates and remains subject to concurrent branch movement.
It does not diagnose or repair an unknown service-side condition.

## What does not work

- A successful commit exit status alone did not prove successful backup to GitHub.
- The narrow tracker query containing both `GH009` and `integrity` returned no results.
  The broader exact phrase `LFS object integrity` found the existing Git LFS issue.
- No force push,
  history migration,
  LFS reinstall,
  or configuration bypass was tried or needed.
  Such changes are not justified by this incident's evidence.

## Upstream filing decision

The `.out-of-scope/` inventory contains no matching exemption.
The existing issue and its comments were read in full.
Its later commenter also reports eventual push success,
but that separate incident does not establish this incident's cause.
There is no new source diagnosis or deterministic reproduction to add.
No upstream message was sent.

1.  Fault:
    a remote rejection is observed;
    the responsible internal condition is unestablished.
2.  Fixability:
    the service implementation is unavailable,
    so no specific upstream fix is established.
3.  Supported use:
    an ordinary existing-branch push was attempted and subsequently accepted.
4.  Contribution policy:
    no client-code contribution or new report is proposed;
    the maintainer directs this diagnostic to GitHub support.
5.  Likely correction:
    the available thread gives no decision about this incident.
6.  Prototype:
    none;
    there is no identified source defect to patch.

The filing artifact is an explicit nothing-to-add record for
[issue 4787](https://github.com/git-lfs/git-lfs/issues/4787).

## Separate protected-branch rejection on 2026-10-15

A later automatic push of `6973251c8342005567353fad9e1c7b3cb4ebdb51`
failed at a different visible boundary:

```text
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: error:
 ! [remote rejected]       main -> main (protected branch hook declined)
```

This incident is not attributed to the LFS failure.
The server provided no specific rejected requirement.
The local commit remained saved,
while the direct remote query still named parent `ef8f58d9226db6b10e45c0676aa41597c5b1111c`.

Read-only checks used the same Git `2.56.0` and repository-owned wrapper:

```sh
# Repository root: inspect the actual required policy and original local signature.
git verify-commit 6973251c83
gh api repos/Aquaticat/Monochromatic/branches/main/protection
gh api repos/Aquaticat/Monochromatic/commits/6973251c8342005567353fad9e1c7b3cb4ebdb51 --jq '{sha: .sha, verification: .commit.verification.verified, reason: .commit.verification.reason}'
```

Local signature verification succeeded.
The protection response required signatures,
reported strict status checking with empty context/check arrays,
required conversation resolution,
and reported `lock_branch: false`.
The commit endpoint returned HTTP `422` with `No commit found for SHA`.
These reads do not identify the rejected requirement or prove a GitHub signature-verification result.

One unchanged ordinary `git push origin main` then exited zero,
reporting `ef8f58d922..6973251c83 main -> main`.
No protection setting,
commit signature,
or history was changed.
As with the LFS incident,
successful retry establishes recovery only.
The internal cause and deterministic reproduction remain unavailable;
no backend patch or new upstream report is justified by these observations.

## Protected-branch rejection followed by concurrent publication on 2026-10-15

The automatic push for local commit `2447ba4224` emitted the same visible
`GH006: Protected branch update failed for refs/heads/main`
and `protected branch hook declined` messages.
The commit remained saved locally.
The diagnostic again did not identify a rejected requirement.

Before retrying,
`git ls-remote origin refs/heads/main` returned
`9e4b09e2d180020194cfbabc9d5d8d38c9c8e449`.
The local log showed concurrent repository work had published that commit,
and `git merge-base --is-ancestor 2447ba4224 9e4b09e2d180020194cfbabc9d5d8d38c9c8e449`
exited zero.
The rejected local commit was therefore already reachable from the actual remote branch.
No additional push,
protection change,
or history rewrite was needed.
This establishes delivery through concurrent publication,
not the cause of the preceding rejection.

## Another protected-branch rejection during the Depot evidence closeout

The automatic push of documentation commit `d50e251a09756504fa07ac004cf3cfc66a1ebffe`
on October 15 local time,
or October 16 UTC,
emitted:

```text
# GitHub remote response relayed by the repository-owned Git wrapper.
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: error:
 ! [remote rejected]       main -> main (protected branch hook declined)
error: failed to push some refs to 'https://github.com/Aquaticats/Monochromatic'
cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.
```

The source commit remained saved.
Before any retry,
`git ls-remote --heads origin main` returned
`c34b44410c5df10f0dd1b5c1a27e76d92d3d0182`.
Both containment checks returned exit status zero:

```sh
# Repository root, Git 2.56.0, against the directly queried remote head.
git merge-base --is-ancestor d50e251a09 c34b44410c5df10f0dd1b5c1a27e76d92d3d0182
git merge-base --is-ancestor a361d83d11 c34b44410c5df10f0dd1b5c1a27e76d92d3d0182
```

The rejected commit and its preceding evidence commit were already delivered through concurrent publication.
No additional push was performed for this incident.

Read-only API inspection found:

- The rejected commit's signature was `verified: true` with reason `valid`.
- Main's classic protection required signatures and conversation resolution.
  Strict status checking was enabled,
  but both required context and check arrays were empty.
- Required pull-request reviews and restrictions were absent;
  administrator enforcement and branch locking were disabled.
- The effective rules endpoint listed an automatic Copilot review rule.
- The commit had zero check runs and zero status contexts.
  The combined status endpoint reported `pending` with an empty status array;
  that alone does not identify a failed required check.

The inspected endpoints were the commit,
its check-runs and status,
`branches/main/protection`,
and `rules/branches/main` under `repos/Aquaticats/Monochromatic`.
The server did not name a rejected requirement.
These post-rejection observations establish delivery and current policy,
not a transient signature,
required-check,
or other backend cause.
Protection settings and history were not changed.
This remains separate from the LFS rejection despite sharing the troubleshooting file.
The unexplained protected-branch rejection is tracked in
[repository issue 1032](https://github.com/Aquaticats/Monochromatic/issues/1032).

## Protected-branch rejection on a feature branch on 2026-10-18

The rejection is not limited to `main`.
The automatic push of documentation commit `a2903488bf` to `translation-repair-rebased`
on 2026-10-18 (UTC) emitted:

```text
# GitHub remote response relayed by the repository-owned Git wrapper.
remote: error: GH006: Protected branch update failed for refs/heads/translation-repair-rebased.
remote: error:
 ! [remote rejected]       translation-repair-rebased -> translation-repair-rebased (protected branch hook declined)
error: failed to push some refs to 'https://github.com/Aquaticats/Monochromatic'
cli-git: commit saved locally, but auto-push to origin failed; run `git push` when ready.
```

The pushes before it to the same branch that day went through.
`gh api repos/Aquaticats/Monochromatic/rules/branches/translation-repair-rebased`
listed only the repository ruleset's automatic Copilot review rule.
Before retrying,
`git ls-remote origin refs/heads/translation-repair-rebased` returned the commit's parent
`dcfb4fefd4cc436edda0136f7e06be666ec6c29f`,
so nothing concurrent had published it.
A plain `git push` then delivered `dcfb4fefd4..a2903488bf`.
As with the incidents on `main`,
the retry establishes recovery only,
and the server named no rejected requirement.
