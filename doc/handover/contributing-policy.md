# Contributing policy decision log

Living record of the repo's contribution-policy work,
updated across sessions.
Tracks what we settled,
 why,
 what stays open,
 and the planned shape of the deliverable.

## Status

Planning complete after a second gap audit;
 ready to draft on the maintainer's go.
`CONTRIBUTING.md` not yet written.
All blocking decisions are settled,
 including the audit findings;
the code of conduct is deferred,
 and a PR-template checklist plus a contributor setup runbook
are noted as optional,
 non-blocking follow-ups (see open items).

## Goal

Write a root `CONTRIBUTING.md` in the spirit of [xkcd 810 "Constructive"][xkcd-810].
The comic imagines defeating spam by forcing every applicant to contribute constructively;
its [comment thread][explain-810] is the rebuttal,
years of readers noticing the gate is gameable.
The deliverable keeps the comic's dream and removes the part that broke.

## The spine

810's gate failed because its rating layer was a crowd,
and crowds are gameable
(vote-rigging,
 sockpuppets,
 and now LLM-cheap "looks helpful" output).
Monochromatic replaces the gameable crowd with two things a crowd cannot be:

- A machine:
   it builds,
   types check,
   lint sits at zero,
   tests hold.
   Objective,
   unfakeable.
- A single accountable judge:
   the maintainer's judgment of project fit,
  in the maintainer's words,
   "does this aquatic cats.
  "

One trusted judge plus a machine is the configuration the comment thread kept reaching for
and never named.

## Decisions

### Voice and register

Straight professional prose,
 manifesto-forward,
 genuinely hoping for contributions.

### Placement

Root `CONTRIBUTING.md`.
This contradicts the `DPL` rule
(root holds only README,
 SECURITY,
 AGENTS,
 CLAUDE,
 LICENSES),
so the rulebook gets updated to bless it:
add `CONTRIBUTING.md` to the `DPL` allowed-root-files list in `AGENTS.md`,
then regenerate `CLAUDE.md` through file-enforcer (`WC2`).

### Depth

Thin frame plus pointers to [`AGENTS.md`](../../AGENTS.md),
 [`README.md`](../../README.md),
and [`SECURITY.md`](../../SECURITY.md).
The file does not restate their mechanics.

### Bots and AI

Welcome,
 no disclosure required.
Maps onto the existing `ready-for-agent` label,
 which already presumes agent work.
A contribution is judged on its result,
 not its author's nature.

### Approval workflow

No PR without an approved issue.
An issue opens as `needs-triage`;
the maintainer approves it by applying `ready-for-agent` or `ready-for-human`,
or closes the door with `wontfix` or `needs-info`.
A PR implements an approved issue.
Merge needs both gates:
 `mise run validate` green,
 and maintainer review.
Label vocabulary lives in [`doc/agent/triage-labels.md`](../agent/triage-labels.md).

The issue-first rule has exactly two exceptions:
the tiny-fix carve-out below,
 and security,
which routes through the private advisory flow and never a public issue (see below).
A PR references its approved issue by number with a `Closes #N` line,
which auto-closes the issue when the PR merges to the default branch,
per [`doc/agent/issue-tracker.md`](../agent/issue-tracker.md).

### Contribution size

Sized at the gate,
 no numeric cap.
One approved vertical-slice issue per PR.
Work too big for one slice is split into several approved issues before any code.
Size is decided when the issue earns its label,
 not argued at the PR.

### Tiny-fix carve-out

A tiny fix may open a PR directly with no issue.
Tiny is defined exactly:
 one typo,
 or one doc paragraph,
 or one comment for one line of code.

### Threat model stays out of the public doc

`CONTRIBUTING.md` states rules only.
It does not enumerate the attacks,
because naming the games in the contributor-facing doc invites them.
The threat-model reasoning is recorded in this handover instead (see below).

### Maintainer obligation

Stay warmly inviting.
Do not advertise "no guaranteed response" now;
flip to an asymmetric stance only if contribution volume actually becomes a problem.

### Test-quality bar

Mutation testing and fuzzing are contributor requirements,
 not maintainer-only tools.
External contributors run them
and are held to at least the bar the maintainer holds for themselves.
Rationale,
 in the maintainer's words:
 no point holding external contributors to a lower bar.

Enforced twice over.
The contributor runs the touched package's mutation and fuzz tasks locally and supplies the result,
so the cost sits on the contributor,
 per the spine.
Running locally is mandatory:
no Podman and baked-image setup means no code contribution to a harnessed package.
Because it is a hard requirement,
 the policy must stay actionable;
it points contributors at the setup they need
(README build steps,
 the per-package `test:mutation` and fuzz mise tasks,
and [`doc/decision/mutation-testing.md`](../decision/mutation-testing.md) for the container model).
CI then re-runs them,
so a contributor who claims green falsely is caught by the machine,
 not taken on trust.
Coverage is per-package today,
but the plan is to make mutation and fuzz exist almost everywhere,
so the bar trends universal rather than uneven.

### Provenance warranty

Include a warranty line:
by contributing,
 you warrant you have the right to contribute the work under the repo's license.
Code is LGPL-3.0-or-later;
 non-code is CC-BY-SA-4.0.
This anchors the no-disclosure bot stance to an accountable submitter under copyleft.

It is a prose line,
 not an enforced DCO sign-off:
opening a PR constitutes the warranty.
Commit signing is not required;
nothing enforces it today,
 and it would wall out the bots this repo welcomes.

### Claiming and duplicate work

No formal claiming convention for now.
The first PR that passes both gates wins;
a claiming or self-assignment scheme is revisited only if volume causes real collisions.
Self-assignment is itself gameable (squatting an issue to block others),
which is part of why it stays out for now.

