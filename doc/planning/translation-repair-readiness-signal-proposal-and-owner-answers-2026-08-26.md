# Readiness signal: the proposal and the owner's answers of 2026-08-26

What the readiness signal first rested on,
what was known and not done,
and the sanitization inventory;
the four decisions the question kept apart and the owner's answers,
including the rejection that sent the work to the actual output;
what changed after the answers the same day,
and what had to come before any re-signal.

Part of [Translation repair: the production readiness signal, and what it rests on](translation-repair-readiness-signal.md),
which lists every file of this record.

## What "ready" rests on

-   The whole-package audit is closed on a measured tally:
    every register entry carries a FIXED,
    CLOSED,
    folded or tracked marker,
    3 BLOCKER,
    28 MAJOR,
    35 MINOR (`doc/audit/translation-repair-package-audit.md`,
    "Closing tally").
-   Every MAJOR and every MINOR group landed with a guard shown to fail when its fix is removed.
-   Whole-suite `buildAndTest` after `4c070f729`:
    819 PASS,
    0 FAIL,
    exit 0.
-   All 38 built commands a `mise.toml` task invokes carry their `import.meta.main` guard (measured,
    not assumed).
-   The provider resilience the owner asked for is verified live:
    both keys are required,
    a half-dark roster is loud (`SEAT` lines end every command),
    and the two calibration arms of 2026-08-26 ran the full ten-model roster with 304 and 302 of 312 voices heard.
-   The three measurements the queue held open are paid:
    overlap,
    the recovery rate
    (3 of 4 re-asked answers came back) and the straggler window's lever 1 (arm C,
    under "The decisions the question keeps apart").

## What is known and not done

-   Guards recorded as owed inside FIXED blocks are proven by the type check or by measurement rather than by
    a mutation:
    repair-8's typography half,
    consolidate-5,
    probes-10 and probes-11,
    calibrate-5 and calibrate-8,
    repair-2,
    document-11,
    slices-5 and slices-8.
    They are coverage debts,
    not defects,
    and each entry says so.
-   The straggler window decision (`doc/decision/translation-repair-straggler-grace.md`) stands at 180000 ms;
    arm C is read and the move is the owner's call (question 4).

## Sanitization inventory

-   Tracked tree,
    measured 2026-08-26 with the standalone scanner against the built deny-list of 10206 corpus
    sentences (`~/temp/agent/deny-rules.txt`,
    from `doc/decision/corpus-deny-list-for-forbidden-strings.md`):
    zero findings over the whole worktree walk.
    The probe was proven able to fire first:
    one rule written into a throwaway file fired as `rule=0` both when
    named explicitly and through the same `--all` walk from inside the worktree.
    The deny-list subtracted the 8 corpus sentences the tree already held on 2026-08-25,
    so the zero means no further sentence of 24 or more characters has entered the tracked tree since.
-   History:
    the branch carries 2342 commits beyond `main` (merge base `88ba0ae2e`,
    first commit 2026-07-16),
    and `main` carries 657 the branch lacks.
    Corpus text quoted in earlier commits and later removed lives only in that history.
-   Outside the repository,
    corpus-bearing and never committed:
    run directories under `~/temp/agent/` and `~/translation-repair-runs-*`,
    including `<runs dir>/rendering-audit-settled/*.json` (document spans and model prose in every row's `report`),
    calibration logs,
    and the deny-list itself.

## The decisions the question keeps apart

Each is its own question,
because answering "yes" to all of them in order must be reachable.

1.  Readiness:
disable branch protection now and sanitize,
or name what is still missing.
2.  Calibration overlap default:
`TRANSLATION_REPAIR_SLICE_OVERLAP` stays opt-in at 1,
or the editor calibration defaults to overlapping slices (only `editor-calibrate` carries the dial today;
    `producer-calibrate` gets the same dial as follow-up work if the default moves).
    Measured on matched arms:
    overlap 4 ran the same 1.74 h of calls in 24.19 min instead of 43.19,
    voices 302 against 304 of 312 (one run per arm;
    the run-to-run band is unmeasured).
3.  Corpus pass slice overlap:
the pass's drivers run slices sequentially by a recorded rationale (`lane-contest-driver.ts`:
    "the client's limiter grants one stream per model,
    so contesting two slices at once queues behind the same slot rather than doubling throughput";
    `translate-document.ts`:
    "aggregate concurrency beyond one stream per model collapses throughput on this plan").
    The calibration arms contradict the first premise under the same limiter,
    and the second predates the multi-provider routing.
    Building the dial into the pass is a design change;
    the proposal is to measure it on the pass the same way before any default moves.
