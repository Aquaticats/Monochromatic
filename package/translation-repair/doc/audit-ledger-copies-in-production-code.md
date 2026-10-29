# Audit ledger: copies of shared code in production

Readers,
predicates and helpers the production code kept in more than one copy,
each copy free to drift from the others.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1: private Han tests and code-point counters, and two Han tests that read one page two ways

Status:
fixed in `67243edae`,
`edb013f38`,
red guard `58d1eefaf` and `37db4ae4b`,
2026-09-28.
`page-name-glossary.ts`,
`corpus-name-index.ts` and `handle-reading.ts` each kept a private copy of the
U+4E00 to U+9FFF test `han-only-text.ts` exports,
and two of them a code-point counter beside `codePointCount`.
The floors' `isHanCharacter` read the unified block alone while the tokenizer's `isIdeograph` added Extension A;
neither read the compatibility block or astral Han.
`isHanCharacter` now covers Extension A,
the unified block,
the compatibility block and Extensions B to H,
and `isIdeograph` delegates to it.
No pinned page carries a character from the added blocks (Extension A,
compatibility and astral counts all 0,
against 210,475 unified-block characters),
so no page reads differently.

## B2: two quote-line readers

Status:
fixed in `43132653d`,
2026-09-28.
The line structure guard and the bilingual pair bound each kept a copy of what a line carries past its `>` markers.
`quote-line.ts` now holds `carriesContent` and `pastQuoteMarkers`,
with its own unit test.

## B3: three copies of whether a footnote relabel moves a note

Status:
fixed in `441e609ab`,
2026-09-28.
The relabel planner (`archive-footnote-relabel.ts`),
its closure (`archive-footnote-closure.ts`)
and the pass that reports the relabel (`pass-footnote-relabel.ts`) each tested whether a from-to relation
folds to another footnote.
`relabelsFootnote` in `footnote-identifier.ts` now serves all three,
with a unit test for respelling and moves.

## B4: two copies of the page assembly's archive-repeat filter

Status:
fixed in `bb836ea31`,
2026-09-28.
`guardPageAssembly` dropped rows repeating the archive's wording on its first read,
and each round of `settlePageRounds` dropped them again before the footnote check,
each with its own copy.
`rowsChangingArchive` in `page-assembly-rounds.ts` now serves both.
A mutant keeping every repeat fails 9 assertions across the guard and heading-collision tests;
its control survives.

## B5: two copies of the shipped-slice reading, one lane's call untested

Status:
fixed in `184b10cf9`,
2026-09-28.
Both lanes built the per-slice list the adjacent-repetition check reads with their own copy of `shippedFor`;
`shippedSliceTexts` in `assembly-invariant.ts` now serves both.
Mutation found the translate lane's call untested:
handing it no surviving rows,
so every slice read as the archive,
failed no test,
because only the repair lane's twin (`repair-assemble-slice-match.unit.test.ts`) drove the reading.
`translate-assemble-slice-match.unit.test.ts` now kills that mutant (2 failures)
and an inverted slice match (3 failures);
its control survives.

## B6: two HTML-comment finding tests beside a third inline

Status:
fixed in `c92394058`,
2026-09-28.
`archive-original-note.ts`,
`entry-notes.ts` and `footnote-protected-ranges.ts` each spelled out the two comment kinds;
`isCommentFinding` in `parse-document.ts`,
where the kinds are defined,
now serves all three,
with a test over every finding kind.

## B7: two readings of one event stream, and a data prefix spelled with its space

Status:
fixed in `91ff1e779` and `440eb0c0d`,
2026-09-28.
The live delta scanners (`stream-delta-scan.ts`,
`anthropic-delta-scan.ts`) read `data:` lines by the event-stream
format (one trailing carriage return dropped,
the field name at the line start,
one optional space removed),
while the three readers folding the drained body (`stream-completion.ts`,
`anthropic-completion.ts`,
`openrouter-chunk-scan.ts`) trimmed every surrounding space first.
A line the format does not count as an event (one indented before `data:`) reached the answer
while the runaway guards never saw it.
`ssePayloadOf` in `sse-data-line.ts` now serves all five;
mutants restoring either trim,
keeping the carriage return,
or stripping every space each fail 2 assertions.
A sixth reader,
`requireBedrockStreamEnd` (`bedrock-stream-end.ts`),
looked for its usage chunk under `data: `
with the space,
so a usage chunk sent as `data:{...}` never counted and a whole stream was refused as cut off;
it now reads through the same function,
and a test sending the tight form fails when the spaced prefix is restored.

## B8: five span-rewrite appliers under three contracts

Status:
fixed in `8f590d8e8`,
2026-09-28.
The name casing sorted its rewrites and dropped any overlapping an earlier one;
the Canadian forms sorted and trusted a comment that dates and spellings never overlap;
the pinyin tones applied rewrites in the order parentheses were read,
sorted or not;
the casing restore and the tag attribute restore relied on their callers' order.
An unsorted or overlapping set splices behind the cursor and repeats page text.
`applySpanRewrites` in `corpus-run/span-rewrites.ts` orders by start,
the longer first,
and withholds overlaps
for all five;
mutants dropping the sort,
the tie-break or the overlap rule each fail its test.
Nested parentheses in a pinyin probe produced no duplicate rewrite,
so no page is known to have hit the defect.

