# Audit ledger: the cap census and the Xiaomi cap

A card that still named the pooled 99th percentile after the cap rule could read its calls (B990),
a census that named every dangling link under the agent root and buried its own summary (B991),
a pairing window of 50 ms that left every long call at the cap unpaired (B1070),
and a seat map that seated no NeuralWatt call and none on the DeepSeek seat's ids from before the 1017 weights (B1071).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B990: the Xiaomi checker card still rode the pooled 99th after the cap rule could read its calls

Fixed on 2026-10-29 (UTC) in `d599b87fab` (shared-contract),
with the census change that left room for its test in `91710ff5d3` (run).
The cases were committed red first,
in `299e1a797e` and `de96820c96`.
The message of `d599b87fab` says "none at the cap",
which was true of the old cap of 13,082 only:
at 5,098,
3 of Xiaomi's 293 calls reach it.
Commit comment 207194488 corrects the message,
and `5f9a726c61` corrects the comment on the card in `model-cards.ts` (ledger M312).

Found on 2026-10-29 by the pre-launch census (`doc/runbook/translation-repair-corpus-pass.md`,
Setup step 6),
which printed `xiaomi/mimo-v2.6-flash: card cap 13082 (pooled placeholder), rule reads 5098`
and `PLACEHOLDER WITH A DISTRIBUTION`.
The census had read 109,489 logs,
447 of them pass runs,
579,058 completed calls.
The card named the pooled 99th (13,082) because it held 78 OpenRouter calls on 2026-09-28,
under the 100 the rule needs (P10 in `audit-ledger-providers-routing-and-seating.md`).
Xiaomi's own endpoint joined the route on 2026-10-22,
and the card still named the pool.

What the rule reads:

- Xiaomi's own endpoint:
  293 completed calls,
  p99 5,098,
  none at the old cap of 13,082.
- OpenRouter:
  1,362 completed calls,
  p99 553,
  none at the old cap.
- The highest p99 of a provider with at least 100 calls,
  floored at the pooled 90th (3,831),
  is 5,098.

The fix:

- The card carries `5_098` with the reading,
  the date and the call counts it rests on,
  the way the other measured cards carry theirs.
  The `max_tokens` sent for this seat falls from 13,082 to 5,098.
- By the rule's own design a cap at the highest p99 cuts about one percent of that provider's calls,
  the trade every measured card makes (`completion-cap.ts`).
  Re-read with the new card,
  the census shows 3 of Xiaomi's 293 calls at the cap (1.02 percent;
  2 with content,
  1 with no stream line to pair),
  so the row now carries `CUTS OVER ONE PERCENT`,
  as seven other seat and provider pairs do (P10).
  No call of either provider ran to the old cap of 13,082.
- No card names the pooled 99th any more,
  so `completion-cap.unit.test.ts` expects none,
  and the census's own case for a placeholder row passes the seats it is told (`placeholderSeats`)
  instead of leaning on a card.
  The default is still read off the cards,
  so a new card that names the pool is flagged again.
- The `RULE READS ... AGAINST` rows of Kimi,
  minimax,
  glm,
  deepseek and haiku are for reading,
  as the runbook says,
  and no other card moved.

Open to the owner's veto:
a higher cap buys the top percent of Xiaomi's answers at the price of a longer runaway,
and the figure is one reading of logs a running pass was still writing.

Recurrence:
none found,
and none expected from the card alone:
the census flag is what names the next placeholder with a distribution.

## B991: the census named every dangling link under the agent root and buried its own summary

