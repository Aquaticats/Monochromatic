# Translation repair history 2026-08-12 to 2026-08-15: the naturalness eligibility filter and the verse rule wording

The naturalness lane refusing most wrapped prose because its filter treated every newline as a hard break,
the measured replacement rule,
and the verse sentence given to the editor that described the translation
instead of the original.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The naturalness lane was refusing four fifths of the prose it exists to read

Session of 2026-08-13,
late.
The question was whether anything needed fixing
 before the run kept accumulating.
Tallying every finding across the settled
 entries answered it:
`refine-skipped (0 eligible paragraphs)` had fired 175
 times.

### What the filter was doing

`refine-eligibility.ts` skipped any paragraph whose text contained a newline.
 Its own module header,
unchanged since it was written,
says an mdast `break`,
a
 soft source wrap,
and an HTML `<br>` are three different things and none of
 them means verse.
The code checked for `\n` and so treated all three as one.

Measured at the pinned corpus commit before touching anything:

```text
prose paragraphs                  2067
carrying an internal newline       811   39.2%
of those, carrying a hard break     29
soft-wrap only                     782
```

The rule was discarding 782 ordinary wrapped paragraphs to protect 29.
The run
 agreed:
`multi-line` was the largest single exclusion at 135 of 386
 paragraph-level skips,
and the lane actually ran on 35 chunks.

The replacement excludes a non-final line ending in two spaces or a backslash,
 the two Markdown spellings of an authored break.
`<br>` needs no new check,
 since `MARKUP_MARKERS` already rejects any paragraph containing `<`.
A trailing
 marker after the last line does not count,
because a break there separates the
 paragraph from what follows rather than dividing it.

This is strictly MORE precise,
not looser.
Every paragraph the old rule refused
 for a real hard break is still refused.
The verse risk in `#79` is not widened
 by it,
because verse is exactly what a hard break marks.

### What this cost, and the restart

Slice cache 20 to 21,
since version-20 slices were refined over a fraction of
 the prose the lane can now reach.
The pass was terminated and the supervisor
 resumed it,
which picks up the new source automatically because the pass runs
 from `src` rather than from a build.

### Two things this did NOT justify

The run did not need restarting for anything else.
Every commit between the tip
 the pass was running (`9cacc3f02`) and the eligibility fix touched only
 measurement,
reporting,
barrels,
or dead code,
and none changes what the
 pipeline produces for an entry.
Checking that before restarting is the reason
 the cache version stayed at 20 through all of them.

Voice loss is no longer a live problem:
2 entries of 21 carry one,
one model
 each.
The channel-marker fix held.

### Verified after the change, not predicted

The paragraph counts above are a property of the corpus and were measured before
 editing anything,
which makes them a reason to change the rule rather than
 evidence the change worked.
The after-state evidence is the shipped
 `selectRefinableParagraphs` run over 60 real corpus pages:

```text
eligible under the old newline rule   120
eligible now                          404
of the 404, cut by the old rule       284   70.3%
hard-break exclusions                  12
```

So the lane may now touch roughly 3.4 times the prose,
and the precise rule
 still refuses genuine authored breaks.
`multi-line` no longer appears as a
 reason at all.

## The verse rule described the wrong side of the pair

`#79`:
the editor replaced three correctly translated lines of `Toka_ls`,
a free
 verse entry,
with invented text,
one of them carrying a correct translation of
 a DIFFERENT line.
A computed predicate and an editor addendum had already
 landed for it.
Both were sound;
the sentence between them was not.

### What was already right

`isLineStructured` reads blank-line-separated blocks and fires when a slice has
 at least five of them at a median length of 30 or less.
It is deliberately
 computed rather than judged,
because an earlier heuristic attempt failed its
 positive control by ranking the one known verse entry 42nd of 54.

`buildEditorAddendum` applies it to the SOURCE and says why:
a translation may
 already have merged the lines that make the original verse,
so a predicate
 reading the target would never fire on the case the rule exists for.

Both claims were verified here against real corpus text through the shipped
 functions rather than taken from the comments:

```text
Toka_ls chunk 0   21 blocks  median  22   lineStructured=true
Toka_ls chunk 1    8 blocks  median  49   lineStructured=false
Toka_ls chunk 2    4 blocks  median  86   lineStructured=false
```

### What was wrong

The sentence handed to the editor opened `This region's CURRENT TEXT IS
 line-structured`.
Measured on the same chunk:

```text
Toka_ls chunk 0 SOURCE    21 blocks  median  22   lineStructured=true
Toka_ls chunk 0 CURRENT   18 blocks  median 101   lineStructured=false
```

So on the single case the rule exists for,
the editor was told something untrue
 about the text in front of it.
Worse,
the instruction continued `keep one
 output line per input line`,
and on a translation that has already merged its
 verse that asks for the merge to be preserved,
which is the opposite of the
 intent.

The rule now states what was measured,
that the ORIGINAL is line-structured,
and
 forbids what was actually observed rather than only the shape:
inventing a
 line,
dropping a line,
and filling one line with content belonging to another.
 A rule about line counts alone would have permitted all three fabrications.

### Corpus reach, re-measured

49 of 275 chunks across 31 entries,
against 55 of 286 across 34 recorded when
 the predicate landed.
The predicate did not change.
The ALIGNER did,
in `#71`,
 and chunk boundaries are its output,
so any figure counted in chunks has to be
 retaken after an aligner change.

### What is still not proven

That the corrected sentence changes what the editor does.
`Toka_ls` has still
 not settled in this pass,
so the direct evidence does not exist.
Nothing
 ENFORCES the rule either:
`isLineStructured` is read in exactly one place,
the
 addendum,
so line structure is requested of the model and never checked on its
 output.
A structural check at the apply gate would turn the request into a
 guarantee,
and it is the obvious next step if `Toka_ls` settles and still shows
 fabrication.
