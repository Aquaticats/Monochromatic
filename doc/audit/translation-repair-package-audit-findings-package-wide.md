# Package audit: findings that cross the package

The A findings,
A-1 to A-6:
the roster sent to a provider that cannot serve it,
and what the README,
the open decisions,
the handover and the test names got wrong.

Part of [Translation-repair package audit](translation-repair-package-audit.md),
which lists every file of this record.

## A-1, BLOCKER, verified: half the roster is sent to a provider that cannot serve it (`#235`)

`src/corpus-run/run-config.ts:611-617`:
`createRunClient` returns the bare Synthetic client when `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY`
is unset,
and `src/synthetic-client.ts:265` sends any model id to the wire with no catalog check,
so the five Charm-Hyper-only roster labels fail every call with HTTP 400 while quorum
(`stage-quorum.ts:167`,
five of ten,
met with `>=`) lets every round stand.
Evidence and the fix design are on `#235`;
the launch that triggered it ran `node dist/...` directly from a fork worktree without secrets.

FIXED 2026-08-26:
guards in `e0010019f`,
fix in `8b289c3ab`.
`createRunClient` requires both keys (a missing one is a stated refusal naming the variable,
exit 6),
the Synthetic client refuses any id outside its catalog before the wire on both surfaces,
every call through the factory's client is counted on `RUN_SEATS`,
and `reportingRefusals` prints `SEAT` lines plus a `SEATS DARK:` line at the end of every command.
Each of five fix lines was removed in turn,
rebuilt,
and its suite failed
(2,
3,
2,
4,
2 failing lines);
each was then restored and passed.
Negative control:
a bare `node` launch with no Hyper key exits 6 with the variable name,
no frames,
no `SEAT` lines.
Live verification,
2026-08-26:
a one-slice `editor-calibrate` under `mise run` from this worktree exited 0
with rounds `10/10` three times and `9/10` twice (never `5/10`),
zero `hf: prefix` refusals,
every one of the ten seats asked five times,
nine usable five of five,
`qwen3.8-max` usable three and thrown two (a wobble,
not a dark seat),
and no `SEATS DARK:` line.
The calibration's own coverage sentence still said six of ten seats wrote something;
that is standing coverage,
and the `SEAT` lines now separate it from darkness.
Correction to the earlier diagnosis:
the fork worktree carries an encrypted `.env.local.json` byte-identical to the main one;
the half-dark run had been launched with bare `node`,
which is what left the key unset.

## A-2, MAJOR, verified: a missing key is reported as a fault, and its name is muted

`src/corpus-run/run-config.ts:45-60`:
`RunConfigError` carries no `messageNamesOnly` marker,
so `reportingRefusals` (`src/corpus-run/cli-refusal.ts`) prints `refused by RunConfigError`
plus a fault stack and exits 5.
The variable name,
the one thing the operator needs,
never reaches the terminal.
Fix:
make it a stated refusal (exit 6),
with the `#235` change.

FIXED 2026-08-26 in `8b289c3ab`:
`RunConfigError` extends `StatedRefusalError`,
so the variable name is repeated at exit 6.

## A-3, MINOR, verified: the README documents the `#235` defect as a feature

`README.md:418-423`,
quoted under the documentation layer.
Fixed with `#235`.

FIXED 2026-08-26 in `8b289c3ab`:
the credentials section now says the second key is required and documents the closing `SEAT` lines.

## A-4, MINOR, verified: the open-decisions document is stale against its own answers

Detail under the documentation layer.
Fix:
a status line per question at the top of the planning document pointing at the record
that answered it.

FIXED 2026-08-26:
a status block under the title names the record answering each of the ten questions.

## A-5, MINOR, verified: the handover exceeds its own cap by 1440 lines

Fix:
move the oldest closed sections verbatim into `doc/handover/translation-repair-history.md`,
per the rule the handover states in its preamble.

FIXED 2026-08-26:
25 closed sections (1844 lines) moved verbatim into the history under a dated marker,
checked
by a line-multiset comparison of the two files before and after;
the handover stands at 1775 lines.

## A-6, MINOR, verified: four test names carry an all-caps verdict word (`TNM`)

`src/corpus-run/bench-report-groups.unit.test.ts:132` (`PASS`),
`src/refine-phase.unit.test.ts:412` (`PASSED`),
`src/corpus-run/run-timing.unit.test.ts:178` (`PASSES OVER`),
`src/budget-routing.unit.test.ts:174` (`FAILS OVER`).
The runner's own verdict prefix is `] PASS `,
so prefix-anchored counts are unaffected,
but a bare-substring grep reports the opposite of the truth on these four.
Fix:
reword (`ACCEPTS`,
`KEEPS`,
`SKIPS`,
`ROUTES TO HYPER`).

FIXED 2026-08-26 in `70a520e7b`:
the four names read RUN,
CLEARED,
SKIPS and SWITCHES TO HYPER,
and two more
found by a package-wide sweep (`PASSES quotas through`,
`PASSES an unsettled ... answer through`) read FORWARDS;
the sweep now finds no all-caps verdict word in any test name.
