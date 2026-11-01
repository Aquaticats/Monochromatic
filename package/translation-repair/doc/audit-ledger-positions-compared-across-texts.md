# Audit ledger: positions compared across texts

Offsets,
headings and slices of one text applied to another that does not line up with it,
a leading byte order mark among the causes.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B83: a heading restore that shipped a section the page never replaced as empty text

Found by T8's twentieth batch,
red in `66927af39`,
fixed in `339ce9118`.
`restoreCollidingHeadings` (`corpus-run/heading-collision-restore.ts`,
class forty-five)
paired the page's headings with the original's and the archive's by position across the whole page,
and checked only that the three totals agreed.
A replaced slice that drops its heading and another that gains one leave the totals equal,
while every heading between them sits one position off.
A slice the page never replaced then had its own heading compared with its neighbour's archive heading;
having no replacement,
its text fell back to the empty string,
and the restored row rides the override a trimmed slice does (`page-assembly-guard.ts`),
so the page would have carried that section as nothing,
with a finding claiming a restoration.
The probe through `guardPageAssembly`
(`b20-shift-probe.mjs` in the audit's scratch folder,
cat-themed)
printed the empty row before the fix and none after.

The restore now pairs by position only where every slice carries as many headings as its archive text;
anywhere else the page is left for the publish guard.
With every slice's count kept,
a slice the page leaves alone carries the archive's own headings at the archive's positions,
so it cannot collide,
and the fallbacks that guarded its absence throw (`nonNullishOrThrow`).

The pass also kept its own replacement-only map of page text,
apart from the contract `assembly-page-text.ts` states for every pass.
`a6edb4665` moves it onto `pageTextBySlice` and `withRewrittenText`,
and adds `pageTextOf`,
one reader of a slice's page text that refuses a slice the map was not built from
(`SliceNotOnPageError`,
`4c0039a57`,
naming the slice)
in place of eight reads in six modules,
six of which took a missing slice for empty text
and two of which already threw without naming the slice;
`70c8c8918` moves the last pass onto it.
No such read was reachable,
since each pass reads only the slices its map was built from.
The message of `a6edb4665` miscounted those reads as seven `?? ''` fallbacks,
and a commit comment on it corrects the count to eight reads in six modules.

Guard shown to fail:
with the per-slice count check disabled,
the case failed alone in its file.
With `pageTextOf`'s refusal disabled,
its own case failed alone in its file
and the B83 case still passed,
since the count check keeps the restore from asking.

## B84: heading passes that paired a slice's headings by position without checking the count

Found by T8's twentieth batch while reading B83's family:
the same pairing inside one slice.
Red in `26e24853e` and `3c35e130b`,
fixed in `57920abed` and `5f059feff`.

- `unifyHeadingSeries` (`corpus-run/heading-series-unify.ts`,
  class sixty-eight)
  numbered a slice's page headings from the original's numbered headings by position.
  A page slice that dropped its first heading gave the third section the second's number:
  `### Part Three: Snowy` shipped as `### Part Two: Snowy`.
  Reading the archive's style the same way,
  an archive slice that opens with an interlude heading of its own had that heading read as the series' third,
  so the archive seemed to use two styles and its own "Part" style was lost.
- `restoreContributorNames` (`corpus-run/contributor-name-restore.ts`,
  class sixty-seven)
  paired a slice's page headings with the original's titles by position,
  though it already refused to pair signatures where the counts differed.
  A page slice that dropped its first heading had the signer's name written over the next section's title:
  `### Four: Doggo` shipped as `### Four: Maomao`.

Each slice now joins the pairing only where it carries as many headings as the original's slice,
and is left as it stands otherwise.
`b20-series-probe.mjs` and `b20-contrib-probe.mjs` in the audit's scratch folder
printed the wrong rewrites before the fixes and none after.
The remaining positional pairings in the page passes already check their counts
(`contributor-name-authorities.ts` for signatures,
`archive-destination-restore.ts` for links,
`jsx-attribute-restore.ts` for tags,
`title-reference-unify.ts` and `heading-distinctness.ts` for headings),
by a search for position-indexed reads over `corpus-run/`.

Guards shown to fail:
with the page-side count check disabled,
the dropped-heading series case failed alone in its file;
with the archive-side check disabled,
the interlude case failed,
and four older cases threw,
since an archive slice that never headed its section now takes the same count check;
with the contributor heading check forced true,
its case failed alone in its file.

