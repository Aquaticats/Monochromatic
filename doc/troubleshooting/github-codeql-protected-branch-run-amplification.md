# GitHub CodeQL default setup on 2026-08-19 scans every wildcard-protected branch and amplifies push runs

## Resolution status

Resolved on 2026-08-19 by commit `000d58464dccc3088e8fd4476692e10bdcf64d2c` and a repository setting change.
The repository now owns `.github/workflows/codeql.yml`,
 and GitHub's default-setup API reports `state: not-configured`.
The advanced workflow batches direct-main coverage daily,
 scans pull requests targeting `main`,
 permits manual recovery,
 and has no `push` trigger.

## Symptom

GitHub Actions shows repeated CodeQL runs named `Push on translation-repair-rebased` after each commit to that branch.
At the 2026-08-19 observation point,
 the Actions API had reached its 1,000-run response cap for the week beginning
2026-08-12,
 and the latest 1,000 runs contained 483 CodeQL push runs across `main` and
`translation-repair-rebased`.

The branch-specific symptom is not an error message.
It is a sequence of successful or concurrent runs that the user did not intend to buy faster hardware merely to repeat:

```text
Push on translation-repair-rebased
Analyze (javascript-typescript)
Analyze (c-cpp)
Analyze (actions)
```

The surface pattern that triggers it is:

- a push to any branch;
- classic branch protection pattern `*` matches every non-default branch;
- CodeQL default setup scans each push to every protected branch.

A separate Copilot review ruleset also targets `~ALL` branches.
It is not needed to explain the CodeQL trigger.

Changing from a GitHub-hosted runner to a Namespace labeled runner changes where these jobs execute.
It does not change when default setup creates them.

## Root cause

### The repository protects every branch

The live GraphQL response on 2026-08-19 contains two classic branch-protection rules.
Pattern `main` matches the default branch,
 while pattern `*` matches every listed non-default branch,
including `translation-repair-rebased`:

```console
$ gh api graphql --raw-field query='query {
  repository(owner: "Aquaticat", name: "Monochromatic") {
    branchProtectionRules(first: 100) {
      nodes { id pattern matchingRefs(first: 100) { nodes { name } } }
    }
  }
}'
{
  "data": {
    "repository": {
      "branchProtectionRules": {
        "nodes": [
          {
            "id": "BPR_kwDOKlVnec4CkrWH",
            "pattern": "main",
            "matchingRefs": {"nodes": [{"name": "main"}]}
          },
          {
            "id": "BPR_kwDOKlVnec4D4xsc",
            "pattern": "*",
            "matchingRefs": {
              "nodes": [
                {"name": "translation-repair-rebased"}
              ]
            }
          }
        ]
      }
    }
  }
}
```

The REST branch API confirms that both the default branch and the working branch are protected.
A complete branch listing returned 25 protected branches and no unprotected branches:

```console
$ gh api repos/Aquaticat/Monochromatic/branches/translation-repair-rebased \
  --jq '{name,protected}'
{"name":"translation-repair-rebased","protected":true}

$ gh api repos/Aquaticat/Monochromatic/branches/main \
  --jq '{name,protected}'
{"name":"main","protected":true}
```

### Default setup treats every protected branch as a push target

GitHub's documentation source at commit `a34bf588b9e6eff791e173fdd3a726dfab26f888` states the trigger directly.
`content/code-security/concepts/code-scanning/setup-types.md:16-20` says:

```md
Default setup ... will be scanned using ...:

* On each push to the repository's default branch, or any protected branch.
* When creating or committing to a pull request based against the repository's default branch,
  or any protected branch, excluding pull requests from forks.
* On a weekly schedule.
```

The live configuration confirms that this repository uses default setup:

```console
$ gh api repos/Aquaticat/Monochromatic/code-scanning/default-setup
{
  "state": "configured",
  "languages": [
    "actions",
    "c-cpp",
    "javascript",
    "javascript-typescript",
    "typescript"
  ],
  "query_suite": "extended",
  "schedule": "weekly",
  "runner_type": "standard"
}
```