## B9: small shared helpers kept twice

Status:
fixed,
2026-09-28,
each with the tests naming its callers passing.

- `longestRunOf` (`character-run.ts`) for the Markdown and prompt fences,
  which counted runs of different characters
  with one body (`304ae283d`);
  its test counts an astral character by code point.
- `runEnd` and `runStart` moved to `corpus-run/text-runs.ts`,
  and the pinyin pass's private `scanEnd` copy removed
  (`c147db8cc`).
- `chunkLabel` exported from `chunk-document.ts` for `coverage-candidates.ts` and `prepare-section-round.ts`
  (`f96ade088`;
  the 46 test files touching sectioning pass).
- `sliceSizesOf` (`displacement-ratio.ts`) for the displacement probe and the window trial (`d80ebe034`).
- `describeBlocks` and `sameShape` from `translate-validate-blocks.ts` for the archive revision shape check,
  over the same `BlockShape` (`6a80392c6`;
  54 test files pass).
- `bothHalvesInserted` (`container-half-pairs.ts`) for lone-half withholding and insertion admission (`957a5ab3e`).
- `slicesInOrder` in `heading-collision-restore.ts` and a new `rowsChangedBy` (`assembly-page-text.ts`)
  for the casing and name gloss restores (`4006a55d4`).

## B10: probes and a benchmark grader that re-carved slices and claimed to match the pipeline

Status:
fixed in `87e42628a` and `2732a2c4e` (red guard `a00128524`),
2026-09-28.
Measured over the 92 pinned pairs (`~/temp/agent/audit-glossary-fix/probe-slicing-extra.mjs`),
aligning sections and subdividing them matches `prepareDocumentPair` in order on every pair
except the front-matter slice the preparation leads with,
so a re-carve's slice numbers run one behind the run's.
The relabel probes find their slice by text,
so no result moved;
they now take the preparation's slices.
The recall benchmark's `gradeSeedDetection` indexed issue records,
which carry the run's slice numbers,
into such a re-carve:
on any document with visible front matter (all 92 pinned pairs) every issue read the next slice,
so an accepted issue at a seed was scored undetected.
It now slices through `prepareDocumentPair` as `repairTranslation` does;
a test with front matter was red before and passes after,
and the older grader test builds its expected slices the same way.
The bench sample keeps its own carve:
its line-structure flags agree with the preparation's on all 1,259 comparable
slices (`bench-line-structure-parity.mjs`),
it leaves the front-matter slice out,
and its numbering is its own.
The slice census and the translate probe measure shapes and texts,
not run indices,
and stay.

## B11: correspondence-list checks kept three times, and a type declared twice

Status:
fixed in `010e36bc5`,
`84c748aba` and `74f56d2bf`,
2026-09-28.
`pair-blocks-wire.ts`,
`pair-sections-read.ts` and `corpus-run/slice-cache-store.ts` each tested
the integer `{ source, target }` list shape;
`isIndexPairList` and `isIndexPairingWire` (`index-pair-list.ts`)
now serve them,
with a test covering ten refused shapes.
`010e36bc5` declared its own `IndexPair` while `pair-agreement.ts` already exported the same shape,
and the package failed its type check at that commit;
`84c748aba` uses the existing type (M47).

## B12: four provider clients with identical limiter and JSON plumbing

Status:
fixed in `23ae677be`,
2026-09-28.
Bedrock,
Hyper,
OpenRouter and Synthetic each kept `limiterFor` and `chatJson`;
each `chatJson` forwarded four named request fields by hand,
so `otherThan` never reached a JSON exchange
(harmless today:
the cross-provider re-ask goes through `chatText`).
`perModelLimiter` and `chatJsonThrough`,
which forwards the whole request but its validator,
now serve all four;
the 25 client test files pass and a direct test pins the forwarding.

## B13: two renderings of prior failed corrections

Status:
fixed in `4830317ef`,
2026-09-28.
`renderPriorCorrections` (`refine-selection-context.ts`) now serves the refine context and the consolidation gate.
Prompt text,
so measured rather than read (`sheet-bytes.mjs`):
hashes of all 38 rendered-sheets fixture sheets
and of both sheets with two prior corrections are identical before and after;
a one-word change to the rendering,
as a positive control,
moved all three hashes.

## B14: artifact readers with private string-list and pair-list readers

Status:
fixed in `8c897d1cc` and `657a81dbe`,
2026-09-28.
`requireStringList` (`artifact-guard.ts`) replaces four private readers and one inline copy;
`requireIndexPairList` (`artifact-exact-guard.ts`) replaces the two section-pairing readers' whole-function copies.
Both readers read one artifact version,
so sharing keeps the freeze.
The 43 artifact test files pass.

## B15: frozen copies kept on purpose

Status:
kept,
annotated in `c0d0eb22a`,
2026-09-28.
`judgeTwoLaneSlice` beside `judgeSlice` (`artifact-two-lane-comparison.ts` already said why)
and `uniqueNaturalnessFindings` beside the live `uniqueFindings`:
each reader copy is an artifact version's rule,
recomputed on read and refused on disagreement,
so merging would remove the check.
Both naturalness sites now say so.

## B18: letter, digit and hex tests kept in many copies, and prose scanners that test ASCII letters only