Measured while here:
`contributor-name-restore.ts` reads ATX headings only,
so a signer named in an HTML heading would never take the signer's rendering.
Over the 92 pinned originals
(`b20-html-signer-probe.mjs` in the audit's scratch folder,
ids and counts only),
13 HTML heading lines stand in 3 entries and none names one of its page's signers,
while 9 ATX headings do (all in XingZ60,
the positive control),
so the ATX-only reading misses nothing at the pin.

Calls made here are open to veto:

- a slice whose heading count moved is left as the page has it rather than paired by some other rule;
- the page-text reader throws a named refusal rather than reading a missing slice as empty text;
- `contributor-name-restore.ts` keeps reading ATX headings only,
  on the measurement.

Recurrence:
pairing items by position is sound only where both sides carry the same count at the level the positions are read;
`mistake-prevention.md`,
"Pairing by position".

## B85: an advancing withdrawal that compared refusal offsets on two different pages

Found by T8's twenty-first batch while reading the uncalled reducer in `assembly-structural-withdrawal.ts`,
red in `9c007f857`,
fixed in `a70c074a3`.
`advancingStructuralWithdrawal`
(class fifty-eight,
reached when no single withdrawal repairs the page)
read where the strict grammar first stops on the page as assembled,
then where it stops on the page with one replacement withdrawn,
and called the withdrawal advancing when the second offset was the larger.
The two offsets index two texts:
a withdrawal before the break changes the length of the text ahead of it,
so the same break sits at another offset without having moved.
A whole replacement 40 characters shorter than its archive text,
before two broken sections,
read as moving the first break 40 characters later,
ranked above the withdrawal that removed that break,
and the guard went on to withdraw all three replacements
(`b21-advance-red-probe.mjs` in the audit's scratch folder,
cat-themed).
The reducer that picks the furthest step had never run,
since every case gave one candidate;
this fixture gave two only through the defect.

`assembledOffsetOf` now maps a refusal on the withdrawn page back to the assembled one.
The two pages differ only in the withdrawn slice,
so a refusal in the shared text before it keeps its offset,
one in the shared text after it moves by the difference in length,
and one inside the slice's own archive text stands where the slice begins.
A withdrawal after which the grammar accepts the page is marked `cleared`
and outranks any that only moves the refusal,
since a grammar that stops at a page's very end
(an expression left open)
leaves the moved offset no later than the page's length.
The guard's finding names such a step as clearing the refusal.
The arms the mapping added are cased in `c17feba5b` and `9244d67d3`.

Guard shown to fail:
with the raw offset compared again,
both B85 cases failed,
and so did the direct B86 left-open case and the nothing-advances case,
each comparing offsets across a length change;
the B86 guard-level case passed.

The reducer still runs in no case.
After the fix,
two withdrawals both advance only where the parser carries state across slices,
so that one slice's text decides how another's is read
(an inference from the mapping:
a withdrawal elsewhere leaves the text around the break as it was).
No fixture built in this batch reached two candidates;
the reducer is kept,
reachable in principle by that route.

## B86: a strict refusal read at the start of what the parser left behind

Found by checking what B85's fix compares,
before it was written,
red in `3086d2ffe`,
fixed in `0e7353948`.
`strictRefusalOffset` names where the strict grammar stopped,
and `advancingStructuralWithdrawal` is its one reader.
It is also the one reader of `MdxParseError`'s `line` and `column`:
of the six other production sites holding a refusal
(`rg 'requireMdxRefusal' src`,
tests and fixtures excluded),
three read only its message,
which `mdxRefusalSite` writes from the parser's own place name and the fix leaves as it was,
one passes it on as a cause,
and two only narrow it.
`mdast-util-mdx-jsx` raises its refusals on leaving the span it names
(`onErrorRightIsTag` and `exitMdxJsxTag` in `lib/index.js`,
3.2.0):
an element left open in a paragraph is refused at the paragraph's end,
and one still open when the document ends is refused with no place at all.
`refusalPlace` read the message's `line` and `column`,
which a `VFileMessage` copies from the span's start,
and `strictRefusalOffset` read a refusal with no place as the start of the body.
On a page whose open element sat beside a broken expression,
the parser named the expression first;
withdrawing it left the open element,
read as offset 0,
so no withdrawal advanced and the guard withdrew every replacement
(`b21-b86-probe.mjs`,
printed `assembly-withdrew-every-replacement` on `a70c074a3`).

