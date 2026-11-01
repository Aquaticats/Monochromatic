# Independent whole-report review of the Depot evaluation

Status:
recorded on 2026-10-18.
This record is the independent whole-report sufficiency review that the evaluation says was still
missing at line 986.
It is an input to Q7 and to the decision-record acceptance;
it is not an acceptance of the plan.

## Scope and pinned revision

The review covers the expanded evaluation,
the execution handover,
their cited audit corpus,
and
the concrete rollout plan that depends on them.
The pinned revision at review start is `98381bfa32e1da9053daa88e1a24bf0ffa01acd7`,
the then
`origin/main`.
The evaluation's frozen evidence cohort remains `e02be6c8b8`,
which is 886 commits and 1,493 files
behind that revision;
the review treats that drift as a finding,
not as a silent substitution.

Four independent read-only surface reviews were run:

- evidence fidelity of the evaluation against its cited audit files;
- feasibility and sequencing of the evaluation's adoption conditions against the repository;
- cost,
  allowance,
  and trust arithmetic and controls;
- governance,
  provenance,
  and decision sufficiency of the evaluation and handover.
The plan itself had already received four separate surface reviews,
  recorded in
[the plan review](depot-expanded-ci-rollout-independent-review.md).
No provider work was dispatched and no account setting was read during the review.

## Verdict

The evaluation is sufficient,
with corrections,
for a bounded pilot and for the shared-understanding
check.
It is not sufficient for a production rollout,
and it is not a complete-coverage,
invoice,
availability,
or universal-performance certification.
The central selective-hybrid recommendation survives;
no finding overturns it.

The plan review reached the same limit:
the plan cannot be confirmed as written.
The current interview has since settled the operating model,
and the closure work addresses the
blockers below.

## Blocker register

Legend:
C,
corrected in report text;
S,
settled by the current interview;
M,
measured or being measured in closure;
P,
converted to a pilot work item with acceptance evidence;
A,
explicitly accepted by the user.

### Plan evidence fidelity

- F-B1,
  ordinary-Linux wording overstates the evaluation:
  C,
  plan rewrite adopts the evaluation's
  "part of its workload allocation" wording.
- F-B2,
  permanent GitHub ownership of Node work contradicts the evaluation:
  S,
  GitHub default for
  Node work with the weekly matrix as capability evidence only.
- F-B3,
  completed-phase benefit claim hides two repetitions and incomplete runs:
  S,
  relative
  targets against a measured baseline replace the benefit claim.
- F-B4,
  Gradle claim hides slower whole warm jobs:
  C,
  cache disposition records the command-phase
  benefit and the 162/177 versus 146/153 second whole-job result.
- F-B5,
  pnpm cache is a measured regression:
  M,
  the pnpm format stays disabled unless a new
  admission record overturns the measurement.

### Plan feasibility

- FE-B1,
  plan contradicts the standing provider decision:
  S and C,
  the adoption record supersedes
  it;
  the plan rewrite removes the contradiction.
- FE-B2,
  validation gates have no enforcement mechanism:
  S,
  advisory-only model.
- FE-B3,
  sequence adds gates,
  admission,
  schedules,
  and native promotion before controls:
  P,
  the
  revised pilot sequence puts boundary controls first and wires one weekly trigger.
- FE-B4,
  native promotion has no in-repository source,
  names no selection,
  and auto-registers
  events:
  P,
  the native closure work names selected files,
  merges disabled first,
  records
  check-name mapping,
  and excludes issuer-bound publication.
- FE-B5,
  no rollback,
  migration owner,
  or per-item definition of done:
  S and P,
  rollback authority
  is in the adoption record;
  the matrix work item carries per-item done conditions.
- FE-B6,
  admission is defined against an inactive plan:
  S,
  Developer no-overage baseline;
  the
  active plan and allowance source are recorded at implementation start.

### Plan cost and trust

- CT-B1,
  Developer plus USD 200 fails the old combined example:
  S and M,
  no-overage capacity
  replaces price arithmetic;
  the zero-overage scenario table records the fit.
- CT-B2,
  subscription wording contradicts the Startup trial:
  C,
  the adoption record states the
  active plan and the outstanding transition.
- CT-B3,
  admission needs headroom while account-wide inspection is excluded:
  S and M,
  overage is
  disabled and operator-supplied allowance caps are the control;
  no aggregate invoice is claimed.

### Plan governance

- PG-B1,
  deferral list is false and confirmation is circular:
  S and C,
  the interview settles the
  deferred decisions and the plan rewrite removes the unbounded deferral.
- PG-B2,
  qualification is circular and criteria-free:
  P,
  per-profile admission records replace the
  undefined qualification.
