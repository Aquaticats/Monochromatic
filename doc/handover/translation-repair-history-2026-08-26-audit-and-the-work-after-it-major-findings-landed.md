# Translation repair history 2026-08-26 audit: the major findings landed

The state of the tree when the fixes began,
and every major finding landed with its commit and its checks.

Part of [Translation repair history: 2026-08-26, the package audit and the work after it](translation-repair-history-2026-08-26-audit-and-the-work-after-it.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The state of the tree when the fixes began, and the major findings landed

### State of the tree

Worktree `/var/home/user/worktrees/translation-repair`,
branch `translation-repair-rebased`,
HEAD `bf7d6afbd` (`#247`),
clean.
`#235` verified live 2026-08-26 (`editor-calibrate -- 1` under `mise run` from this worktree,
exit 0):
rounds `10/10` three times and `9/10` twice,
zero `hf: prefix` refusals,
all ten seats asked five times,
nine usable five of five,
`qwen3.8-max` usable three and thrown two,
no `SEATS DARK:` line.
The run wrote its `SEAT` lines to stderr and its `METERS` and `round:` lines to stdout;
its throwaway runs directory is named in `~/temp/agent/vub-calibrate-current.txt`.
`#247` verified live 2026-08-26 (`probe-sensitivity` under `mise run`,
exit 0,
output under
`~/temp/agent/vub-sensitivity-*`):
eighteen `SENSITIVITY` lines with `list=` and `issue=`,
three checker seats
usable eighteen of eighteen,
no dark seat;
the rendered prompt measurably silences two of three probers on
`deletion/mislabelled` (`removal=1`,
`preExisting=2`) where the withheld prompt raises `removal=3`,
and the screen moves nothing on any region.
`#257` landed in `f199b70cf` and `bab444350` (HEAD):
the round-three instruments read legacy artifacts from the root again.
Note for the record:
`f199b70cf`'s message overclaimed,
one of its two legacy cases was failing when committed;
`bab444350` fixed the fixture and says so in its message,
per the no-amend rule.
`#257` verified live 2026-08-26 (`probe-relabel` under `mise run` against the round-three draw,
exit 0,
output under `~/temp/agent/vub-relabel-*`):
5 damaged regions and 10 controls rebuilt,
each under all three arms,
three checker seats 45 of 45 usable,
no dark seat;
that run is also the relabel half of `#247`'s verification.
No run is in flight.

### The major findings landed, each with its commit

`#237` landed in `c19d203c6` (lint tidy `82db8fb4e`,
HEAD):
the four stdout printers go through `refusalText`,
and the two comparison checks name differing fields instead of quoting rows.
`#238` landed in `1a96979ad` (HEAD):
a silent stage is neither cached nor resumed,
and this worktree's caches
were scanned clean (150 files,
none carrying the finding).
The day's first whole-suite `buildAndTest` (after `#238`) failed 6 of 764 suites that the per-commit single-suite
runs had not covered:
the marked-class inventory (`message-names-only.unit.test.ts`) did not list the two new
stated refusals,
`RunConfigError` forwarded an arbitrary message while inheriting the marker,
and a Synthetic wire
test asked the Synthetic client for a Charm Hyper label.
Fixed in `6d19c68a0` (`RunConfigError` now writes its own
sentence from a variable name);
a second whole-suite run is in flight to confirm 0 failures.
LESSON,
in force:
a fix is not landed until `buildAndTest` passes whole;
the per-suite runs after each commit are
necessary and not sufficient,
because inventory-style suites read the whole source.
The second whole-suite run,
at `6d19c68a0`,
passed 759 of 759.
`#239` landed in `7103ae59c`:
a sole-adopted envelope records an `adopted` round and keeps its authors;
the whole suite passed 759 of 759 behind it.
`#240` landed in `fd8ac6c7e`:
the re-ask takes the Synthetic slot it releases;
the whole suite passed 759 of 759.
`#241` landed in `bbbd5f2c5` and `995b76f30`:
the roster-to-catalog proof is a value and a test;
the whole suite passed 760 of 760.
`#242` landed in `3bc37b4f1`:
a tool answer is kept apart from the prose written before it;
the whole suite passed 760 of 760.
`#243` landed in `caee057fd`:
stale-lock eviction is a rename and release is owner-checked;
the whole suite passed 760 of 760.
`#244` landed in `0fb6a8ad8`:
the marked size refusal keeps the gateway's words off its message;
the whole suite passed 761 of 761.
`#245` landed in `4eaa89ede`:
pairing agreement is counted over every voice's pairs,
strictly increasing;
the whole suite passed 762 of 762.
`#246` landed in `b00f9d3b2`:
ledger files are named per launch,
so a relaunch appends;
the whole suite passed 763 of 763.
`#248` landed in `b88d6a947` (HEAD):
the damage sheet tells its grader it mixes flagged and silent items,
and the two ungraded sheets on disk were reframed in place;
a whole-suite run is in flight.
EXPECTED REBUY,
not a cache defect:
`#245` changed what the pairing stages emit,
so the pipeline digest moves and
every pairing cache (`pairing.` and `section-pair.` namespaces) retires on the next run;
the first run after it
buys every pairing round again,
once.
Advisor checkpoint 2026-08-26 (after `#248`):
the queue through `#248` needs no reopening;
for `#249` and `#255`
record the section pairing in the artifact the way `blockPairing` was recorded (omitted when nobody was asked,
named absence on read,
no generation bump if exact-keys tolerates an absent listed key),
carve the two probes
from settled artifacts through one shared helper,
give the rendering verifier a third outcome `unverifiable`
for artifacts without a stored section pairing,
and label the census's artifact-less rows as the deterministic
baseline.
Build order:
the preparation field and helper,
then `#255`,
then `#249`,
then `#250`.
The owed doc passes (A-4,
A-5,
A-6) stay queued behind the MAJORs and close `#236`.
`#249` and `#255` LANDED 2026-08-26 in three commits,
each GFP-proven and gated on types,
oxlint and the touched
suites before commit:
`26f346de8` (the artifact records `preparation.sectionPairing` as `deterministic` or
`supplied`,
always written,
read as `unrecorded` on older files;
guards fail 2,
3 and 4/7 cases),
`d2c580995`
(`rebuildPreparation` and the three-outcome provenance verdict `verified`/`refused`/`unverifiable`;
guards fail 2
cases each),
`459482191` (`settled-carve.ts`,
both probes walk settled entries,
the census labels its rows and
opens with `CENSUS carve:`;
guards fail 2,
3,
2 and 2 cases).
POPULATION FACT learned at the boundary:
this worktree's runs directory (`node_modules/.monochromatic/
translation-repair-runs/artifacts`) holds 56 LEGACY artifacts,
none two-lane,
so the probes carve nothing there;
`~/translation-repair-runs-20260817` holds 6 two-lane artifacts and `~/translation-repair-runs-flagged-20260818`
holds 5,
none with a recorded pairing half,
and `~/translation-repair-v2-archive` holds 4 nested ones.
Point an instrument at a population with `TRANSLATION_REPAIR_RUNS_DIR=<dir> mise run ...`;
the 2026-08-17 directory
carved 6 of 6 (all halves defaulted),
the census reported 6 settled-partial rows and 86 baseline,
and one capped
fidelity trial ran over it with all ten seats usable (logs under `~/temp/agent/vub-*-current.txt`).
Every artifact a future pass writes records `sectionPairing`,
and `blockPairing` whenever the roster shell ran,
so
the complete-recipe path is exercised by the next settled pass;
until then every rebuild names its defaulted halves.
Whole-suite `buildAndTest` after `459482191`:
769 PASS,
0 FAIL,
exit 0 (`~/temp/agent/buildAndTest-current.txt`).
`#250` LANDED 2026-08-26 in `2c9886a55`:
the recall scorecard is kept under `recall-scorecard/<stamp>-<tip>.json`,
written atomically;
GFP fails 4 of 4 cases under either mutation;
plan mode verified at the boundary.
Whole-suite `buildAndTest` after `2c9886a55`:
770 PASS,
0 FAIL,
exit 0.
`#251` LANDED 2026-08-26 in `a8bc69508`:
no slate is bought for a slice with no standing text;
GFP fails 2 cases
with the refusing client's throw when the purchase is restored.
Whole-suite `buildAndTest` after `a8bc69508`:
770 PASS,
0 FAIL,
exit 0.
`#252` LANDED 2026-08-26 in `8af2b2bde`:
`CorpusReadError.kind` (`missing-object` or `other`,
measured off git
2.55's stderr),
`isMissingCorpusObject` as the one steppable failure in all five catchers,
and the pass's walk in
`pass-eligibility.ts` printing `INCOMPLETE <id>: <side> page absent at the pin (...)`;
GFP fails 2 and 3 cases
under the collapsed classifier and 2 under the re-widened catch.
Whole-suite `buildAndTest` after `8af2b2bde`,
first run:
VOID,
and the line committed in `4e23e3d5c` claiming
"730 PASS,
0 FAIL,
exit 0" was WRONG (the exit was 1;
the "exit 0" was typed,
not read).
The run reported
`test files failed:` for 29 files,
each with `ERR_MODULE_NOT_FOUND` on a hashed dist chunk,
because the plan-mode
boundary run (`mise run ...:corpus-pass -- --plan`,
whose task depends on `build`) was launched concurrently and
rewrote `dist/` at 06:31:44Z,
inside the suite's window.
Solo rerun of `buildAndTest` after `8af2b2bde`:
771 PASS,
0 FAIL,
exit 0 (read from the runner's exit,
then counted).
LESSON:
never launch a mise task that depends on `build` while `buildAndTest` is running;
the suite imports hashed
chunks from `dist/` and a concurrent build deletes the ones it holds.
Count verdicts by the runner's exit code first
(TLY);
a PASS count with a non-zero exit is the count's bug.
Plan mode over the real corpus (that concurrent run) printed exactly one `INCOMPLETE` line (the one one-sided
entry,
its source page absent) beside `pending=92` and `PLAN ok`;
that measurement stands.
`#253` LANDED 2026-08-26 in `17bcaf46c`:
a pair verdict resting on a transient reader failure is marked
`transient`,
is not persisted,
and is refused on resume together with every record from before the field;
GFP fails
3 and 2,
2,
and 3 cases under the three mutations.
Whole-suite `buildAndTest` after `17bcaf46c`:
772 PASS,
0 FAIL,
exit 0 (solo run,
nothing else building).
`#254` LANDED 2026-08-26 in `7a2a21ed8`:
CRLF is folded to LF at `readCorpusFile` (141 endings on the one such page,
1 block before and 45 after),
the block splitter folds again,
and the invisible-line mask judges around the return;
GFP fails 2 cases under each of the three mutations.
Whole-suite `buildAndTest` after `7a2a21ed8`:
773 PASS,
0 FAIL,
exit 0 (solo run,
nothing else building).
`#256` LANDED 2026-08-26 in `68062ccf7`:
the report reads `--run` and `--against` through the args module's refusing
reader;
GFP fails 3 cases under a lenient reader;
the CLI refuses `--run` written last with exit 6 and no frames.
Whole-suite `buildAndTest` after `68062ccf7`:
774 PASS,
0 FAIL,
exit 0 (solo run,
nothing else building).
EVERY MAJOR FROM THE AUDIT IS LANDED (`#237` to `#257`).
