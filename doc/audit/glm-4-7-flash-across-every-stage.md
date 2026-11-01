# GLM-4.7-Flash across every stage

Read 2026-08-20 against the authorisation to drop it,
which was conditional on it being disproportionately bad versus every other
model on every stage.

## Verdict: the condition is not met, so it stays

It is the worst model on some stages,
middling on others,
and clearly better than two of its peers on one.
Three separate stages have a worse model than this one.

## What was measured

Twenty-two settled version 2 artifacts,
every stage that records per-model attribution,
each reported as a rate over that model's own opportunities.
A count of failures without a count of contributions says nothing:
a model asked less often fails less often.

## Critic: fifth of six, and not last

Claims that survived adjudication,
over claims proposed:

-   `Kimi-K3` 185/270,
    69 percent.
-   `Qwen3.6-27B` 129/190,
    68 percent.
-   `GLM-5.2` 142/213,
    67 percent.
-   `Nemotron-3-Super` 131/216,
    61 percent.
-   `GLM-4.7-Flash` 59/116,
    51 percent.
-   `gpt-oss-120b` 86/181,
    48 percent.

READ,
NOT ONLY COUNTED.
Its accepted claims are specific and correct:
a causal adverb rendered as a temporal one,
an omitted character trait,
active coercion softened into a preference,
a dropped clause tying one trait back to the narrator.
Its rejected claims are mostly fine-grained nuance the panel declined to sustain,
not fabrication,
and several read as defensible calls that simply lost.
This is a useful critic with a lower yield,
not a noise source.

## Translate judge: third best, and far from worst

Abstentions over ballots cast:

-   `GLM-5.2` 0/108 and `Kimi-K3` 0/120,
    0 percent.
-   `Qwen3.6-27B` 1/121,
    1 percent.
-   `GLM-4.7-Flash` 6/117,
    5 percent.
-   `gpt-oss-120b` 32/120,
    27 percent.
-   `Nemotron-3-Super` 65/121,
    54 percent.

A model that declines to rank on more than half its ballots is a weaker judge
than one that answers nineteen times in twenty.
This is the stage where the authorisation's condition fails outright.

## Historical voice loss: second, behind GLM-5.2 by a factor of three

`stage-voice-lost` findings naming each model,
across the same artifacts:

-   `GLM-5.2` 71.
-   `GLM-4.7-Flash` 25.
-   `gpt-oss-120b` 5.
-   `Kimi-K3` 2.
-   `Qwen3.6-27B` 0 and `Nemotron-3-Super` 0.

## Where it IS worst

TRANSLATE SHIP RATE,
candidates that shipped over candidates offered:
9/132,
7 percent,
against `Nemotron-3-Super` at 10 percent and `Kimi-K3` at 11 percent.
Worst,
but inside the spread.

STRUCTURALLY INVALID TRANSLATIONS:
46,
against 29 to 35 for everyone else.
One of its rejected translations rendered a single original paragraph as
forty-seven blocks.

FAILING TO REPAIR ITS OWN INVALID TRANSLATION:
12 `translate-repair-unresolved`
findings,
against two or fewer for every other model.
This is the one historical stage where it is disproportionately bad.

LANE CONTEST:
4 lost voices in 26 rounds,
15 percent,
while every other model on
the roster lost none.
Four failures in three shapes:
a stop mid-string,
a degenerate repetition the
runaway guard cut at 131096 characters,
and an empty completion.

A MID-FLIGHT READING OF 50 PERCENT ON THIS STAGE WAS WRONG.
It came from three losses in the first six rounds.
The completed 26 rounds put it at 15 percent,
still uniquely bad and not the
figure first reported.

## What the finish reason established

The stop is not a token ceiling.
With `finish_reason` now read,
one truncation reports `finish_reason=stop`:
the model emitted a stop token in the middle of a JSON string value and the
provider considers that completion finished.
Raising a ceiling cannot fix it.

## What outranks dropping it

Judging one contested slice,
every judge shown the documents' declared names,
three of six penalised a candidate for carrying a declared alias:
`GLM-5.2`,
`GLM-4.7-Flash` and `Qwen3.6-27B` each called it an addition the
original does not support.
`GLM-4.7-Flash` is not unusual here.
It is the majority behaviour among the models that reasoned about it at all.

So the translate judge's own faithfulness criterion overrides the declared-names
context it is already given.
That is a prompt defect at a production stage,
it explains the translate lane
stripping accurate detail that
`doc/audit/eight-entries-read-against-the-original.md` recorded,
and it costs more than any one model's reliability.
The lane contest received exactly this fix on 2026-08-20;
the translate judge has not.
