# Translation repair history 2026-08-26 after the audit: the readiness signal, the owner's rejection and the output reading

The sanitization inventory of the tracked tree,
the final whole-suite gate before the readiness signal,
the owner's four answers quoted verbatim and what the rejection means,
and the reading of published pages that found defect classes no gate measures.

Part of [Translation repair history: 2026-08-26, the package audit and the work after it](translation-repair-history-2026-08-26-audit-and-the-work-after-it.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Sanitization inventory of the tracked tree

SANITIZATION INVENTORY (2026-08-26).
The tracked tree was scanned with the standalone `forbidden-strings` binary
against the built deny-list of 10206 corpus sentences (`~/temp/agent/deny-rules.txt`):
zero findings over the whole
`--all` walk,
validated by a positive control first (one rule written into a throwaway file fired as `rule=0` both
named explicitly and through the same walk from inside the worktree;
the file was deleted at once).
Findings print on
stderr as `PATH:LINE rule=<token>`,
never the text.
The branch carries 2342 commits beyond `main` (merge base
`88ba0ae2e`,
since 2026-07-16).
The proposal behind the `#219` question,
with the four decisions it keeps apart,
is
`doc/planning/translation-repair-readiness-signal.md`.

## The final gate before the readiness signal

Whole-suite `buildAndTest` after `2829fd4da` (the last code commit),
run after arm C as the final gate before the
readiness signal:
819 PASS,
0 FAIL,
exit 0 (solo run).
The advisor checkpoint before `#219` was made and asked
for this verdict to be read first,
the status head refreshed,
the arm reader kept beside its logs,
and question
2 scoped to `editor-calibrate`;
all four are in this commit.

### The owner's answers to the readiness signal, and what the rejection means

THE OWNER'S ANSWERS TO THE READINESS SIGNAL (2026-08-26,
the first genuine user input since the audit began;
quoted
verbatim).
Readiness:
"Not yet.
You didn't even look at its actual output."
Overlap default:
"Measure the run-to-run
band first (repeat arm A)".
Pass overlap:
"Build the dial into the pass and measure it there;
default stays 1 until
measured".
Window:
"Run arm D (overlap 4 at 300000 ms) before moving it".

WHAT THE REJECTION MEANS.
Every gate the signal rested on was a process gate:
guards,
suites,
wall clock,
voices
heard,
scan results.
Nobody in this session read what the pipeline PRODUCES,
the published `*.en.md` pages and the
shipped text inside settled artifacts,
against the Chinese source and the archive English.
That reading is now the
first item,
ahead of the three measurements the other answers authorize,
and readiness is not to be claimed again
until the output has been read and the reading recorded.
The three authorized measurements:
arm A repeated as arm A2
(overlap 1,
built-in window,
same four slices) to size the run-to-run band;
arm D (overlap 4,
300000 ms) to reprice
the window under overlap;
the overlap dial built into the corpus pass's drivers,
default 1,
then measured on the pass
with matched runs.
Both arms launch back to back and solo,
since a concurrent run would share provider slots and spoil
the band.

### The output reading of published pages

THE OUTPUT READING (`#259`,
2026-08-26) is `doc/audit/translation-repair-output-reading-20260826.md`:
four published
pages from the 2026-08-22 and 08-24 builds read sentence by sentence against source and archive,
six defect classes no
gate measures (comma splices and present tense for the dead,
in-page terminology drift,
the handle used as a name,
伙伴
rendered "partner",
U+2011 and U+2019 introduced by models,
a source hyperlink lost by inheriting an archive
sentence),
and tasks `#263` (the calibration coverage sentence misdiagnoses a seat that answered every ask as silent;
found by reading arm A's printed report,
whose SEAT line contradicts it),
`#264` (introduced punctuation),
`#265`
(source hyperlinks lost).
The four-slice EDITOR standing swings by twenty points between identical runs (GLM-5.2 52.2%
in A,
26.7% in B,
10.0% in C),
so a four-slice standing is noise;
that goes into the band answer.
Next:
after the
arms,
a `corpus-pass --only` over the six entries of the 08-20 reading plus `wangzihao980` and `dogesir_` into a
throwaway runs dir on the current build,
read the same way.
