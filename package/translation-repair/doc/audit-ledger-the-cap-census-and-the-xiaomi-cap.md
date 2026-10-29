# Audit ledger: the cap census and the Xiaomi cap

A card that still named the pooled 99th percentile after the cap rule could read its calls (B990),
and a census that named every dangling link under the agent root and buried its own summary (B991).

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
