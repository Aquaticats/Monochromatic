# Audit ledger: futile returns replayed and the memory of skipped slices removed

The pass's memory of the translate slices it had skipped,
which the owner removed on 2026-10-29,
what the rule of B1010 spends when it is replayed over XingZ6020,
and the cache account of the round.
These continue [short bench floors and the waits that cross them](audit-ledger-short-bench-floors-and-the-waits-that-cross-them.md),
whose B1010 is the rule itself.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1011: the pass kept a memory of the slices it had skipped, and the owner removed it

Status:
done on 2026-10-29 (UTC) in `c4d4c80277`.
It supersedes the memory of B834,
and nothing else in B834 or B835.

The owner's answer,
through the question tool on 2026-10-29:
rounds that cannot decide below the floor of two voices (lane contest,
consolidation,
translate select) are skipped and the skip recorded.
A per-pass memory added for the same purpose saves about nothing (the payload store already replays).
The owner chose "Skip,
drop the memory (Recommended)":
keep the skips,
remove the per-pass memory.

What was removed:
`FloorLimitedMemory`,
`FloorLimitedRecord`,
`floorLimitedKey`,
`rememberFloorLimited`,
`notAskedAgain` and `AskDecision` from `translate-floor-limited.ts`,
the `floorLimited` parameter from `translate-slice-settle.ts`,
`translate-document.ts` and `document-lanes.ts`,
`pass-floor-limited.ts` (`floorLimitedByEntry`) from the corpus pass with its test,
the `floorLimited` handed through `pass-entry-settle.ts`,
`pass-entry.ts`,
`pass-lanes.ts` and `corpus-pass-run.ts`,
the exports of both packages,
and the case of `corpus-pass-run.unit.test.ts` that checked an entry is handed an empty memory.
The translate lane no longer writes the finding `translate-not-asked-again`
or the line that said a slice was not asked again.

What stayed:
the three skips and the artifact's record of each.
The lane contest opens no round for a slice whose bench reads under two,
the consolidation buys no slate for a slice whose slate bench reads under two with an eligible standing,
and the translate lane buys no candidate for a passage the archive never translated
(`selectBelowFloor` and `skippedBelowFloor`).
Each ends as before,
with `benchBelowFloor` recorded from artifact generation 21 (B840).

What changes in a run:
a later attempt of the same pass at the same reach skips the same passage again,
records the mark again and says why again,
where the memory made it silent.
A slice that is not skipped,
because the archive translates it,
and that ended unfilled under the floor,
is asked again by a later attempt,
and the payload store replays its calls,
which is why the memory saved about nothing
(B834 measured 0 NeuralWatt calls and 0 Wh in the second attempt of 18 slices).

The case that shows it:
`translate-floor-limited.unit.test.ts` holds three attempts at a reach of one,
each of which makes no call,
ends unfilled as `no-candidate-backed` with the mark and the finding that names the bench,
and says why on one line of its own,
with no memory between them,
and a reach of two that buys the slice.
With `selectBelowFloor` changed to return false,
four of its eight cases fail,
among them both new ones.

The message of `c4d4c80277` says that four of seven cases fail and that three cases replace the memory cases.
The file holds eight cases and two are new,
so the message is wrong on both counts and this entry is the correction.

No cache version moved:
the memory was in the process,
and an unfilled slice is not cached.

## B1012: what the rule spends on XingZ6020

Status:
measured on 2026-10-29 (UTC),
keyless and from the log alone.
The script is outside the repository:
`futile-replay.ts` in this round's scratch folder,
adapted from the scripts of option (c) in `d123-recheck2.md`.

