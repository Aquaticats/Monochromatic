# Depot expanded CI rollout handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

## Purpose

Resume the accepted Depot expanded CI rollout after a planned PC shutdown.
The user is isolating a room issue and will continue in a few hours.
This handover records the work item state,
the evidence,
the open decisions and the next action.
Do not rerun the full repository build on the local host.

Canonical documents:

- [Accepted rollout plan](../planning/depot-expanded-ci-rollout.md),
  revision `d94e50bec4`.
- [Adoption decision](../decision/depot-ci-adoption.md),
  accepted.
- [Work item status](../planning/depot-expanded-ci-rollout-status.md).
- [Baseline register](../planning/depot-expanded-ci-baseline-register.md).
- [Independent review record](../planning/depot-expanded-ci-evaluation-independent-review.md).

## Current state

The plan and decision records are accepted.
All nine planned work items now have implementation records.
The advisory profile is implemented and qualified.
Admission,
fallback and the allocation ledger are implemented and exercised.
The weekly native matrix ran four bounded dispatches and recorded its category evidence.
Cache admission,
trust controls and native qualification are recorded.
Fast-subset measurement and provisional relative targets are recorded.
The complete build and whole-repository unit baseline are still not known-good.
BuildKit and the VM matrix category remain unqualified for environment-specific reasons.
The subscription is still the Startup trial;
moving to Developer is a user action.

## Completed work

### Accepted operating model

- Q7 confirmed the revised plan at `d94e50bec4`;
  the adoption record is accepted in
  `doc/decision/depot-ci-adoption.md`.
- Provider and cache supersession notes are in place.
- The plan is advisory-only,
  Developer no-overage,
  Startup trial for the pilot,
  explicit weekly
  native dispatch,
  public GitHub for untrusted source,
  main-only cache writes and bounded paid
  profiles.

### Baseline

- Owner map refreshed at `98381bfa32`:
  225 packages,
  210 active,
  13 paused,
  2 archived,
  14,301 files,
  2,600 authored task declarations,
  2,464 expanded records.
- The evaluation's 223,
  208,
  13,860 and 2,425 counts are historical for its frozen cohort.
- Fast subset:
  `module/logger`,
  `module/p-map-fork`,
  `module/toml-edit`,
  `cli/forbidden-strings` and `linter/monochromatic-lint`,
  with their unit or Nextest tests.
- GitHub run `39199930641` passed the fast-subset build and unit in 10 minutes 17 seconds.
- Whole-tree static baseline:
  85,719 findings across 1,046 files at `4f3d5a65`.
- Whole-repository unit baseline is invalid:
  it ran without built prerequisites and was dominated
  by `ERR_MODULE_NOT_FOUND` and missing `dist/final/node` modules.
- Complete build baseline is blocked:
  run `39197594870` failed on glib,
  gdk-pixbuf,
  cairo,
  Podman,
  Android,
  Qt,
  cargo vendor downloads,
  unresolved workspace imports and `canvg`.
  These are environment,
  toolchain,
  build-order and dependency failures,
  not product attribution.

### Advisory profile

- Workflow:
  `.github/workflows/advisory-core.yml`.
- Push gate:
  2 per rolling minute,
  4 per hour,
  8 per day;
  newest skipped tip marked pending.
- Pull requests run the same profile and do not consume the push budget.
- Manual dispatch bypasses the push budget for qualification.
- Changed-path static;
  five-owner fast subset build and unit.
- Static and build continue on failure;
  unit runs after a successful build;
  one final enforce step
  fails the job if any profile failed.
- Results are advisory check runs and a step summary;
  no required checks and no branch-protection
  change.
- First admitted push run `39201617300` found a changed-file static failure in
  `doc/handover/slint-ide-0x.md`.
- Manual qualification run `39202293518` passed the gate,
  build,
  unit and enforce steps.

### Native evidence

- Push and pull-request registration proven on both landing paths.
- Duplicate execution observed on GitHub and native Depot for one side-effect-free workflow.
- Native check names observed as `<workflow> / <job>` from app `219785`.
- Paused-session qualification did not reach a job and remains unqualified.
- Evidence files are under `doc/audit/`.

## Residuals

All nine planned work items have implementation records.
These residuals are explicit and are not claimed as green:

- The complete build and whole-repository unit baseline are not known-good.
  Run `2ls35xfpds` recorded the prepared-environment failure classes and skipped unit.
- Weekly matrix BuildKit is qualified by rootful Podman runs `r1h68z8tzv`,
  `391g4q5t98` and `k1d5q6x48p`.
- Weekly matrix VM is qualified by run `s6ljx0bvw8` (Linux lifecycle).
- Release-run attachment relies on check runs on the commit,
  not an explicit release workflow
  step.
- Gradle cache writes are disabled and restore-only until cache-off,
  invalidation and stale-output controls pass.
- The per-finding static triage and the full unit failure set are class-level,
  not individual product defects.
- The subscription transition to Developer is a user action.

## Open questions and blockers

- Which prepared environment or prerequisite image is used for the complete build baseline.
- The user reports product caps are configured but did not record values.
  Native CI,
  Tests and storage caps are not documented as configurable.
- Paused-session remains unqualified and is excluded from the weekly matrix.
- Release-run attachment currently relies on check runs on the commit,
  not an explicit release
  workflow step.
- The subscription transition to Developer is a user action and has not happened.

## Next action

All nine planned work items have implementation records.
The remaining work is the residual list above.
The next practical steps are:

- BuildKit is qualified;
  no further work is needed there.
- Continue triaging the complete-build failures until the unit baseline can run.

Use bounded CI for any build or unit measurement.
Do not run local full builds.
Do not enable overage or raise a cap automatically.
Record every admission and recovery attempt in the ledger.

## Environment notes

- No local build,
  test or maintenance process was running when this handover was written.
- Remote GitHub or Depot runs are independent of the local machine and may complete after shutdown.
- The local main branch was fast-forwarded to the remote tip before this handover.
- Existing uncommitted worktree changes belong to concurrent work;
  do not restore,
  stash or revert
  them.
