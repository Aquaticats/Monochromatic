# Task-slice owner vet report

Status:
 reopened vet for the chosen candidate;
 scoring complete in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md),
 recommendation given,
 adoption depth open.

Lifecycle phase:
 scored and recommended;
 this report adds the adoption-depth audit
 (license,
 provenance,
 maintenance,
 production source,
 human auditability)
 for the chosen candidate.

Subject:
 task slice owner.

Decision scope:
 select the owner for Mise's task-execution slice:
 task discovery,
 per-package task addressing,
 dependency edges,
 task bodies,
 watch,
 failure tolerance,
 and machine-readable listing,
 at 222 packages and 1,811 tasks.

Start date:
 2026-10-29.

Last updated:
 2026-10-29.

Governing skill commit:
`37117e36397e30233062828581a5e38217da9458`.

Governing skill SHA-256:
`552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.

Compatibility fingerprint:
`e4a30abd2c6ffb3c3bb2dfa178aae27a546962df505282d609158352d361a123`.
Built from schema version 1,
 subject,
 decision scope,
 hard constraints,
 deployment,
 trust boundary,
 incumbent,
 base categories,
 and overlays,
 canonicalized by sorted-key JSON
 and hashed with SHA-256.

Active audit owner:
 the agent session that ran the eight pilots
 on 2026-10-29.

Prior compatible report:
 none.
The 2026-09-16 monorepo manager vet
 ([`tech-monorepo-manager-vet-2026-09-16.md`](tech-monorepo-manager-vet-2026-09-16.md))
 carries a different subject
 and a different decision scope
 (file-enforcer,
 watch daemon with RPC,
 cgroup sandboxing,
 one-file shipping),
 so its fingerprint does not match
 and its "no candidate is recommendable" result
 is not reusable here.

## Context

The task slice is one slice of the monorepo manager.
The accepted endpoint
 ([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md))
 names `meow` as the monorepo-manager owner,
 and `meow` is plan-only for the task slice,
 so the slice has no implemented owner today.
The incumbent is Mise,
 measured with two named holes:
 nothing is cached at all
 ("Nothing is cached,
 so the aggregates are slow by construction"),
 and root `mise run test` runs only the three root suites
 and misses 15 packages' tests
 ([`doc/planning/monorepo-manager-route-research/mise-task-usage.md`](../planning/monorepo-manager-route-research/mise-task-usage.md)).

The scoring axis is capability for the slice job.
The user rejected parity with the current status quo
 as an axis
 ("Parity with the current status quo is poor scoring axis.
Assume we're fine with huge huge refactors and everything."),
 so migration and shape cost score nothing
 and are recorded as implementation notes.
The tightening that records this lives in `AGENTS.md` as `OPT`:
 "Score capability for the target workload,
 never fidelity to incumbents."

## Hard constraints

The slice's capability catalog,
 each a hard requirement of the dev loop:

- partial invocation of one package's one task;
- paths containing spaces;
- sequential and parallel groups;
- missing optional package builds;
- intentional failure tolerance;
- tests continuing after build failure where required;
- retained failure causes;
- machine-readable task listing;
- correct aggregate exit status.

The acceptance boundary
 `package/dev-script/task-util/src/build-and-test.unit.test.ts`
 carries the failure-tolerance and tests-after-build-failure duties.

## Base components and overlays

- Base:
  inspectable open-source local technology
  (build tool or task runner,
  installed locally,
  source inspectable).
- Overlay:
  incumbent dependency replacement
  (Mise's task-execution slice).
- Overlay:
  high-trust execution
  (the owner executes arbitrary package task bodies
  in the developer loop and in CI).

No managed service,
 no proprietary component,
 no native or prebuilt artifact without source mapping
 is under consideration.

## Frozen criteria and weights

Nine criteria,
 weight 1 each,
 because the user named no priority.
Full definitions and thresholds are in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md),
 section "Scoring axis".

- A,
  edge fidelity;
- B,
  body model;
- C,
  failure semantics;
- D,
  optional-task robustness;
- E,
  watch;
- F,
  caching and affected selection;
- G,
  machine-readable introspection;
- H,
  scale at 222 packages;
- I,
  maintenance depth.

Documentation trust is a scored deduction of `4 - trust` points,
 not a criterion.

Weight sensitivity is published in the comparison doc
 (duty only,
 duty dominant,
 equal).
Pants is first under every weighting tested.

## Candidate ledger and where its evidence lives

Discovery,
 screening,
 and validation ran as eight measured pilots
 against the task-slice catalog,
 in disposable fixtures whose paths contain spaces.
The per-gate measurements,
 score arithmetic,
 sensitivity runs,
 and per-adjacent-pair reasons are in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md).

- Pants 2.33.1:
  validated,
  scored 32,
  recommended,
  chosen by the user 2026-10-29.
  Adoption-depth audit in this report.
- Please 17.34.0:
  validated,
  scored 26,
  runner-up.
- Buck2 2026-10-14:
  validated,
  scored 26,
  third on trust tie-break.
- Nadle 0.6.1:
  validated,
  scored 25.
- Tilt 0.37.8:
  validated,
  scored 21.
- pnpm 12.16:
  validated,
  scored 19.
- projen 0.104.19:
  validated,
  scored 13.
- Poe the Poet 0.49.0:
  validated,
  scored 12.
- meow as designed:
  not scorable,
  plan-only,
  and the custom-implementation gate
  is not met while Pants clears every named duty.
- Keep Mise:
  incumbent baseline,
  scored 24,
  trust not audited.

## Ranking and adjacent reasons

The full ranking with every adjacent reason
 and its flip condition is in
[`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md),
 section "Why each adjacent pair".
Condensed order:

1.  Pants 2.33.1
2.  Please 17.34.0
3.  Buck2 2026-10-14
4.  Nadle 0.6.1
5.  Keep Mise
6.  Tilt 0.37.8
7.  pnpm 12.16
8.  projen 0.104.19
9.  Poe the Poet 0.49.0

## Adoption-depth audit for Pants 2.33.1

Filled from the maintenance and source audits
 recorded below.
Every claim cites a primary source,
 a repository path,
 or a reproduced command.

### License and provenance

Pending:
 license text,
 dependency surface,
 and release provenance for 2.33.1.

### Maintenance

Pending:
 the 10 most recently updated issues and pull requests,
 maintainer concentration,
 merge latency,
 release recency,
 and upstream acknowledgement of the three
 pilot-measured defects
 (dependents silently omitted from the run summary,
 named missing address poisoning `::`,
 the 30s default process timeout).

### Production source

Pending:
 `adhoc_tool` argv handling,
 `peek` JSON listing,
 `--loop` watch,
 local cache and affected selection,
 and failure reporting.

### Human auditability

Pending:
 non-test source lines and file count,
 dependency counts,
 and where credentials,
 network,
 filesystem,
 and process execution concentrate.

## Confidence and evidence limits

- The pilot measurements are fixture-scale
  except pnpm and Keep Mise,
  which were exercised at 222 packages.
- Category judgments on scale
  (Pants,
  Please,
  Buck2,
  Tilt)
  are not corpus probes.
- The Buck2 and Please totals tie at 26
  and the tie breaks on documentation trust;
  a reader who does not weigh trust
  should treat them as co-second.

## Recommendation

Pants 2.33.1,
 chosen by the user on 2026-10-29
 when asked which owner he wants
 for the task slice.
This report is the reopened vet
 that adoption waits on;
 it is complete when the
 adoption-depth audit
 above is filled.
