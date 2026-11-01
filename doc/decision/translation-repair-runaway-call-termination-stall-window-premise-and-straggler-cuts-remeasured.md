# Runaway call termination: the stall windows and the straggler grace re-measured on production traffic

The idle windows' premise re-derived from first production traffic,
with the tail evidence that keeps them at 600000,
and the straggler grace's cut population counted to the end of the pass.

Part of [Ending runaway calls on our side of the wire](translation-repair-runaway-call-termination.md),
which lists every file of this record.

Superseded on 2026-10-21 (UTC):
the constants this record keeps at 600000 are now `STREAM_FIRST_CONTENT_MS` (180000 ms) and `STREAM_IDLE_MS` (120000 ms),
and they count generated text only.
See the section "The windows armed on 2026-10-21" at the end of this file.
The rest of this record is kept as it was written.

## The idle windows' premise, re-derived from first production traffic

STATE THE SPREAD BEFORE THE NUMBER,
per this repository's own rule.
Read separately from each of
the three logs `doc/audit/stream-guards-first-production-traffic.md` names,
`flagged-pass-2.log`
(2025 progress lines after partial-text retention),
`corpus-pass-20260817.log` (2166),
and `resume-run-output.log`
(2888),
the first-byte median runs 1032,
1207,
and 1237 ms,
five days and more than one roster
composition apart.
That band,
not any single figure inside it,
is what "the median" means here.
`grace-remeasure-snapshot.log` stays excluded:
independently re-run,
`cmp` confirms it is byte for
byte the first 811723 bytes of `corpus-pass-20260817.log`,
so counting it would count part of one
run twice.

THE 95.6 S MEDIAN `STREAM_FIRST_BYTE_MS` WAS SET FROM IS SUPERSEDED BY 77 TO 93 TIMES,
not a round
eighty read off one comparison:
95600 / 1032 = 92.6,
95600 / 1237 = 77.3,
95600 / 1207 = 79.2.
Working no longer looks anything like the old premise's stalled-and-silent shape at the median.

WHAT THIS CORRECTS IS THE STATED REASON,
NOT THE CONSTANT.
`STREAM_FIRST_BYTE_MS` and
`STREAM_IDLE_MS` stay at 600000,
because the tail refuses to agree with the median.
Pooling all
7079 rows across the three logs,
a fuller read than the audit's own mid-flight counts since two of
the three logs are static files and `flagged-pass-2.log` kept growing after the audit was written,
finds a completed,
not cut,
first byte at 183755 ms and a completed,
not cut,
mid-stream gap of
124992 ms on `hf:zai-org/GLM-5.2` (first byte 794 ms,
otherwise an ordinary stream).
A separate,
uncensored 2026-07-26 run,
PASS 7 RUN 014 in `doc/handover/translation-repair-history.md` (a different,
seven-model roster,
three weeks before this traffic),
recorded a completed call at 347099 ms,
WITHIN 3.6 PERCENT OF THE CURRENT 360000 MS DEADLINE,
with 9 of 764 sampled calls landing at or
past 300 s.
Any window tight enough to meaningfully beat the deadline would,
on the evidence in
hand,
have killed at least one of these healthy completions.
That is why the correction lands in
the constants' TSDoc rather than in their values:
commit `3893825b2`.

THE REGIME SHIFT ITSELF IS UNEXPLAINED AND STAYS THAT WAY.
Time to first byte read 45.8 s (RUN 013,
median,
censored at the 240 s deadline live then),
55.2 s (RUN 014,
median,
uncensored),
95.6 s
(the sentinel probe this file's constants were built from,
same day),
and roughly 1.1 s (this
traffic,
three weeks later).
Roster size moved from seven models to six somewhere in between,
and
provider-side conditions across those three weeks are entirely unrecorded.
Nothing here explains a
swing that size;
it is named so the next reader does not mistake today's fast median for a settled
baseline.

BOTH TESTS ARE SHOWN TO FAIL,
per `GFP`.
With `STREAM_FIRST_BYTE_MS` and `STREAM_IDLE_MS` each set
to 30000 in turn,
`stream-idle-guard.unit.test.ts` refuses both:
"keeps both windows above the
per-call deadline so the guard measures without killing" (`expected 30000 to be above 360000`) and
"keeps both windows above the highest first-byte wait and mid-stream gap observed in production so
far" (`expected 30000 to be above 347099` against the sabotaged first-byte constant,
`124992`
against the sabotaged idle constant).
Restored,
both pass.

## The straggler grace's cut population, counted to the end of the pass

THE AUDIT'S "NINE OF TEN" WAS A FLOOR,
taken mid-flight;
the pass log is no longer growing.
Read to
the end of `flagged-pass-2.log`,
the re-derivation counts 19 cuts,
not 10,
over 2025 streams:

