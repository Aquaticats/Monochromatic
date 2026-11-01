# Cli-git post-commit tree latency

## Symptom

A PATH-shadowed `git commit` completed the native commit and printed its success line,
but the wrapper did not exit for another eighteen seconds.
The full dirty-worktree fixture measured `18.64 s`.

The visible hk work made hk stashing a plausible cause,
but the native Git and hk path completed in less than one second in the same disposable repository.

## Environment

The reproducer used:

- cli-git from `package/git-policy/cli`;
- hk pre-commit stashing with five unstaged files;
- one staged `README.md` change;
- the enabled-by-default `final-newline` policy;
- a landed tree comparable in size to this repository.

The fixture lived at `/var/home/user/temp/agent/hk-latency-20260711`.

## Reproduction

The slow command was:

```console
/usr/bin/time --format='elapsed=%e user=%U system=%S' \
  /var/home/user/Monochromatic/node_modules/.bin/git \
  commit --message='fixture c' -- README.md
```

It printed the landed commit before eventually reporting:

```text
elapsed=18.64 user=6.93 system=11.64
```

A repeat after the core final-newline fix used the same dirty-worktree shape and reported:

```text
elapsed=0.74 user=0.42 system=0.53
```

A main-repository run then measured `20.85 s` while its trusted configuration snapshot still contained
the pre-fix forbidden-strings plugin.
A later run with cli-git rebuilt but the trusted snapshot unchanged measured `16.77 s`.
Those observations isolated the same unchanged-candidate fan-out in scanner materialization
and proved that rebuilding the executable alone could not update already-trusted bundled policy code.

## Diagnosis

The wrapper runs a post-commit lifecycle after native Git succeeds.
`package/git-policy/cli/src/policy-engine/post-commit-facts.ts` exposed every file in the landed tree
as an immutable candidate,
but labeled every candidate `unchanged`.

`package/git-policy/cli/src/policy-engine/final-newline-policy.ts` then read every regular candidate.
Each lazy byte read invoked `git cat-file` separately.
The landed tree therefore turned one changed-file commit into repository-wide subprocess fan-out.

The process behavior distinguished this from native commit latency:
Git printed the successful commit immediately,
then the PATH-shadowed wrapper remained active with no hk output.
The same repository's `/usr/bin/git commit` plus hk completed in less than one second.

## Root cause

Post-commit facts described the complete landed tree but did not distinguish its changed paths.
Content policies consequently treated unchanged tree entries as work requiring blob reads or scanner materialization.

The expensive operation was not required for correctness.
The private-index transaction had already checked predicted changed content before forwarding the commit.
Post-commit enforcement needs ground-truth bytes for the landed delta,
while complete tree metadata may remain available to policies that need it.

## Fix

`loadLandedCandidates` now loads these facts concurrently:

- complete tree metadata from `git ls-tree`;
- changed paths from one NUL-delimited `git diff-tree --root -r -z -m` invocation.

Landed candidates receive `modified` only when their path is in that diff.
Other retained tree entries receive `unchanged`.

The core `final-newline` policy and the forbidden-strings plugin now skip unchanged post-commit candidates
before requesting bytes or materializing scanner inputs.
Root commits remain covered because `git diff-tree --root` reports every initial path.
Merge commits remain covered because `-m` reports differences against each parent
and the implementation deduplicates paths.

File-enforcer must copy the canonical forbidden-strings source into cli-git's statically shipped optional-policy source.
After rebuilding cli-git,
`git cli-git trust --yes` must replace an existing trusted TypeScript bundle
that contains the old policy implementation.

The TypeScript trust builder now calls Rolldown directly instead of using tsdown's programmatic middle layer.
Rolldown is imported only when a trust build is required,
and `await using` closes the bundle and its native workers explicitly.
The real main-repository trust rebuild completed in `0.29 s`.

## Verification

The following checks passed:

- post-commit lifecycle regression proving changed and unchanged classification;
- final-newline regression proving unchanged landed bytes remain unread;
- forbidden-strings regression proving only landed-delta candidates reach scanner materialization;
- dirty-worktree commit with hk stashing five files,
   reduced from `18.64 s` to `0.74 s`;
- main-repository commits with the old trusted forbidden-strings bundle,
  which reproduced the second fan-out at `20.85 s` and `16.77 s`;
- real main-repository TypeScript trust rebuild through direct Rolldown,
   which completed in `0.29 s`.

The measured fixed command is below the required `2,000 ms` real-Git operation ceiling.

## Rejected hypotheses

### hk stashing caused the delay

Rejected because native Git plus the same hk dirty-worktree path completed in less than one second.
The delayed process remained after hk had restored unstaged changes and Git had printed commit success.

### Transaction finalization caused the delay

Rejected because transaction finalization performs bounded commit and tree identity checks.
The long tail correlated with repository file count and disappeared when unchanged post-commit blob reads were removed.

### Mise bootstrap dominated the command

Rejected because bootstrap also appeared in the fixed run,
which completed in `0.74 s`.

## Upstream status

No upstream report is appropriate.
The defect was in cli-git's post-commit candidate consumption,
not Git,
 hk,
 mise,
 or the Pi process harness.
