# Translation repair handover close: 2026-08-26

Part of the [current translation repair handover](translation-repair.md).

## Current stop condition

Historical evidence only.
Candidate M failed on 2026-09-01,
and no corpus run,
model call,
spent-prompt retry,
or successor implementation is authorized by this snapshot.

## Work while the arms ran (2026-08-26)

Arms A2 and D (`bw9dhhs6c`) hold `dist` from 15:54Z;
nothing below is built or tested yet.
Source edits,
`lint:oxlint` and `lint:types` ran (the type-check emits nothing under `dist/final/node`;
it writes only a
`.tsbuildinfo`,
the tsconfig has `noEmit`).
Every test file that imports the bundle type-errors against the stale
bundle until the next build;
those are the only lint findings left and they are not defects.

THE ADVISOR CHECKPOINT ON THE READING PLAN (16:00Z,
answered 16:02Z) changed the plan in four places,
all done
or scheduled:
the fresh pass adds the hard cases (`Toka_ls`,
`XIEPT2`,
and `XingZ60` in a second launch after
the ten,
since `XingZ60` alone projects at 385 minutes against the 420 minute cap and the pass orders `--only`
by the corpus listing);
sol reads each fresh page as a second reader (`scratchpad/sol-read.py <entry> <runs-dir>`
attaches the three whole files with `@file` arguments,
which `pi --help` documents,
and backgrounds the call);
every fresh-page defect is traced into the artifact (slice,
lane,
ballots) before it is filed;
and the reading
document's preamble was made true (it quotes one rendering;
it said it quoted none).

`#264` WAS MEASURED INSTEAD OF ASKED.
Over all 92 archive `page.en.md` files at the pin,
85 carry typographic
quotes and the corpus holds 1173 U+2019,
so a U+2019 the pipeline writes is house style;
the defect narrows to
invisible variants (U+2011,
U+00A0,
U+00AD) written where neither side has them.
The decision,
recorded in the
reading document with veto invited:
normalize that class at candidate intake,
before the deciders judge,
so both
still judge the bytes that ship (`#162`).
Not built yet.

`#263` LANDED IN SOURCE (`d8a040edf`,
`04f1ebafd`).
The refine stage now returns `heard` (the refiners with a
usable answer,
proposal or not),
`settleRefinedSlice` threads it out as `refinersHeard`,
the calibration's slice
record carries `refinerHeard`,
and `readStandingCoverage` takes a fourth input,
`answered: SeatAnswers`,
which is
`recorded` with ids or `unrecorded`.
The coverage has a fourth state,
`answeredUnslated`,
with its own line
(`ANSWERED AND WAS NEVER SLATED`);
the silent line reads `ANSWERED NOTHING USABLE` where answers are recorded and
`NO CANDIDATE OF THEIRS REACHED ANY SLATE ... this seat does not record who answered` where they are not.
The
editor and translate seats are unrecorded (`#266`):
their stages carry only a heard count out,
and the chunk
outcome is persisted,
so carrying ids needs the in-memory path `settleRefinedSlice` uses.
The seat report moved
to `editor-calibrate-standing.ts` as `standingReportLines`,
which returns lines so a test reads it without the
console;
the first name chosen collided with `seat-tally.ts`'s `seatReportLines`,
which the barrel union refused.
Owed after the build:
the suites,
GFP on the producer-silence guard (remove the `answered` input from the
refiner's `produced` wiring and see the `#263` cases fail),
the whole suite solo,
and a live `editor-calibrate`
reading with the REFINER coverage line checked against its SEAT lines.

`#265` LANDED IN SOURCE (`fd4f7546f`).
`publishFixedPage` now takes `sourceText`,
reads both sides' destinations
off the pipeline's own parse (`parseBodyTolerant` is exported `@internal` from `parse-document.ts`;
front matter
split,
invisible lines and HTML comments masked,
strict MDX with the plain-markdown downgrade) unioned with a
linear scan for `http://` and `https://` runs,
and returns `{ path, destinations }`.
The pass prints
`DESTINATIONS <id> source=N page=M dropped=K` beside the tally (counts only;
the addresses go to the run log at
info,
a warn line carries the count).
A dropped destination is a finding,
never a refusal:
the page is what both
deciders approved.
`pass-entry.ts` reached its line budget and the capped failure text moved to
`tally-error-text.ts`;
the destination exports live in `publish-barrel.ts` because `corpus-barrel.ts` reached
its budget too.
Owed after the build:
the suites,
GFP (drop the union of the scan and see the bare-run cases
fail),
and a live reading of the `DESTINATIONS` lines on the fresh pass.

