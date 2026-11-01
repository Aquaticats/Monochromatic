# Audit ledger: loops that copy what they built

Folds,
maps and scans that copied or re-read everything built so far at every step.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B70: folds that copied what they had built at every step

Found by the sixteenth coverage batch,
which rewrote `scanNearest` (`pair-sections-steps.ts`) as a loop:
a `reduce` whose step returned its answers so far spread into a new list,
so each step cost as much as every step before it,
the quadratic shape rule ITR in `AGENTS.md` rules out.
Nothing read that shape:
`text-accumulators.unit.test.ts` sees only a `let` begun as text and grown in a loop.
A scratch probe (`fold-copies-probe.mjs`) over the 1,068 production files
read 159 `reduce` calls and flagged 16 folds and one loop,
with controls:
the pre-refactor `scanNearest` and a synthetic file of every shape were reported,
and a numeric sum and an appending fold were not.
The guard written from it found three more loops the probe's rule missed.

The guard's 19 findings sort four ways.
Seven copied a list that grows with the input,
among them `stripStubMarkersWithOrigins` (`archive-stub.ts`),
which copied every line kept so far at each line of a page
while its summary called the pass linear.
Four spliced text one edit at a time,
last first,
at offsets into the original:
patch operations,
italic titles,
a list's gaps,
and a URL's trailing marks shed one slice at a time.
Six are sequential or bounded and are named in the guard with why:
four whose each step reads what the step before wrote
(the archive block repair,
name glosses,
seeded errors,
the slice splice),
two folds over fixed tables,
the merged regions `grownSpans` checks each run against,
and `collectRounds`' next-round queue,
rebuilt once per round and bounded by `maxRetryRounds`.
A spread inside a record of fixed fields is not read,
since it costs the same at every step.

One splice was a defect in output,
not only in cost.
`italicizeSlice` (`archive-italic-title-restore.ts`) reads curly and straight quotes,
which can nest.
With a title quoted inside a quoted title,
the inner went in first,
its form read its two spaces as one,
and the outer then cut at its original end,
one unit past its closing quote,
taking the mark after it;
its findings also named the inner rewrite the outer had overwritten.

Fixed (`42d7a0f08` red guard,
`be0897f00` the growing lists,
`f033d19e8` red case for the nested titles,
`66b77b040` the splices):
each growing list is a loop that appends,
removing in place where a step removes;
the splices go through `spliceDisjointEdits` (`disjoint-splice.ts`),
which sorts by start,
copies each untouched run once,
and refuses two edits that share a unit or a start (`OverlappingEditsError`),
since each caller builds its edits apart;
the italic pass keeps a nested span only where it opens first
and names only the spans it writes;
and the URL trim steps back to its cut and slices once.
The whole unit suite at `66b77b040` prints 1,472 PASS lines and no FAIL line.
With the old copying fold put back in `respaceList`
the guard's package case failed
("expected [ …(9) ] to deeply equal [ …(8) ]"),
and with the helper's refusal disabled
its two refusal cases failed and nothing else;
both were restored.
The first whole-suite run of the splice change failed the message inventory alone,
since the new error class was unlisted;
it is marked and listed in the same commit.

Reading the italic pass for the splice left one finding open.
`protectedRanges` (`prose-ranges.ts`) reads emphasis as prose,
so a quoted title inside an emphasis span is rewritten in asterisks there too.
Parsed with `mdast-util-from-markdown` 2.0.3
(`italic-in-emphasis-probe.mjs`,
cat-themed inputs),
the result inside `*…*` or `_…_` is an emphasis node inside an emphasis node,
and inside `**…**` an emphasis node inside a strong one.
Inside emphasis the title then shows in the same italics as the words around it,
and the quotes that set it apart are gone
(an inference from the default styling:
the local site checkout holds only `data`,
so its stylesheet was not read).
B72 measured it over the pinned corpus and closed it,
with the archive side's reading of italics.

Recurrence:
a fold or loop over the input never copies what it has built;
it appends,
and several edits into one text are written in one pass from offsets into that text.
`fold-copies.unit.test.ts` fails on a new one,
and an exemption names why a fold's copies are bounded or are its meaning.

## B74: map entries rebuilt as copies at every repeat of their key

Found reading `readCapLog` (`corpus-run/cap-census-read.ts`) for B73.
Each completed stream spread its label's queue into a new list,
and each pairing set the queue to a `toSpliced` copy,
so a log's streams of one label cost the square of their count:
the shape B70 removed from folds,
moved into a map.
`fold-copies.unit.test.ts` read `reduce` callbacks and loops reassigning a binding as a copy of itself,
not a map entry set to a copy of what the map held,
so it could not see this.

