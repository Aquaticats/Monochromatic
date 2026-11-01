# Audit ledger: code no unit test runs, census at `65bd3a645` and the runner entry files

The whole-suite census of 2026-10-06 and the library clusters cut from it,
the quote-loss refusal kept cold on purpose,
and the runner entry files moved into modules,
with what stayed open at the close of that passage.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Whole-suite census at `65bd3a645` and the clusters cut from it

WHOLE-SUITE CENSUS AND THE CLUSTERS CUT FROM IT (2026-10-06 UTC):
a whole-suite census was taken at `65bd3a645` (`census-5RzoUC`),
with the unit suite at 1,621 pass lines.
Its report reads:
library source,
103 files,
137 stretches over 380 lines,
4 functions never called;
entry files,
5 files,
17 stretches over 714 lines,
9 functions never called;
invariant throws counted apart,
41 stretches,
all in library source;
and 36 bundles no test loaded,
carrying 39 sources and 14,029 physical lines.
B231 records the fourteen chain tails that census listed as cold.
The clusters of this passage were cut from that census,
by file,
and given to agents in side checkouts.
The lead merged each patch,
with red cases committed apart from their fix;
what the lead measured stands in each commit's message,
and M121,
M123 and M124 record what the lead's reading and checks missed.

The library's cold stretches went in three clusters.
The files from a to l:
cases for twenty-two library files,
four states made loud,
one dead operand removed,
and one defect (B271),
in `f18d0b9b1` and `b8e30dce2`.
The `corpus-run` files:
cases in seventeen test files and one new one,
five defaults for states their callers rule out made loud,
and two defects (B272,
B273),
in `96dad6b2c` and `5723cd071`.
The files from m to z:
cases in seventeen test files and six new ones,
eleven silent defaults made loud or total,
and three defects (B274 to B276),
in `f45278048`,
`71974714f` and `230623bec`.
The agents of the first two clusters each report a census over their test files
that lists no stretch of the cluster's files the whole-suite census listed,
and all three list their new throws among the invariant throws
(the agents' accounts,
not rerun by the lead).
A fourth piece closed what the third left,
the fallbacks of `request-pace.ts` and the rethrow of `transient-retry.ts` (B292).

## The quote-loss refusal kept cold on purpose

One stretch of library source stays cold on purpose.
`translate-slice.ts` keeps its quote-loss refusal,
which ships the whole archive by the owner's rule of 2026-09-27 and which no input reaches,
since the floor counts the same passages.
The lead's ruling,
open to the owner's veto:
it stays as a defence the owner decided,
and the final census is expected to list exactly this stretch in library source.
The owner ruled on 2026-10-10 (UTC),
asked in this form,
that the refusal stays as it is,
over turning it into an `unreachable:` throw,
which would crash where a run must ship,
and over removing it;
the card for B113 on the review page states the same call.

## Runner entry files moved into modules

The runner entry files came next,
the work this entry had left for after the library.
A runner's procedures moved out of its entry file into modules beside it,
each with a test file of its own over scripted clients and readers,
and each command gained a suite that runs it as built,
in a child process that holds no key (B291).
The entry file keeps its wiring and one call.
Nine clusters,
34 commands,
were merged by `f8e99b84d`,
with entry lines after and before by each agent's count:

- `ledger-report`,
  `run-timing-report`,
  `spend-report` and `meter-report`
  (58,
  32,
  28 and 34 lines,
  from 378,
  271,
  463 and 496),
  in `5e116395a` and `387af02b7` (B278 to B281);
- `probe-sensitivity`,
  `probe-verify`,
  `probe-relabel`,
  `checker-sensitivity` and `audit-sensitivity`
  (61,
  57,
  78,
  50 and 118,
  from 190,
  264,
  306,
  363 and 523),
  in `139a31f4a` and `abb506b7b` (B282,
  B283);
- `score-attribution`,
  `score-verify`,
  `score-agreement`,
  `score-probe` and `score-crosscheck`
  (32,
  42,
  34,
  39 and 38,
  from 280,
  325,
  353,
  401 and 414),
  in `a13f71ba8` and `1088e72d9` (B284 to B287);
- `cap-census`,
  `slice-cost-report` and `cache-account-audit`
  (17,
  16 and 40,
  from 443,
  533 and 462),
  in `53e25b458` (B288 to B290);
- `sentinel-probe`,
  `displacement-probe`,
  `coverage-control-probe`,
  `coverage-probe` and `translate-probe`
  (35,
  62,
  37,
  75 and 44,
  from 212,
  588,
  321,
  437 and 394),
  in `7d2ca2604` and `da8a963cc` (B294);
- `model-health`,
  `model-catalog`,
  `roster-card`,
  `draw-sample`,
  `damage-sample` and `budget-sample`
  (40,
  38,
  42,
  42,
  55 and 41,
  from 223,
  129,
  180,
  427,
  412 and 174),
  in `17a66fa66` and `097b6ac40` (B295 to B298);
- `editor-calibrate` and `editor-width-probe`
  (51 and 56,
  from 596 and 312),
  in `327ea1de0` and `77a213478` (B299);
- `corpus-pass`
  (116,
  from 675),
  in `ea16d3b51` and `9a68e3c78` (B302 to B304);
- `producer-calibrate`,
  `recall-benchmark` and `roster-bench`
  (65,
  51 and 73,
  from 315,
  446 and 405),
  in `cd78288d7` and `f8e99b84d` (B305 to B307).

B308 records the counts those moves found standing before a fixed word.
Each agent reports a census over its own test files that shows its bundles loaded,
and no cold stretch in its entry files or new modules beyond the invariant throws its commit names
(the agents' accounts,
not rerun by the lead;
the editor cluster's is in its report and not in its commit's message;
two entry files of the sampler cluster each keep one cold character,
which B296 records).
What a keyless child cannot reach stays unreached,
and each commit says what:
a run to its end,
a round that reaches a provider,
a real settlement.

## The suite at the close of that passage, and what stayed open

The whole suite at the close of this passage,
on `f8e99b84d` in a side checkout:
1,938 `[PASS]` lines and 2 `[FAIL]` lines,
both for one case that fails under load (B300,
B305),
the lint finding nothing,
and the 41 source scans passing.
Until `63f8f838a` every count of pass lines left out the cases B277 records.

Still open at `f8e99b84d`:

- three runner clusters,
  seven commands,
  were not merged at that commit
  (`verify-published` and `window-trial-probe`;
  `slice-census` and `coverage-census`;
  `judge-fidelity-probe`,
  `rendering-audit-settled` and `rendering-audit-settled-report`);
- no whole-suite census has been taken since `65bd3a645`,
  so the counts this passage rests on are the agents' own,
  and the recount B231 owes is still owed;
- the moved procedures of `ledger-report` and `meter-report` set the exit code themselves (B281),
  where the rule for the later clusters is that a procedure returns its code;
- the clusters' observations left to rule on,
  each in the entry of its cluster under "Open".
