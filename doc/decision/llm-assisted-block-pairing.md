# Paragraph pairing is done by a model

Decision record.
Decision:
pairing source blocks to translation blocks is LLM-assisted,
taken 2026-08-20 by the user,
in their words:
"I believe LLM-assisted pairing is the only way."

## What forced it

Six of eleven slices on `saurikissa` paired unrelated paragraphs,
with the artifact reporting no alignment finding at all,
and every downstream stage then behaved correctly on wrong input.
`doc/audit/the-critics-are-shown-the-wrong-paragraph.md` carries the reading.

The deterministic aligner has three signals and this corpus exhausts all three.

BLOCK KIND carries nothing in prose,
because every block on both sides is a
paragraph,
so the kind term is constant across every candidate pairing.

SHARED TOKENS carry nothing across scripts.
The matcher counts Latin words,
digit
runs and component names,
and Chinese prose has none of them.
The anchors that
DO exist,
transliterated names of people and places,
are invisible to it because
the Chinese side is not Latin.

LENGTH is what remains,
and it was miscalibrated:
a fixed expansion constant of
1.8 against entry medians running 1.49 to 4.10.
Estimating expansion per document
is now landed and moves `saurikissa` from two correct pairings in eight to four.

FOUR IN EIGHT IS WHERE DETERMINISM STOPS.
Sweeping the length weight over 1,
2,
3
and 4 changes nothing,
so the remaining errors are not a tuning question.
They are
a one-to-two correspondence,
where the English splits a paragraph the Chinese
keeps whole,
which the step vocabulary cannot express at all:
it has `pair`,
`skip-source` and `skip-target` and no merge.

## Why a model rather than more machinery

The remaining work is reading comprehension across two languages,
which is the one thing this pipeline already has six models for
and no amount of scoring can substitute for.
Adding merge steps and a transliteration table would raise the ceiling
and still leave a scorer guessing at meaning it cannot read.

## What this does NOT decide

Whether the deterministic aligner is removed.
It stays as the fallback when the
roster cannot be reached or cannot agree,
and its per-document expansion fix
stands on its own.

The shape of the model stage:
ensemble size,
prompt,
how disagreement resolves,
and what happens to a block the roster refuses to pair.
Those belong in the
implementation and in `#131`.

## What it obliges

A PAIRING THE ROSTER REFUSES MUST NOT SILENTLY PROCEED,
which is what `#71`
already demanded of the section aligner:
a wrong pairing is worse than no
pairing,
because it manufactures issues rather than skipping work.

RE-MEASURE EVERYTHING DOWNSTREAM AFTERWARDS.
Conclusions recorded this week about
the window,
the panel,
the lanes and the judges all rest on runs whose critics
were shown the wrong paragraph on a fifth to a half of slices.