THE SITE'S OWN PARSER,
on the owner's pointer.
The corpus repo (`~/one-among-us/data`,
`package.json` at the
pin) builds each `page.md` itself:
`scripts/build.ts` rewrites `<!--` to `{/* ` and `-->` to ` */}`,
splits
front matter with `markdown-yaml-metadata-parser`,
and `scripts/mdx.ts` compiles with `@mdx-js/mdx` `compileSync`
under `remarkMath` and `rehypeKatex`,
no GFM.
`one-among-us/web` (cloned shallow to `~/temp/agent/oau-web-20260826`)
renders the compiled page and uses `marked` only for metadata.
The pipeline parses with `remark-mdx` plus
`remark-gfm` after masking comments to whitespace.
Measured at the pin:
two or more `$` on 34 source pages,
`[^` on 23,
`<!--` on 17,
a JSX component on 53.
`#267` holds the reconciliation question;
the destination
check is unaffected because it scans bare runs as well as the tree.
VERIFIED WITH THE SITE'S OWN RENDERER
(shallow clone at `~/temp/agent/oau-data-20260826`,
deps from its lockfile via npm,
`renderMdx` driven by
`scratchpad/render-probe.ts`):
`[^1]` compiles to literal text,
`$x^2$` to KaTeX markup,
a JSX comment
vanishes,
a GFM table stays literal.
Classified at the pin:
6 source pages carry a math-like `$...$` pair
(14 occurrences);
the other 171 `$` are lone.
So math is real on the site for six pages and the pipeline
reads it as prose;
footnotes are structure to the pipeline and literal text on the site,
which loses
nothing since the text is preserved either way.

ARM A2 IS IN (`#260`),
and the band is wider than one of the two effects it was meant to size.
Wall clock from
the log's first to last timestamp (the `run-timing-report` task depends on `build`,
which arm D forbids,
so the
span was read with a timestamp scan):
A 43.18 min,
A2 58.95 min,
on an unchanged build over the same four
slices.
That is a run-to-run band of at least 15.8 min,
37% of A,
and the calls themselves were slower,
not the
pipeline:
stream sum 9294 s against 6312 s,
p50 7017 ms against 5260 ms,
p95 136 s against 78 s,
8 cut voices
against 6 (all `qwen3.8-max` and `hf:Qwen/Qwen3.8-27B`),
312 of 320 voices heard against 304 of 312,
no
recovery round against two.
Provider speed between 11:08Z and 15:54Z moved more than any dial did.

NORMALIZED AGAINST CALL TIME the picture is clean.
Wall clock over stream sum:
A 0.41,
A2 0.38 (band about
0.03),
C 0.43,
B 0.23.
Overlap 4 (B) sits 0.18 below A,
six times the band,
so the overlap effect stands.
The 300000 ms window (C) sits 0.02 above A,
inside the band,
so its cost is UNMEASURED at this scale rather
than the +24.7% a single pair of runs suggested;
the 2-of-6 voices it bought back is also inside what
provider speed alone moved (A2 lost 8 with no window change).
Arm D (overlap 4 at 300000 ms) is running;
read
it the same two ways.
What goes back to the owner with D:
the overlap default can be decided on this evidence;
the window cannot be decided on single runs,
and the honest options are interleaved repeats (A,
C,
A,
C on one
afternoon) or leaving it at 180000 ms with the dial available.

