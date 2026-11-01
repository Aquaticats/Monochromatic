# GitHub Actions at docs revision e1e4aa9 starts unrelated cli-git performance runs on every main push

## Symptom

The workflow named here, `.github/workflows/cli-git-performance.yml`, was deleted on 2026-10-13 in commit `724d70bfa7`
when the performance harness was retired; every statement below describes it as it was at the cited revisions.

Before commit `3ab61f0d3bfddbf017127617591d8b8c7abd4910`,
`.github/workflows/cli-git-performance.yml` started the complete packed lifecycle benchmark for every push to `main`,
even when a commit changed only an unrelated document or package.

The 500-run window from run `29973695489` through run `30441334531`,
returned by GitHub on 2026-07-29,
contained 499 `push` runs and one `pull_request` run.
Every run had a `failure` conclusion.
All 499 push revisions were available in the local Git history for path classification.
The dependency and infrastructure scope evaluated before the narrower version-boundary decision matched 39 revisions
and did not match 460 revisions.
This is a classification of that 499-run sample,
not a claim about every historical run.

The pre-fix red state had a second repository-local cause.
The package had been renamed to `@monochromatic-dev/git-policy-cli`,
but the workflow still asked pnpm to install the old `@monochromatic-dev/cli-git...` dependency closure.
Run `30440155739` consequently reached the build without `rolldown` and emitted:

```text
sh: 1: rolldown: not found
Error: fanout child tasks failed: //package/git-policy/cli:build:js:node
```

## Root cause

### Every main push is an explicit trigger

The pre-fix workflow at
`4054134fe2eb15b9cbac5b92164f28daf2fd49cf:.github/workflows/cli-git-performance.yml:3` to `9`
declared `push` with only a branch filter:

```yaml
on:
  pull_request:
    branches: ['main']
  merge_group:
    branches: ['main']
  push:
    branches: ['main']
```

There was no `paths` filter.
The job in the same revision at lines `14` to `16` also had no condition:

```yaml
jobs:
  lifecycle-latency:
    runs-on: ubuntu-latest
```

GitHub was therefore following the repository's configuration.
The workflow was introduced by commit `7244d1b94f75488b9a018c8ed36aa5c803676bc5`,
whose commit body explicitly said it would run the benchmark "on every mainline change.
"

GitHub's documentation source was inspected at `github/docs@e1e4aa937308f21c411c248b4966873536bb0cba`.
`data/reusables/actions/workflows/triggering-a-workflow-paths1.md:1` to `3` identifies path filtering as the supported
restriction for `push` and `pull_request`:

```md
When using the `push` and `pull_request` events, you can configure a workflow to run based on what file paths are changed.

Use the `paths` filter when you want to include file path patterns or when you want to both include and exclude file path patterns.
```

The same source at line `11` states that branch and path restrictions are conjunctive:

```md
If you define both `branches`/`branches-ignore` and `paths`/`paths-ignore`, the workflow will only run when both filters are satisfied.
```

The observed behavior is not a GitHub Actions defect.
The repository configured a broad `push` trigger and omitted the documented filter.

### The merge-group trigger has no current repository role

The `main` branch protection response currently has an empty `required_status_checks.checks` array.
The only active repository ruleset is `Copilot review`;
it does not require this workflow or configure a merge queue.

GitHub's documentation source,
`data/reusables/actions/merge-group-event-with-required-checks.md:1`,
explains the relevant role of `merge_group`:

```md
If your repository uses GitHub Actions to perform required checks or if you require workflows via organization rulesets on pull requests in your repository, you need to update the workflows to include the `merge_group` event as an additional trigger.
```

That condition does not hold for this repository today.
Keeping `merge_group` would add work if a merge queue were enabled later,
but it provides no current protection.

### The package rename left the install selector stale

`package/git-policy/cli/package.json:3` declares the current package name:

```json
"name": "@monochromatic-dev/git-policy-cli"
```

The pre-fix workflow at
`4054134fe2eb15b9cbac5b92164f28daf2fd49cf:.github/workflows/cli-git-performance.yml:39` to `40`
still selected the removed name:

```yaml
- name: Install cli-git dependency closure
  run: mise exec node pnpm -- pnpm install --frozen-lockfile --filter @monochromatic-dev/cli-git...
```

