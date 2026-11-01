# CI provider

Decision record for where this repo's CI runs.
Decision:
 stay on GitHub Actions standard hosted runners.
No migration is scheduled or anticipated;
the only option kept open is adding GitHub self-hosted runners for ubuntu x86
if deliberately heavy per-push jobs ever exceed the hosted concurrency budget.
Non-GitHub providers are rejected below so they do not get re-proposed.

## Superseded in part on 2026-10-18

[The Depot CI adoption decision](depot-ci-adoption.md) replaces the provider decision above for
this repository.
The replaced decision is the "stay on GitHub Actions standard hosted runners" choice at lines 4
to 9,
including its "no migration is scheduled or anticipated" and "non-GitHub providers are rejected"
clauses.
The current decision keeps GitHub-hosted runners as the default executor and the only executor
for untrusted pull-request source,
and admits Depot-managed GitHub runners and selected native Depot CI workflows as bounded
additional capacity under a Developer no-overage baseline and an advisory-only model.
The original 2026-06-11 measurements and the "Kept open:
GitHub self-hosted runners" section are
retained unchanged as history.
The rejections of OSS-Fuzz,
GitLab CI,
Woodpecker CI,
Forgejo Actions and
Buildkite remain in force;
only the Depot replacement is affected by this note.

Measurements in this file were taken on 2026-06-11 from the live repo
(`gh run list`,
 `git log`) and current GitHub documentation.

## Findings

### Billing: no monthly minute limit exists for this repo

`Aquaticat/Monochromatic` is public.
GitHub Actions standard hosted runners are free for public repositories with no minute quota:
"GitHub Actions usage is free for self-hosted runners and for public repositories
that use standard GitHub-hosted runners"
([GitHub billing docs](https://docs.github.com/en/billing/concepts/product-billing/github-actions)).
Every workflow in `.github/workflows/` runs on `ubuntu-latest`,
 a standard hosted runner.
There is no monthly budget to run out of;
question one of the original assessment ("will we hit monthly limits") is moot for minutes.

### Measured usage (week ending 2026-06-11)

- 800 workflow runs totaling about 1,058 runner-minutes (about 4,500 minutes/month pace).
- 483 commits in 7 days,
   1,799 in 30 days,
   auto-pushed per commit.
- Peak burst:
   51 commits in one hour (2026-06-05 22:00 local),
  59 workflow-run starts in one hour.
- Per-workflow averages over the recent window:
  `forbidden-strings` and `scorecard` under one minute each (about 223 to 225 runs/week),
  `toml-edit-fuzz` about 3 minutes (19 runs/week,
   path-filtered).

### Limits that actually apply

From [GitHub Actions limits](https://docs.github.com/en/actions/reference/limits):

- 20 concurrent jobs on the Free plan;
   this table applies to GitHub-hosted runners only.
  Excess jobs queue rather than fail.
- 6 hours per job on hosted runners;
   5 days per job on self-hosted runners.
- Queue caps:
   500 workflow runs per 10 seconds,
   100 runs per concurrency group,
  1,500 trigger events per 10 seconds per repository.

At measured peak (59 run-starts/hour at 3 minutes or less per job),
average occupancy is about 3 of the 20 concurrent slots.
The cap binds only if individual jobs become long.

## Scaling plan for fuzz and mutation workflows

The repo has about 106 packages (101 `package.json`,
 5 `Cargo.toml`);
fuzz plus mutation coverage of 50%+ of them means about 55 new workflows.
This stays within hosted-runner limits if the workflows follow three rules:

- **Path-filter every per-package workflow**,
   as `.github/workflows/toml-edit-fuzz.yml` already does.
  A push then triggers only the workflows whose package it touched,
  so total run count scales with files changed per push,
   not with workflow count.
- **Run mutation testing on `schedule:` cron (nightly or weekly),
   not per-push.
  **
  Mutation runs execute the test suite once per surviving mutant,
   so they are structurally long,
  and long jobs colliding with per-commit auto-push is the one path to saturating
  the 20-concurrent-job cap.
  When mutation feedback is wanted on PRs,
   use incremental modes
  (`cargo-mutants --in-diff`,
   Stryker incremental) so the per-push job stays short.
- **Put `concurrency:` groups with `cancel-in-progress: true` on per-push workflows**
  so a burst of auto-pushes cancels stale runs instead of queueing them.
  (`cargo-publish.yml` deliberately sets `cancel-in-progress: false`
  because publishes must not race;
   that exception stands.
  )

## Kept open: GitHub self-hosted runners for ubuntu x86

If the repo ever deliberately wants heavy per-push jobs
(for example,
 full mutation runs on every push during a hardening sprint),
the escape valve is registering self-hosted ubuntu x86 runners,
 not migrating:

- Workflows stay as-is except for `runs-on` labels.
- Free:
   "free to use with GitHub Actions,
   but you are responsible for the cost
  of maintaining your runner machines"
  ([self-hosted runner docs](https://docs.github.com/en/actions/concepts/runners/self-hosted-runners)).
- The 20-concurrent-job table does not apply to self-hosted runners,
  and jobs may run up to 5 days instead of 6 hours.
- Security caveat to resolve before adoption:
  a public repo running workflows from fork PRs on self-hosted runners
  needs fork-PR approval settings locked down,
  because fork code executes on the runner machine.

Revisit triggers:
 sustained job queueing visible in `gh run list`,
or a deliberate decision to run mutation testing per-push.

## Rejected alternatives

Culled on 2026-06-11;
 none were taken through full vendor vetting
because the stay-on-GitHub recommendation made it moot.
If any is ever reconsidered,
 run the `choosing-technology` vetting layers first.

- **OSS-Fuzz**:
   acceptance requires "a significant user base and/or be critical
  to the global IT infrastructure"
  ([acceptance criteria](https://google.github.io/oss-fuzz/getting-started/accepting-new-projects/)),
  which this repo does not clear today,
   and it covers only fuzzing,
   not CI.
- **GitLab CI**:
   the Free tier includes 400 compute minutes/month
  ([GitLab pricing](https://about.gitlab.com/pricing/)),
  far below the measured 4,500 minutes/month pace;
  the 50,000 minutes/month open-source program is application-and-renewal gated;
  migration would rewrite every workflow and lose `merge_group` and `gh`-based tooling.
- **Woodpecker CI / Forgejo Actions**:
   self-hosted platforms require bringing compute anyway,
  so versus GitHub self-hosted runners they add a full workflow rewrite
  and lose marketplace actions and GitHub-native integration,
   for no compute gain.
- **Buildkite**:
   closed-source SaaS;
   same bring-your-own-compute shape as above,
  so it is strictly behind GitHub self-hosted runners for this repo
  and fails the open-source-default rule with no offsetting capability.