Status:
fixed,
2026-09-28.
Letter tests were kept as named functions in eleven modules (`72bbe6500` merged them into `ascii-letters.ts`)
and inline in more (M48),
several named for Latin while testing ASCII only,
so an accented letter ends a word for every scanner that reads English prose with one.
The settled pages carry 4 such words in 17,492 (2 of 40 entries),
which is why nothing has visibly broken.
Done so far,
each with the same admitted characters as before unless stated:

- `9375057c5`:
  `latin-letters.ts` holds the Latin blocks and combining marks the glossary's form edges and
  the Canadian spelling pass each kept;
  the glossary edges now also admit Latin Extended Additional,
  of which only letters NFD keeps whole reach folded text,
  and none occurs in the pinned corpus or the settled pages
  (control U+00E4 found twice).
  The sheet hashes are unchanged,
  and mutants dropping the sign exclusion,
  the Additional block or the marks are each caught.
- `4197af843`:
  the inline letter-or-digit,
  digit and lower-case hex tests route through `ascii-letters.ts`;
  one of the digit tests (`image-reading-sense.ts`) was never called.
- `1c3286271`:
  eight source files opened with a blank line,
  each since its first commit.
- `8d88a08e1` and `f4aa85d9a`:
  the casing restore's `latinWords` opens a word on a Latin letter and continues
  through letters and combining marks,
  and takes the opening letter before its continue loop,
  which the mutation check showed could otherwise hang the pass (M50).
  Old against new (`~/temp/agent/audit-glossary-fix/letter-harness.mjs`):
  `latinWords` differs on exactly
  the 11 inputs the census names,
  `titleRuns` on 4.
  The page-assembly guard replayed old against new over 222 settled artifacts on their own slicing
  (`page-assembly-letter-replay.mjs`) differs on none;
  lowering the pass's `MIN_USES` makes it differ on one,
  so the replay can see this pass.
- `00a22f08e`:
  the name-casing pass's `midSentence` and glued-match check read Latin letters,
  digits and marks.
  A name after `café` now counts as mid-sentence,
  and an occurrence running on into `ō` or `ū` is another word;
  over the corpus `phraseOccurrences` drops one such false occurrence in each of 2 inputs,
  and no page changes.
- `218720cfe`:
  the address floor's word scan reads Latin letters;
  the pre-letter build refused a rendering
  addressing a cat named `Heřmánek` as if it said "he",
  and the current build accepts it.
  No finding changes over 131 source and target pairs;
  a control removing `you` makes 20 differ.
- `340b56ef5`:
  the suicide-method floor's word scan,
  the same;
  no finding changes,
  and a control breaking
  the suicide stem makes 21 differ.
- `3e28464fa`:
  those three scans were one scan in three copies;
  `latinWordSpans` and `lowerCaseLatinWords`
  in `latin-letters.ts` replace them,
  matching HEAD on every measure this entry records for the three.
  Mutants making the shared scan ASCII at the start or in the loop are caught by 6 assertions each.
- `bdc0112e6`:
  heading affinity's `latinTokens` come from the shared scan,
  folded by the new `foldLatinWord`
  (NFD,
  marks dropped,
  lower case),
  so a handle written with its accent composed,
  combining or left off
  scores against itself;
  `Kätzchen` no longer reads as `tzchen`.
  Token sets differ on the 13 inputs with such letters,
  and `prepareDocumentPair`'s slices on none of 92 pairs;
  with affinity disabled XingZ60's slicing moves,
  so the comparison sees this path.
- `ee99ee294`:
  the preservation gate's `contentTokens` run on Latin letters,
  digits and marks,
  and `properNouns`
  opens a name on any capital Latin letter (the new `isLatinCapital`),
  so a deleted `Émile` counts as a name lost.
  Token lists differ on 13 inputs and names on 9;
  `checkPreservation` replayed over 9,456 recorded repair regions
  (`preservation-replay.mjs`) changes no verdict,
  while raising the minimum name length flips 92.
- `a5ae23a8f`:
  the neutral-pronoun count no longer counts `TA` inside an accented word or before a combining mark.
  `sourcePronounLines` is unchanged on all 92 sources;
  counting occurrences inside words makes 31 differ.
- `23d2bea2a`:
  a missed quote's note counts accented words as one Latin token (diagnostic only).
- `fa0983503`:
  the picture-reading refusal screen reads accented words whole;
  the pre-letter build discarded a short reading of a `Noël` card as a refusal (`no` plus `l`),
  and the current build keeps it while real refusals still refuse.
  Word lists change on the 11 inputs with such letters;
  no recorded picture readings were replayed.
- `5f82122aa`:
  the quote and refusal tests add words opening on an accented letter,
  which closed mutants that tested only a word's first character against ASCII.
- `b4029fa3d`:
  content survival and the damage log read `foldedLatinWords`,
  so `château` is one specific of seven letters instead of `ch` and `teau`,
  both too short to count.
  Survival counts change on 6 of 131 archive-to-settled and archive-to-itself pairs,
  each by one accented word,
  and one settled page's lost list gains one such word.
- `435ba036f`:
  the lexical restoration grade (read by the recall benchmark only) folds accented words;
  word sets change on the 11 inputs with such letters and on no other.
- `c72cee0cf`:
  the line guard reads an accented-only line (`Å` beside its Han line) as the original's own English;
  the pre-letter build refuses that fixture's one-line rendering and the current build accepts it.
  `compareLineCounts` is unchanged on 131 pairs;
  forcing `isOwnEnglish` false makes 31 differ.