Commit `b18e8836b6cd070a3fa03aaa1295c048994ff06e` renamed the package and updated workflow paths,
but its workflow diff did not update this filter.
The failure is emitted later because the filtered install command completes without providing the build tool,
then `mise run //package/git-policy/cli:perf:lifecycle-latency` invokes `rolldown`.

## Verification

### Version and harness

- Repository revision first inspected:
  `bbbeb340b785fb16629fcf76fdf7c235b6dbd9c8`.
- Corrected workflow revision:
  `3ab61f0d3bfddbf017127617591d8b8c7abd4910`.
- GitHub documentation source:
  `github/docs@e1e4aa937308f21c411c248b4966873536bb0cba`.
- Workflow:
  `.github/workflows/cli-git-performance.yml`.
- Version source:
  `package/git-policy/cli/package.json`.
- Failed pre-fix run inspected:
  [run 30440155739][failed-run].
- Successful corrected workflow run:
  [run 30442283425][corrected-run].

Retrieve the trigger and recent-run evidence with:

```sh
gh api repos/Aquaticat/Monochromatic/branches/main/protection
gh api repos/Aquaticat/Monochromatic/rulesets --paginate
gh run list \
  --repo Aquaticat/Monochromatic \
  --workflow cli-git-performance.yml \
  --limit 500 \
  --json databaseId,event,headSha,createdAt,updatedAt,conclusion
gh run view 30440155739 \
  --repo Aquaticat/Monochromatic \
  --log-failed
gh workflow run cli-git-performance.yml \
  --repo Aquaticat/Monochromatic \
  --ref main
gh run watch 30442283425 \
  --repo Aquaticat/Monochromatic \
  --exit-status
```

### Corrected hosted run

Manual dispatch run `30442283425` completed successfully at revision
`5ba6d1b8ed68b158ee669da391ea39c52c0ed937`.
The `detect-version-bump` job selected the manual exception,
the lifecycle job installed the corrected `@monochromatic-dev/git-policy-cli...` dependency closure,
enforced every measured budget,
and uploaded artifact `cli-git-lifecycle-latency-1`.
The artifact response reported 7,133 bytes and `expired=false`.

The exact inline detector was also extracted into a disposable Git repository and exercised against a committed base
manifest.
The result catalog was:

```text
manual=true unchanged-push=false changed-pull-request=true missing-base=error invalid-version=error
```

Neither the workflow implementation commit nor the documentation commits created a `push` run,
confirming that changes outside `package/git-policy/cli/package.json` do not satisfy the automatic trigger.

### Runs that start the benchmark after the correction

- A pull request targeting `main` starts detection when it changes
  `package/git-policy/cli/package.json`.
  The benchmark starts only when that manifest's `version` differs from the pull request base revision.
- A push to `main` starts detection under the same manifest path filter.
  The benchmark starts only when `version` differs from the push event's before revision.
- A manual dispatch starts the benchmark without requiring a version change.
  This explicit exception supports incident reruns and budget validation.

### Runs detected but skipped after the correction

- A dependency or metadata edit in `package/git-policy/cli/package.json` with an unchanged `version` starts only the
  detection job.
- A change outside `package/git-policy/cli/package.json` creates no automatic workflow run.
- A missing automated base revision or non-string manifest version
  fails detection instead of silently running or skipping
  the benchmark.

### Historical runs outside the automatic scope

- Revision `bbbeb340b785fb16629fcf76fdf7c235b6dbd9c8` changed only
  `doc/planning/prefer-readonly-return-substitution.md`,
  but started run `30440155739`.
- Revision `98b3f8f3c3149d258960cae813186589e52cf2ce` changed only the
  `prefer-readonly-parameter-type` plugin and its test,
  but started run `30438732081`.
- Revision `84021bbb4a67cc36b5273905be36c9e499bc75bf` changed `cli-markdown-lint`,
  but started run `30436916736`.

These inputs cannot change the packed cli-git artifact,
its direct workspace dependencies,
or the lifecycle harness.

## Verified workarounds

### Gate the benchmark on the cli-git package version

Commit `3ab61f0d3bfddbf017127617591d8b8c7abd4910` implements the selected policy.
`.github/workflows/cli-git-performance.yml:3` to `12` restricts automatic detection to the cli-git manifest and retains
manual dispatch:

