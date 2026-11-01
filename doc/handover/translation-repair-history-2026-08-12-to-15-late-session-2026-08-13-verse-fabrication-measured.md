# Translation repair history 2026-08-12 to 2026-08-15: the late session of 2026-08-13: verse fabrication measured and two measurement bugs

The late session of 2026-08-13:
the verse fabrication fix and the measurement bugs in its acceptance check,
the unit at which the predicate must be measured,
and the governance rule as a union of slice and chunk readings.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-13 late: verse fabrication measured, and two measurement bugs of my own

### `#79` is fixed, and the number that says so is not the one I first reported

The acceptance check I wrote earlier compared the two texts BY LINE INDEX.
That
is wrong the moment a line is inserted,
because everything after shifts,
so its
`emptied` and `changed` counts past the first insertion measure the shift rather
than the edit.
It also judged the WHOLE DOCUMENT on line count,
and `Toka_ls` is
one verse chunk plus two prose chunks,
so it condemned legitimate prose reflow.

Rewritten to measure the only thing that decides the question,
net line delta
inside the slices the addendum GOVERNS:

        run          tip          governed edits   net   prose edits   net
        pre-fix      95f72e591          55         +24        14         0
        source-fix   91ba66671          13           0        17        +5

`+24` fabricated lines inside verse becomes `0`.
The `+5` on the later run is
entirely in prose slices,
where reflow is what a repair is for.

The check lives at `scratchpad/verse-check.ts` and recomputes governance from the
shipped rule rather than assuming it.

### The predicate needs the CHUNK, and I measured it at the wrong unit twice

`isLineStructured` refuses to answer below five blocks,
because under that a
stanza and a couple of short paragraphs are indistinguishable.
Two readings I
took were invalid for exactly that reason and are deleted rather than kept:

-   one fed it whole-document BLOCKS and concluded `Toka_ls` has no verse;
-   one fed it individual replaced REGIONS and concluded no repaired region was
    line-structured.

Both could only ever return false.
Anything reading "0 line-structured" from that
period is an artifact of the unit,
not a finding.

At the correct unit the entry is unambiguous:

-   source,
    34 blocks,
    median 23 chars:
    line-structured
-   target,
    30 blocks,
    median 106 chars:
    not

That gap IS `#79`.
The published translation had already flattened the verse into
prose,
so a predicate reading the translation could never fire on the case it
exists for.
Only the original carries the structure.

### Governance is a union, and chunk-inheritance alone was a regression

Deciding per slice dropped the rule on most of the verse:
the `Toka_ls` verse
chunk trips at 21 blocks,
median 22,
subdivides into seven slices,
and only ONE
still trips.
Four of the other six sit at medians 20,
22,
23 and 29.

But moving the decision to the chunk alone was not a widening,
it REPLACED the
slice reading and lost ground.
Measured deterministically across the 92 entries
at the pinned corpus commit,
no model calls:

        slice only      55 governed slices
        chunk only     195 governed, but FOUR entries go BACKWARDS
                       interrgned 5 -> 1, three others 1 -> 0
        union          203 governed, zero entries lose ground

Those four are stanzas inside a section whose prose dominates the chunk median.
The union is not a compromise:
the predicate returns false both for "not verse"
and for "cannot tell",
so a true from either side is evidence and neither false
is evidence against.

The corpus-wide effect is a 3.7x widening of a prompt-shaping rule,
55 slices to
203 across 28 of 92 entries.
That is intended,
but it had never been measured,
and a `--only` run cannot see it.
`scratchpad/governed-widening.ts` recomputes it.

### Runs in flight

-   `translation-repair-runs-verse2`,
    `--only Toka_ls` at tip `080adcafa`.
    That
    tip carries chunk-only governance,
    NOT the union.
    For this entry the two
    agree,
    because pair 0's union adds nothing beyond its chunk verdict,
    so it
    remains valid evidence for `Toka_ls` and does not need restarting.
    It is not
    evidence about the union.
-   `translation-repair-runs-pass13` accumulation,
    restarted 19:41,
    working
    `MTF_0615`.