- `59e6f47c4`:
  the italic title restore reads a title opening on an accented capital (`Été des chats`).
  `archiveItalicSpans` is unchanged over 92 preparations;
  accepting any first character makes 18 differ.
- `94dca38d8` and `d107043be`:
  handle gloss placement reads word edges as Latin letters,
  digits and marks,
  where it had read any cased letter (so a kaomoji's Greek or Cyrillic letter joined a handle's word)
  and no combining mark (so a gloss could land between a letter and its accent).
  Page assembly replayed over 222 settled artifacts changes nothing;
  with the edge test always true,
  7 differ.
  The first commit's test never reached the path it named (M53);
  the second puts the accented word where
  the pre-letter build does insert the gloss before the accent.
- `d8a90d528`:
  a handle's reading keeps a space from an accented word beside it (`YumaoÉmile` no longer).
  `nameAuthorities` is unchanged over 92 preparations;
  tagging every romanised run changes one signer rendering.
- `81f2ce048`:
  a ballot finding naming a longer accented word (`tabbyé`,
  or a combining accent) blames no candidate;
  the wire reads model replies,
  so no corpus replay applies,
  and the test pins both spellings.
- `b44711b68`:
  block alignment tokens are folded Latin word and digit runs instead of ASCII runs by code point.
  Token sets change on 13 inputs,
  and `prepareDocumentPair`'s slices on none of 92 pairs;
  with tokens disabled the slicing of 10 pairs moves.
- `baed88f59`:
  tests written negated (`(character < '0') || (character > '9')`) route through `ascii-letters.ts`:
  four whole-text digit tests become `isAsciiDigits` and a lower-case letter test `isAsciiLowerLetter`,
  each admitting the same characters as before,
  the empty text included.
  The full suite after it:
  0 FAIL,
  1,303 PASS.
- `16bd8c142`:
  the apostrophe and nested quotation readings read the combining character sequence before a straight quote
  (`sequenceBefore` in `quote-neighbours.ts`),
  and `bindsWord` tests its base,
  so `café's` written with a combining acute curls as its composed spelling does,
  and no longer counts as an opening quote that kept a later trailing apostrophe straight.
  The whole sequence is kept for the other tests,
  so a mark on a space (a standalone accent) is still no space a quotation opens after;
  a first version that read the base alone curled that case,
  and the build before the change leaves it straight.
  No straight quote follows a mark in the sources or the archive's English;
  in 372,489 distinct artifact strings 5 do,
  each a variation selector on a heart in judge prose,
  where both readings agree.
  Page assembly replayed over 222 settled artifacts changes nothing;
  a control tree whose `bindsWord` never binds makes 62 differ.
  `restoreTypography` also runs inside the repair and refine stages,
  so the pre-launch cache check (M28) reads this commit.
- `a7b08bbfc`:
  the Canadian date and spelling passes read a word written with a combining accent whole.
  `isWordCharacter` (a cased letter or a combining mark) moved to `canadian-date-parts.ts`;
  `continuesWord` and every word run read it,
  the initial test counts letters with marks left out,
  and an underscore's or dot's far side reads a mark as the word it ends before the word,
  and as no word after it.
  Before the fix,
  `4 May` with a combining acute after it became `May 4` with the accent on the digit,
  and an initial,
  a heading's small word,
  a May compound and an identifier each read differently from their composed spellings.
  No combining mark occurs in the sources or the archive's English,
  and in run artifacts only 3,
  all in judge ballot reasons the passes never read
  (control:
  the same census over composed Latin letters finds 2,972),
  so no page changes.
  Mutants:
  8 of 9 caught;
  the month-run and range-word switches are equivalent
  (a mark at a month's end is refused by `continuesWord`,
  and a range word needs a space after it)
  and are kept so every word run reads whole words.

The package now tests for a combining mark in two ways,
on purpose:
`isCombiningMark` (`latin-letters.ts`) takes the Combining Diacritical Marks block,
U+0300 to U+036F,
the marks NFD splits off Latin letters,
for scanners that read Latin words;
`MARK_CODE_POINT` (`quote-neighbours.ts`) takes general category M,
variation selectors included,
because a quote's neighbour can be any character a mark sits on,
an emoji among them.

The census bounds every switch:
the characters an ASCII test and a Latin letter-or-mark test disagree on
occur in 2 of 92 sources (7 characters),
9 of 92 archive pages (20) and 2 of 40 settled pages (4).

The last copies were three `opensTag` tests,
one per scanner that reads markup:
- `7a46c8be8`:
  the typography mask took ASCII letters and `/`,
  the markup atom scan the same and `!`,
  and the prose ranges `/` and any cased letter.
  `mdx-tag-start.ts` now answers for all three as `micromark-extension-mdx-jsx` 3.0.2 does
  (`dev/lib/factory-tag.js`,
  `startAfter` and `nameBefore`):
  a space,
  tab or line end after `<` leaves it text,
  other whitespace is stepped over,
  and `/`,
  `>` or an identifier start (`$`,
  `_` or Unicode ID_Start,
  Han included) opens a tag;
  anything else fails the compile.
  The atom scan names `<!--` explicitly,
  which the corpus build rewrites into a JSX comment.
  Before the fix a tag named in Han,
  or opening on `_` or `$`,
  or after a no-break space,
  was prose to all three,
  so its quoted attributes were curled (which compiles nowhere) and respelled.
  The new test checks 19 probes against the package's own MDX parser.
  Over the sources,
  the archive's English,
  the settled pages and 372,489 artifact strings,
  what follows `<` is an ASCII letter,
  `/`,
  `<!--`,
  markdown whitespace,
  or `|`,
  `]` and `@`,
  on all of which every version agrees.
  Page assembly replayed over 222 settled artifacts against the tree before the change differs on none;
  a control tree whose test never opens a tag makes 38 differ and 96 fail the guard.

