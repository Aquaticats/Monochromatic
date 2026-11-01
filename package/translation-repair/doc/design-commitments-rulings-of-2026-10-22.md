# Design commitments: rulings of 2026-10-22

Two rulings of 2026-10-22 made by the lead while fixing the defects the XingZ6018 read found,
each open to the owner's veto.
They answer questions the owner's own rulings left open,
and neither is the owner's answer.
Part of [Design commitments](design-commitments.md),
which lists each beside the earlier ruling it extends.

If the owner vetoes one,
the entry here is replaced by the owner's words and the date,
and the ledger entry named in it is updated.

## The entry in flight when a run stops stays resumable

Ruled by the lead on 2026-10-22 (UTC),
open to the owner's veto.
This extends "A run always ships" in
[shipping and the archive](design-commitments-shipping-and-the-archive.md)
and rests on the owner's ruling that the router's every-provider-dry refusal ends the run.
The question was whether "no matter what,
we must ship" covers the entry that is in flight when the run stops.

The entry in flight stays `INCOMPLETE` and stopped,
resumable,
keeping its slice cache.
The pages of the entries settled earlier ship,
each written when its entry settled,
so the run ships what it settled.

Filing the interrupted entry on the path of 2026-09-27 ("Keep archive,
ship":
a slice with no valid wording keeps the
archive's and the page ships) would mark it settled.
A settled entry discards its slice cache and nothing revisits it,
and an interrupted entry has slices no repair has reached,
not slices with no valid wording,
so that filing would freeze them unrepaired,
a worse page than resuming the entry on the next launch.

Two pieces are missing for any other design,
from the hand-back of the worktree `tr-w291`:
a degraded finish across the stages that makes no provider call,
and an artifact status for a partial entry.
Ledger:
B821 in `audit-ledger-the-every-provider-dry-refusal.md`.

## Only dryness of every provider ends the run

Ruled by the lead on 2026-10-22 (UTC),
open to the owner's veto,
and queued as D4 in the worktree `tr-w293`.
At `495341919a` the code still ends the run per model.
The owner's ruling that the refusal ends the run rests on one reason,
"this one leaves nothing,
so no seat's voice can be heard" (`stage/src/stage-run-end.ts`).
The router raises the refusal per model:
the branch of `provider/src/budget-routing.ts` that builds `meters read ... wet but not serving this model` throws it
when every provider that serves the model asked is dry,
while other providers are wet.

Only dryness of every provider the run holds ends the run.
A model whose own providers are all dry while others are wet is a lost seat:
the seat is dark,
and the bench reseats it the way it reseats an unserved seat.
A call that finds every provider dry still ends the run.
A dry decisions route with other providers wet is therefore a lost seat too,
which is what `provider/src/provider-router-decide.ts` already does.

The change updates `doc/decision/translation-repair-direct-providers.md`,
which states the per-model ending as designed ("a fleet that is dry except for Xiaomi still ends the run for every other
model").
That record describes the code as shipped and stays as written until the change lands.
Ledger:
B822 and B824 in `audit-ledger-the-every-provider-dry-refusal.md`
and `audit-ledger-unreachable-branches-kept-as-quiet-answers.md`.
