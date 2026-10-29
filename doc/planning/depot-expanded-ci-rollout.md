# Expanded GitHub and Depot CI rollout

Status:
accepted on 2026-10-18 after Q7.
This revision replaces the earlier rollout draft.
Implementation is permitted and proceeds through the work items below.

The [evaluation](depot-expanded-ci-evaluation.md) supplies the evidence and its limits.
The [independent review record](depot-expanded-ci-evaluation-independent-review.md) records the
whole-report review and the closure controls.
The [adoption decision](../decision/depot-ci-adoption.md) records the accepted operating decisions.
The [handover](../handover/depot-trial-grilling.md) records the interview answers and the
2026-10-18 settlement.

## Purpose and accepted constraints

The permanent design uses GitHub-hosted execution,
Depot-managed GitHub runners and selected native
Depot CI capabilities.
GitHub remains the forge,
the check consumer,
the release holder and the default executor.
It also remains the only executor for untrusted pull-request source.
Ordinary per-push work stays on GitHub-hosted compute.
Depot adds bounded capacity for selected work and one bounded weekly native matrix.
There is no target of progressively draining ordinary work from GitHub.

The operating account is Developer at USD 20 per month with overage disabled on every product.
The pilot runs on the remaining Startup trial;
phase 2 runs on Developer.
No paid overage is authorized.
The [adoption decision](../decision/depot-ci-adoption.md) governs the subscription transition.

The combined planning ceiling remains USD 200,
but with overage disabled it is a capacity
ceiling,
not a spending authority.
The user reports that product caps are configured;
the exact values are not recorded.
Current Depot documentation exposes configured caps for container build minutes and GitHub
Actions runner minutes only.
Native CI,
Tests and storage caps are not documented as configurable.
That is a recorded residual control gap,
not a passing control.

## Delivery model

### Gates and results

CI results are advisory.
No required status checks are configured and no branch protection is changed.
Direct commits and auto-push remain.
A failure is reported,
not blocking.
The operator and reviewing agent use results to keep,
revert,
review or promote.

Default-branch and PR results attach to the run and to the release run for visibility.
Release workflows keep their existing behavior;
this plan adds no release or promotion gate.
Adding a blocking gate later requires a new decision record.

### Execution routing

GitHub-hosted runners are the default executor for ordinary Node type,
lint,
unit and process
validation,
for Windows and Mac validation,
for untrusted pull-request source and for publishing.

The default-branch advisory run executes a fast static,
build and unit subset.
The static part is changed-path scoped.
The first fast subset is five owning build tasks and their tests:
`logger/core`,
`upstream-fork/p-map`,
`toml-edit/core`,
`forbidden-strings/cli` and
`linter/monochromatic-lint`.
Desktop,
Android,
Qt and Podman stay in the weekly prepared environment.
The complete profile runs in the weekly matrix,
not on every push.
This profile is implemented by `.github/workflows/advisory-core.yml` on GitHub-hosted Linux.
Its results are check runs on the commit and a step summary;
no required checks and no branch-protection change follow.
Pull requests to `main` run the same profile and do not consume the push budget;
manual dispatch is available for qualification.
The default-branch run uses the sliding-window limiter currently used by the Scorecard workflow as
its starting point:
2 admitted starts per rolling minute,
4 per hour and 8 per day.
The limits adjust from measured capacity.
Excess events are skipped and recorded;
the newest skipped tip is marked pending,
not retried
blindly.

Depot-managed GitHub runners and native Depot CI execute only admitted profiles.
The repeated complete profile runs only through one bounded weekly native matrix.
The matrix is triggered by explicit `depot ci dispatch`,
not by automatic push.
It covers these capability categories in one bounded run:
static checks;
Node and Rust test consumers;
browser and E2E consumers;
packaged artifacts and host consumers;
OS,
architecture and application consumers;
VM,
desktop and Android consumers;
BuildKit and Bake container producers;
Depot Tests;
cache and Disk;
SDK VM profiles.