What remains tests letters in any script on purpose:
`isCasedLetter` in the Canadian passes
(a Cyrillic letter glued to a listed word makes a longer token),
their capital tests,
the quote neighbours' general categories,
and `declared-name-survival.ts`'s letters and numbers after NFC
(handles are written in Han).
A last shape census over non-test source,
asserted and negated forms,
character-code ranges,
case-fold comparisons and regex classes,
finds nothing else outside `ascii-letters.ts` and `latin-letters.ts`,
and no regex uses `\w` or `\b`,
which are ASCII-only in JavaScript
(control:
the same search finds the `\s` in `mdx-tag-start.ts`).

## B19: copies inside one file, which the cross-file census never showed, and no guard against a new copy

Status:
fixed,
2026-09-28.
The census behind B1 to B15 grouped bodies kept in two or more files,
so a body kept twice in one file never showed.
Widened to any two places,
it found three more beside B15's frozen pairs:
- `f72d96d18`:
  `rendering-audit-settled-repeat.ts` built a repeat pair twice (now `repeatPairOf`);
  `artifact-two-lane-read-evidence.ts` read each lane's slice texts twice (now `requireEvidenceRows`;
  both parsers read one artifact version,
  so they may share);
  and `slice-cache-store.ts` kept `isCachedPairing` and `isCachedSectionPairing`,
  one check under two type guards (now `isCachedPairingRecord`).
  Both copies of that check tested only that findings were a list,
  where both record types promise text,
  so a record with a non-text finding is no longer resumed.
  The pairing caches are exported for a new test (`slice-cache-pairing.unit.test.ts`),
  which round-trips a record through each and refuses a numeric finding,
  a non-index pair (`23ce27694`) and a bare list;
  mutants restoring the list-only finding check or dropping the pair check are each caught by 2 assertions,
  and the control survives.
- `4e79127ad`:
  `duplicate-bodies.unit.test.ts` parses every non-test source file with rolldown's parser
  (`parseSync` from `rolldown/utils`,
  as the repository's import-attributes plugin reads source;
  TypeScript 7 exposes no stable compiler API),
  compares each function body with its comments cut out and whitespace dropped,
  and fails on a body of 80 or more such characters kept in two places,
  except B15's frozen copies,
  listed with their reasons;
  a listed copy that no longer stands fails it too.
  Its fixtures first show the scan finds a copy across files whatever the comments and layout,
  and a copy inside one file,
  and leaves bodies differing in one name and short ones.
  A mutant restoring one merged copy and a mutant drifting a frozen copy are each caught;
  a comment control survives.
  `rolldown` joined the package's devDependencies from the catalog,
  and the lockfile gained that importer entry only.

The other cache readers (`slice-cache-store.ts`,
`consolidate-cache-store.ts`,
`lane-contest-cache-store.ts`,
`reading-cache-store.ts`) check that a record's fields are present and its lists are lists,
not every element.
That was read as deliberate rather than a defect:
each lane cache is opened with the built pipeline's digest as its generation
(`pass-entry-caches.ts`,
`pass-prepare.ts`,
`pass-consolidate.ts`),
and a namespace whose generation moves is deleted (`slice-cache-namespace.ts`),
so a file these readers accept was written by the same built code from values of these types;
the shape checks are the belt the module comment names beside that brace.
The lookup cache,
which outlives builds,
already checks every hit (`isLookupRecord`).

## B56: the missing-path check written out in three shapes, one reading the error's text

Found after the ninth coverage batch (census-VW2tHL),
reading the ten catches whose rethrow arm no test reached
(`page-republish.ts`,
`published-tree-listing.ts`,
`runs-lock.ts`,
`settled-carve.ts` twice,
`sheet-path.ts`,
`sheet-write.ts`,
`slice-cache-dir-read.ts`,
`window-trial-ledger.ts`
and `bedrock-ledger.ts`).
Every catch that reads "nothing stands at the path" as an ordinary answer wrote the check itself,
in three shapes:
`Error.isError && 'code' in && code === 'ENOENT'` twelve times inline and once as `settled-carve.ts`'s own `isMissingPath`,
`isJsonRecord(error) && code === 'ENOENT'` in `bedrock-ledger.ts` and `prompt-payload-store.ts`,
and in `image-ocr.ts` a substring,
`String(error).includes('ENOENT')`,
deciding that tesseract is not installed from the error's text,
which carries the tool's own output.
Two defects rode with the copies.
`sheet-write.ts`'s `exists` said in its comment that any other failure "is still an absence for this purpose"
while its code rethrew it,
the two written together in `ebc53f24f`.
And the substring:
a probe of Node's `execFile`
(`~/temp/agent/audit-glossary-fix/enoent-spawn-probe.mjs`)
shows an absent program rejects with code `ENOENT`
and a program exiting 3 or 1 with that number and no `ENOENT` in its text,
so the code decides the question and the text only happens to agree.
No real outcome read differently in the probe.

