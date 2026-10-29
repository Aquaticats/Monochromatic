# aube vs pnpm: staying on pnpm

Decision record for whether this monorepo should replace pnpm with aube (`jdx/aube`) as its package manager.
Decision:
 rejected on 2026-06-21,
 by Aquaticat.
We stay on pnpm indefinitely.
aube is not a viable option for this repo,
 we are not betting on it becoming one,
 and we have no plans to re-evaluate it.

## Decision

- Keep pnpm as the package manager for the foreseeable future.
- Do not adopt aube.
   It cannot manage this repo today,
   and we are not relying on it to improve.
- Do not monitor aube or schedule a re-evaluation.
   If the topic returns,
   this record is the answer.

## Why pnpm stays

pnpm is comfortable,
 and comfortable is the point.
 It already does everything this repo needs,
 with no pain driving a change:

- It natively supports the machinery our supply-chain hardening depends on:
  the catalog and named catalogs,
   `workspace:` deps,
   and the full `overrides` surface in `pnpm-workspace.yaml`.
  That surface includes version floors for CVE patches,
   dependency removal (`'jspdf>canvg': '-'`),
  `link:` shims (`node-domexception`,
   `proper-lockfile`),
   a `workspace:` stub substitution for `js-yaml`,
  and `.pnpmfile` policies.
- It is mature,
   has a content-addressable store already,
   and resolves and installs the whole 117-package
  workspace without surprises.
- There is no driver (speed,
   disk,
   or security) strong enough to justify trading a working setup for a young one.
  Anything we want from a newer tool can be approached inside pnpm without a migration.

The honest summary:
 nothing is wrong,
 so nothing needs replacing.

## Why aube is rejected

We trialed aube 1.22.0 firsthand in a throwaway worktree against this repo's real config.
The rejection rests on concrete findings,
 not vibes:

- Hard blocker on our config.
  aube validates our existing `pnpm-lock.yaml` ("Lockfile is up to date"),
  but a forced re-resolve fails with `ERR_AUBE_NO_MATCHING_VERSION` on our `workspace:` stub override
  (`no version of js-yaml matches range workspace:@monochromatic-dev/pnpm-override-stub-throwing@*`).
  That means it cannot add a dependency,
   bump a version,
   or regenerate the lockfile here.
  That is day-to-day package management,
   not an edge case.
- Immaturity,
   and a version number that oversells it.
  aube's first public beta was 2026-04-18,
   and it only moved to jdx's namespace on 2026-06-09.
  It is roughly two months old.
   The 1.
  x line is not a tooling artifact:
  aube cut `v1.0.0` titled "First stable release" on 2026-04-23,
   five days after `v1.0.0-beta.1`,
  and reached 1.22.0 by 2026-06-17.
   A 1.
  x version conventionally signals a committed,
   stable public API;
  the reality is a tool still shipping weekly "pnpm-lockfile parity sweep" releases two months in,
  one of which we tripped over.
   The version signals more maturity than exists.
- It fails our test bar,
   and does not even measure it our way.
  Production Rust is around 72k lines against around 40k lines of Rust test code (roughly 1.8 to 1 the wrong way),
  plus 1491 BATS integration tests (around 30.6k lines of shell) for a closer-to-even effort ratio.
  Our standard is test code at least double production code (a 1 to 2 ratio or better);
   aube is nowhere near it.
  aube tracks parity-completeness against pnpm,
   not a coverage ratio or percentage,
  so it will never report a number we can gate on.
- Breadth we would carry but never use,
   and it will not go away.
  aube reimplements four lockfile formats (pnpm,
   npm,
   yarn,
   bun).
  For a pnpm-only repo,
   the npm,
   yarn,
   and bun support is dead surface:
  code we would inherit,
   that widens the audit and attack surface of a tool that runs lifecycle scripts,
  and that is already a demonstrated bug source (scoped-package misresolution on bun.
  lock import).
  This surface is permanent,
   not transitional:
   multi-format is the `aube-lockfile` crate's stated identity,
  the four parsers are compiled in unconditionally with no feature flag to drop them,
  and "reads any existing lockfile" is aube's core adoption pitch.
  Breadth is a migration on-ramp for aube's adoption goals,
   not an engineering virtue for us,
  and we should not expect it to shrink.
- Reduced bug visibility.
   aube's GitHub issues are disabled by deliberate jdx policy;
  feedback funnels through a single rolling discussion,
   with only triaged items promoted to the tracker.
  This is a defensible maintainer-bandwidth choice,
   but it means we cannot browse an open-bug list
  to gauge the real defect load of a tool we would depend on.

## On aube's future, and why we still say no

This is a stance,
 not a forecast we are tracking.
aube is competently engineered:
 it ports pnpm's own test suite for parity (`test/PNPM_TEST_IMPORT.md`),
runs real installs against an offline Verdaccio registry,
 merges fixes quickly,
 and has a serious security model
(a lifecycle-script jail,
 a release cooling period,
 and typosquat detection).
A competent young tool can become a good mature tool.

We are choosing not to wait on that,
 and not to spend attention finding out.
The single technical crux is the re-resolve failure on our `overrides`,
 which their parity work could close.
We are recording that only so a future reader understands the decision,
 not as a trigger to revisit.
Staying on pnpm costs us nothing we value;
 chasing aube's maturity would cost attention we would rather not spend.
If someone reopens this in the future,
 the burden is on the new evidence to overturn a setup that already works,
not on us to keep checking.

## Alternatives considered

- Stay on pnpm.
   Chosen.
   Mature,
   supports our advanced overrides and catalog,
   no migration,
   no pain.
- Bun's built-in installer.
   Rejected.
   We already use Bun as the runtime,
   so there is a consolidation argument,
  but it is still a migration,
   its catalog and overrides parity differs from pnpm,
  and it would not preserve our `link:` and stub and pnpmfile hardening cleanly.
- aube now.
   Rejected,
   per the findings above.
