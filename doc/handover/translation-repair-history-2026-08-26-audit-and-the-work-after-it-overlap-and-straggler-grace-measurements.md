# Translation repair history 2026-08-26 after the audit: slice overlap and straggler grace dials and what they measured

The slice overlap dial for the editor calibration and its guard proofs,
the serial and overlapped arms measured over the same four slices,
the recovery round paid and the straggler grace dial,
and the longer grace window measured against the serial arm,
with the caveat that each arm ran once.

Part of [Translation repair history: 2026-08-26, the package audit and the work after it](translation-repair-history-2026-08-26-audit-and-the-work-after-it.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The overlap dial lands for the editor calibration

`#213` LANDING (2026-08-26).
The overlap dial landed in `ce5ca2368`:
`TRANSLATION_REPAIR_SLICE_OVERLAP` admits that
many slices at a time through `p-limit` in `editor-calibrate.ts`,
unset reproduces the sequential driver exactly,
and
an unreadable value or a value under one is a stated refusal read before the sample is drawn,
so two sequential runs
can never pass as an overlap comparison.
The run client stays shared across slices in flight,
on purpose:
the saved
prototype rebuilt it per slice,
which predates `acfc7ad22`'s once-per-run client,
and the per-model limiter is what
production routes through.
The progress line and the `SliceRounds` type moved to `editor-calibrate-slice.ts`,
numbered
by sample position rather than arrival,
which keeps the entry module at 298 code lines and exports nothing from it.
Guards:
`slice-overlap.unit.test.ts` (four cases,
one describe) and `editor-calibrate-slice.unit.test.ts` (three
cases);
four GFP rounds,
one mutation each:
an unreadable dial falling back to one fails only `REFUSES a value that is
not a number`,
zero admitted fails only `REFUSES zero`,
numbering from zero fails `NUMBERS` and `RENDERS`,
the reach
note dropped fails only `SAYS`;
restored,
both suites pass.
Boundary,
keys stripped:
a dial of `four` exits 6 with the
refusal and no other output;
a dial of `2` prints `1 slices, 10 models editing and judging each, 2 slices in flight`
and then the key refusal;
the built command keeps its `import.meta.main` guard.
Next:
the whole suite (in flight),
then arm A (`TRANSLATION_REPAIR_SLICE_OVERLAP=1`) and arm B (`4`) back to back over four slices under `mise run` from
this worktree,
which carries `.env.local.json`;
`mise env` here lists both key names.
Whole-suite `buildAndTest` after `ce5ca2368`:
816 PASS,
0 FAIL,
exit 0 (solo run).
Both arms were launched back
to back at 2026-08-26T11:08Z into `~/temp/agent/overlap-arm-serial-20260826` and
`~/temp/agent/overlap-arm-four-20260826`,
logs beside them under the same names with `.log`.

### Overlap measured against the serial run

`#213` MEASURED (2026-08-26).
Arm A (`TRANSLATION_REPAIR_SLICE_OVERLAP=1`) and arm B (`4`) ran back to back over the
same four slices,
full roster,
both exiting 0,
33 rounds each.
`run-timing-report` on each log:
A,
43.19 min of round
time (sequential,
so also the wall clock),
37.08 min of it waiting after quorum (85.9%),
8 voices never heard,
calls
in flight mean 2.44 peak 10,
1.75 h of calls;
B,
24.19 min of run for 51.96 min of overlapped round time,
46.98 min
waiting (90.4%),
10 voices never heard,
mean 4.31 in flight,
peak 34,
1.74 h of calls.
So the same call time ran in
56% of the wall clock,
and voices heard were 304 against 302 of 312.
Losses by cause:
A,
6 grace cuts (all
`qwen3.8-max`) and 2 `schema-mismatch`;
B,
7 grace cuts (`qwen3.8-max` 4,
`hf:Qwen/Qwen3.8-27B` 2,
`hf:zai-org/GLM-5.2` 1) and 3 `schema-mismatch`,
one of them the same `deepseek-v4-flash-0731` panel answer failing
its re-ask.
QNB CAVEAT:
one run per arm,
and the run-to-run band on an unchanged build is unmeasured,
so the two-voice
difference is not a finding either way;
the wall-clock gain is the arithmetic of one call budget spread over four
lanes while 86% to 90% of every round is waiting,
which a repeat could shrink but not undo.
WHAT IS NOT DECIDED:
whether overlap above one becomes the calibrations' default,
and whether the corpus pass should overlap its slices the
same way;
both are design changes for the owner,
queued for the `#219` question.
Logs and run directories:
`~/temp/agent/overlap-arm-serial-20260826` and `~/temp/agent/overlap-arm-four-20260826`,
read only through their own
templates (`~/temp/agent/compare-arms.py`,
beside the logs it reads).

### The recovery round paid

`#230` PAID (2026-08-26).
Both arms carry `91f0c8ba5`,
and each logged two `recovery round for 1 unreadable answers`
lines:
A recovered both (`panel` and `critic`,
each `1/1 heard`),
B recovered one (`introduced-defect-probe` `1/1`;
`panel` `0/1`,
the same model answering badly twice),
so 3 of 4 re-asked answers came back readable.
The split comment
is CONFIRMED rather than corrected,
and `2829fd4da` records why in `stage-call.ts`:
the only non-`ok` kinds `chatJson`
returns are `refusal-shaped` and `schema-mismatch`,
a stream the idle,
runaway or degeneration guards cut throws into
the not-answered catch,
and a grace-abandoned straggler is classified in `stage-round.ts`;
live,
all four re-asked
voices were `schema-mismatch` and none of the thirteen grace cuts was re-asked.

### The straggler grace dial lands

`#229` LEVER 1,
DIAL LANDED (2026-08-26).
`4c070f729`:
`src/grace-override.ts` reads
`TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` (unset or empty means the built-in `STRAGGLER_GRACE_MS`;
unreadable or not
positive is a `StatedRefusalError`),
`runGatherRound`'s `graceMs` default resolves through it per call,
and
`editor-calibrate` and `corpus-pass` resolve it first thing and print `STRAGGLER GRACE OVERRIDDEN by ...` when it
differs.
`shippedAuthors` moved to `editor-calibrate-slice.ts` (driver at 297 code lines).
Guards:
nine cases in
`grace-override.unit.test.ts`,
two `shippedAuthors` cases;
four GFP rounds each bit only their own cases (NaN returned
fails the prose and `300s` cases,
zero admitted fails the zero case,
the note inverted fails both note cases,
per-issue authors dropped fails `CREDITS both halves`);
restored,
both pass.
Boundary,
keys stripped:
`five` exits 6
in one line;
`300000` prints the note after the header and then the key refusal;
`corpus-pass --plan` on a throwaway
runs dir prints the note after START and exits 0,
and refuses `soon` with exit 6.
Whole-suite `buildAndTest` after
`4c070f729`:
819 PASS,
0 FAIL,
exit 0 (solo run).
Arm C launched at 2026-08-26T12:23:14Z:
overlap 1,
`TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=300000`,
the same four slices,
into `~/temp/agent/overlap-arm-grace-20260826`
with the log beside it;
it is matched against arm A.
The proof that the window reached the rounds is a round with more
than 180000 `ms in grace` or an `abandoned 300000ms` line.
Nothing may rebuild `dist` until it ends;
the whole suite
after `2829fd4da` (a comment-only commit that passed oxlint and `lint:types`) runs then.

## The longer straggler grace measured against the serial run

`#229` LEVER 1 MEASURED (2026-08-26).
Arm C (overlap 1,
`TRANSLATION_REPAIR_STRAGGLER_GRACE_MS=300000`,
the same four
slices,
launched 12:23:14Z,
exit 0 at 13:17:15Z) against arm A (overlap 1,
180000):
round time 53.87 min against 43.19
(+24.7%),
waiting after quorum 48.33 min (89.7%) against 37.08 (85.9%),
voices never heard 5 against 8 (306 of 311
heard against 304 of 312;
C ran 32 rounds to A's 33 because it needed one recovery round to A's two),
grace cuts 4
against 6,
all `qwen3.8-max` in both,
one `schema-mismatch` (`critic`,
recovered `1/1`) against two.
The window's
effect is legible in the rounds themselves:
four rounds burned the full 300000 ms (the four cuts),
and two rounds
spent 214010 ms and 263265 ms in grace,
which are voices that arrived after 180000 ms and would have been cut under
the built-in window.
So on this sample the longer window bought back 2 voices of the 6 the built-in one loses,
at
10.68 min of wall clock,
about 5.3 min per voice,
and the other four stragglers reason past five minutes and are cut
anyway.
Under overlap that price changes,
since waiting after quorum is what overlap fills:
a longer window under
overlap 4 was not run and would be the arm to run if overlap becomes the default.
QNB:
one run per arm again;
the two
180000 arms cut 6 and 7,
so C's 4 sits below both and the two in-band rounds are direct evidence rather than a
difference of totals.
The dial stays opt-in and `STRAGGLER_GRACE_MS` stays 180000 until the owner decides (question 4
of the readiness signal).
Log and run directory:
`~/temp/agent/overlap-arm-grace-20260826`.
