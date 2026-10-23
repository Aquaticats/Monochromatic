# Audit ledger: the source text under a note and the page grammar

Findings of the second re-check of the round that made a page carry the original of an untranslated passage.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B832: the page could stop over an original the grammar refuses, and four texts said more than the code did

Status:
fixed on 2026-10-23 (UTC) in the worktree `tr-w317`,
except M5,
which is a flagged choice.
Reds `a024fdf1d0` (M3),
`4edf0025ab` (M4) and `dbcf9aed27` (M2,
M1,
N7).
Fixes `0763fa31b8` (M4,
the finding text),
`449bfb369d` (M2,
M3,
M1,
N1 to N4,
N7),
`93ecd78335` (M5,
the comment and a pin case),
`be1c689889` (N1,
`run`),
`2eea925d80` (M4,
the log and the comment,
N1 and N2 in `translate-lane`)
and `0257149051` (X6,
a test only).
The rulings of this entry are the lead's,
open to the owner's veto.

The red `dbcf9aed27` was wrong in two ways that its fix `449bfb369d` corrects:
it left the source-scan suite red (two top-level `describe` calls in one file),
and a case asserted that the warnings numbered exactly one where the composition's line is one of several;
the cases now filter the composition's own lines.

### M2: the grammar can refuse the original

`assertPageParses` stopped the entry with `UnparseablePageError` when an original held a bare `{`,
an unclosed `{`,
an unclosed `<details>`,
a lone `</details>` or an indented `{x} <y>` line.
`composeSourcePage` ships each run as recorded when the page parses,
and otherwise:

1.  writes,
    as literal text,
    only the runs that do not parse as a fragment of their own;
2.  writes every run as literal text;
3.  ships the page without the source text of the slices and adds the defect `page-source-unparseable`.

Each step taken writes one warn line naming the slices and the step.
Literal text is a backslash before each `{`,
`}` and `<` and nothing else;
`>` is not escaped,
because it is syntax only inside a tag and escaping it would turn a block quote of the original
into a line starting with the sign.
Fenced code,
a code span closed on its own line,
an HTML comment and an escape the original already wrote are copied
as they are.
Known limits kept in the module comment:
a code span that runs across lines and math between dollar signs are read as text,
so a brace in them shows its backslash.
The bytes of the original differ from the artifact's only on the encoding paths.

What each hostile case reached under the real grammar:
a bare `{` in a sentence,
an unclosed `{`,
an unclosed `<details>`,
a lone `</details>` and the indented `{x} <y>` line
all reach step 1;
the control that parses ships as written;
a page with one parsing run and one failing run reaches step 1 and encodes only the failing run's slice;
a `<details>` pair split across two slices of one run ships as written.
No text found with the real grammar reaches step 2 or 3
(300 `[` throws `MarkdownParseError` for the original alone,
in `droppedDestinations`).
Steps 2 and 3 are covered by cases that hand the composition a grammar of their own,
not by real-grammar text.

Known limit of step 3,
by design of the no-marker page:
the page differs from what the artifact says it ships,
so the page-agreement check reads it as disagreeing on a later republish
or `verify-published` against the stored artifact.
`withoutSourceRecord` makes the agreement check at publish time weigh the artifact without its record of the original,
and a page marker was not added.
The page-agreement check accepts the as-recorded or the literal form of a run.

### M1, M3, M4, N1 to N7, X6

- M1:
  the docs say that the page carries each slice's text as the artifact records it,
  joined by one blank line,
  and that text between slices is not carried;
  the test is renamed,
  and a case with a single newline between slices is added.
- M3:
  `shippableReplacements` defaults `includeSource` to `false`;
  the guards and the consistency pass cannot carry the original,
  and only `publishFixedPage` passes `true`.
- M4:
  the finding `source-passage-unfilled` and its TSDoc say the page carries the source text under a note
  unless the source recorded for the slice renders as nothing or the artifact records none,
  because the only caller is in `corpus-pass/src/corpus-run/pass-entry.ts`,
  which this round does not edit,
  so the reading cannot be passed in.
  The comment and the warn in `translate-lane/src/translate-slice-buy.ts` say the same.
- N1,
  N2,
  N3,
  N4:
  lines over 120 characters,
  article-opening `@param` and `@returns`,
  a ragged tail,
  an empty suite name and a double cast.
- N7:
  a guard case with English-looking words and straight quotes but no footnote definition shows that the
  original stays out of the spelling and quote passes too.
- X6:
  a test over the cat page with its last section cut from the archive shows that the consistency pass reads that
  slice as carrying no English,
  writes nothing there,
  leaves the refrain place there unlocated,
  and carries neither the original nor the note into the slices it rewrites (the pass wrote two,
  the positive control).
  It was not shown to fail,
  because making the pass read the original takes an edit to the fenced pass source.