DOCS UPDATED 2026-08-26 on the owner's instruction "Please update all docs now":
this file;
the package README
(destinations line,
the fold at intake,
the site's grammar,
the four coverage states,
status);
the corpus pass
runbook (destinations check);
`doc/planning/translation-repair-readiness-signal.md` (since the answers);
`doc/planning/translation-repair-open-decisions.md` (questions 11 and 12,
the two dials,
with rankings);
`doc/decision/translation-repair-straggler-grace.md` (addendum:
the dial and the band);
`doc/audit/translation-repair-package-audit.md` (calibrate-1 superseded in part by `#263`);
`doc/troubleshooting/translation-repair-invisible-characters.md` (the fold);
`doc/troubleshooting/translation-repair-unread-signals.md` (the misread coverage line,
the destinations line);
`doc/audit/translation-repair-output-reading-20260826.md` (tooling,
the second reader's dry run).
`#268` holds
the reading tooling's move into the package.

ARM D IS IN (`#262`):
overlap 4 at a 300000 ms window,
same four slices,
launched 16:53Z after A2,
solo.
Wall clock 29.31 min (log first to last timestamp) against B's 24.18 at the built-in window,
with calls that
were slower than B's (stream sum 7591 s against 6249 s,
p50 5839 ms against 5318,
p95 101 s against 89 s),
so
normalized as wall clock over stream sum D is 0.23,
the same as B.
Voices heard 318 of 320 against B's 302
of 312;
cut 2 (one `qwen3.8-max`,
one `hf:Qwen/Qwen3.8-27B`) against B's 7,
A's 6,
A2's 8 and C's 4;
no
recovery round.
Time in grace was 4591 s against B's 2819 s,
and overlap filled it:
rounds waited up to 300 s
and the other three slices used the wait.
So under overlap the longer window costs nothing measurable and
buys the fewest cut voices of any arm,
which is the pairing the question 12 ranking in
`doc/planning/translation-repair-open-decisions.md` did not have when it was written;
it now does.
The
`#263` coverage line did not fire in D:
every seated model drew a row in both tables.

THE BUILD AFTER THE ARMS FOUND FOUR CASES WRONG (whole suite 17:23Z:
828 PASS,
8 FAIL counting each case
twice).
Two were the fixes' own tests:
the fold test had lost its literal invisible characters to the tool that
wrote it (the module's own table had survived,
checked by code point),
and the coverage case's four-model
roster had a silent fourth model the expectation omitted.
Two were a real defect in `#265`:
a GFM autolink
literal in Chinese prose runs into the full-width punctuation after the address,
so the tree reader and the
scanner disagreed on one link and the union counted it twice;
every destination is now cut at the first
stopper and shed of trailing punctuation before comparison (`trimDestination`).
Fixed in `5edd6a3b8` and
`fbf23ceae`;
the seven touched suites pass and the package lints clean (0 warnings,
0 errors).
The
guard-removal rounds (`scratchpad/gfp-three-landings.py`,
eight rounds) and a second whole-suite run follow
serially,
then the fresh pass.
WHOLE SUITE AFTER `fbf23ceae`:
829 PASS,
0 FAIL,
exit 0 (solo run,
17:29Z).
The guard script's first launch crashed on a path it built wrong (`src/src/...`) before touching anything;
relaunched after the fix.
GUARD ROUNDS (17:30Z):
seven PROVEN,
one NO SUITE.
`#263`:
removing the
`answered` reading from the coverage split fails the two `#263` coverage cases and the standing-report case;
removing the refine stage's heard list fails both refine-stage cases;
the calibration's own wiring is an
entry module with no suite and gets the live check.
`#264`:
removing the fold at each of the three intakes
fails that intake's fold case.
`#265`:
dropping the bare-run scan from the union fails the destination case;
emptying the publisher's source text fails the publish case.
Tree restored and `dist` rebuilt after.
THE FRESH PASS LAUNCHED at 17:31Z from `907d14ea2` (the task builds what it runs,
so today's landings are in
it):
`--only` over the ten entries into `~/temp/agent/fresh-read-20260826`,
log beside it,
at production
defaults,
solo;
`XingZ60` follows in a second launch.
Every page is read twice as it lands.