- PG-B3,
  spend authority precedes reservations:
  S,
  no-overage caps replace spend authority.
- PG-B4,
  no disable,
  rollback,
  or incident path:
  S and P,
  rollback authority is recorded and the
  single weekly dispatch is the off switch that must be verified.
- PG-B5,
  required-check membership is undecided and blocking has no gate on auto-pushed main:
  S,
  advisory-only model.
- PG-B6,
  untrusted-PR boundary is contradictory and the fork precondition is dropped:
  S and P,
  public GitHub only for untrusted source;
  fork approval and Dependabot exclusion are verified
  before any Depot dispatch.
- PG-B7,
  a proposal asserts decisions and schedules no decision-record step:
  C and A,
  the adoption
  record and supersession notes are drafted;
  explicit acceptance remains.

### Evaluation governance

- EG-B1,
  frontier is misstated and paid-priority allocation was never asked:
  S and C,
  the handover
  and evaluation frontier text now records the settlements and the Q7 exception.
- EG-B2,
  adoption allocation contradicts the corrected GitHub-retention framing:
  S and C,
  the
  evaluation allocation is replaced with the GitHub default plus weekly native matrix.
- EG-B3,
  standing decision records are never reconciled:
  C,
  the provider and cache supersession
  notes are drafted for acceptance.

### Evaluation feasibility

- EF-B1,
  allocation contradiction:
  C,
  same correction as EG-B2.
- EF-B2,
  native promotion prerequisites are missing:
  P,
  the parallel-step correction is complete,
  native `pull_request` registration and push-on-main registration are proven,
  and duplicate-publication behavior,
  check-name mapping and paused-session qualification remain open.
- EF-B3,
  frozen owner map is stale:
  M,
  the closure measurement refreshes the package,
  task and
  file counts at the pinned revision.
- EF-B4,
  protected-environment and reusable-workflow boundary is unsupported:
  C,
  the evaluation
  now records that no such boundary exists and names the real push-publishing boundary.
- EF-B5,
  report says the review never ran and Q7 is pending:
  C and S,
  this record closes the
  review gap;
  Q7 remains the plan-confirmation exception.

### Evaluation fidelity

- EFI-B1,
  465 "owning commands" is actually 465 scheduled invocations:
  C,
  the evaluation now says
  297 owning target tasks plus 168 shared prerequisite invocations.

Every listed blocker has a disposition;
none is left silent.
The remaining acceptance steps are the adoption record,
the two decision-record notes,
the
subscription transition,
and Q7.

## Corrections applied

The evaluation received dated in-place corrections for the 465 invocation count,
the Mac and Windows
pass/fail provenance,
the artifact-consumer import attribution,
the Semgrep rule-pack wording,
the
sealed and complete wording against the manifest flags,
the 2,425-task and 13,860-file labels,
the
mutation dry-run,
the 120-page ledger status,
the Go mechanism,
the Developer Windows citation,
the
catalog Mac shape,
the namespace-init explanation,
the approval and reusable-workflow boundary,
and
the review status.
The evaluation and handover also received the frontier replacements that record the advisory-only
model,
the Developer no-overage baseline,
the weekly native matrix,
the public-GitHub untrusted
boundary,
the main-only cache writes,
the review gate,
the per-product disposition rule,
and the Q7
exception.

The handover line 78 now records the 2026-10-18 settlement and names Q7 as the only open
confirmation question.
The handover review-pending statements are marked historical by a dated correction note.

## Closure evidence recorded after this review

Owner-map refresh at pinned revision `98381bfa32`:

- 225 package directories,
  210 active,
  13 paused and 2 archived.
- 14,301 tracked files.
- 2,600 authored task declarations and 2,464 expanded `mise` task records.
- The evaluation's 223,
  13,860 and 2,425 counts are historical for its frozen cohort and are not
  a current starting scope.

Closure workstreams:

- A bounded native experiment list E0 to E7 was prepared with provider,
  minutes,
  cost ceiling,
  admission,
  cleanup and acceptance evidence.
  E1,
  the parallel-step correction dispatch,
  completed as native run `gns8864w4s`.
  The state and consumer jobs finished;
  the two parallel jobs failed intentionally and both receipts passed:
  `parallel-default` returned with `siblingFinishedAtReturn: false`,
  and
  `parallel-allrun` returned with `siblingFinishedAtReturn: true`.
  Both receipts record `descendantCleanupGuaranteeAsserted: false`.
  The workflow and job failure statuses are the intentional failure controls,
  not product failures.
  Artifact hashes are retained from the run logs.
- The default-branch advisory window starts from the Scorecard pattern,
  2 admitted starts per
  minute,
  4 per hour and 8 per day,
  and adjusts from measured capacity.
  Derived zero-fallback options are recorded separately;
  the pilot starts with the Scorecard
  pattern as the user directed.