Native automatic registration is proven for `pull_request` and for `push` on the default branch,
for both PR-merge and direct-push landing.
See [the registration evidence](../audit/depot-registration-control-2026-10-18.evidence.json).
The matrix still uses explicit dispatch so admission and duplicate execution stay under control.

Paused-session SSH,
`--ssh-after-step`,
tmate and interactive session strings are excluded from
the matrix.
The bounded paused-session qualification did not reach a job and remains unqualified
([paused-session evidence](../audit/depot-paused-session-qualification-2026-10-18.evidence.json)).
A future debug-feature decision may revisit it.

### Current baselines

The current-source baseline is not known-good.
At revision `4f3d5a65`:

- static:
  `mise run lint:monochromatic-lint` exits 1 with 85,719 findings across 1,046 files,
  dominated by 81,700 semantic line-break findings,
  2,077 rustdoc findings and 1,894 bare-URL
  findings;
- unit:
  `mise run test:unit` exits 1 with 1,979 `ERR_MODULE_NOT_FOUND` and 987 `Cannot find` errors;
  the run lacked built prerequisites,
  so the 751 named files are a missing-prerequisite cascade,
  not product failures.
  Rerun after a successful build;
- build:
  the local attempt was stopped as host stress;
  the bounded CI run `39197594870` provisioned glib,
  Qt6,
  Podman and the workflow token and still failed after 14 minutes 26 seconds,
  so the unit step was skipped.
  Failure classes include `glib-sys`/`gdk-pixbuf-sys`/`cairo-sys-rs` system builds,
  a Podman image build,
  `cargo vendor` downloads,
  unresolved workspace imports,
  Android and design builds,
  and `canvg`.
  A generic Depot GitHub runner is insufficient;
  the baseline needs the prepared environment or a documented prerequisite image.
- fast subset:
  GitHub run `39199930641` passed the five-owner build and unit tasks in 10 minutes 17 seconds;
  the only failure annotation is the tolerated tool-install step.
  This is the first known-good fast-subset baseline.

The [baseline evidence](../audit/depot-current-baseline-2026-10-18.evidence.json) retains the
compressed logs.
The static baseline is advisory;
the fast-subset unit baseline is valid;
the whole-repository unit baseline is invalid until a prepared build succeeds;
the complete build profile requires the prepared environment.
The pilot must separate product failures from harness,
environment and resource failures.

### Workload-to-provider map

The map is a maintained artifact,
not prose hidden in this plan.
It starts with:

- GitHub-hosted Linux:
  fast static,
  build and unit advisory subset;
  Node type,
  lint and unit
  default;
  ordinary per-push work.
- GitHub-hosted Windows and Mac:
  platform validation and publication boundaries.
- GitHub-hosted public fallback:
  any admitted profile that cannot run inside the no-overage cap.
- Depot-managed GitHub runners:
  admitted complete-profile work and the weekly matrix where the
  measured profile and resource shape qualify.
- Native Depot CI:
  the weekly matrix categories,
  explicit dispatch only.
- Public GitHub only:
  untrusted pull requests,
  fork source and Dependabot updates.

Every result keeps provider,
machine,
source,
case and artifact identities.

## Spend and admission

Every paid profile declares resource and execution bounds,
retention exposure and a conservative
reservation.
No profile starts without an admission record.
The active plan and allowance source are recorded at implementation start.
The repository does not assume unused included credits.

When an included allowance is exhausted or a cap stops a run:

- an ordinary Linux,
  Windows or Mac workload falls back to free public GitHub when a qualified
  equivalent exists;
- a capability with no equivalent is marked incomplete for that period;
- publishing waits or stays on GitHub-hosted compute;
- no overage is incurred and no cap is raised automatically.

Storage is the weak meter.
The 25 GB included storage is an hourly-sampled monthly average,
and no combined hard cap was
documented.
The pilot records the storage average,
retention settings and an operational stop near 25 GB.
If storage becomes the binding constraint,
the plan reports it rather than assuming headroom.

