# Lane outcome vocabulary handover: flagged-pass findings and fixes, 2026-08-18

The measured findings and landed fixes that followed the flagged pass on 2026-08-18:
relocation locality and line structure loss,
the semantic wrap and the bundling trap it exposed,
the translate lane deleting English and the quote guard that replaced the classifier,
and the reader that joins flagged slices to their replacements.

Part of [Handover: the lane outcome vocabulary and what is mid-flight](two-lane-outcome-vocabulary.md),
which lists every file of this record.

## TWO NEW MEASURED FINDINGS, BOTH COMMITTED

RELOCATION IS LOCAL.
Over all 92 probed entries,
1260 slices:
80 flagged,
51 contiguous runs,
LONGEST RUN THREE,
mean 1.57,
and every relocation pair adjacent.
`~/temp/agent/slide-shape.mjs`,
written up in `doc/audit/relocation-signature-at-flagged-slices.md`.
This bounds the `#107` fix:
a judge shown n-1,
n,
n+1 would have the moved passage in view in every case measured.

THE DAMAGE IS VISIBLE IN A FINISHED DOCUMENT.
The repair lane's `lintong` output offers the same
thing twice within four lines and repeats an invitation the paragraphs above already made:
each of
the two distinguishing noun phrases occurs twice in the repair document and once in the translate
one.
The repair lane edits the incumbent in place,
so a one-sentence original against a
four-sentence incumbent leaves three sentences behind that slice 2 then supplies again.
The
translate lane,
which writes each slice from its source,
is clean here.

LINE STRUCTURE LOSS,
`doc/audit/line-structure-loss-when-a-replacement-ships.md`.
Of 64 shipped
replacements with an incumbent:
33 lose soft wrapping,
17 flatten to a single line,
9 lose a
blank-line paragraph break,
2 are a rewrap and nothing else,
0 are byte-identical.
The repository
already owns the fix:
`package/cli/markdown-lint`'s `semantic-line-breaks` rule is `fixable: true`
and ADD-ONLY,
so it cannot destroy a break the model got right.
Tracked as `#122`.

## THE SEMANTIC WRAP LANDED, AND IT EXPOSED A BUNDLING TRAP (2026-08-18 evening)

`#122` code is in.
Four commits:
`d8d3b933f` the wrap,
`34f4320a8` the satteri external fix,
`59384774c` the tests,
`3c2b945e1` the lint warnings.
Full suite exit 0,
types clean,
`lint:oxlint` 0 warnings 0 errors,
15 new cases.

`src/semantic-wrap.ts` calls `fixSource` with exactly the `semantic-line-breaks` rule.
`src/repair-wrap.ts` and `src/translate-wrap.ts` apply it at the top of `assembleRepair` and
`assembleTranslation`,
the one point per lane where both the replacements and the lane wordings
read from the same list,
because the delivery invariant demands those two agree byte for byte.
Only text a lane PRODUCED is wrapped;
`changed` is re-derived so a wrapping-only difference is
recorded as the retention it is.

MEASURED BEFORE WIRING,
zero quota,
over all 64 shipped passages:
findings 326 to 0,
no passage
loses a non-newline character,
ten gain continuation prefixes,
second application changes nothing.
GFP shown both ways and restored.

STILL OWED:
re-run `~/temp/agent/wrap-census.mjs` over a pool settled AFTER this landed.
Needs a
fresh pass,
so it rides the next one.

`#447` IS THE TRAP.
Importing markdown-lint bundles it,
because the rolldown config always bundles
`@monochromatic-dev/**` and keeps only DECLARED dependencies external.
That inlined Sätteri's napi
loader into `dist/final/node/`,
where its binding cannot be found,
so every import of
translation-repair threw before reaching pipeline code.
Declaring `satteri: catalog:` in the
consumer keeps it external.
`packageExtensions` does NOT apply:
nothing is missing from Sätteri's
own location,
the code was copied out of it by the bundler.
The user asked for the writeup to live
in the issue rather than in `doc/troubleshooting/`,
so `de0aa4676` added a doc and `471966830`
removed it again;
the evidence is all in `#447`,
which asks for the doc and then a migration.