`stopPointOf` now reads the end of the span a message names,
or the point it names;
a refusal with no place reads as the end of the body.
Micromark's own refusals name a point,
and its columns count UTF-16 units as the offsets here do
(`b21-place-probe.mjs`:
a break after an emoji refuses at the offset its index gives).

Guards shown to fail:
with the position-less refusal read at the body's start again,
the document-end case and both left-open withdrawal cases failed;
with the span read at its start again,
the paragraph-end case failed alone.

Calls made here are open to veto:

- a refusal is placed where the parser stopped,
  the end of the span it names,
  not where the span begins;
- a withdrawal after which the page parses outranks any that moves the refusal,
  however far.

Recurrence:
an offset means something only in the text it indexes,
and a parser's position is where its message says it stopped;
`mistake-prevention.md`,
"Offsets and positions".

## B257: the parse counted a leading byte order mark as nothing, so every later offset was short

Red in `5abbb9c66`,
fixed in `68df23b05`.

Found on 2026-10-05 (UTC) by an agent's probe of the parse entries.
`micromark` skips a `U+FEFF` that opens its input without counting it,
so every position of the tree sat one character short of the body as written,
and a column on the first line one short as well.
A paragraph after the mark read as text starting with the mark and ending one character before its end:
the envelope's `baseText` lost its last character,
a container's tags and a footnote reference were placed one too early,
and `strictRefusalOffset` named an offset one short.
The mark also opens a body after front matter.

The fix:
`parseMdxBody` and `parseMarkdownBody`,
where the offsets are made,
count the dropped mark into every position:
every node and JSX attribute moves one offset,
and one column on the first line;
the root keeps its start.
`MdxParseError` carries the dropped columns,
so a first-line refusal names its column in the body as written.
The red cases are in the `parse-mdx`,
`parse-document` and `strict-refusal-offset` test files.

The agent's measurement,
not rerun by the lead:
109 positions of two bodies under both grammars,
compared with the mark-free parse shifted by one,
0 wrong.
Measured by the lead on the pinned corpus,
by bytes and with a control on the same test (M128):
none of the 276 Markdown files under `people` opens with a byte order mark;
one archive,
`people/Toka_ls/page.en.md`,
holds three lines that are a lone mark,
which `maskInvisibleLines` blanks before the page is parsed.
Whether any slice of that page is parsed starting at such a line was not measured,
so no real page is known to exercise the leading-mark path.

For B257 to B262 together,
measured by the lead:
136 named test files pass,
with the 37 source scans and a clean type check,
and the slice census over the pinned corpus prints the same sixteen summary lines as before the change
(92 complete pairs,
1 incomplete,
1,259 slices).
The whole suite ran on `68df23b05` in a side checkout:
1,633 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 37 source scans passing.

Open:
a byte order mark before the opening front matter fence leaves the document without front matter
(`front-matter.ts`,
not changed here).

Open to the owner's veto:
a block starts after a leading mark and never holds it.

Recurrence:
`mistake-prevention.md`,
"Offsets and positions":
a parser that drops input without counting it moves every offset it reports,
so the shift is undone where the offsets are made and not in each reader.

## B273: a replacement for a slice the page lacks was taken with the archive read as empty text

Red in `96dad6b2c`,
fixed in `5723cd071`.

Found by the coverage cluster of B272.
`restoreNameGlossLines` (`name-gloss-restore.ts`) read the archive of a slice the pairs lack as empty text
and passed the row on.

The fix:
it reads through `pageTextOf`,
which throws `SliceNotOnPageError`,
the refusal the other page passes give.
The page text of a slice in `quote-style-unify.ts` is read through `pageTextOf` too,
over a map built from the same slices.
On the lead's red tree the case failed with `Expected act to throw, but it returned`.

Open to the owner's veto:
the refusal,
where passing the row on is the alternative.

Recurrence:
`mistake-prevention.md`,
"Pairing by position".

## B324: a page opening with a byte order mark was read as having no front matter

Red in `75ea14bff`,
fixed in `3ce947dce`.