Fixed on 2026-10-29 (UTC) in `4c850b45cc` (walk,
report and summary line),
`91710ff5d3` (placeholder seats),
`af7d6ddbf8` (the built command's pinned summary line) and `f16cfb3636` (lint rules).
The cases were committed red first,
in `4b7b4d33c3`.

Found on 2026-10-29 running the census over `~/temp/agent`:
315 lines of `cap-census: cannot read <path>`,
312 of them `ENOENT` on `stat` and three `EACCES` on `scandir`,
among the summary and the seat rows the operator had to filter them out to read.
The 312 were links to nothing,
none under a name ending `.log`:
browser profile `SingletonLock` and `SingletonCookie` links,
scratch `bin/git` links,
and links whose targets are gone.
The three were directories nobody may list.
The summary counted all 315 as unreadable.

What each kind means for the count:

- A link to nothing holds no log.
  A directory it stood for would be there to list and a file it stood for to read,
  so under a name the census would never take (anything not ending `.log`)
  it cannot change the result.
- A link to nothing named like a log may stand for a log,
  so it is still named and counted as unreadable.
- A directory the walk cannot list may hold a run log,
  so it is still named and counted.
- A path named on the command line is always reported.

The fix:

- `capCensusLogsUnder` lists directories with their entry types.
  A regular file that is no log is dropped by its name before any stat.
  A link is still followed,
  since it may lead to a directory that holds logs,
  and one that leads to nothing under a name no log has is passed over silently
  and returned as `danglingLinks`.
- The summary line carries the count after the unreadable paths,
  and the lines left on stderr are the ones that could have held a log.
- The runbook's pattern `^cap-census:` still matches those lines,
  and step 6 now says what they and the new count mean.

Re-read after the fix,
the summary line counts 109,526 logs,
447 pass-run logs and 579,238 completed calls,
then `3 paths unreadable, 312 dangling links with no log name passed over`.
The three listed paths are the three directories,
the 312 are the links,
and the pass-run logs are the same 447.
The log count is 37 higher than the first read's 109,489,
so the two reads are not one snapshot.

Recurrence:
`mistake-prevention.md`,
"Searches and censuses that miss":
a census reports what it could not read where that could change its figure,
and counts apart what could not.

## B1070: the census paired a spend line with its stream within 50 ms, so long calls at the cap read unpaired

Status:
fixed at 23:51 UTC on 2026-10-29 in `a85074bf3d` (run),
the cases committed red first in `5beb18316b` at 23:45 UTC,
and a stream an ambiguous line has seen kept contested,
at a constant cost per line,
in `a0e99a3fed` after the red `5b7aef0e8f`.
The lead's decision of 2026-10-29 (open to the owner's veto):
"Census pairing by file order:
pair each SPEND line with the latest earlier stream line of the same label that no SPEND has taken,
within a bound of whole seconds you justify from the measured gaps;
when two candidates qualify,
mark the call `ambiguous` (a new column or state,
counted apart) instead of guessing."
No cache version moved:
the census reads logs and keys nothing.

Found on 2026-10-29 by the read of the DeepSeek calls that ran to the cap.
Seven of the 63 direct DeepSeek calls (`provider=deepseek model=deepseek-flash`)
completed at 13,082 tokens with `finish_reason=length` and no content:
`corpus-pass-XingZ6019.log:407`,
`corpus-pass-XingZ6020.log:1657`,
`:10238`,
`:19841`,
`:26556`
and `:29966`,
and `corpus-pass-XingZ6021.log:470`.
The census printed every one of them as `unpaired`,
and the same for 233 of OpenRouter's GLM-5.3-Flash calls at the cap and 34 of Hyper's minimax-m3 calls.

Why.
`PAIR_WINDOW_MS` was 50,
and the premise comment said the stream line and the spend line of one exchange are written within milliseconds.
They are written in that order,
but the gap grows with the body the stream read.
In the seven cases the stream line sits 139 to 589 ms ahead of its spend line
(`XingZ6021` line 469 is 150 ms ahead of 470,
`XingZ6020` lines 1656,
10237,
19840,
26555 and 29965 are 141,
148,
139,
406 and 589 ms ahead,
and `XingZ6019` line 406 is 167 ms ahead),
and each of the seven read 4,100,450 to 4,112,827 raw characters.
Under 300,000 raw characters the gap is 1 to 10 ms.
A window of 50 ms could not hold any of them.

What the pass-run logs under the agent root show
(447 logs,
582,710 reported spend lines,
each against the latest completed stream of its label that no earlier line had taken,
read by a scratch script of the first form of the rule,
before a stream seen by an ambiguous line was kept contested):

- The gap is 1 ms at the median,
  4 ms at the 90th percentile,
  40 ms at the 99th and 115 ms at the 99.9th.
  36 calls sit past two seconds,
  and the longest gap is 13,520 ms.