4.  Straggler window:
keep 180000 ms or move it.
Measured on arm C (overlap 1,
300000 ms,
the same four slices) against arm A (overlap 1,
    180000 ms):
    53.87 min against 43.19 (+24.7%),
    voices never heard 5 against 8,
    grace cuts 4 against 6,
    all `qwen3.8-max`.
    Two rounds spent 214 s and 263 s in grace,
    the two voices the longer window bought back;
    four rounds burned the full 300 s and were cut anyway.
    About 5.3 min of wall clock per voice bought while slices run one at a time;
    under overlap the wait is what overlap fills,
    so a longer window under overlap 4 is the unrun arm that would reprice it.

## The owner's answers (2026-08-26)

1.  Readiness:
"Not yet.
You didn't even look at its actual output."
The signal rested on process gates;
    the output itself,
    the published pages and the shipped text,
    had not been read.
    That reading comes first,
    and readiness is not claimed again until it is recorded.
2.  Overlap default:
measure the run-to-run band first,
by repeating arm A.
DECIDED later the same day,
on arms A2 and D:
    `editor-calibrate` defaults to overlap 4.
3.  Pass overlap:
build the dial into the pass and measure it there;
the default stays 1 until measured.
4.  Window:
run arm D (overlap 4 at 300000 ms) before moving it.
DECIDED later the same day,
on arm D:
the calibration runs under 300000 ms together with overlap 4;
    the pass keeps 180000 ms until the overlap dial is measured on the pass.
    Record:
    `doc/decision/translation-repair-calibration-overlap.md`.

## Since the answers (2026-08-26, updated the same day)

WHAT THE REJECTION CHANGED.
The signal had rested on process gates;
the deliverable is the published page,
and none had been read by the session that put the signal.
The actual-output reading opened as the gate that now precedes any re-signal:
read pages against source and archive,
trace every defect into the artifact (slice,
lane,
ballots) before filing it,
and have a second reader (sol,
whole files attached) read each page too.
The advisor checkpoint on that plan (16:00Z) added the hard entries this project's own history names (`Toka_ls`,
`XIEPT2`,
`XingZ60`) to the sample,
since eight short entries answer only whether the output is publishable on easy input.

WHAT THE FIRST READING FOUND,
on four older-build pages (`doc/audit/translation-repair-output-reading-20260826.md`):
six defect classes no gate measured.
Three became code that is landed in source and awaits the build the running arms forbid:
a refiner that answered every ask reported as silent
(recurred on arm A2 on two seats),
invisible-variant punctuation
(U+2011 for the hyphen,
folded at every lane's intake;
U+2019 measured as the archive's own convention and kept),
a source hyperlink absent from the page
(now a `DESTINATIONS` line per entry with the addresses in the run log).
The other three (name rendering where the source uses an alias,
lexical ambiguity introduced by a change,
coverage misdiagnosis) are recorded in the reading and re-read on the fresh pass before anything is built for them.

WHAT THE MEASUREMENTS SAID.
Arm A2 put the run-to-run band at 37% of wall clock on an unchanged build,
driven by provider speed (stream sum 9294 s against 6312 s).
Normalized as wall clock over stream sum:
A 0.41,
A2 0.38,
C 0.43,
B 0.23.
So question 2's overlap effect stands (six bands wide) and question 4's
window cost is unmeasured at this scale rather than the +24.7% one pair of runs suggested.
Arm D is running as this is written.
Question 3's dial is designed and waits for the fresh pass to launch,
because the pass task builds what it runs and a driver edit present at launch would ship into the reading.

WHAT COMES BEFORE ANY RE-SIGNAL,
in order:
arm D read;
whole suite and the guard-failure rounds on the three landings;
the fresh pass at production defaults over ten entries (`XingZ60` second);
each page read twice and traced;
a spot re-read on the fixed build if the reading finds publishability blockers;
then this document gains a section that says what the pages showed,
and the readiness question is put again.