```yaml
on:
  pull_request:
    branches: ['main']
    paths:
    - 'package/git-policy/cli/package.json'
  push:
    branches: ['main']
    paths:
    - 'package/git-policy/cli/package.json'
  workflow_dispatch:
```

The detector at lines `18` to `66` reads the current manifest,
loads the same manifest from the event's base revision,
and writes `should_run=true` only when the version differs or the event is manual.
The benchmark job at lines `68` to `71` consumes that result:

```yaml
lifecycle-latency:
  needs: detect-version-bump
  if: ${{ needs.detect-version-bump.outputs.should_run == 'true' }}
  runs-on: ubuntu-latest
```

Line `95` also selects the renamed package correctly:

```yaml
run: mise exec node pnpm -- pnpm install --frozen-lockfile --filter @monochromatic-dev/git-policy-cli...
```

Tradeoffs:

- GitHub path filters cannot distinguish one JSON property from another.
  A non-version manifest edit creates a detector job,
  but it does not create the benchmark job.
- Code and dependency changes no longer receive immediate performance coverage.
  The next cli-git version change is the automatic release boundary.
- A version-changing pull request followed by its merge can run the benchmark for both events.
  Keeping both preserves pull-request feedback and direct-main coverage.
- The detector uses a full-history checkout so the event's base manifest
  is available across multi-commit pushes and pull
  requests.
- Manual dispatch intentionally bypasses the version-change requirement.

### Preserve a future required check with an always-reported workflow

The current workflow is not required.
If it becomes required later,
do not require the path-filtered `lifecycle-latency` workflow directly.
GitHub documents at
`content/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks.md:53` to `59`
that a path-filtered required workflow remains pending and blocks merging.
At that point,
trigger a lightweight required workflow for every pull request,
use version detection to condition the benchmark job,
and let the required job report success after either a benchmark pass or an intentional skip.

Tradeoff:
every pull request would create a workflow run,
but only a version change would allocate the benchmark runner and container work.

## What does not work

- Dependency-wide path filters still run before a version change,
  which does not meet the selected release-boundary policy.
- Filtering only on `package/git-policy/cli/package.json` runs the benchmark for dependency and metadata edits too.
  The detector is required to compare the `version` property.
- Version detection without the manifest path filter creates a lightweight workflow run for every commit.
- `concurrency` can cancel superseded runs,
  but unrelated pushes still satisfy an unfiltered trigger.
- Manual dispatch alone turns the budget into an optional check.
- Pull-request-only automation omits direct-main version changes.
- A schedule adds runs without a version change and weakens attribution to the release boundary.
- Keeping `merge_group` for a check that is not required and a queue that is not configured adds no current coverage.
- Fixing only the stale pnpm selector stops `rolldown: not found`,
  but leaves the every-main-push trigger unchanged.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry names GitHub Actions event path filtering or this workflow's trigger scope.
Open and closed `github/docs` issues and pull requests were searched for the path-filter and required-check behavior.
No upstream report is needed because the current documentation already describes the observed behavior.

1. **Is it really upstream's fault?**
   No.
   The repository explicitly requests every push to `main` and omits `paths`.
2. **Can upstream fix it?**
   No product defect was established.
   The fix belongs in this repository's workflow configuration.
3. **Are they supporting this use case?**
   Yes.
   GitHub documents `paths` for `push` and `pull_request`,
   plus `workflow_dispatch` for manual runs.
4. **Would the repo welcome our contribution?**
   Not applicable because the deciding upstream documentation is correct.
5. **Will they likely fix it?**
   No upstream change is needed.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes at the consumer configuration boundary.
   Commit `3ab61f0d3bfddbf017127617591d8b8c7abd4910` adds manifest path filters,
   compares the version against the event base revision,
   removes the currently unused merge-group trigger,
   retains manual dispatch,
   and corrects the stale package selector.

### Filing artifact

~~~md
Do not file as-is.

There is no GitHub Actions defect to report.
Monochromatic configured an unfiltered push trigger and retained a stale package selector after renaming cli-git.
Commit `3ab61f0d3bfddbf017127617591d8b8c7abd4910` corrected both at the repository boundary.
~~~

[corrected-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/30442283425
[failed-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/30440155739