THE TWO DIALS ARE DECIDED (owner,
17:35Z,
both the recommended option):
`editor-calibrate` defaults to
overlap 4 (`CALIBRATION_OVERLAP`,
`slice-overlap.ts`,
whose `readOverlap` now takes the caller's fallback) and
runs under 300000 ms (`CALIBRATION_STRAGGLER_GRACE_MS`,
adopted by `adoptCalibrationGrace` through the variable
a launch can set,
printed as `straggler window <ms>ms (calibration default|override)`);
the pass keeps 1 and
180000 until `#261`.
Landed in source after the fresh pass built,
so the pass does not carry it;
suites run
after the pass.
Record:
`doc/decision/translation-repair-calibration-overlap.md`.

THE OWNER ALSO REPORTED (17:40Z) that `qwen3.8-27b` is now available on Charm Hyper.
The Synthetic seat
`hf:Qwen/Qwen3.8-27B` is one of the two seats the window keeps cutting (1 to 2 per arm,
beside `qwen3.8-max`),
and Hyper has no slot limit,
so routing that seat through Hyper is the next roster change;
the catalog reading
and the id check come first.
DONE (`d40ac07d3`):
Hyper's public `/v1/models` lists `qwen3.8-27b` (1M context,
128000 max output,
vision,
$0.5 in,
$3 out,
$0.1 cache hit per million;
10,
60 and 2 Hypercredits),
so
`HYPER_MODELS` pairs it with `hf:Qwen/Qwen3.8-27B` as one panelist and the router's overflow can send that seat
to Hyper once its Synthetic slot saturates;
a scratch probe under `mise exec` answered 3 of 3 with the forced
tool shape in about 2 s each,
so `toolChoice: 'forced'`.
Regenerating the price table from the same endpoint
found six rows drifted since 2026-08-25 (`gemma-4-26b-a4b-it` up about 9%,
and five non-roster models),
so
every row is re-read and `HYPER_PRICE_READ_ON` is 2026-08-26.
`model-catalog` compares only Synthetic against
the compiled catalog;
the Hyper half is `#270`.
The route is not measured yet:
the next calibration's SEAT and
cut counts for that seat,
read per provider,
are the measurement.

THE SECOND READER'S DRY RUN CHANGED THE READING METHOD.
sol read the older `wangzihao980` page (2026-08-22
build) and returned 15 items where this session's own reading had two:
1 inherited blocker (the day of death,
"the next day" where the source says the early hours of that day),
5 inherited majors,
5 inherited minors,
and
introduced findings including a restored link shipped inside a malformed sentence beside a terminology split;
verdict,
not publishable.
The first reading had never looked at the front matter,
never checked dates as facts,
and graded faithfulness without naturalness.
All three are now in the method
(`doc/audit/translation-repair-output-reading-20260826.md`,
"The second reader on the older page").
One new
scope gap fell out:
the front matter's `desc` is translated prose and no stage ever reads or repairs it
(`#269`;
`document-preparation.ts` reads front matter as identity data only).

THE `#263` MISREPORT RECURRED IN A2,
now on two seats:
`WROTE NOTHING AT ALL: minimax-m3,
deepseek-v4-flash-0731` beside `SEAT minimax-m3 asked=32 usable=32` and `SEAT deepseek-v4-flash-0731 asked=32
usable=32`.
The landed fix reports them as answered-but-unslated;
the live check after the build reads this
same line on a fresh calibration.

## Sections aged out into the history (2026-08-26)

Moved verbatim into `doc/handover/translation-repair-history.md` when this file passed its cap by
more than 1500 lines (register item A-5),
because each is closed work whose conclusion is already
encoded in the code or in a decision record.
Their headings,
in the order they now sit in the history:

-   The settled artifacts already carry editor rounds,
    and they do not support the reseat
-   Zero editor rounds does not mean nothing was repaired (`#200`)
-   How many slices an editor calibration needs,
    measured from production
-   The 14-slice editor calibration finished,
    and it settles no seat (`#200`)
-   What the suite actually reaches,
    measured (2026-08-24)
-   `#217` is built,
    GFP-proven and parked (2026-08-25)
-   The parked work is now build-and-test verified together,
    not just apply-clean (2026-08-25)
