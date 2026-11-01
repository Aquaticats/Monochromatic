# Weekly native matrix design

Status:
work item 4 design.
Source plan:
[the expanded CI rollout plan](depot-expanded-ci-rollout.md).
Admission:
[the admission ledger](depot-admission-ledger.md).
Registration evidence:
[the registration control](../audit/depot-registration-control-2026-10-18.evidence.json).

## Purpose

Run every relevant Depot native capability once per week in one bounded,
explicitly dispatched matrix.
The matrix is capability evidence,
not a required check and not a product verdict.
Publication stays on GitHub.
Paused-session SSH,
`--ssh-after-step`,
tmate and interactive session strings are excluded.

## Trigger and duplicate control

The only trigger is explicit `depot ci dispatch`.
There is no automatic push or pull-request trigger for the native matrix.
The admission gate decides the reservation before dispatch.
The matrix records its run identity,
the reservation identity and the resulting attempt.
A duplicate dispatch is a defect because the workflow is dispatch-only and the ledger reservation
is single-use.

## Categories

Each category runs a named owning task or command and emits at least one case or artifact identity.
The category list follows the Delivery model:

- static checks;
- Node and Rust test consumers;
- browser and E2E consumers;
- packaged artifacts and host consumers;
- OS,
  architecture and application consumers;
- VM,
  desktop and Android consumers;
- BuildKit and Bake container producers;
- Depot Tests;
- cache and Disk;
- SDK VM profiles.

A category with no qualified command at dispatch time is recorded as incomplete for that period.
It is never replaced with an unrelated command.

## Provisional bounds

These bounds are the reservation basis for the first dispatch.
They are conservative and are replaced only by a measured admission update.

- Provider:
  native Depot CI on one 8x32 runner.
- Wall cap:
  120 minutes.
- Native compute reservation:
  240 native minutes.
- Reserved dollars:
  10 USD against the 200 USD combined ceiling.
- Retention:
  the standard run and artifact retention for the period.
- Per-category timeout:
  static 10 minutes;
  Node and Rust 25;
  browser and E2E 30;
  packaged 20;
  OS,
  architecture and application 25;
  VM,
  desktop and Android 30;
  BuildKit and Bake 25;
  Depot Tests 20;
  cache and Disk 10;
  SDK VM 15.
- The sum of category timeouts exceeds the wall cap by design.
  Categories run within the one bounded run and the wall cap stops overrun.

## Evidence format

Every category records:

- category name;
- provider and runner identity;
- owning task or command;
- case or artifact identity;
- start and finish timestamps;
- conclusion and failure class;
- retention reference.

The matrix records the admission reservation identity and the final wall and meter usage.
The evidence register links the matrix run,
the reservation attempt and every category record.

## Acceptance

- One explicit dispatch produces every named category or a recorded incomplete reason.
- Every produced category has a provider,
  case or artifact identity,
  and conclusion.
- No duplicate execution occurs.
- Wall and native meter usage stay within the reservation.
- The ledger attempt is appended after the run.
- A second clean dispatch is idempotent in identities and evidence shape.

## Current state

The design and provisional bounds are recorded.
The native workflow,
reservation file and first dispatch are not built.
The admission gate already handles the reservation decision.