The resulting chain is:

1. Classic branch-protection pattern `*` matches `translation-repair-rebased`.
2. A commit is auto-pushed to that protected branch.
3. CodeQL default setup creates one workflow run because the push target is protected.
4. The run fans out into the detected language jobs.
5. Another pushed commit starts another run without canceling the prior run.

The earlier reading that Copilot review ruleset `9126851` made the branch protected was unsupported.
The classic `*` branch-protection rule is direct evidence and independently explains the protected status.

### A runner-provider change does not alter the chain

GitHub does allow default setup to use a labeled runner.
`content/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/edit-default-setup.md:27-35`
contains:

```md
Optionally, to use labeled runners ... select **Labeled runner**.
Then, next to "Runner label," enter the label of an existing self-hosted or GitHub-hosted runner.
```

That setting only selects a runner label.
The trigger remains the protected-branch behavior in `setup-types.md:16-20`.
Namespace's migration guide likewise describes changing the job's `runs-on` label,
not changing GitHub event generation
(<https://namespace.so/docs/solutions/github-actions/migration>).

The earlier hypothesis that a faster runner provider could remove the run flood was wrong.
A faster runner can shorten a generated job,
 but no cited Namespace feature suppresses the push event,
changes CodeQL default-setup triggers,
 or cancels an older GitHub workflow run.

## Verification

### Versions and evidence points

- GitHub.com service observed on 2026-08-19.
- GitHub documentation commit:
   `a34bf588b9e6eff791e173fdd3a726dfab26f888`.
- Classic wildcard branch-protection rule:
   `BPR_kwDOKlVnec4D4xsc`.
- CodeQL default setup update timestamp:
   `2026-07-20T15:49:57Z`.
- Working-branch example run:
   `32279201781`.
- Default-branch example run:
   `32204388026`.
- Scheduled example run:
   `32250832355`.
- Advanced workflow commit:
   `000d58464dccc3088e8fd4476692e10bdcf64d2c`.
- Advanced workflow database ID:
   `338017899`.
- Successful manual advanced run:
   `32284761248`.

### Runnable probe

Run from this repository with a GitHub CLI identity that can read Actions and repository rules:

```console
cd -- /var/home/user/Monochromatic

gh api graphql --raw-field query='query {
  repository(owner: "Aquaticat", name: "Monochromatic") {
    branchProtectionRules(first: 100) {
      nodes { id pattern matchingRefs(first: 100) { nodes { name } } }
    }
  }
}'

gh api repos/Aquaticat/Monochromatic/branches/translation-repair-rebased \
  --jq '{name,protected}'

gh api repos/Aquaticat/Monochromatic/code-scanning/default-setup

gh run view 32279201781 \
  --json workflowName,createdAt,updatedAt,conclusion,jobs,url
```

### Before-state positive controls

- A push to `main` starts CodeQL because `main` is the default branch.
  Run `32204388026` is the positive control.
- The weekly schedule starts CodeQL independently of pushes.
  Run `32250832355` is the positive control.
- Each run contains separate `actions`,
   `c-cpp`,
   and `javascript-typescript` analysis jobs.

### Run-amplifying patterns

- A push to `translation-repair-rebased` starts CodeQL because classic protection pattern `*` matches it.
  Run `32279201781` is a completed example.
- Closely spaced pushes create overlapping workflow runs
  because default setup exposes no repository workflow file
  in which to add a concurrency group.
- Selecting a labeled runner routes the same generated jobs elsewhere and preserves their trigger count.

## Verified remediation

Commit `000d58464dccc3088e8fd4476692e10bdcf64d2c` added the advanced workflow and concurrency groups to
seven replaceable validation workflows.
The CodeQL workflow uses:

- `pull_request` targeting `main`;
- daily `schedule` at `03:17 UTC`;
- `workflow_dispatch`;
- no `push` trigger;
- separate event-scoped concurrency so scheduled work cannot cancel a manual run;
- `build-mode: none` for `actions`,
  `c-cpp`,
  and `javascript-typescript`;
- `security-extended` queries;
- local threat sources in addition to the default remote threat model.

The deployed trigger and concurrency shape is:

```yaml
on:
  pull_request:
    branches: ['main']
  schedule:
  - cron: '17 3 * * *'
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.event_name }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name != 'workflow_dispatch' }}
```

The event component separates scheduled,
pull-request,
and manual groups.
The ref component keeps distinct pull requests independent.
The cancellation condition also prevents one manual dispatch from cancelling another.

All seven replaceable validation workflows use this separate shape:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

Including `github.workflow` prevents cross-workflow cancellation,
while `github.ref` limits supersession to work for the same branch or pull request.

GitHub recognized `.github/workflows/codeql.yml` as active workflow ID `338017899` before default setup was disabled.
The file fetched through GitHub's Contents API had the same SHA-256 as the local committed file.
The default-setup endpoint then accepted `state: not-configured`.

Manual run `32284761248` crossed the consumer boundary:

- the workflow event was `workflow_dispatch`;
- all three language jobs succeeded;
- each initialization log showed `build-mode: none`,
  `threat-models: local`,
  and `security-extended`;
- each analysis log reported a successful result upload and complete upload status;
- the Code Scanning analyses API returned distinct advanced-workflow categories for all three languages.

The commit that introduced the advanced workflow was pushed while default setup was still active,
 so that commit intentionally has both dynamic default-setup analyses and advanced manual analyses.
Three later pushes made after default setup was disabled provide the negative trigger probe:

- `7560d3ab9f6665d750cd1ca39ae6167f4267ae06` created Scorecard and forbidden-strings runs;
- `d29a9dd263019e294d312e457b28c9598a2a5b3f` created Scorecard,
  forbidden-strings,
  and toml-edit-fuzz runs;
- `4e2e0e65e87949e46b7475a3802cde3cc3c2b1e2` created Scorecard,
  forbidden-strings,
  and toml-edit-fuzz runs.

The Actions API returned no CodeQL run for any of those commit SHAs.
The other runs are positive controls proving that GitHub processed each push event.

Two manual final-newline dispatches exercised the shared validation concurrency shape.
Run `32285688509` ended as `cancelled` after run `32285692427` entered the same workflow-and-ref group;
the superseding run completed successfully with every step green.

Disposable draft pull request
[#450](https://github.com/Aquaticat/Monochromatic/pull/450)
exercised the pull-request path and same-ref supersession:

- CodeQL run `32288000285` started all three language jobs for the first revision;
- a second revision created CodeQL run `32288031286` in the same
  `CodeQL-pull_request-refs/pull/450/merge` group;
- GitHub cancelled all three jobs in the first run with the higher-priority-request concurrency diagnostic;
- all three jobs in the superseding run succeeded;
- the Code Scanning API returned three successful `.github/workflows/codeql.yml:analyze` analyses on
  `refs/pull/450/merge`;
- first-revision final-newline run `32288000482` and forbidden-strings run `32288000457` were also cancelled;
- their superseding runs `32288031268` and `32288031205` succeeded.

The probe PR was closed without merging,
and its local worktree and local and remote branches were deleted.
The first natural scheduled run,
[32330328949](https://github.com/Aquaticat/Monochromatic/actions/runs/32330328949),
completed successfully on 2026-08-20.
GitHub created it at `04:01:16 UTC`,
44 minutes and 16 seconds after the configured `03:17 UTC` trigger.
All three language jobs succeeded,
used `build-mode: none`,
`threat-models: local`,
and `security-extended`,
and uploaded successful analyses on `refs/heads/main`.
The only annotations were the documented `MissingPushHook` warnings.

Classic branch-protection pattern `*` remains unchanged.
A final GraphQL query returned classic patterns `main` and `*` with required-status-check enforcement enabled,
but empty `requiredStatusCheckContexts` and `requiredStatusChecks` for both.
Disabling default setup therefore did not leave a required dynamic-CodeQL check name that can block merging.
The fix preserves force-push,
 deletion,
 and conversation-resolution governance while decoupling CodeQL scheduling from protected-branch status.

### Expected missing-push annotation

Pinned `github/codeql-action` commit `ff2f1c621b7f889edc0d3c761ac2e6a3f8cdb0dd` emits coded workflow diagnostic
`MissingPushHook`:

```text
Please specify an on.push hook to analyze and see code scanning alerts from the default branch on the Security tab.
```

The emitting condition is `src/workflow.ts:190-199` in `github/codeql-action` tag `v4.37.7`.
It reports whenever a workflow has `pull_request` but neither `push` nor `workflow_call`;
the check does not account for `schedule` or `workflow_dispatch` default-branch analysis.
Run `32284761248` nevertheless uploaded three `refs/heads/main` analyses,
 and the Code Scanning API exposed alerts whose most recent instance uses that advanced-workflow commit.

The annotation is accepted because no `push` trigger is the requested batching policy.
Adding a misleading no-op push trigger merely to silence the diagnostic would not improve analysis coverage.

Closed upstream issue
[github/codeql-action#1339](https://github.com/github/codeql-action/issues/1339)
requested suppression for intentional no-push layouts.
The maintainer's suggested workaround adds a push trigger and skips every job on push.
That would recreate an Actions run record for every main push,
which directly conflicts with this repository's run-volume objective.
No current suppression input exists in pinned source.

## What does not work

### Migrating default setup to a faster labeled runner

This can reduce duration if the selected machine performs better.
It retains every protected-branch trigger and therefore does not remove redundant runs.
Namespace also introduces a separate paid or sponsorship-gated compute allocation.

### Increasing concurrency alone

More concurrent slots drain a burst faster but execute the same number of analyses.
For this repository,
 GitHub's public standard runners are free,
while Namespace's standard pricing meters compute and gates Windows runners behind Team or higher
(<https://namespace.so/pricing.md>).

### Adding path filters to default setup

Default setup does not expose a repository workflow file.
GitHub directs users who need custom workflow triggers to advanced setup at
`content/code-security/concepts/code-scanning/setup-types.md:68-74`.

### Splitting the repository

A split would change repository-level event boundaries,
but the trigger amplification originates in an all-branch protection rule
combined with documented default-setup behavior.
Changing repository topology is not required to change either input.

## Upstream filing artifact

### Upstream filing decision

- **Is it really upstream's fault?**
   No.
  GitHub documents the protected-branch trigger explicitly.
  The surprising volume comes from this repository applying
  classic branch protection pattern `*` to non-default branches.
- **Can upstream fix it?**
   GitHub could add default-setup concurrency controls,
  but no defect was established in the documented behavior.
- **Are they supporting this use case?**
   Yes.
  GitHub supports default setup,
   labeled runners,
   and advanced setup for custom triggers.
- **Would the repository welcome the contribution?**
   No issue is needed.
  Searches of open and closed `github/docs` issues and pull requests for
  `default setup protected branch wildcard scans every push` found no duplicate,
  but the current documentation already states the decisive behavior.
- **Will they likely fix it?**
   Not applicable.
  There is no demonstrated documentation error or service defect to fix.
- **Have we prototyped a minimal fix compatible with their architecture?**
   No.
  The relevant change belongs in this repository's CodeQL or branch-protection configuration,
  not in GitHub's documentation source.

`.out-of-scope/` contains no GitHub Actions or CodeQL exemption.
The upstream-filing gate still stops at the first constraint because this is expected behavior.
There is nothing additive to file or comment upstream.