-   `#216` was half wrong,
    and reading the source before building found it (2026-08-25)
-   `#215`:
    a run now says where its wall-clock went,
    and a CLI reads it back
-   `#205`:
    the two-lane artifact family is named for its shape,
    not a version
-   The landing was rehearsed on a throwaway,
    and it works
-   The three report CLIs are documented,
    and the landing was re-checked for collisions
-   The refiner column is thinner than the editor column,
    by about four times
-   The run's power inputs,
    measured at 38 of 40 slices
-   An unreadable run file printed itself,
    and the fix was a whole class rather than one CLI
-   A second calibration is in flight,
    to pay four measurements the landing left owed
-   The same defect had a second half,
    reaching a sink through a catch (`#224`,
    2026-08-25)
-   A third shape,
    found by asking which other parsers quote (`#225`,
    2026-08-25)
-   Does any error class quote what it was handed?
    Scanned,
    not sampled (2026-08-25)
-   Every entry point now reports its refusals,
    settled by measurement (`#226`,
    2026-08-25)
-   The cause sweep,
    which the message scan had missed (2026-08-25)
-   A stream the provider cut short was the one transport failure that never retried (`#228`,
    2026-08-25)
-   Which error messages may be repeated,
    decided by a rule rather than an audit (`#227`,
    2026-08-25)
-   The guard proof found the guard was somewhere else (`#224`,
    2026-08-25)
-   Four guard proofs,
    run once the bundle was free (`#225`,
    `#228`,
    2026-08-25)
-   The pipeline now keeps what each model wrote (`#212`,
    2026-08-25)
-   The ledger has a reader,
    and writing it found a real gap (`#212`,
    2026-08-25)
-   `#211` is proved at the wire,
    and the fix is in (2026-08-25)

Moved on the evening of 2026-08-26,
when this file stood at 1984 lines against the cap of 2000:

-   AUDIT IN PROGRESS:
    the whole package,
    before any new work (`#236`,
    2026-08-26).
    The audit is closed;
    its register with every marker is `doc/audit/translation-repair-package-audit.md`,
    and the closing
    verification (814 PASS,
    every entry marked) stands in its "State of the tree" subsection,
    now in the
    history,
    and in the register.

## Where this session stopped (2026-08-26 evening)

Written for a fresh agent with no transcript,
at the owner's request,
because the session's remaining budget ran short.
The in-flight code work has its own document:
`doc/handover/translation-repair-overlap-dial.md`.

### The state of the tree

Worktree `/var/home/user/worktrees/translation-repair`,
branch `translation-repair-rebased`,
auto-push on.
Tip is `748b54841`.
The only untracked path is `.idea/.name`,
which is not ours to touch.
Build,
`lint:oxlint` (0 warnings,
0 errors) and `lint:types` are all green at that commit,
and the two new suites pass.
The last whole-suite run,
before the two commits above,
was 829 PASS,
0 FAIL,
exit 0.

### A corpus pass ran and was stopped without settling anything

Launched 2026-08-26 at 17:31:54Z from tip `907d14ea2`,
stopped with `SIGINT` at 20:47:24Z when the session ended:

```sh
mise run //package/module/translation-repair:corpus-pass -- \
  --only ArtsEpiphany,gaoyanger,Zha_Ke,Weideriche_,keyword233,Acheron,wangzihao980,dogesir_,Toka_ls,XIEPT2
```

Moved on 2026-10-18 from `//package/module/translation-repair:corpus-pass`
to `//package/translation-repair/corpus-pass:corpus-pass`.

-   Log:
    `~/temp/agent/fresh-read-20260826.log`.
-   Runs directory:
    `~/temp/agent/fresh-read-20260826`,
    holding `artifacts`,
    `fixed`,
    `ledger`,
    `slice-cache`.
-   Progress is read from the pass's OWN template lines only,
    never by quoting the log:
    `round: N/M heard`,
    `abandoned <ms>ms after quorum`,
    `TALLY <id> status=`,
    `DESTINATIONS <id> source= page= dropped=`,
    `INCOMPLETE`,
    `CLEANUP`,
    `ONLY ...`,
    `START tip=`.
    At 20:43Z:
    145 rounds,
    25 abandoned,
    no entry settled yet.