### Footnote

Credit [xkcd 810 "Constructive"][xkcd-810] as the inspiration.

## Threat model (kept here, deliberately not in CONTRIBUTING.md)

Each item is an attack the comic or its comment thread named,
 paired with this repo's answer.

- Plausible slop ("easier to look constructive than be it"):
  the machine gate is objective,
   so plausibility earns nothing.
- Tests that look like tests but assert nothing:
  Stryker mutation testing and fast-check fuzzing require that a test kill mutants
  and survive generated input,
   not merely execute lines.
- Faked green (a contributor claims the tests pass but they do not):
  CI re-runs mutation and fuzz,
   so a false claim is caught by the machine,
   not taken on trust.
- Gameable crowd rating:
  there is no crowd,
   only one machine and one named judge,
   neither sockpuppetable.
- Flood or denial-of-service against the gate (infinite LLM output,
   finite evaluation):
  the governing principle is that the gate must cost the contributor more than the maintainer.
  The contributor runs validate and supplies the green;
  an issue must earn a label before anyone writes code;
  `needs-triage` to `wontfix` is a near-free reject.
- "I did everything right and got rejected on taste":
  approval-before-PR fires the taste gate before the work,
   not at PR time.
- Provenance under copyleft:
  whoever opens the PR is accountable for license and provenance,
   however the code was generated.

## Open items and tensions

- Code of conduct (deferred,
   not blocking):
  deliberately out of the first `CONTRIBUTING.md`.
  Planned for a later pass after the maintainer's own deliberation;
  the machine-plus-judge design enforces constructiveness through gates,
   so it is not urgent.
- Visibility of this doc:
  it is committed,
   so it is repo-visible.
  The threat model is mild to expose,
   but if exposure is unwanted,
   rename to `.local.md`.
- Sequencing:
  the DPL bless and the `CLAUDE.md` regen happen alongside writing `CONTRIBUTING.md`,
   on "go.
  "
- PR-template checklist (optional follow-up):
  a pull-request template encoding the gate (linked approved issue,
   validate green,
  mutation and fuzz supplied locally) would operationalize the policy;
  decide keep-or-drop later,
   it does not block the first `CONTRIBUTING.md`.
- Contributor setup runbook (optional follow-up):
  if the README and mutation-testing pointers prove insufficient
  for standing up the Podman-based mutation and fuzz runs,
  write a `doc/runbook/` setup guide.

## Planned CONTRIBUTING.md outline

- Opening:
   the one idea (keep 810's dream,
   delete the crowd),
   stated without naming attacks.
- Same door for bots and humans,
   no disclosure.
- How to contribute:
   open an issue,
   wait for a `ready-*` label,
   PR against it,
   both gates must pass.
- Two exceptions to issue-first:
   the tiny-fix carve-out,
   and security.
- Tiny-fix carve-out,
   defined exactly.
- Security:
   vulnerabilities go through `SECURITY.md`'s private advisory,
   never a public issue.
- Size:
   one approved slice per PR,
   big work split into approved issues first.
- The bar:
   validates clean,
   lint at zero,
   locally-run mutation and fuzz survivable tests,
   TSDoc,
  docs placement,
   Conventional Commits,
   no AI-attribution trailers;
   full rules in `AGENTS.md`.
- Setup pointer:
   mutation and fuzz run locally and need Podman and baked images;
  point to README and the mutation-testing doc.
- Issues count as contributions.
- Pointers:
   `AGENTS.md`,
   `README.md`,
   `SECURITY.md`.
- License plus a prose warranty line (no DCO,
   no required signing).
- Footnote crediting xkcd 810.

## Verification

Before declaring the deliverable done:

- Pass the repo's own doc bar:
   dprint format,
   remark,
   Harper prose,
   and forbidden-strings in CI.
  Root `CONTRIBUTING.md` is not in `.remarkignore`,
   so it is linted like any other doc.
- Confirm every internal link resolves and the xkcd links are correct.
- Run file-enforcer and confirm `CLAUDE.md` reflects the `DPL` edit.
- The contribution policy must itself clear the contribution gate it describes.

## Files in scope

- `CONTRIBUTING.md`:
   new,
   repo root.
- `AGENTS.md`:
   add `CONTRIBUTING.md` to the `DPL` allowed-root-files list.
- `CLAUDE.md`:
   regenerate from `AGENTS.md` via file-enforcer.
- `doc/handover/contributing-policy.md`:
   this log.

## Changelog

- 2026-06-26:
  decision log created.
  Captured voice,
   placement,
   depth,
   bot stance,
   approval workflow,
   size,
  tiny-fix carve-out,
   rules-only public doc,
   warm maintainer obligation,
  mutation and fuzz contributor bar,
   provenance warranty,
   footnote.
- 2026-06-26:
  resolved the test-quality enforcement open item.
  Contributor supplies the passing mutation and fuzz result,
   CI re-runs to catch a faked green;
  harnesses are being rolled out toward almost every package.
  Added a code-of-conduct keep-or-drop open item.
- 2026-06-26:
  deferred the code of conduct out of the first `CONTRIBUTING.md`;
   it lands in a later pass.
  Plan now complete and ready to draft.
- 2026-06-26:
  second gap audit.
  Added a security exception to issue-first (private advisory),
   a self-lint verification step,
  and a PR-to-issue linking convention.
  Decided:
   mutation and fuzz run locally with a setup pointer,
  no claiming convention,
   a prose warranty (no DCO),
   and no required commit signing.
  Noted optional follow-ups:
   PR-template checklist and a contributor setup runbook.

[xkcd-810]: https://xkcd.com/810/
[explain-810]: https://www.explainxkcd.com/wiki/index.php/810:_Constructive