Method:
the log is `corpus-pass-XingZ6020.log`,
the same as option (c) used.
From it the script reads the 228 hold waits (`JUDGE SEATS phase=... waiting N ms`),
the 395 refusals of Synthetic with the hold each set (`held out for N ms`) and its 1,657 served calls
(`SPEND provider=synthetic`).
Every one of the 395 refusals read the meter as unreadable and held Synthetic out for 300000 ms,
and none named its return,
so every hold was a refusal cooldown.
The script marks a refusal as the budgets would
(a hold of some time ended,
the refusal arrived after the end,
and no call was served since the end),
lets the next served call clear the mark,
and counts a logged wait as skipped when it began under a standing mark.
The baseline reproduces option (c)'s figures for the rule in force:
31,509.8 s and 146.8 min for the lane contest and the consolidation,
four consolidation waits followed by a served call.
The replay is first order:
a wait that is skipped changes what the stages do afterwards,
which a log cannot show.

The lane contest and the consolidation,
as option (c) reported them:
- The rule in force:
  31,509.8 s,
  146.8 min,
  four waits followed by a served call (the return of 05:06:50 UTC and its 20 calls).
- Reading A,
  the mark forgotten at the end of each phase,
  the scope of the measurement:
  lane contest 1 wait,
  216.5 s,
  3.6 min;
  consolidation 5 waits,
  1,255.5 s,
  9.7 min;
  together 1,472.0 s and 13.3 min,
  with no kept wait followed by a served call.
- Reading B,
  the scope of one entry that the code implements,
  with the marks the whole log gives:
  lane contest 0 s;
  consolidation 4 waits,
  974.5 s;
  together 974.5 s and 5.0 min,
  with no kept wait followed by a served call.
- Reading C,
  the cooldown give-up alone,
  which every one of these holds was:
  0 s in both phases.

Why reading A is not the 497.5 s of option (c):
the first wait of each phase is 216.5 s and 281.0 s,
which sum to 497.5 s and are exactly what option (c) kept.
Four further consolidation waits
(log lines 37077,
37085,
37117 and 37167,
from 05:12:24 UTC to 05:15:00 UTC,
all ending at 05:17:24 UTC)
begin after Synthetic had served 20 calls between 05:06:50 UTC and 05:12:12 UTC,
and a served call clears the mark.
Option (c) said only the first cycle of each phase is waited,
without a served call clearing anything.
Whether those 20 calls happen without the wait of the return at 05:06:50 UTC
the log cannot say,
because the stages seat Synthetic once its hold has ended,
whether or not a reader waited for it.
Gates bought:
0 in every reading.
The three gates (slices 27,
28 and 33) came from the four waits that ended at 05:06:50 UTC,
and every reading skips them,
because the mark set at 04:35:12 UTC stands until the first served call at 05:06:59 UTC.
A stage that starts after the return could still seat Synthetic and buy them,
which the replay does not count.

The other phases,
which option (c) left out because the rules differ only on a floor-only lift:
- The translate lane:
  112 waits and 28,850.0 s in the log,
  8 waits and 2,012.1 s under readings A and B,
  4 waits and 925.2 s under C.
  The 4 waits followed by 25 served calls at 01:42:52 UTC are skipped by A and B and by C.
  The mark that skips them is the refusal at 00:34:14 UTC.
- The lanes keep their 4 waits under A and lose them under B and C;
  the page consistency pass keeps its one wait under A and loses it under B and C.

What the figures do not say:
the replay cannot say what the stages would have done with the time,
and the mark of reading B is set by phases that ran before the one it skips.

## B1013: the cache account of the round and the ids it did not use

Cache account:
no cache version moved.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 24a0c34d16..HEAD -- package/translation-repair` prints nothing,
and the same search over `c265e8df95~1..c265e8df95` prints `c265e8df95`,
so the search works.
The marks are in the process.
They enter no cache key and no stored record.
The reader's choice not to wait changes which seats a stage reads,
not what any record means.
The artifact generation stays 21:
the skips and their `benchBelowFloor` mark are the same,
and the finding `translate-not-asked-again`,
which the removed memory wrote for a slice it kept,
is a finding string that no reader of the artifact checks.

Ids:
B1010 to B1013 are used,
and B1014 is not.