The allocation ledger records each attempt with its provider,
run,
job,
attempt,
resource shape,
timestamps,
reservation and source receipt.
Overlapping recovery attempts are counted once.

## Caches

Cache writes come only from `main` pushes and explicit dispatches.
Pull requests,
merge groups and untrusted source restore only.
This applies to GitHub Actions cache,
Depot Actions cache,
Depot project BuildKit cache and
organization-global Disk.

The measured cache decisions are:

- the pnpm store archive is a measured regression and stays disabled;
- the archived Rust target is a reuse proof,
  not a Nextest speed or job speedup;
- the Kotlin Gradle task-output cache is admitted only for the `test --rerun jar` command boundary,
  not as a whole-job saving;
- BuildKit layers,
  Disk mounts,
  prepared tools and snapshots require their own executable,
  invalidation,
  secret-exclusion and reader controls before enablement.

No cache contains shipped,
attested or published bytes.
Release producers stay fresh.
On a cache incident,
writes stop and the affected family is invalidated before resuming.

## Trust and publishing

Untrusted pull-request source,
fork pull requests and Dependabot updates run only on public
GitHub-hosted runners.
Depot executes trusted `main` pushes,
explicit dispatches and the weekly trigger only.
Fork approval settings and Dependabot exclusion are verified before any Depot runner receives work.

Egress filtering and Tailscale are not assumed.
No secret-bearing workflow runs on Depot unless separately admitted.
Publishing stays on GitHub-hosted runners:
npm trusted publishing excludes self-hosted runners and
the pnpr binding names the exact GitHub workflow.
Native CI does not sign or publish.

## Recovery and rollback

One automatic recovery attempt is reserved for a classified provider or startup failure.
Both attempts are reserved and the original diagnostic and attempt identities are retained.
Source failures keep their failure result;
a passing provider does not erase them.
An unclassified failure is reported and does not become an automatic retry.

Removing the weekly dispatch disables every paid native profile.
The operator may disable Depot admission without further approval on repeated classified failures,
an allowance or storage breach,
a trust incident,
or evidence of a cache or isolation defect.
Safety disablement records the date,
profile,
reason and fallback.
Re-enabling requires the review gate and user notice.
Advisory-only results leave no required check to unwind.

## Optional products

The per-product dispositions are recorded in the
[adoption decision](../decision/depot-ci-adoption.md).
The default is out of scope.
Enabling a product requires a new dated decision record with eligibility,
meter,
cost and trust
boundary.
"Applicable capabilities" language authorizes nothing.

Depot Code,
Depot Agents,
Dagger,
GPU,
managed AWS,
customer AWS,
private connectivity,
external
observability sinks,
Fast CDN,
server-side registry promotion and unbounded autoscaling are out.
Custom GitHub runner images and registry pull-through are out for the pilot and conditional for
phase 2 on the recorded measurements.

## Work items

Each item has an owner,
an acceptance artifact,
a threshold and a rollback.
The implementing agent owns the engineering steps;
the user owns acceptance,
account changes and
the subscription transition.

1.  Baseline refresh and triage.
    Pin the implementation candidate.
    Re-run the owner census and the static,
    unit and build baselines.
    Run the baselines in the bounded CI environment or a resource-capped container,
    never on the shared development host.
    Split product failures from harness,
    environment and resource failures.
    Produce a failure disposition register with exact case identities.
    The register and the refreshed coverage manifest define the admitted owner set used by later
    coverage targets.
    Acceptance:
    the register lists every observed failure class with a disposition and the baseline commands
    reproduce it.
    Rollback:
    none;
    this is measurement.

2.  GitHub-hosted advisory profiles.
    Add the fast static,
    build and unit subset,
    the changed-path static selector,
    the
    sliding-window limiter and release-run visibility.
    Keep direct commits and auto-push.
    No required checks and no branch-protection change.
    Acceptance:
    a direct push and a pull request each produce an advisory result;
    a skipped event is recorded and the newest tip is marked pending;
    a release run shows the result without changing release behavior.
    Rollback:
    disable the workflow;
    no branch protection to restore.