Fixed as a quality call open to veto (`9a02d8e0b`):
`missing-path-error.ts` holds `isMissingPathError`,
a Node error whose code is `ENOENT`,
and `rethrowUnlessMissingPath`,
which throws the caught value unchanged otherwise,
the narrowing-helper shape of `requireMdxRefusal` (`f792174a2`);
the catches read `rethrowUnlessMissingPath(...)` and then their "not there" answer,
so each rethrow arm is the helper's,
tested once over real `ENOENT` and `ENOTDIR` failures.
Sharing the check left `bedrock-ledger.ts` and `window-trial-ledger.ts` with one body word for word,
which the duplicate-body guard refused,
so both read through `readTextOrEmptyIfMissing` (`read-text-if-present.ts`),
and so does the cache marker read.
`sheet-write.ts`'s comment now says what its code does,
and `image-ocr.ts` reads the code.
`declined-entries.ts` still reads the code through `filesystemReason`,
since its refusal names the code.

Recurrence:
a catch that answers "not there" uses `rethrowUnlessMissingPath` or `isMissingPathError`;
a comment explaining what a catch does with the other failures is read against the arm it describes.

## B700: the pinned lint binary version said 0.2.0 while `mise.lock` installed 0.3.0

Status:
fixed on 2026-10-20 (UTC),
`e262b18601`.
The lock moved in `bca9a2856e`,
a `chore(mise)` commit of 2026-10-19 at 20:55 UTC that touched `mise.lock` alone.
It recurs an earlier move of the same constant,
`ea791a3840` of 2026-10-19 (0.1.2 to 0.2.0),
which no ledger entry records.

`PINNED_LINT_BINARY_VERSION` in `package/translation-repair/typography/src/semantic-wrap-binary-constants.ts`
is one of two writings of one fact:
the version of `monochromatic-lint` the shipped-text wrap is written against.
The other is the tool's entry in `mise.lock`,
since `mise.toml` asks for the latest release of a tool this repository publishes.
With the lock at 0.3.0 and the constant at 0.2.0,
every wrap of shipped text threw `LintBinaryVersionError` under `mise`:
the unit tests of page-check failed 13 cases on `214ddb2bdb`,
and a corpus pass on that build would have stopped at its first wrap.
It was first read as an environment fault in the check of the page-pass fix,
and a baseline run on `main` showed it was not.

Between the tags `monochromatic-lint-v0.2.0` and `monochromatic-lint-v0.3.0` the crate's source adds only the
`rust/avoidable-closure` rule and its closure step;
`markdown_semantic_breaks.rs` and the Markdown rule settings are unchanged.
The wrap the pin names writes the same bytes,
so no cached stage changes.
After the change the unit tests of typography (43 pass,
0 fail) and of page-check (59 pass,
0 fail) pass.

Recurrence:
the handover's rule that each `monochromatic-lint` release moves `mise.lock`,
and that `PINNED_LINT_BINARY_VERSION` follows in the same change.
A search of this audit's files for the constant and its error found no earlier entry;
the nearest mention,
a bare `node` resolving 0.1.1 on the path while the pin was 0.1.2,
is another cause (the program found on `PATH`,
not the lock).
The first move,
`ea791a3840`,
followed a merge with `main` whose lock installed 0.2.0 and failed 341 failure lines across the family suites
in gate g112.

A guard exists.
The agreement case in `semantic-wrap-binary.unit.test.ts`
(`400582c281`,
"HOLDS THE PINNED VERSION TO THE ONE mise.lock RECORDS")
reads the tool's entry in `mise.lock` and compares its version line with the constant,
so it fails whenever the two differ.
It is a unit test of typography,
and `bca9a2856e` changed no file of the package,
so nothing ran it when the lock moved;
whether it was run on `214ddb2bdb` is not recorded,
and it was not run for this entry.
No code was added for this entry.

Prevention:
`mistake-prevention-shared-code-loops-and-keyed-tables.md`,
"Copies of shared code".

## B980: the toolchain cutover broke the lock reader of B700 and the gate's lookup of the lint program

Status:
fixed on 2026-10-29 (UTC).
The reader of the lock is fixed in `dbfcb006d7`.
The gate setup is scratch tooling outside the repository,
so it has no commit;
the order that now prevents the break is in
`mistake-prevention-tasks-builds-and-bulk-output.md`,
"Gate trees and their tools".

Three breaks followed the move of the repository's tools to MPM.
Main deleted `mise.lock` in `4cb480344f` (2026-10-26),
and the merge `30d8d23218` of main into the translation-repair branch brought that deletion in on 2026-10-29 (UTC).

The first break is a second reader of the fact of B700.
The agreement case of `semantic-wrap-binary.unit.test.ts` read the tool's entry in `mise.lock`,
so it failed with `ENOENT` once the file was gone.
The message of `dbfcb006d7` names gate g221 as the gate that showed it.
The case now reads `tools["cargo:monochromatic-lint"].version` from `toolchain.lock.json`
and reaches the table and the entry through `isJsonRecord`,
so a lock whose table or entry is not an object fails the case by an assertion and not by a thrown error.

