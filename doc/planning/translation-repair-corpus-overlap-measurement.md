# Corpus-pass overlap measurement

## Question

Choose corpus-pass slice-overlap fallback from matched live runs,
without confusing provider speed,
cache resumes,
or entry mix for an overlap effect.
Fallback remains `1` until this measurement is read.

## Launch invariant

Every arm uses:

- same committed build and built-in `180000` ms straggler window;
- same entry in each pair;
- no other provider-consuming process;
- a fresh run root,
  including separate artifacts,
  published pages and slice cache;
- explicit `TRANSLATION_REPAIR_SLICE_OVERLAP=1` or `4`;
- launch through the package `corpus-pass` mise task so both provider credentials are injected;
- overlap log line as arm attribution:
  `OVERLAP <entry> value=<n> source=TRANSLATION_REPAIR_SLICE_OVERLAP`.

No source rebuild or code commit lands between paired arms.
Documentation may be updated because running bundles do not import it.

## Provider authority and live gate

The owner authorized unrestricted use of both providers on 2026-08-27.
Provider spend and allowance consumption do not limit evidence depth;
matched-arm isolation and live availability still protect measurement validity.

`budget-sample` immediately before launch must report both providers wet.
The 2026-08-27 preflight did:
`~/temp/agent/pre-overlap-pass-budget.log`.
A provider outage voids timing from the affected pair rather than becoming evidence against its arm.

## Staged entry set

### Smoke pair

Run `keyword233` first at overlap `1`,
then overlap `4`,
in disjoint roots.
It is a live positive control for environment attribution,
provider routing,
interleaved slice admission and artifact publication.
If preparation yields fewer than two eligible slices in a measured driver,
use `Weideriche_` as the smoke pair before starting the decision set.

### Decision pairs

Use these entries,
one pair at a time:

- `ArtsEpiphany`,
  a one-slice null control;
- `Zha_Ke`;
- `keyword233`;
- `Weideriche_`;
- `Toka_ls`,
  a historical editor-fabrication hard case;
- `XIEPT2`,
  a historical pairing hard case.

Alternate arm order by entry so first-arm timing is not confounded with time of day.
The smoke pair may count as `keyword233`'s decision pair only after its launch mechanics and outputs verify.

If the paired normalized effects disagree or remain inside their own spread,
repeat pairs interleaved rather than forcing a conclusion from one run each.

### Smoke result, 2026-08-27

Both `keyword233` arms settled two slices on pipeline digest
`sha256-tree-v1:711ef62a473323e52f727b29ea62bd0d481ae10a0b616e508379c5a43d15f068`.
Both providers were wet throughout,
both entries settled without a pass error,
and `verify-published` accepted both pages.

- Overlap `1`:
  38.50 minutes wall over 1.68 hours of calls,
  normalized `0.382`,
  9 voices unheard and peak 10 calls in flight.
- Overlap `4`:
  31.12 minutes wall over 1.78 hours of calls,
  normalized `0.291`,
  8 voices unheard and peak 19 calls in flight.

The overlap arm used 19.2 percent less wall time and 23.9 percent less normalized wall time.
It bought more calls because the live ballots produced more issues and rounds,
so spend from this pair cannot be attributed to overlap.
The result is a positive control,
not the fallback decision:
only two slices could overlap,
and refinement's no-memo caveat was not exercised observably.
Logs and roots begin `~/temp/agent/corpus-overlap-smoke-`.

### `Toka_ls` pair result, 2026-08-27

Both arms settled 15 slices on digest
`sha256-tree-v1:711ef62a473323e52f727b29ea62bd0d481ae10a0b616e508379c5a43d15f068`.
Both providers remained wet throughout both arms,
each arm lost 61 voices,
and `verify-published` matched one artifact to one page while exposing one promised silent passage.

-   Overlap `1`:
    313.24 minutes wall over 12.44 hours of calls,
    normalized `0.420`,
    peak 10 calls in flight.