```text
model                                               streams   cuts     rate
hf:zai-org/GLM-5.2                                      328     16   0.0488
hf:Qwen/Qwen3.6-27B                                      354      2   0.0056
hf:moonshotai/Kimi-K3                                    322      1   0.0031
hf:openai/gpt-oss-120b                                   351      0   0.0000
hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4        352      0   0.0000
hf:zai-org/GLM-4.7-Flash                                 318      0   0.0000
```

SIXTEEN OF NINETEEN CUTS,
84 PERCENT,
ARE STILL ONE MODEL,
and `hf:Qwen/Qwen3.6-27B` has now
entered the cut population with two,
both among the largest deliveries in the log (3258415 and
3209277 characters),
the same producing-not-stalled shape as `GLM-5.2`'s biggest cuts.
Three of
six models remain at zero,
not four;
the finding narrows rather than reverses.

WHETHER TO CHANGE `STRAGGLER_GRACE_MS` IN RESPONSE IS A POLICY QUESTION,
not a measurement one:
the
input the original 180000 ms derivation used,
per-model whole-call latency percentiles under a
bench roster,
has no fresh equivalent under the current six-model roster in anything the re-derivation read,
and estimating one would mean spending quota this task was not authorized to spend.
Options,
ranked,
are in `doc/planning/translation-repair-open-decisions.md`,
Question 9.
The idle-window
re-arming posture "The idle windows' premise,
re-derived from first production traffic" leaves open,
whether 600000 should ever be lowered given that section's
tail evidence,
is the same document's Question 10.

## The windows armed on 2026-10-21

Set in `w263` on the owner's brief to fix the classes the read of `zheermao22` found and to choose the bounds by measurement,
and recorded as ledger B804 in `package/translation-repair/doc/audit-ledger-provider-streams-usage-and-retries.md`.

The reason the windows stayed at 600000 was that no number below it cleared every regime measured.
The windows were also longer than the per-call deadline of 360000 ms,
so keeping them there meant they could not fire at all,
and the artifact of `zheermao22` stamped the pair without anything reading it.
Eleven `inception/mercury-2.5` streams that sent only `: OPENROUTER PROCESSING` comment lines were ended only by the
120 s straggler grace.

What replaced it:

- The first-content window is 180000 ms and the idle window 120000 ms.
  The basis is the largest healthy observation in 19542 completed streams that delivered text:
  a first byte of 115725 ms and a silence of 93736 ms.
  The idle margin is 28 percent.
  The first-content margin is 56 percent over the first byte.
  The window began at 120000 ms,
  a margin of 3.7 percent,
  and was widened before landing because the logs record the time to first byte and not the time to first text.
  The first byte is a lower bound on the wait the window bounds,
  so the margin over it could have been negative,
  and a healthy slow stream cut.
  The stream progress line now writes `firstContent <N>ms` beside `firstByte <N>ms`,
  which is how the real figure is measured on the next read of the logs.
- Progress is a content or a reasoning delta.
  A comment line,
  a blank keep-alive line or an event with an empty delta does not reset a window.
- The configuration is refused when a bound is not a positive finite number shorter than the per-call deadline.
- The older tail evidence this record kept the windows for
  (a first byte of 183755 ms,
  a completion at 347099 ms and a gap of 124992 ms)
  would still be cut,
  each by the window it exceeds
  (the first byte of 183755 ms is 3755 ms over 180000 ms).
  It came from rosters that are gone.

Question 10 of `doc/planning/translation-repair-open-decisions.md` is answered by this arming,
on the evidence of the current roster,
and stays open for the owner to loosen the first-content window if the next runs show healthy streams cut,
or to tighten it once `firstContent` figures show how long healthy first text takes.
The first-content figure of 180000 ms was chosen by the parent session and is open to the owner's veto.

Status of 2026-10-22 (UTC):
the first-content window is now also a per-card setting,
and the `inception/mercury-2.5` card names 90000 ms
(ledger B809,
red `731c541fa2`,
fix `9614ff8336`);
every other card keeps the shared 180000 ms.
The evidence is that,
in the 24 pass logs on disk (`CuspariaKLSY16` to `XingZ6017`),
4636 Mercury streams delivered text and the longest of them ended at 56772 ms,
no Mercury stream that ran 60 s or more without text ever delivered any (130 such streams),
and 26 rounds of `XingZ6017` waited about 301 s each
on one stream that sent only `: OPENROUTER PROCESSING` comment lines.
The 90000 ms is bounded from above by the elapsed time of streams that delivered text,
because `XingZ6017` predates the `firstContent` field this decision names,
so it is not the measurement of first text that this decision waits for.
The window is 59 percent over that longest elapsed time.
It is open to the owner's veto
(`firstContentMs` in `package/translation-repair/shared-contract/src/model-cards.ts`),
and the owner may ask for it to wait until `firstContent` figures from the next pass log are read.