The second break is the size of the index.
The merge and the stamps entries took `audit-ledger.md` to 1009 lines,
past the bound of 1,000 that the line-limit scan holds it to.
`e6a586cd62` moved the process mistakes to `audit-ledger-process-mistakes.md`,
which the main index lists.

The third break is the gate's tool lookup.
Gate g219 on `30d8d23218` passed 3181 cases and failed 694,
and the source scans exited 1.
`confirmPinnedVersion` in `package/translation-repair/typography/src/semantic-wrap-binary.ts` runs
`monochromatic-lint` by name,
and on the gate's `PATH` that name resolved to the shim of `mise`,
which printed `mise ERROR No version is set for shim`.
The repository's tools now live in a directory of each worktree,
`.cache/toolchain-bin`,
which `mise run toolchain:sync` makes after the worktree's install.
The gate's `PATH` builder had dropped the entries of the main checkout,
and the fresh gate trees had never been synced,
so nothing on the path held the real program.

Round 8 met the same break.
Its own red cases in `tr-w357` were the same missing `monochromatic-lint`:
ten cases of the page-check unit tests failed with `LintBinaryMissingError`,
and three cases of the source scans failed with `spawn monochromatic-lint ENOENT`.
That tree predated the cutover and was never synced.
The report of the round recorded the lookup errors word for word and installed nothing.

The fix of the gate setup,
which is the order the prevention names:
the gate's `PATH` puts `<tree>/.cache/toolchain-bin`,
`<tree>/.cache/cli-git-bin` and `<tree>/node_modules/.bin` first,
and leaves out the directory of the `mise` shims.
A fresh tree runs `mise run prepare:pnpm:install` and then `mise run toolchain:sync` before a gate,
since the sync needs `node_modules`.
A subagent never runs `toolchain:sync`,
so the lead syncs a subagent's tree after the subagent merges main across a change to the toolchain.

Not verified here:
the counts of g219 and g221 are the lead's and no log of them is in the repository,
and the commit message of `dbfcb006d7` calls g221 the first gate after the merge
while g219 ran on the merge commit itself,
so which gate showed the `ENOENT` first is not settled by the repository.

Prevention:
`mistake-prevention-tasks-builds-and-bulk-output.md`,
"Gate trees and their tools",
and `mistake-prevention-shared-code-loops-and-keyed-tables.md`,
"Copies of shared code",
whose rule on the lock follows the move to `toolchain.lock.json`.

Recurrence:
B700,
which records the fact and its first reader,
and M278 and M280,
which record a build that ran another tree's tools and a tree tested on another commit's install.

## B1015: the lint binary pin said 0.3.0 while the lock recorded 0.4.0, and the refusals named dropped tools

Status:
fixed on 2026-10-29 (UTC) in `6253eb52b`.
`b750a8899f` (2026-10-29,
`chore(toolchain)`) moved the `"cargo:monochromatic-lint"` entry of `toolchain.lock.json`
to 0.4.0 when another session released the crate,
and it touched no file of typography.
The owner ruled on the same day through the question tool,
verbatim:
"Tell it to release 0.4 and move typography to 0.4".

This is the move of the constant that follows the two of B700 (0.1.2 to 0.2.0 and 0.2.0 to 0.3.0),
and the same fact in two writings.
With the lock at 0.4.0 and the constant at 0.3.0,
the lead's gate g230 failed one step,
`typography.test-unit` (exit 1);
the gate's whole-suite summary read 3271 passing and 15 failing,
counted in `[PASS]` and `[FAIL]` lines across every package,
not in cases,
and the failing lines are all typography's
(an earlier draft of this entry,
from the lead's own message,
called the 15 "cases" of typography alone,
which was wrong);
the step's log (`gate-g230/typography.test-unit.log`,
lines 34 to 37) shows the refusal
`LintBinaryVersionError: monochromatic-lint is version 0.4.0, but shipped text is wrapped by version 0.3.0 only`
from the cases that wrap through the real program.
A corpus pass would have stopped at its first wrap.
The agreement case that B980 moved onto `toolchain.lock.json` would have failed too,
and it is a unit test of typography,
which the commit that moved the lock did not touch,
so whether anything ran it then is not recorded.

Whether the wrap rule changed was read from the linter's history,
`monochromatic-lint-v0.3.0..monochromatic-lint-v0.4.0` over `package/linter/monochromatic-lint`,
and then measured.
No commit changes what `markdown/semantic-line-breaks` does:

- `71352ac59d` reflows comments in `markdown_semantic_breaks.rs`,
  the rule's only file,
  and changes no code.
- `2deb5bfa79` replaces `partition_point` with named `binary_search` folds in `markdown_positions.rs` and
  `rust_source.rs`,
  which the position map the rule reads goes through,
  and keeps the count each search returned.
- `c7adda99a5` moves the arena walk to `markdown_traversal.rs` unchanged,
  and `fd9df3030c` marks the `catch_unwind` closure in `markdown_source.rs` with a lint expectation.
- `44c650dae0`,
  `baf9ead5ac`,
  `79ff3e02e1`,
  `2fad6fd4cf` and `de947eb794` add the nine `shell/` rule ids and the reading of workflow `run:` steps
  (`.github/workflows` and composite actions),
  and `d34026bb22` and `d659a87347` change only the failure text for an unregistered rule name.