### M5: flagged choice, not implemented

The ruling was that `standingUnendorsed` leaves out the slices that carry the
`ineligible-standing-replaced-by-incumbent` finding.
The artifact cannot support that:
`ArtifactConsolidateSlice` has no `findings` field,
the finding text is in no field of the XingZ6020 artifact,
and the only trace is the shipped kind `incumbent`,
which also marks the archive-dispute stand-in that the stage warns about.
The options:

- Add the finding to the consolidation record.
  Pros:
  exact.
  Cons:
  a change to what the generation writes,
  which is a generation question for the owner.
- Leave out every slice of shipped kind `incumbent`.
  Pros:
  needs no new field.
  Cons:
  the count on XingZ6020 would drop by up to the 6 slices of that kind and then contradict the stage's lines for
  the archive-dispute ones,
  which B831 matched.
- Reword the comment and keep the count.
  Pros:
  true to the artifact and no behaviour change.
  Cons:
  the count can exceed the lines.

Ranking:
reword the comment and keep the count > add the finding to the record > leave out the shipped kind,
because the first is true today and changes nothing a reader relies on,
the second is exact but moves a persisted shape on a decision that is the owner's,
and the third trades a documented count for a different wrong one.
This round took the first (`93ecd78335`) and pinned that an `incumbent` slice is counted.

Recurrence:
a comment that says a count "asks the same question the log did" is checked against what the artifact records.
A claim about a page is checked against what a reader of the page sees.

## Corrections to earlier entries

### A8

The header and the finding text of `publish-completeness.ts` say that the page carries the source text
only where the artifact records a source that renders as something,
and that a page the grammar refuses with the source on it ships without it (ledger B832).

### B827

- The text of B827 that says the passes read the page with `includeSource: false` is now the default
  of `shippableReplacements`,
  which moved to `page-check/src/corpus-run/shippable-replacements.ts`;
  only `publishFixedPage` passes `includeSource: true`.
  The consistency pass cannot carry the original,
  for two reasons that the new test in
  `corpus-pass/src/corpus-run/pass-consistency-source-ships.unit.test.ts` shows:
  the artifact it composes has no `lanes`,
  so it reads the slice as carrying no English,
  and the helper it shares with the guards leaves the source text out unless asked.
- The two citations for "a slice it leaves unfilled is written to no cache" are
  `slice-cache/src/corpus-run/entry-reattempt.ts` and `translate-lane/src/translate-slice-buy.ts` (lines 168 to 191
  at the fix of this follow-up),
  where the slice returns `unfilled` and the warn says it is not cached.
- "The source text follows as written" overstated what the page carries.
  The page carries each slice's text as the artifact records it,
  the slices of one run joined by one blank line,
  and no text between slices (an HTML comment the original holds between two blocks is not on the page).
  The docs now say so,
  and a test keeps each slice's text and the join.
- The page grammar can refuse the original (a bare `{`,
  an unclosed `{`,
  an unclosed `<details>`,
  a lone `</details>`,
  an indented `{x} <y>` line),
  and `assertPageParses` then stopped the whole entry.
  The ruling of this round:
  the page ships each run as recorded whenever it parses;
  otherwise it takes up to three steps,
  parsing after each (the runs that do not parse alone as literal text,
  then every run as literal text,
  then the page without the source text and the defect `page-source-unparseable`),
  with one warn line for each step taken,
  and the entry never stops on this path.
  The bytes of the original differ from the artifact's only on those paths.
  Measured against the real grammar,
  every hostile case found reaches the first step and none the second or the third;
  those two are covered by cases that hand the composition a grammar of their own.

### B830

The subject of the fix commit says "structure and section-pairing",
and the code logs every finding that begins `alignment ` (every kind the aligner writes) and every
`section-pairing` one,
so the subject names fewer kinds than the code logs.
The commit is not amended;
the corrective comment is in the round's notes.

### B831

The comment at the call said the stage writes the line only for a valid standing,
so the count asked the same question.
The artifact does not record that.
The stage's findings are not projected per slice (`describeConsolidateSlice`),
and a shipped kind `incumbent` is written both where an ineligible standing was replaced by the incumbent
(a different warn,
no "remains retryable" line) and where an archive dispute made the incumbent stand in over a valid
standing (the line is written).
So `standingValid` is `true` for every kept standing,
the count can exceed the lines by the first kind,
and the comment and the TSDoc say so.
On the XingZ6020 artifact the consolidation record holds 49 slices ending `slate-declined-standing/unchanged`,
6 ending `slate-declined-standing/incumbent`,
7 `consolidated` and 1 `gate-kept-standing/unchanged`
(a read-only script over `XingZ60.json`).