- The user reports that product caps are configured but does not record the values.
  Current documentation supports configured caps for container build minutes and GitHub Actions
  runner minutes only;
  native CI,
  Tests and storage caps are not documented as configurable.
  This remains a residual control gap,
  mitigated by the usage pages,
  an operational stop and
  graceful degradation.
- The product disposition research is recorded:
  11 products are out for pilot and phase 2;
  custom runner images and registry pull-through are out for the pilot and conditional for
  phase 2.
  The adoption proposal carries the list and its activation conditions.
- Paused-session qualification was attempted from a linked worktree and a depth-1 clone.
  Both local runs reported a default-branch detection failure,
  skipped the patch,
  created zero-workflow runs,
  and were cancelled.
  Paused-session remains unqualified and is excluded from the weekly matrix.
  The attempt is recorded in
  [the paused-session evidence](../audit/depot-paused-session-qualification-2026-10-18.evidence.json).
- Automatic event registration was investigated with a side-effect-free direct-push probe.
  The probe is not a valid negative for three reasons:
  the marker push arrived at `00:49:35`,
  during the add push's native run
  `ps_hf4mk4q5l6` from `00:49:30` to `00:50:03`;
  the listing command observes executions,
  not registered triggers;
  and the workflow was removed before a later push could exercise a registration that completed.
  Native `pull_request` registration is already proven in this repository on a non-default base:
  six native runs,
  Depot checks from app `219785`,
  and a required-check 405 control.
  Push registration on `main` is now proven on both landing paths:
  the PR-merge landing and the direct-push landing each produced a native marker run
  (`ps_4dzmpnfm09` and `ps_9hmw58cpsd`),
  a finished probe job,
  and a Depot check from app `219785`.
  The corrected control is recorded in
  [the registration evidence](../audit/depot-registration-control-2026-10-18.evidence.json);
  the primary-source research remains in
  [the registration research](depot-native-registration-research.md).
- Duplicate execution was observed with the same side-effect-free workflow on both providers:
  GitHub run `39187939149` and native run `ps_sjdqrdsgtl` for marker commit `3cf4d341f1`,
  with checks `probe` from `github-actions` and `duplicate-control / probe` from
  `depot-code-access`.
  The evidence is in
  [the duplicate control](../audit/depot-duplicate-control-2026-10-18.evidence.json).
  The adoption proposal uses explicit dispatch for the weekly matrix to avoid duplicate execution.
- The current static,
  unit and build baseline was measured at revision `4f3d5a65`.
  Static:
  `mise run lint:monochromatic-lint` exits 1 with 85,719 findings across 1,046 files,
  dominated by 81,700 semantic line-break findings,
  2,077 rustdoc findings and
  1,894 bare-URL findings.
  Unit:
  `mise run test:unit` exits 1,
  dominated by 1,979 `ERR_MODULE_NOT_FOUND` and 987 `Cannot find` errors;
  the run lacked built prerequisites,
  so the 751 named files are a missing-prerequisite cascade,
  not product failures.
  Build:
  the local attempt was stopped as host stress;
  the bounded CI run `39197594870` provisioned glib,
  Qt6,
  Podman and the workflow token and still failed after 14 minutes 26 seconds,
  and the unit step was skipped.
  Failure classes include `glib-sys`/`gdk-pixbuf-sys`/`cairo-sys-rs` system builds,
  a Podman image build,
  `cargo vendor` downloads,
  unresolved workspace imports,
  Android and design builds,
  and `canvg`.
  A generic Depot GitHub runner is insufficient;
  the baseline needs a prepared environment or a documented prerequisite image.
  Fast subset:
  GitHub run `39199930641` passed the five-owner build and unit tasks in 10 minutes 17 seconds,
  with only the tolerated tool-install annotation.
  The static baseline is advisory;
  the fast-subset unit baseline is valid;
  the whole-repository unit baseline is invalid until a prepared build succeeds;
  the complete build profile requires that prepared environment,
  not the shared development host.
  The evidence is in
  [the current baseline](../audit/depot-current-baseline-2026-10-18.evidence.json).

## Open closure items

The following are not satisfied by this review record and remain work:

- paused-session remains unqualified;
- build environment provisioning and unit rerun after a successful build;
- no-overage sliding-window admission and the operator cap verification;
- subscription transition from the Startup trial to Developer;
- implementation work items from the accepted plan.

## Limits

This record verifies the review inputs that were read and sampled;
it does not re-execute the
evaluation's provider campaigns.
It does not replace the operator's account records,
an invoice,
or a complete-coverage claim.
It does not authorize a paid profile;
the adoption record and Q7 do.