- `Cargo.lock` differs by the crate's own version line alone,
  and the Markdown parser crates stay at `satteri-pulldown-cmark` 0.6.3,
  `satteri-arena` 0.3.1 and `satteri-ast` 0.5.3.

The measurement ran 0.3.0 (the release artifact,
installed by `cargo binstall` into a scratch directory)
and 0.4.0 (the tree's `.cache/toolchain-bin/monochromatic-lint`,
a link into `~/.cargo/bin`)
over 8476 inputs,
one process at a time,
with typography's `semantic-wrap.config.jsonc`,
the file name `passage.md` and the arguments `semantic-wrap-binary.ts` passes:

- 21 passages of the typography tests,
  copied by hand from the inline literals of the three `semantic-wrap*` test files
  and `semantic-wrap-binary.unit.test.ts`,
  the 1.1 million character one included.
- The sources (`people/*/page.md`) and archives (`people/*/page.en.md`) of 92 pages at corpus commit `a41fc607ea`,
  read with `git show`:
  92 whole files and 2756 paragraphs of the sources,
  92 whole files and 2672 paragraphs of the archives,
  and,
  beyond the ask,
  the 92 files and 2751 paragraphs of `page.zh_hant.md`.
  A paragraph is a run of lines between blank lines outside a code fence,
  and 95 front matter paragraphs of each of the sources and archives are among them.

All 8476 gave the same standard output,
exit status and standard error under both:
status 0 and empty standard error throughout,
no difference in any of the 21 test passages,
5612 source and archive inputs,
or 2843 `page.zh_hant.md` inputs.
The comparator was shown able to find a difference first:
it flags one appended space and one changed status on a real reply,
and 0.3.0 itself changed its input in 15 of the 21 passages,
1586 of 2672 archive paragraphs,
91 of 92 archive files,
59 of 2756 source paragraphs and 32 of 92 source files,
so the wrap acts on this material and the comparison is not between two programs that do nothing.

The cache decision is that no cache version moves.
`wrapReplacementText` writes the bytes `monochromatic-lint` returns,
the bytes are the same under both versions on every input measured,
and `semantic-wrap.ts` wraps text on its way out of a cache rather than into it,
so a stored record holds the lane's text and a replay returns what it returned.
The cache-account audit (`mise run //package/translation-repair/slice-cache:cache-account-audit`,
run without `--runs-under`) lists `6253eb52b` among the source commits that no account names,
as it lists every commit of typography,
with all eight versions as the ones it rides inside;
it read no slice-cache record in this tree (`slice-cache records read: 0`),
so it says nothing about the caches of a launch,
and the decision rests on the byte comparison of the measurement.

The refusal text was written before the repository's tools moved to `toolchain.lock.json`.
`LintBinaryMissingError` and `LintBinaryVersionError` told the reader to run `mise install`,
named `mise.toml` as the place the tool is pinned,
and asked for a start under `mise exec --`;
`semantic-wrap-binary.ts` said `mise.toml` pins the tool.
They now advise `mise run toolchain:sync` (the tool is `"cargo:monochromatic-lint"` in `toolchain.lock.json`),
the `cargo binstall monochromatic-lint@0.4.0` line,
and a start under `mise run` or with the tree's `.cache/toolchain-bin` directory first on the `PATH`,
which is the form the root `mise.toml` and the gate trees of B980 use.
The cases in `semantic-wrap-binary-error.unit.test.ts` pin the new words
and that `mise install`,
`mise.toml` and `mise exec` no longer appear,
in the case of `LintBinaryMissingError` and,
after `8c6de0fbcc`,
in the case of `LintBinaryVersionError` too;
`6253eb52b` had pinned only the absence of `mise install` for the second.
`semantic-wrap-binary.unit.test.ts` pins `mise run toolchain:sync` and the `cargo binstall` line
in the missing-program case and `mise run toolchain:sync` in the wrong-version case.

`mise exec --` is not named as a remedy any more.
The first reason given for that in this entry's fix,
that nothing in the repository shows what it adds to the `PATH`,
was weak:
the root `mise.toml` `[env]` section does set the `PATH` for mise,
and the family's docs record probes run under `mise exec -- node`.
The reason that holds is in the file.
Its `_.path` list reads `".cache/search-shadow-bin"` and then `".cache/toolchain-bin"`,
so the tree's tool directory is second,
not first,
and the comment on the list says the second entry is there
"so tasks resolve the locked tools before any ambient or global one".
It speaks of tasks,
and no file read here says that `mise exec` applies the list,
so the advice names `mise run` and the directory on the `PATH`,
which the gate trees of B980 set directly.
A probe of the `PATH` under `mise exec --` would settle it and was not run,
since the task for this reading allowed a file read only.

Not verified here:
the release binaries were not rebuilt from the tags,
so that the released binary was built from the `Cargo.lock` read here rests on the measurement alone.
The inputs are the paragraphs and whole files of the pinned pages,
not the slices a lane produces,
and the hand-copied test passages stand for the inline literals as they stood on 2026-10-29.

Prevention:
`mistake-prevention-shared-code-loops-and-keyed-tables.md`,
"Copies of shared code",
whose rule on a tool version written in source names this entry beside B700.

Recurrence:
B700 and B980,
which record the constant,
its guard and the toolchain move this entry follows.