Found on 2026-10-06 (UTC),
a residual the report behind B257 to B262 left,
measured by the agent of this fix.
`splitFrontMatter` looked for the opening fence at offset zero,
so a page whose first character is U+FEFF read its YAML as body:
a thematic break and a setext heading,
translated,
judged and counted in slices.
With a mark on the archive only,
the metadata slice was an insertion at offset zero,
so a rewrite wrote the new front matter before the mark
and left the archive's own in the body.
By the agent's census,
the pinned corpus holds no page opening with a mark (0 of 184),
so the defect is unreached by it.

The fix:
the fence is read after a leading mark (`leadingMarkWidth`,
`front-matter.ts`);
`FrontMatterBlock.raw` excludes the mark,
and the new required `startOffset` says where the fence stands;
every offset stays an offset into the text as written;
the metadata slice's spans start at each block's own fence,
and an insertion lands after the archive's mark (`targetInsertionOffset`);
a boundary after nothing but the mark is the start of the document;
and the archive intake (`corpus-run/pass-archive.ts`) keeps a leading mark and still folds every other.
The lead wrote the red commit's nine marks in the front matter cases as escapes,
where the agent's file held the character itself (M135).
The whole suite on `3ce947dce`,
run in a side checkout:
2,012 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 44 source scans passing.

Open to the owner's veto:
the ruling itself,
against the site:
`markdown-yaml-metadata-parser` 3.0.0 opens front matter only at `^---`,
and `readFileSync` keeps the mark,
so the site reads such a page as having none;
the alternative is to start at zero in `splitFrontMatter`,
everything else staying correct under both readings.
The intake keeping a leading mark while folding the others
(the alternative is to exempt U+FEFF from the fold);
and the mark outside the metadata slice,
so no lane rewrites it.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a rule about where a document starts is asked of one reader every consumer shares,
and a test pairs a page with and without the mark.

## B695: a heading series the archive heads by name alone lost the original's numbers

Status:
fixed on 2026-10-20 (UTC),
red `e3fcc3094f`,
fix `bc645360f6`,
in the integrated tree at `191a8bc38d`.
Neither page-pass nor page-check declares a cache version
(a search of their sources for `CACHE_VERSION` finds none),
so no number moves for it;
the fix commit lists the stored page assembly findings and page text as the results it alters.

