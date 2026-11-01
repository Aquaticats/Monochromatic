# The recovery round re-asks with the complaint appended

Decided by the owner on 2026-09-03 ("Re-ask with the guard's complaint quoted"),
asked with three options
on issue 473.

## The conflict the question named

- The uniqueness rule (`prompt-uniqueness-client.ts`):
  one model,
  one completed prompt,
  never sampled twice;
    a repeat is served from memory.
  Deliberate,
  privacy- and spend-motivated.
- The recovery rule (`stage-quorum.ts`):
  an answer that arrived but could not be read is re-asked once,
    measured on 2026-08-25 as the cheap half of 36 losses (13 answered badly,
  23 were silence).
- Together they made the recovery round a no-op:
  five recovery rounds over two passes on 2026-09-02 were
    answered from the cache with the same unreadable bytes in 0 to 1 ms.

## The rule

- The recovery round sends a different prompt:
  the original messages plus a complaint naming what happened
    and what to do (`RECOVERY_NUDGE`).
  The digest is new,
  so uniqueness holds,
  and the re-ask carries
    information the first ask did not.
- The complaint is general,
  not the guard's own words:
  every stage guard is a type predicate and carries no
    message.
  Where a guard grows a message,
  the complaint quotes it;
  nothing here forbids that.
- One round,
  never a ladder,
  as before.

## Options rejected

- Delete the round and answer the gemma item by a guard reading or a drop:
  throws away the measured
    recoverable half.
- Leave the no-op:
  a round that logs "recovery" while recovering nothing misleads every later reading.

## What landed

- `6323f05d8` (2026-09-02) built the re-ask with the complaint appended,
  ahead of the decision,
  and the
    decision ratifies it.
- 2026-09-03:
  the round logs `<stage>: recovery round heard N of M re-asked voices`,
  so a run log says
    whether the round earns its call.
- Measured the same day on the keyword233 passes that ran recovery rounds (pairing each round's line with
    its stage's next `round: x/N heard`):
  5 of 11 re-asked voices came back readable,
  none served from the
    cache.
  The misses were one endpoint answering unreadably twice (MiniMax on Parasail,
  since ignored).
    The round earns its call;
  issue 473 closed on this.

## Where the evidence lives

- Issue 473,
  with the five measured no-op rounds.
- `doc/planning/translation-repair-openrouter-2026-09-03.md` for the pass that first carries the count.
