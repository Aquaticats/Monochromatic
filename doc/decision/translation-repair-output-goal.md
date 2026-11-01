# What the translation pipeline owes its output

Decision record for what `@monochromatic-dev/module-translation-repair` is FOR.
Decision:
 the pipeline yields a good translation of the original,
 taken on 2026-08-13 by the user,
even where the translation fed in does not make sense.

This supersedes the framing every stage was built under,
 which treated the input translation as the thing being repaired
 and the pipeline as a defect-fixing loop over it.

## What prompted it

A drawn sample surfaced sections whose English covers a fraction of their
 Chinese.
Measured across the 56 settled entries,
 60 of 172 aligned chunk pairs differ in block count,
 and the extremes are not close calls:

```text
XingZ60   source=76 blocks  target= 5
XingZ60   source=62 blocks  target= 1
XIEPT2    source=24 blocks  target= 1
```

The pipeline repaired them as ordinary translations with defects in them.
Critics filed omission after omission,
 the panel accepted them,
 and the editor wrote English to fill gaps that are the corpus's own state.

The question put to the user was which of three things to do with such a
 section:
 repair it as now,
 skip it,
 or report it as a corpus gap for a human translator.

## The answer, and why the question was wrong

The user rejected the question as an X/Y problem.
All three options assumed the deliverable is a REPAIRED VERSION OF THE INPUT,
 so all three were arguments about when to give up on repairing.
If the deliverable is a good translation of the ORIGINAL,
 a section that was barely translated is not an edge case at all:
 it is simply a section the pipeline has to translate.

The user's words:
 the pipeline should yield a good translation at the end
 even if the translation fed in does not make any sense,
 and this indicates a re-design may be necessary.

## What this decides, and what it does not

Decided:
 the output is judged against the ORIGINAL,
 not against how far it improved on the input translation.
An input translation is EVIDENCE about what the original says
 and a starting point worth preserving where it is right,
 never the standard the result is measured by.

Decided:
 an input that is absent,
partial,
or nonsensical is ordinary input.
It is not a reason to decline,
skip,
or file the shortfall as a defect for
 someone else.

NOT decided:
 what the re-design is.
The user named a re-design as likely,
not as chosen,
 and its shape belongs in `doc/planning/` until they take it.

## What already points the same way

The probe question was re-anchored on 2026-08-12 for an unrelated reason,
 from "did the replacement introduce a defect the BEFORE text did not have"
 to "does the AFTER text misrepresent the ORIGINAL".
That change was made because the old question made the pre-edit translation the
 standard of accuracy,
 and every claim it produced argued from that text,
 one of them reporting a corrected mistranslation as damage.
The same error at stage level is what this record decides against.

A second user decision on the same day runs with it:
 accurate detail a translator ADDED,
 a citation's translator,
publisher and ISBN where the original names only the
 work,
 is kept rather than stripped to match the original.
Faithfulness to the original is what the output is judged by;
 it is not a licence to delete correct information a reader benefits from.