-   Rebuilding `dist` did not disturb it.
    There are no dynamic imports,
    and this was verified rather than assumed.
    What a rebuild DOES change is what the next launch ships,
    since the pass task builds before it runs.

In 3 hours and 15 minutes it settled NOTHING.
The log carries no `TALLY` line,
`artifacts` is empty,
no `page.en.md` was published,
and the run was still inside its first entry:
147 stage rounds,
25 abandoned exchanges.
That is the plainest argument for `#261` anyone has produced so far.

The per-slice work it did buy is in `~/temp/agent/fresh-read-20260826/slice-cache`,
1.7 MiB of it,
and it will NOT resume under a current build.
The generation is a digest over the bytes in `dist/final/node`
(`src/corpus-run/pipeline-digest.ts` explains why it is the built directory rather than the git tip),
and `dist` has been rebuilt since,
with modules that did not exist when the pass started.
That cache resumes only for a tree checked out at `907d14ea2` and rebuilt.
Treat a relaunch as a cold run,
or accept the loss and read whatever a fresh pass settles.

`pass.lock` in the runs directory still names process 2166853,
which is gone.
Nothing needs to be done about it:
`src/corpus-run/runs-lock.ts` evicts a lock whose holder has exited,
by atomic rename,
and logs `taking over a stale lock in <dir> from gone process <pid>` when it does.

To relaunch,
follow `~/temp/agent/reading-instruments/launch-fresh-pass.txt`,
which carries the whole invocation including the environment and the redirection.
Launch it solo:
a pass and a calibration arm running at once make each other's timings unreadable.

### What to do as each entry settles

Nothing settled in the stopped run,
so this is the procedure for the NEXT one.
For every `TALLY <id> status=settled` line,
in this order:

1.  `python3 ~/temp/agent/reading-instruments/page-read.py <id> ~/temp/agent/fresh-read-20260826`
    writes the source,
    archive and published triplet,
    their diff,
    a code-point census
    and the address counts under `~/temp/agent/read-<id>/`.
2.  `python3 ~/temp/agent/reading-instruments/sol-read.py <id> ~/temp/agent/fresh-read-20260826`
    sends the same triplet to the second reader and leaves its answer at `~/temp/agent/sol-<id>/answer.log`.
    It is backgrounded and may take a long time.
    Never poll it and never kill it;
    read the answer later.
3.  Read the `DESTINATIONS` line beside the tally.
    This is the live check `#265` still owes.
4.  Run `verify-published`,
    the rendering audit and the damage probe over the settled artifact.
5.  Trace every defect the reading finds back into the artifact:
    which slice,
    which lane,
    which ballots.
    A defect nobody can trace to an artifact is not yet a finding.
6.  Extend `doc/audit/translation-repair-output-reading-20260826.md`,
    which holds the rules of the reading,
    the tooling,
    and the second reader's corrections.

When the ten entries finish,
launch the second pass over `XingZ60` into the SAME runs directory,
following `~/temp/agent/reading-instruments/launch-fresh-pass.txt` verbatim.
That entry is the one with the historically hardest section pairing,
so it is read separately and on purpose.

### What the reading method has to include

The second reader found 15 items on a page where the first reading had found 2,
which is why the method now says,
explicitly:

-   Read the front matter,
    not just the body.
    `#269` came out of this:
    the `desc` field is never translated or repaired and ships the archive's wording untouched.
-   Check dates,
    ages and stated relationships as FACTS against the source,
    not as prose.
-   Grade naturalness separately from accuracy.
-   Record defects the archive already had (inherited) apart from defects this pipeline introduced.
    Only the second kind is a regression;
    the first kind is what the pipeline exists to fix and is a miss,
    not a break.

### Two live checks are owed, and they are one run

Both are paid by a single fresh `editor-calibrate` after the pass ends:

-   `#263`:
    the REFINER seat's coverage line must be read against the SEAT lines in the same output.
    A refiner that answered and was never slated must read as
    "ANSWERED AND WAS NEVER SLATED",
    never as silent.