The guard now also reads,
inside a loop,
a call `R.set(key, value)` whose value copies what `R.get` read,
directly or through a name bound to that read,
by the copying shapes it reads elsewhere,
and it reads a path through `??` or `||` from its left side.
Its fixture case gains a map entry spread into a new list,
one cut with `toSpliced` through a bound name,
one appended in place and one copied outside any loop;
the first two are found and the last two are not.
At `9e01f2650` its package case found five unnamed
(a probe built from the guard's own functions printed them):
`aggregateClaims`' `byComponent`,
`censusByGeneration`'s `byDigest`,
`indexProposers`' `index`,
`readCapLog`'s `waiting`
and `gatherRelabelCases`' `byRegion`,
each rebuilding a list held in a map entry at every repeat of its key.

Fixed (`9e01f2650` red,
`fde3a7298`,
then cases in `81ec51782`):
`aggregateClaims` groups with `Map.groupBy`,
which keeps member order within a group and the groups in the order their first members come;
the other four push onto the entry's list,
`readCapLog` splicing a paired stream out in place,
`indexProposers` copying a claim's first proposers once so a record's own list is never appended to,
and `gatherRelabelCases` gathering through a local type whose positions list is writable.

With each append disabled in one build,
the cases of `aggregate-claims`,
`attribution-report`,
`artifact-eligible` and `pool-generation` failed,
and those of `cap-census` and `probe-relabel-gathering` passed:
no case had two completed streams of one label waiting,
and none drew one region twice.
`81ec51782` adds both,
the second drawing the windowsill edit again at position 7,
the position `DAMAGED_CASES` records as the same edit drawn under a second accepted issue.
With the two appends disabled again,
both new cases failed
(the cap census read `[ 11, 'unpaired' ]`);
with the queue's splice removing nothing,
the cap census case read `[ 22, 22 ]`;
and with the guard's map check disabled,
its fixture case failed.
Each was restored,
and `git diff` printed nothing after.

Recurrence:
a loop that groups or queues by key appends to the entry's list,
or groups with `Map.groupBy`,
and removes in place;
it never sets an entry to a copy of what the entry held.
`fold-copies.unit.test.ts` fails on a new one.

## B137: a scan its summary called one linear pass read the text again at every address

Pinned in `855553c3b`,
changed in `4008eeb99`.
Found on 2026-10-05 (UTC) reading the MiMo trial's production diff.
`scanUrlRuns` (`corpus-run/dropped-destinations.ts`) found the next address through `nearestScheme`,
which searched for each of the two schemes apart with `indexOf` and took the nearer answer.
A scheme the text lacks,
or carries only far ahead,
was read for to the text's end at every run,
so the scan cost the square of the text,
while its TSDoc called it one linear pass.
`cited-reference-scan.ts` already scanned for the same two schemes in one pass,
by searching for the stem both share and testing each stem where it stood.
Measured by a delegated agent on its build of `80e129926`,
over a text repeating one `https://` address,
median of five calls a size with other work sharing the machine:
2,000 runs 34 ms,
4,000 runs 132 ms,
8,000 runs 543 ms,
16,000 runs 2,168 ms,
four times the time at every doubling
(spread at 4,000 runs 136 to 178 ms).

No page was harmed:
the scan's results were right,
and a page of the pinned corpus is far shorter than the texts measured.

The fix:
`scheme-start-scan.ts` holds the one search,
`nextSchemeStart`,
and both readers call it,
each keeping its own stop characters and trimming.
The same script then read 0.6,
1.2,
2.4 and 5.2 ms at those sizes.
Both readers' whole output is the same before and after over 30,079 texts
(79 written by hand for the shapes a scheme search can get wrong,
30,000 from a seeded mix of tokens):
the recorded outputs differ in no line,
and a control that lower-cases the text first differs in 9,784.
A text with no scheme and many bare `http` stems costs about twice what it did
(0.30 ms against 0.13 ms at 132,000 characters).
No test asserts a timing;
the cases pinning both readers were committed green in `855553c3b` and stay green.
The whole suite on `4008eeb99` printed 1584 [PASS] and 3 [FAIL] lines,
all three from one lookup-cache case and its suites (ledger B140),
the lint `Found 0 warnings and 0 errors.`
and the scans 35 [PASS] lines.

Open to the owner's veto,
both the agent's choices:
"no scheme from here" is the text's length,
where the two readers disagreed before,
so a caller that forgets the test reads an empty run,
which `scanUrlRuns`' guard and `withoutTrailingPunctuation` both refuse;
and a stem that opens no scheme is passed by one character,
so the step does not depend on the stem having no self-overlap.

Recurrence:
`mistake-prevention.md`,
"Loops that copy what they built".

## B429: the repeat finders grew with the square of their input

Red `cda53b4338`,
fixed in `566de2e511`.

`maximalRunsOf` extended every window group,
so a paragraph copied whole cost the square of its length
(8000 elements per copy took 2329 ms);
the refrain finder compared every pair of clauses sharing four leading elements;
and the line lookup sliced the page once per occurrence.

The fix:
a window group that is the previous one moved by one element is skipped
(the same copy takes 26 ms),
clauses are bucketed by both ends,
and line numbers come from one table of line starts.
The maximal runs of 20000 random streams and the groups of the 92 pinned pages are unchanged.
A page of clauses that nearly all share both ends is still quadratic,
as the TSDoc of `refrainFramesOf` says.
The largest pinned page takes tens of milliseconds.