-   Overlap `4`:
    104.37 minutes wall over 11.82 hours of calls,
    normalized `0.147`,
    peak 37 calls in flight.

Overlap `4` used 66.7 percent less wall time and 64.9 percent less normalized wall time.
Its call sum was 5.0 percent lower,
while metered Hyper spend was 104.2 percent higher because added concurrency overflowed subscription seats to Hyper.
Price is not decision constraint under provider authorization,
but routing is measured consequence rather than hidden cost.

Output reading found blocker independent of overlap:
source-only factual death paragraph was recorded unfilled and knowingly published in both artifacts.
No page from either old-digest arm is readiness evidence.
Current ten-model roster passed live coverage control on three damageable cases:
two targeted cuts flipped to `absent`,
third to `partly-carried`,
and all three equal-size decoys stayed `carried` with zero absence votes.
Fixed-build overlap-4 `Toka_ls` at 180-second grace settled complete page in 114.72 minutes,
normalized `0.132`,
with both providers wet.
Omission blocker,
destination loss and invisible bytes are fixed.
Strict output gate still failed one inherited major:
first-person intrusion that repair corrected but translate retained;
contest heard 8 of 10 and tied 4 to 4,
retaining archive.

Same-digest overlap-4 arm at 300-second grace settled in 127.43 minutes over 15.15 call-hours,
normalized `0.140`,
with both providers wet and 35 voices unheard against 72 at 180 seconds.
Strict first and independent readings find no blocker or major;
page is publishable as-is.
The arm retained date,
time,
place,
age and destination;
medical-cause generalization follows pinned corpus reader-protection policy.
This is one observation and does not decide grace default.
It also does not close declined-archive consolidation seam,
which remains visible on two other slices.
Proceed to remaining decision arms at built-in grace.

### `ArtsEpiphany` one-slice null pair, 2026-08-27

Both arms settled one unchanged slice with zero unheard voices,
verified one complete page,
and produced byte-identical pages with zero archive diff hunks.
No driver had second slice to overlap.

-   Overlap `1`:
    9.88 seconds wall over 36.93 call-seconds,
    normalized `0.268`.
-   Overlap `4`:
    14.87 seconds wall over 45.73 call-seconds,
    normalized `0.325`.

This is expected null control:
dial attribution holds,
output holds,
and timing difference where no work could overlap is provider variation rather than overlap effect.

## Readings per pair

Read only package-owned templates from logs.
For each arm record:

- TALLY status and wall milliseconds;
- stream elapsed sum,
  median and tail from `run-timing-report`;
- wall milliseconds divided by stream elapsed sum;
- rounds heard,
  configured voices,
  straggler cuts and recovery rounds;
- stage and provider failure counts;
- spend and provider meter state;
- settled artifact and `verify-published` result;
- persisted slice counts by namespace when an entry fails.

The effect is credible when paired wall-over-stream ratios improve consistently outside their observed spread.
Raw wall time alone cannot decide:
unchanged calibration arms moved 37 percent with provider speed,
while their normalized spread was about 0.03.

## Quality gates

Overlap cannot become fallback when it adds an entry error,
loses a page or destination,
fails `verify-published`,
increases voice loss beyond the paired run's provider evidence,
or produces an unexplained persistence difference.

Refinement is the one driver without an in-run twin memo.
Check logs and artifacts for repeated refinement questions,
stream-sum inflation and refine-only between-arm differences before closing that caveat.

## Output-reading continuation

After deciding the fallback,
run the remaining fresh-reading entries under the selected arm:
`gaoyanger`,
`Acheron`,
`wangzihao980`,
`dogesir_`,
plus any decision-set entry without a settled readable artifact.
Run `XingZ60` separately under the selected arm.

Each settled page is read against source and archive,
then checked by `verify-published`,
rendering audit and damage probe.
Every defect is traced to artifact slice,
lane and ballots before it is filed.
The durable reading record remains
`doc/audit/translation-repair-output-reading-20260826.md`.