- Among the 34,255 calls of 3,800 completion tokens or more,
  which takes in the cap of every card,
  the longest gap is 2,096 ms and 3 calls sit past one second.
- The calls that read as ambiguous grow with the window:
  33 at 50 ms,
  502 at one second,
  871 at two,
  1,335 at three,
  2,947 at five and 20,578 at thirty.
  The calls left unpaired fall from 3,878 at 50 ms to 69,
  36,
  23 and 15 at one,
  two,
  three and five seconds.

The fix (`cap-census-read.ts`):

- `PAIR_WINDOW_MS` is 3,000.
  It holds every call of 3,800 tokens or more with room (2,096 ms is the longest),
  leaves 23 of the 582,710 calls outside it,
  and costs 464 more ambiguous calls than two seconds would.
- A spend line pairs only with a completed stream of its label that comes before it in the file
  and that no earlier spend line took.
  One such stream is its own and is taken.
  None reads `unpaired`.
  Two or more read `ambiguous`,
  and all stay waiting as contested,
  so the next spend line of the label reads the same streams and is ambiguous as well.
- A stream an ambiguous line has seen is contested for as long as it waits.
  A later line that finds only that stream left reads `ambiguous` and takes nothing,
  because an earlier line may own it.
  This came from the post-commit standards review of `a85074bf3d`
  (`doc/audit/standards-review-gate-shadow.md`),
  which found both the false certain pairing and the cost:
  counting candidates scanned the whole queue,
  so n streams and n spend lines of one stamp cost n squared.
  The queue keeps a front index,
  a waiting count and a sequence number for each stream,
  and a line costs a constant amount.
- A stream older than the window against the newest line of its label is dropped from the queue,
  so a label that never gets a spend line (`credits`,
  `quota`,
  `models`)
  holds no more than three seconds of lines.
- `CapSample.content` gains `ambiguous`,
  `ProviderCapReading` gains `atCapAmbiguous`,
  and the provider line ends `, N ambiguous` after the unpaired count.
- The premise comment no longer says the two lines are written within milliseconds.

Re-read with the fix over the same agent root (447 pass-run logs,
582,710 calls),
before and after the contested change alike,
the calls at the cap of all seats and providers moved as follows:

- Unpaired:
  289 before,
  0 after.
- Ambiguous:
  0 before,
  4 after
  (GLM-5.3-Flash on Synthetic 2,
  Qwen3.8-27B on Synthetic 1,
  minimax-m3 on Hyper 1).
- The seven direct DeepSeek calls are 0 with content and 7 with none,
  where they were 7 unpaired.
  Six came after a NeuralWatt runaway of the same model
  (the pairs `XingZ6021` lines 456 and 470,
  `XingZ6020` lines 1626 and 1657,
  10171 and 10238,
  19794 and 19841,
  26490 and 26556,
  and 26575 and 29966),
  and each was the nudged re-ask (B1072),
  the last made from within the stage's recovery round (B1073),
  while the seventh (`XingZ6019` line 407) is a first ask.
- OpenRouter's GLM-5.3-Flash calls at the cap read 14 with content and 225 with none,
  where they read 3 with content,
  3 with none and 233 unpaired.
  Hyper's minimax-m3 read 13 with content,
  252 with none and 1 ambiguous,
  where they read 9,
  223 and 34 unpaired.
- The rule's reading and the flags of the rows did not change because of this entry.
  The one row that moved moved for B1071.

Why a wider window finds 464 more ambiguous calls at three seconds than at two was not determined.
A stream with no spend line of its own,
in reach of a neighbour of the same label,
would explain it,
and the logs were not read for one.

Open to the owner's veto:
the window of three seconds is a choice made from the measured gaps,
and a log written under heavier load than these would need a wider one.
The census would then print more `unpaired`,
never a guessed pair.

Recurrence:
`mistake-prevention.md`,
"Searches and censuses that miss":
a pairing window is measured on the calls it must hold,
and what two lines could each be is counted apart,
never chosen.

## B1071: the census seated no NeuralWatt call and none on the DeepSeek seat's ids from before the 1017 weights