3.  Admission,
    caps and fallback.
    Record the active plan and allowance source.
    Record that caps are configured,
    without recording their values,
    and record the documented gaps.
    Implement reservation,
    graceful degradation,
    public-GitHub fallback and the allocation ledger.
    Acceptance:
    an exhausted allowance or cap produces the documented fallback and no overage;
    the ledger reconstructs the attempt.
    Rollback:
    stop paid dispatch and retain the ledger.

4.  Weekly native matrix.
    Define the explicit-dispatch matrix,
    its per-category bounds and its evidence set.
    Include every capability category listed in Delivery model.
    Exclude paused-session.
    Keep publication on GitHub.
    Acceptance:
    one weekly dispatch produces the named categories with provider,
    case and artifact identities;
    duplicate execution is absent because the matrix is dispatch-only;
    the run stays within its reservation.
    Rollback:
    remove the dispatch;
    retain evidence.

5.  Cache admission.
    Keep main-only writes and the disabled formats.
    Admit the Gradle command boundary and any later format only after the consumer,
    invalidation
    and stale-output controls pass.
    Acceptance:
    cache-off,
    invalidation and stale-output rejection are demonstrated for each admitted format.
    Rollback:
    disable the format and invalidate its family.

6.  Trust and identity controls.
    Verify fork approval,
    Dependabot exclusion,
    publishing-host bindings,
    secret boundaries and
    cache isolation.
    Acceptance:
    a fork or Dependabot event cannot reach a Depot runner;
    a publish workflow stays GitHub-hosted;
    cache writes from non-main refs are rejected.
    Rollback:
    disable Depot admission;
    publishing is unaffected.

7.  Native evidence closure.
    Keep the registration and duplicate controls.
    Decide whether to rerun the paused-session qualification from a checkout with working
    default-branch detection.
    Record the result either way.
    Acceptance:
    the evidence register states exactly which native features are qualified and which are not.
    Rollback:
    none;
    evidence only.

8.  Measurement and relative targets.
    Measure the configured profiles against the public GitHub baseline for the same workload.
    Record duration,
    failure results and allocation exposure.
    Finalize the relative targets:
    25 percent faster core feedback,
    100 percent admitted-owner
    coverage,
    zero unclassified failures and spend at or below the Developer fee with no overage.
    Acceptance:
    a review point compares the measured results with those targets before the recurring schedule
    is relied on.
    Rollback:
    stop the schedule and fall back to public GitHub.

9.  Governance follow-through.
    The decision records are accepted and Q7 is answered.
    The subscription transition from the Startup trial to Developer remains a user action.
    Keep the records current when measured bounds change.
    Acceptance:
    the active plan is recorded and any later decision change has a dated note.
    Rollback:
    none;
    this is governance.

## Verification

- Every result keeps provider,
  machine,
  source,
  case and artifact identities.
- Gates are advisory;
  a missing or unexecuted consumer is not a pass.
- Setup,
  discovery and artifact-integrity failures are reported as failures.
- The baseline register retains exact case identities and commands.
- The weekly matrix uses explicit dispatch and retains its admission record.
- The review record and the decision records are linked from every measured result.

## Limits

The evaluation is sufficient for a bounded pilot,
not for a production certification.
It does not establish complete coverage,
an aggregate invoice guarantee,
provider availability or a
universal speed multiplier.
The source cohort and the pinned baseline are recorded;
later source changes require a fresh
baseline.
The plan does not claim every product feature or source branch is covered.
Runner labels,
cache encodings,
retry timers and workflow file names are engineering details chosen
during implementation.
Required-check membership,
spend authority,
trust boundaries and the weekly matrix membership are
not engineering details;
they are recorded above or remain explicitly open.

## Shared-understanding check

Q7 was answered on 2026-10-18:
the user confirmed this revised plan and permitted implementation.
[The adoption decision](../decision/depot-ci-adoption.md) is accepted.
This section is retained as the record of the shared-understanding check.