Found in the read of XingZ6015,
the third entry of the readiness rotation.
The original heads its sections 其一 to 其十.
Both lanes rendered them "Part One:
Irako" to "Part Ten:",
the lane contest kept the numbers,
and the repair panel logged the archive's bare "Irako" as an accuracy omission of minor weight.
Page assembly then wrote nine `heading-series-unified` lines at 2026-10-20T00:05:26.988Z
(slices 9,
10,
49,
66,
71,
76,
78,
84 and 103,
each ending "in the style of none,
the archive's own style"),
and the shipped page lost every number the original gives.
Slice 48 drew no line,
because its page text already carried the bare archive heading.

The rule that did it is class eighty-nine's of 2026-09-23:
the archive's style wins wherever the archive heads the series.
An archive that heads its sections by name alone has no numbering style,
so the rule read "no style" as a style and removed the number from the heads of both lanes.
The pass also picked among lane renderings by plurality (`majorityStyle`),
so one original and one archive could ship different series from different lane output.

`unifyHeadingSeries` now writes a number on every heading of the original's series,
in one style that depends on the original and the archive alone:
the archive's style where the archive numbers the series in one style,
else the house style "Part One:"
(`HOUSE_SERIES_STYLE`).
The house style is the one 14 of the 18 numbered lane headings of XingZ6015 took,
and the one XingZ622 shipped;
no human archive in the corpus numbers a heading series,
so none offers a precedent.
The lane plurality is removed.
The safeguard of B84 stays:
a slice joins the pairing only where it carries as many headings as the original's slice.
The finding now ends "the house style" where the archive does not number the series.

Three cases were red before the fix:
a bare archive with lane renderings "Part One:"
and "The Fourth:"
shipped bare headings,
one bare archive shipped different series for different lane renderings,
and an archive that never headed the series,
or headed it in two styles,
shipped bare headings on a plurality of bare renderings.
Each now expects "Part N:"
on every numbered heading.
The existing cases that encoded the plurality now expect the house style,
and the cases for an archive that numbers the series are unchanged.

Recurrence:
none of an earlier entry.
A rule that takes the archive's form as the page's is asked first of an archive that has no such form;
the three red cases of this entry hold the bare,
the mixed and the unheaded archive.

## B709: a quote copied from a sheet that cut the notes between translators was searched in the stored text

Status:
fixed on 2026-10-20 (UTC),
red `074fd701f1`,
fix `a9ed3d4ec3` (document-structure),
`fe8c5c15a7` (typography)
and `768b081ee5` (the callers),
pins `0d4a78bca0`,
cache accounts `64d40cb4ff` and `bb7052ce9d`,
source scan fix `53f8f409cf`,
merged onto main as `1851eab22e`,
gated as g128
(145 steps exited 0,
2608 `[PASS]` lines,
no `[FAIL]` line,
49 source scans pass)
and landed with this entry.

The cut of B708 shows each judge,
critic and auditor the English with the notes between translators taken out,
for display only.
Six places read a model's quote back against the text:
the critic's issues (`resolveCriticIssue` through `locateQuote`),
the coverage voices (`weighVoice`),
the introduced-defect prober (`screenEvidence`),
the rendering audit (`screenFinding`),
the reference attestation and its screen of attested additions,
and the consistency pass (`locatedRendering`).
Each searched the stored text,
so a quote that spans a place where a note was cut matched nothing there,
and the claim was dropped,
counted unanchored or left unlocated.
Over the 92 pinned English archives 3 of the 34 notes stand where a quote can cross:
zheermao101,
XingZ60 and shihai4h,
the first two in the readiness rotation.

`shownTranslation` and `storedStretchesOf` (`document-structure/src/translators-note-view.ts`) map a span of the shown text
back to the stored text as one stretch for each run of kept words it reaches,
with whitespace trimmed at the edge a cut touches.
`locateQuote` takes the original the sheet was cut against,
searches the quote in the text as shown,
judges ambiguity there,
and binds each stretch to its blocks,
so no anchor holds a note.
Without the original nothing changes.
One span covering the note was measured and refused:
a replacement over it deleted the note at all three sites,
and the carry (`archive-note-carry.ts`) puts a lost note back only after the slice's last block,
outside the nested quote at zheermao101 and after an unrelated heading at XingZ60.
With the stretches each replaced alone the note stays in place at all three,
and `deriveEditableEnvelopes` keeps separate stretches as separate envelopes (pinned in `shared-contract`).

The consistency pass is left as it was:
it counts a place's end as its start plus the core's length and copies the core between places,
so a core mapped across a note would carry it.
A test pins that a crossing quote leaves its place unlocated,
so the place is left alone,
and that quotes either side of the note still locate.
A quote that drops the `>   > ` marker across a quoted soft break fails in the shown text too,
and stays out of scope.

Over the pinned archives,
built windows of 10,
40 and 120 characters either side of each cut and whole lines,
trimmed as a model would give them:
245 cross a cut in 8 entries,
0 resolved before and 245 resolve now,
none anchors on a note or on characters other than the stored text's
(zheermao101 10 of 10,
XingZ60 9 of 9,
shihai4h 10 of 10).
Untrimmed,
1 window resolved before and not now,
one that ends in a lone line feed before a paragraph break,
which the shown text reads as a hard break.
These are built windows,
not quotes models gave.

No cache number moves.
`SLICE_CACHE_VERSION` 40,
`REFINE_CACHE_VERSION` 11,
`TRANSLATE_SLICE_CACHE_VERSION` 22,
`LANE_CONTEST_CACHE_VERSION` 13 and `CONSOLIDATE_CACHE_VERSION` 28 each carry this change inside them:
none of the 12 run logs starts on a tip that holds the commit that set any of them
(`git merge-base --is-ancestor` against each `START tip=` line,
with a control that holds),
so no record stands at those numbers.
The rendering audit opens no cache,
and the consistency cache holds ballots only.

Two commit subjects name the wrong type,
and each carries a comment that says so:
`64d40cb4ff` has a `docs(*)` subject over lines that read `fix(...)` for TSDoc-only changes,
and `768b081ee5` lists `fix(translation-repair-insertion)` for a change to a comment only.

Recurrence:
of B708.
A text cut for display keeps a map back to the text it was cut from,
and every reader of a model's reply that cites the shown text reads through that map,
tested on a quote that crosses the cut.