Status:
fixed at 23:51 UTC on 2026-10-29 in `a85074bf3d` (run),
the cases committed red first in `5beb18316b`.
The lead's decision of 2026-10-29 (open to the owner's veto):
"add `neuralwatt` and the retired deepseek ids to `SEAT_OF_SERVED`
(read `model-cards.ts` for the NeuralWatt side)
so their calls count under the seat."
No cache version moved:
the census keys nothing.

Found on 2026-10-29 with B1070:
`SEAT_OF_SERVED` in `cap-census-rule.ts` keyed the card sides of seven providers and none of NeuralWatt's,
and the ids Hyper and OpenRouter served for the DeepSeek seat before it moved to the 1017 weights on 2026-10-20
(`deepseek-v4.1-flash` and `deepseek/deepseek-v4.1-flash`) are on no card.
So 132,085 calls printed as `on ids no card names`:
5,310 of them NeuralWatt's `deepseek-v4.1-flash`,
1,758 NeuralWatt's `glm-5.3-flash`,
6,785 Hyper's `deepseek-v4.1-flash`
and 64,526 OpenRouter's `deepseek/deepseek-v4.1-flash`.

The fix:

- `SEAT_OF_SERVED` adds `neuralwatt <card id>` from each card's NeuralWatt side.
- `RETIRED_SERVED_IDS` adds the DeepSeek seat's two ids from before the migration,
  with the evidence on the constant:
  `roster-changes.md` says the seat was served as `deepseek-v4.1-flash` on Hyper
  and `deepseek/deepseek-v4.1-flash` on OpenRouter from its approval on 2026-09-11
  and as the 1017 ids since 2026-10-20,
  and the card's cap was measured on the older ids.
- The other ids that print as `on ids no card names` stay unlisted:
  `deepseek-v4-flash-0731` was a separate version whose ratings "did not transfer" (`roster-changes.md`),
  and no card or document ties `deepseek-v4-pro-0813` or the rest to a seat.

Re-read over the same agent root,
the count of calls on ids no card names fell from 132,085 to 53,706,
which is 5,310 + 1,758 + 6,785 + 64,526 fewer.
What remains:
15,675 on `typesafe/jev-1.13`
(OpenRouter's decisions endpoint,
a decision-only seat the census does not seat),
7,720 and 7,542 on `deepseek-v4-pro-0813`,
6,510 and 5,393 on `deepseek-v4-flash-0731`,
4,524 on OpenRouter's `qwen/qwen3.8-27b`,
2,627 on its `z-ai/glm-5.3`,
1,742 on Synthetic's Nemotron,
959 on its `GLM-5.2`,
719 on Hyper's `qwen3.8-max`
and 295 on its `glm-5.2`.

One row of the census changed its reading,
the DeepSeek seat's (`deepseek-v4.1-flash`,
card cap 13,082):

- Before:
  `rule reads 9234`,
  flagged `RULE READS 9234 AGAINST THE CARD'S 13082`.
  The seat held 63 direct DeepSeek calls,
  335 Hyper calls and 1,081 OpenRouter calls.
- After:
  `rule reads 13082`,
  and that flag is gone,
  replaced by `CUTS OVER ONE PERCENT`:
  Hyper 142 of 7,120 calls at the cap (1.99 percent),
  NeuralWatt 63 of 5,310 (1.19 percent),
  OpenRouter 241 of 65,607 (0.37 percent),
  and the direct 7 of 63.
  Of the 142 on Hyper,
  5 carry content and 137 none.
- The card's cap did not change.
  The rule reads 13,082 now because the 6,785 pre-1017 Hyper calls join the seat,
  and Hyper's 99th percentile over its 7,120 calls is 13,082 (4,651 over the 335 calls of the 1017 id),
  while OpenRouter's over its 65,607 calls is 7,556 (9,234 over the 1,081 of the 1017 id).
  The calls of the retired ids are from before the migration,
  so they measure the earlier weights and not the 1017 ones.
  Whether 13,082 is right for the 1017 weights is the open question the card already states
  ("carry over as an assumption until the first runs on 1017 measure them again"),
  and this entry does not answer it.

Recurrence:
the census seats a provider the moment a card carries its side,
and `cap-census.unit.test.ts` counts a NeuralWatt call under the seat of each of the two models it serves.