-   The Qwen seat on Charm Hyper (`qwen3.8-27b`,
    added 2026-08-26 after the owner reported it available)
    has been probed 3 of 3 under the forced tool shape but never measured in a calibration.

The calibration now defaults to four slices in flight under a 300000 ms straggler window,
so its header should read `4 slices in flight` and
`straggler window 300000ms (calibration default)`.

### The open register

These numbers are this session's task list,
NOT GitHub issues.
Anything durable about them lives in this document,
in the audit document,
or in `doc/decision/`.

-   `#219` readiness signal:
    REJECTED by the owner on 2026-08-26 with "Not yet.
    You didn't even look at its actual output."
    Re-signal only after the reading in `#259` is recorded.
    The signal itself is put through `AskUserQuestion`,
    and the owner disables branch protection at that point.
-   `#259` read the actual output:
    in progress,
    gated on the running pass.
    This is the task that gates `#219`.
-   `#261` the overlap dial:
    in progress,
    two commits landed.
    See `doc/handover/translation-repair-overlap-dial.md`.
-   `#263`,
    `#264`,
    `#265`:
    all three landed and GFP-proven on 2026-08-26.
    `#263` and `#265` still owe the live checks named above.
    `#264` folds invisible variants (U+2011,
    U+00A0,
    U+202F,
    U+00AD,
    U+200B,
    U+2060,
    U+FEFF)
    at all three model intakes and names each fold as a finding;
    U+2019 stays,
    because it is the archive's own convention.
-   `#266`:
    the editor seat cannot say who answered,
    because heard editor ids stop at the editor stage,
    so its coverage line can only hedge.
-   `#267`:
    the pipeline reads math as prose and footnotes as structure;
    the site,
    verified with its own renderer,
    does the reverse.
    Six source pages carry a `$...$` pair.
-   `#268`:
    make the page reading reproducible as a package CLI
    (triplet,
    diff,
    code-point census,
    address counts).
    The Python instruments are throwaway and live outside the repository.
-   `#269`:
    front matter is never translated or repaired.
-   `#270`:
    `model-catalog` compares only Synthetic against the compiled catalog;
    the Charm Hyper half has no drift check,
    and six price rows drifted in a single day.
-   `#271`:
    contest can decline archive and settle on neither lane,
    consolidation calls that `no-standing-text` and buys nothing,
    then final assembly revives archive.
    Fixed `Toka_ls` reproduced this with 9 of 10 voices calling archive flawed on one slice.

### Instruments, and where they live

The session scratchpad is session-local and will vanish.
Copies of everything worth keeping are at `~/temp/agent/reading-instruments/`:
`page-read.py`,
`sol-read.py`,
`launch-fresh-pass.txt`,
`gfp-three-landings.py`,
`compare-arms.py`,
`hyper-probe.ts`.
`~/temp/agent/gfp-overlapped-map.py` is the guard-failure proof for the newest helper.
`#268` exists because these should be a package command instead.

### Standing constraints that outlive this session

-   The corpus repository `one-among-us/data` is UNLICENSED.
    Read it through `git show <sha>:<path>` from `~/one-among-us/data`,
    pinned at
    `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.
    Entry ids may be named in commits and documents;
    passages must not be quoted.
    Never print raw run-log lines:
    extract only the pipeline's own template fields.
-   Never echo an API key value and never read `/proc/<pid>/environ`.
    `TRANSLATION_REPAIR_SYNTHETIC_API_KEY` and `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY`
    are injected by the root `mise.toml` from `.env.local.json`;
    launch through `mise run` or `mise exec` from the worktree.
-   Never set `thinking`,
    `budget_tokens` or `reasoning_effort` on any model call.
-   Never write `Closes #N` in a commit message:
    the numbers here are not GitHub issues.
-   The owner's standing instructions:
    make non-design decisions yourself and prioritise quality over cost;
    prototype and measure rather than argue;
    nothing is blocked on the owner;
    the pipeline must be resilient to provider trouble;
    a full 92-entry pass is never needed;
    the producers stay at three;
    corpus text in commits is tolerated,
    and sanitization happens at the end.