## THE TRANSLATE LANE WAS DELETING ENGLISH, AND THE FIX CHANGED SHAPE (2026-08-18 evening)

Found from the grader's item 8.
Measured:
`dogesir_/translate` 3493 to 2042 characters,
`wangzihao980/translate` 1952 to 1091.
The repair lane retained the same passages.
`doc/audit/deleting-english-the-chinese-never-said.md`.

THE RATIO GUARD DOES NOT REACH IT:
`MAX_INCUMBENT_TO_SOURCE_RATIO` is 16,
the cases measure
15.49 and 8.71.

CLASSIFYING TARGET-ONLY CONTENT IS UNSOUND.
`splitTargetOnlyRun` reaches 2 of the 9 known
transcripts and fails three ways:
transcript before the shared markup,
several per document with
only the last reachable,
one merged into a paired block.
Deeper,
"has no source counterpart"
cannot be decided by matching bytes,
since translation changes bytes by construction;
a
corpus-wide exact-match scan returns 234 hits that are almost all ordinary translated quotations.

DELETION IS DECIDABLE,
and that is what shipped as `refused-quote-loss`:
a replacement may not
leave fewer blockquote BLOCKS than the archive.
4 of 64 flagged rows,
0 of 69 natural rows,
reproduced through the built artifact.
Counter-case:
the repair lane's correct removal of an
invented paragraph was prose,
so the guard stays quiet on it.

ONE LANE ALREADY KNEW.
On `wangzihao980` the repair lane's critics rejected the claim that the
transcript was an improper addition 5 votes to 1;
elsewhere it recorded a translator's comment
`needs-human` rather than acting.
The translate lane deleted both with no flag,
because a passage
with no source span is invisible to a stage reading only source spans.

CRLF BUG FOUND:
one of 184 corpus markdown files uses CRLF throughout,
where a `\n\n` splitter
reads the whole document as one block and counts zero quotes.
Shared splitter now folds first.

BOTH FAIL-SHOWN-FIRST CHECKS DONE and restored,
for the quote guard and for the archive window.

LANDED:
32caa4f02,
c03847232,
7da2825f5,
6e6bc863a (window on,
both sides),
4d041313c (house
policy:
name what a passage points at,
drop dead punctuation),
79c8120c0 (`readsImages`),
25fd63639 (quote guard),
1a3325ca3 (audit).

STILL OWED ON `#111` B:
image path resolution,
the wire shape for an image part,
and the stated
rule for "the OCR does not make sense" written down BEFORE it is implemented.

## The `#107` join reader is built, validated, and waiting

`~/temp/agent/join-107.mjs`.
Outside the repository,
no build,
no quota,
safe while the pass runs.

  node ~/temp/agent/join-107.mjs --archive ~/translation-repair-runs-flagged-20260818/artifacts

THE INDEX SPACE WAS CHECKED,
not assumed,
because `#99` established that `chunkIndex` means three
different things depending on who stamped it.
Over all six settled entries the probe's `sliceCount`
equals the artifact's delivery row count in BOTH lanes,
and delivery indices run contiguously from
zero.
The reader re-checks per entry and THROWS on a mismatch rather than joining silently.

VALIDATED against three already-recorded figures:
translate 37 of 50 at 0.7400,
repair 32 of 50 at
0.6400,
69 shipped pooled.
POSITIVE CONTROL passes,
per `QPC`:
with `--control shipped` it reports
1.0000 flagged against 0.0000 unflagged,
so it is shown able to display a difference before being
trusted to report the absence of one.
An empty flagged bucket prints `NO SLICES IN THIS BUCKET`
rather than a rate of zero.

Flag lists for the five entries,
union of every flag kind,
16 flagged slices against 26 unflagged:
lintong 2 3 4;
wangzihao980 3 4;
GLaDOSister 2 3 4;
dogesir_ 2 3 8 9;
saurikissa 3 4 8 9.
