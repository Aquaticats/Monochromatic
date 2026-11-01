# Audit ledger: text by code point, and words inside words

Text read by UTF-16 unit or ordered by locale where code points were meant,
and fixed words found inside longer words.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B21: the other recurring families have no recorded code-reading pass

Status:
fixed,
2026-09-29,
recorded the same day so the work outlived the session's task list;
every family is read and fixed:
family one (text indexed by UTF-16 unit) as B22,
family two (matches without word boundaries) as B23,
family three (straight against curly quotes) as B24,
family four (trimmed text compared with raw text) as B26,
family five (quorum denominators counting unreachable seats) as B27,
family six (sheets missing blocks) as B28,
and family seven (silent fallbacks) as B29,
which names the one gap it leaves open under its own heading.
B1 to B20 came from the duplicate-body and letter-predicate censuses.
The class history names other families that recurred across stages,
and this ledger records no code-reading pass over them:
text indexed by UTF-16 unit (class ninety-six),
substring or prefix matches without word boundaries
(class one hundred sixty-three;
area one fixed the glossary's matcher,
and no pass over the other sites is recorded),
straight against curly quotes
(classes thirty-eight,
seventy,
one hundred sixty-eight and one hundred seventy-five),
trimmed text compared with raw text
(classes sixty-five and one hundred one),
quorum denominators that count unreachable seats
(classes twenty-six and fifty),
sheets missing blocks,
and silent fallbacks.
Fix:
read each family's sites across translate,
repair,
consolidation,
the provider clients and the corpus-run driver;
each finding gets its file and line and a probe that reproduces it,
and is fixed with a guard shown failing first.

## B22: text read by UTF-16 unit where the test reaches past the first plane (B21, family one)

Status:
fixed,
2026-09-29.
Class ninety-six read a quote's neighbour one UTF-16 unit at a time,
so the low half of a script letter was no letter.
The pass over the family read every test whose domain reaches past the first plane
(the Han blocks from Extension B on,
`\p{L}`,
case)
and every scan that feeds one:

- Declared names.
  `nameProjection` (`declared-name-survival.ts`) scanned by unit,
  so a handle in mathematical script projected to nothing
  and one holding an Extension B ideograph fell under the shortest checkable form;
  neither was ever checked.
  Before the fix,
  no declared name or contributor form at the pin held a letter beyond the first plane,
  and over 835 form and text pairs from the 92 archives and 214 settled pages
  no survival answer changed,
  where a script letter planted inside a form's occurrence did.
  Red `409114af6`,
  fix `6346909fc`;
  a mutant restoring the unit scan is caught.
- The preservation tokenizer.
  `contentTokens` (`preservation-tokens.ts`) read `text[index]`,
  so the gate never saw an Extension B ideograph,
  although audit area six widened the Han test to every block (`han-only-text.ts`).
  `348e97c58` moved `codePointAt` and `codePointBefore` from `quote-neighbours.ts` into `code-points.ts`
  and dropped the private copy in `mdx-tag-start.ts`;
  red `328a35e6e`,
  fix `9816ae18a`.
- The Han residue floor.
  `translate-han-residue.ts` built its runs from single units,
  so such an ideograph left alone in prose passed,
  and a run it opened was named from its second character.
  Red `08f797e48`,
  fix `fdaf1752a`.
- The Canadian and pinyin page passes.
  `runEnd`,
  `runStart` and 19 other character reads in the date,
  spelling and capital readers took one unit,
  so a date or listed word glued to a Deseret letter was rewritten
  where one glued to a Latin letter is not.
  The pinyin pass never took an Extension B ideograph for Han,
  and its comment that every ideograph pinyin-pro reads is one unit was false:
  pinyin-pro 3.29.3 gives a reading for 196 of the 65,312 code points in Extensions B to H
  (`~/temp/agent/audit-glossary-fix/b21-pinyin-astral.mjs`).
  Red `4b4066c86`,
  fix `73e158c8b`;
  `52fe27efd` adds one Latin-twin case per converted read,
  and a unit read restored at any of them breaks the comparison,
  except the date cursor,
  where no month name starts beyond the first plane,
  so the whole letter opens no month either;
  that mutant is equivalent.
- Two readings of a cased letter.
  `quote-neighbours.ts` read one by general category
  (class ninety-six:
  script letters are cased letters with no case mapping),
  the Canadian passes by case mapping,
  so a script letter bound an apostrophe but was no letter beside a date,
  and the capital and small-letter tests read a script capital as neither.
  `cased-letters.ts` holds one reading for both;
  red `9369541f4`,
  fix `f94718959`.
  Over 306 pages,
  settled and archived,
  the Canadian pass wrote the same text under either reading,
  and a control page glued to a script letter differed.
- Text excerpts.
  Eleven limits at thirteen cuts
  (an error body excerpt,
  a stream opening,
  a refused reading,
  a raw reply preview,
  bench and probe failure details,
  a ledger report,
  a tally line,
  a rendering-audit drop reason and a blockquote note)
  could keep half an emoji in a log line or a persisted run row.
  `wholeOpening` (`code-points.ts`) cuts on a whole character;
  red `95c13f282`,
  fix `9830a6a4c`.
  `fixed-length-cuts.unit.test.ts` lists every other `.slice(0, LIMIT)` and `.slice(-LIMIT)` with its reason,
  so a new fixed-length cut is classed before it lands.
  `NOTE_WORDS` counted units and is `NOTE_OPENING_UNITS`.
- The shared readers themselves.
  `code-points.ts` and `cased-letters.ts` shipped tested only through their callers;
  the text barrel now exports both,
  and each has its own unit test.
  Those tests found two faults with a lone surrogate:
  `codePointCount` counted every unit that is not a second half,
  so a lone second half counted as nothing,
  and `wholeOpening` dropped a lone first half that no pair followed.
  Both now read a lone half as one character,
  as the string's own iteration does.
  Red `a6193fc8f`,
  fix `f6e93ed5f`;
  9 mutants are caught,
  and a comment control survives.
  `lone-surrogate-census.mjs` finds no lone surrogate in the pinned corpus
  or in any string of the 265 stored artifacts,
  while its control finds both kinds.

Read and left as they stand:
`translate-neutral-pronoun.ts` compared one unit with U+2E80,
and a surrogate half compares above it as its whole code point would
(B23 has since replaced that comparison with `tokenStarts`);
the recurrence watcher's tails are compared unit for unit with the buffer they came from;
the refusal windows are searched for markers a cut character cannot match;
and `latin-letters.ts`,
the ASCII tests and the handle scans test first-plane blocks only,
where a unit read answers as a code-point read does.

Exposure at the pin:
the pinned sources and archives,
the settled pages and the 370 stored artifacts hold no Han,
kana-supplement or case-mapped letter beyond the first plane;
what lies beyond it is bold script
(328 letters on pages,
12,390 in artifacts)
and emoji,
which every site this entry lists reads the same either way,
the cased-letter reading aside,
measured under "Two readings of a cased letter".
The translate,
consolidation,
contest,
refine and repair versions carry rides-inside accounts;
the page passes and the excerpts key no cache.
Prevention:
`mistake-prevention.md`,
"Text by code point".

## B23: fixed words and phrases found inside longer words (B21, family two)

Status:
fixed,
2026-09-29;
every site the census classed as a finding now reads its word,
name or address as one.
Class one hundred sixty-three found a glossary term inside a longer word,
and area one bounded the glossary's matcher (C1).
This pass reads every other site that looks for a fixed word or phrase in text,
starting from the 460 of 668 `includes`,
`indexOf`,
`startsWith` and `endsWith` calls whose needle is not a literal
(`~/temp/agent/audit-glossary-fix/b23-needle-census.mjs`).
Those that test membership of a list were set aside,
and each text search was classed as bounded already,
a deliberate substring
(quote and evidence containment,
the declared-name projection,
a pinyin prefix),
a needle with Han edges only,
or a finding.

### The refusal readers

`detectRefusalShape` (`refusal.ts`),
the picture refusal phrases (`image-reading-sense.ts`)
and the inability markers (`reading-refusal.ts`)
matched raw substrings of text with straight quotes:
"as an ai" fired inside "as an aide",
"i cannot" inside "taxi cannot",
"load" inside "download",
"quality" inside "equality",
and a reply opening with a typographic apostrophe
("I won’t be able to")
was no refusal at all.
`word-bounds.ts` finds a needle whose Latin edges stand at word boundaries,
the glossary's rule;
both opening windows fold their quotes to ASCII first (`normalizePunctuation`),
and the inability markers are read as the starts of words.
Red `d090a0f0a`,
fix `23974cfa8`;
18 mutants,
each undoing one part,
are caught,
and a comment control survives.
A neighbour is read as a whole character,
and since every Latin letter the package names sits in the first plane,
a unit read decides the same;
that mutant would be equivalent and is not run.

Reading markers as word starts dropped every form a prefix joins to one.
Every word of `/usr/share/dict/words` that carries a marker past its start was read
(`b23-inability-lexicon.mjs`).
The forms that still name access to the picture are listed on their own:
download,
upload,
reload,
the stems inaccessib and unaccessib,
and unprocessable and unloadable,
which that list lacks.
The rest
(workload,
payload,
overload,
inequality,
uncorrupted and the like)
name no inability.
Download,
upload and reload stay inability markers
although an absence report could name a download icon,
because the costs differ.
Two absence reports overrule the deterministic reader
and confirm a picture textless (`image-reading-pair.ts`),
so an access failure read as absence is silent.
An absence read as inability leaves the picture unread,
and the visual evidence check then pauses the entry and says why.
The red guard in `d090a0f0a` that took a download icon for absence
became a boatload in the fix.

Measured with the frozen build `2f26f440d` against the fix
(`b23-screen-replay.mjs`):
the 707 distinct stored picture readings,
from 447 reading-store files,
get the same verdict under both.
Over the 409,436 distinct strings of the 370 stored artifacts,
the refusal detector newly reads 22 openings as "i won't be able to"
and 1 as "i can't help",
and no longer reads 2 as "as an ai".
The picture screen newly refuses 291 strings
(274 as a refusal,
17 as an absence report),
every one through a typographic "I can’t" in its opening,
and passes 1 it refused,
a romanized name ending in i before "cannot".
None of those strings is a picture reading;
they show what the phrase list now refuses whichever apostrophe a reading uses.
That list is a heuristic stated as one,
and an English transcription opening with "I can’t" is refused,
as one with a straight apostrophe always was.

No cache version moves.
A gather treats a reply read as refusal-shaped and an unparseable one alike,
both off-shape (`recovery-nudge.ts`),
so the detector changes a log line and the scorecard's counts,
not whose voices a round hears.
The picture screen's verdict reaches a slice key through the reading's words,
which the translate and consolidation keys carry,
and the reading store is stamped with the build's digest.

### Declared names and the link-name floor

The census had classed the survival guard's projection containment as deliberate,
and it is,
for what the projection sees through:
a `\_` escape,
a space or underscore where the other side writes none,
a name wrapped behind a `> ` prefix.
It also drops the spaces between words,
so containment found a declared key across a word edge:
a candidate that lost "Ann" but said "cannot" kept the name,
and a base that said only "cannot" put an "Ann" it never held at stake.
The guard's header claimed that widening what counts as carried
could only turn a missed loss into a caught one;
that was wrong in both directions.
The link-name floor compared the same projection,
and matched a Latin source form,
a mention and a page handle as raw substrings,
so a tomcat named Tom and `@mi-mi-420` carried `@mi-mi-42`.

The edge rule came from the glue itself,
read by general category only (`b23-projection-bounds.mjs`).
Where a declared key sat inside a longer word in the stored texts,
the long keys and the one settled page were glued by a case change or a digit
inside a handle,
and the lowercase-to-lowercase glue was prose collision
(one handle stem inside a common first name among them).
`name-projection.ts` keeps,
per projected unit,
where its character sits in the text as composed,
and `carriesName` counts a key only where each Latin end meets a word edge,
a letter meeting a digit and a small letter meeting a capital being edges.
The form-inside-form check that reports a lost long form once reads the same way,
so a shorter form running on inside a longer one is its own loss.
The link floor reads its source forms,
mentions and page handles at handle edges,
where a hyphen and an underscore join.
Red `83a3457d0`,
fix `11d029fde`,
the before-edge case `edac4d209`;
15 mutants,
each restoring containment at one site or dropping one edge rule,
are caught,
and a comment control survives.

Measured with the fix
(`b23-projection-final.mjs`):
over every declared and contributor form against the texts of its own entry,
all 285 carriages in the 92 archives
and all 544 on the 214 settled pages stand as names,
and 1,422 of 47,389 in the stored artifacts existed only across a word edge.
Replayed against the frozen build `2f26f440d` over 1,264 archive slices and 3,975 would-ship slices
(`~/temp/agent/audit-floor-replay/names-replay.mjs`):
the survival guard now refuses four would-ship wordings of two slices,
each of which dropped a declared form the incumbent carried as a word
and kept its letters only inside a longer word;
contributor authority moves nothing;
the link floor's three moves on archive slices come from the account-handle rule of `9c085dcd3`,
which the frozen build predates,
and none from this change.
The translate,
consolidation,
contest,
refine and repair versions carry rides-inside accounts
(`3a68938ef`);
the refine and repair accounts rest on the translate replay,
since no stored rewrite or patch was replayed.

### The page-name note

`declaredNameNote` (`linked-title-declared-name.ts`) matched the source form
and the declared rendering as raw substrings,
while the link-name floor reads the same link at handle edges and as a name.
The note now reads it the floor's way,
so a title rendered "Tomcat" draws a note for a declared "Tom",
an archive writing the handle with an escaped underscore draws none,
and a Latin source form inside a longer word names no one.
The handle reading moved into `handle-token.ts`,
which both read.
Red `b571f4303`,
fix `c60a2f4ff`;
7 mutants are caught,
and a comment control survives.
Over the 92 entries the frozen and current builds write the same linked-title lines,
and the page-name lines ride in the identity context every stage key hashes.

### The title reference passes

`referenceScope` (`title-reference-scope.ts`) took the first framed line holding the bare title,
so a footnote naming the title without brackets,
ahead of the footnote that brackets it,
held the quote search to the wrong line.
The scope and the unify pass's gate now read one `bracketsTitle`.
`rewriteLocated` (`title-reference-unify.ts`) read any located span
ending with the heading's rendering as the heading's,
so a footnote quoting a longer rendering was never unified.
A quoted,
bracketed or linked span now has to equal the rendering,
and a glossed run,
whose lead-in words the locator cannot separate from the title,
may end with it only at a word edge (`endsWithWords` over `wordStarts`).
Red `10c60dae8` and `4bdb2f614`,
fixes `089bd9388` and `89f4fcba3`.
Restoring a bare gate in the unify pass survived the first mutation run;
a case pinning that a slice naming the title without brackets is left alone now catches it,
and all 6 mutants of the two passes are caught,
with a comment control surviving in each run.
At the pin,
6 of 264 Han heading titles are referenced in brackets
and none has an earlier bare framed line;
on the 273 settled page files no quoted,
bracketed or linked span ends with one of the page's 1,061 heading renderings without being it.

### The name-gloss restore

`nameGlosses` (`name-gloss-restore.ts`) counted an archive's uses of a glossed name by splitting on it,
and `restoreNameGlossLines` placed the gloss with `indexOf`,
so a longer word carrying the name's letters ("Pipit" for "Pip") counted as a use
and decided where the gloss went.
Both read the name through `wordStarts`.
Red `a6b8a6177` and `5cef56405`,
fix `6ef70915a`;
3 mutants are caught,
and a comment control survives.
At the pin the archives carry 1 gloss line,
and neither its use count nor its place on the 14 settled pages of its entry moves.

### The work-title lookup

`namesWork` (`work-title-lookup.ts`) tested the bare title with `includes`,
so a result titled "Catcraftopia" read as naming 《Catcraft》,
drew no neighbour warning and could sort first.
It reads the title with `carriesWord`,
and a Han title is found as before.
Red `8c39cc87c`,
fix `a49c3ac80`;
2 mutants are caught,
and a comment control survives.
Over the 58 cached lookup records at hand
(290 hits,
11 titles carrying Latin letters or digits)
no warning moves,
and the lookup lines ride in the identity context every stage key hashes.

### The unwrapped-link floor

`unwrappedLinkFindings` (`translate-unwrapped-link.ts`) asked
whether the rendering still carries the href with `includes`,
so a bare address that only begins with it (`windowsill-nap.html2`) counted as the destination kept,
and the model was told its destination was intact when it had changed.
It now reads the rendering's `link-url` atoms,
the destination floor's own reading.
That first fix read a rendering the strict grammar refuses as carrying no destination.
Where the original is refused too,
the validator runs only the floors that need no grammar and otherwise answers unknown,
which translate-repair lets stand as written,
so a genuine unwrap there went unrefused.
Such a rendering is now read under plain markdown,
the page side's downgrade,
through the skeleton's own atom walk (`walkAtoms`).
Plain markdown reads a bare destination inside a raw html block as html,
so the floor is silent there,
and its header says so.
Red `49f64c4e3` and `fbe758d77`,
fixes `fc3b3373e` and `28a4ceb63`;
4 mutants are caught,
with a comment control surviving in each run.

Replayed against the frozen build over 1,264 archive slices and 3,975 would-ship slices
(`~/temp/agent/audit-floor-replay/floor-replay.mjs`),
the floor fires on none under either build.
Its positive control (`unwrap-control.mjs`) shows the builds disagree on the longer address
and agree on a genuine unwrap and on a kept link.
`unparseable-link-census.mjs` finds 1 archive slice whose original the strict grammar refuses,
with no link syntax,
and no would-ship slice.
That doubly refused branch runs no destination floor at all,
by the design of F-5;
the census bounds its reach at the pin,
and it is left as it is.

### The neutral pronoun floor

`neutralPronounFindings` (`translate-neutral-pronoun.ts`) counted TA,
Ta or ta only between listed marks,
whitespace or characters at or above U+2E80,
compared by UTF-16 unit.
The listed 「」 was redundant,
not 『』 missing:
both sit above U+2E80.
What the list missed sat below it:
a doubled Chinese dash or a slash after han in an original,
so the floor never applied to that slice,
and an em dash,
an ellipsis,
an arrow or emphasis in a rendering,
so it passed a kept pronoun.
`pronoun-disagreement-census.mjs` read every occurrence
where the list and a word-boundary reading disagree,
in the pinned originals and archives and every string of the stored artifacts,
by the key holding it and a masked context.
In originals and renderings each one was a pronoun the list missed
(originals of XingZ60,
Mizuki_Yuuki and qianyuanakg;
renderings of XingZ60 and noname),
and the rest sat in judges' reasons.
A bare word-boundary reading would lose what the floor's tests pin:
a handle,
a path,
an address and the interjection "ta-da" are not the pronoun.
`tokenStarts` (`word-bounds.ts`) reads a word standing as a token of its own:
the run of Latin word characters and address joiners around it,
trimmed of joiners at its ends,
is the word alone,
and a word after a mention mark is a handle.
The floor counts with it,
which retires its UTF-16 comparison too.
"he/TA" and "Well...Ta" read as one token each,
as under the list,
and the census found neither.
Red `79d588b5e`,
fix `2c0cc08d8`;
7 mutants are caught,
and a comment control survives.

`pronoun-entry-census.mjs` reads TA in 2 of the 92 sources,
Ta in 7 and ta in 8 under the token reading,
the figures the floor's header gave on 2026-09-04;
the list read Ta in 6.
Replayed against the frozen build (`floor-replay.mjs`),
4 slices fire under both,
none clears and none is added:
three archive slices of XingZ60,
whose archive keeps a bare TA,
and one would-ship slice of SS3B_0016,
a settled wording the frozen build already refused.
The frozen build predates the F-9 source gate,
so the 4 were also read under the floor as it stood before this fix
(`pronoun-previous-check.mjs`),
which fires on all 4.
The positive control (`pronoun-control.mjs`) shows the builds disagree on a dash in a rendering.

### Cache accounts

No cache version moves for these sites.
The unwrapped-link and neutral pronoun floors give the same verdict on every replayed slice,
the page-name and lookup lines ride in the identity context,
the gloss and title passes move nothing on the settled pages,
and every per-entry cache is stamped with the build's digest.
Prevention:
`mistake-prevention.md`,
"Words inside words".

## B95: text ordered by the runtime's locale

Found 2026-10-01 (UTC) by a census of the package's orderings
(`~/temp/agent/audit-glossary-fix/locale-order-census-report.md`),
red in `169218442`,
fixed in the commit adding this entry.
`orderBySourceSize` in `corpus-run/bench-draw.ts` broke a source-size tie with `localeCompare` on entry ids,
which are the corpus's own directory names,
so the order of tied slices,
and with it which slices `pickSpreadSample` drew for a benchmark,
followed the locale and collation data of the machine that ran it:
a collation interleaves capitals with lower case,
and code points put every capital first.
Nine more `localeCompare` calls ordered candidate ids,
claim and cluster ids,
coverage sources and bundle names,
and sentinel rows;
the census measured each on its value domain and found every tested locale agreeing with code-point order,
so none moved an output today,
but each rested on its domain staying closed.
Three tests ordered with it too,
one of them under a description saying it ordered by code unit.
`preservation-check.ts` sorted lost document words with the default order,
which compares UTF-16 units and puts a character past U+FFFF before one between U+E000 and U+FFFF.
`compareCodePoints` (`code-points.ts`) now orders text by code point,
the same on every machine,
and every one of those sites uses it.
`locale-orderings.unit.test.ts`,
a source scan,
finds `localeCompare`,
the `toLocale` casing and formatting members
and `Intl.Collator` in source and tests;
it found none outside its fixture once the sites moved.
`memberName`,
the scan's reader of a member's name,
moved from `wall-clock-reads.unit.test.ts` to `source-scan.test-fixture.ts`
so the two scans share it.

Reach:
a benchmark drawn before this fix and after it can differ wherever a source-size tie was broken by entry id.
The other orders the census measured do not move.
Sorts of entry ids with the default order
(`corpus-pass.ts` and `pass-decline.ts`,
both for log lines)
and comparators using `<` on ids are left as they are:
they are the same on every machine,
and differ from code-point order only for a character past U+FFFF in an id.

Calls made here are open to veto:

- code-point order for every text the package orders,
  in place of keeping `localeCompare` where its domain measured stable;
- the default order kept for entry ids sorted only for log lines.

Recurrence:
text is ordered by `compareCodePoints` and cased without the locale;
`mistake-prevention.md`,
"Text by code point";
the scan refuses a new locale-dependent call.

## B122: sorts that skipped the code-point order

Red in `e29363b19`,
fixed in `ae0a90f71`.
B95 made `compareCodePoints` the package's order for text
and guarded against locale orderings,
but nothing guarded the other way around it.
An argument-less `sort` or `toSorted` orders by UTF-16 unit,
so an astral character sorts before one in U+E000 to U+FFFF,
against code-point order.
A comparator written `left < right ? -1 : 1` orders text the same way
and never answers zero for two equal keys,
which the comparator contract requires.

`locale-orderings.unit.test.ts` gained `codeUnitOrderings`:
an argument-less sort in package source,
and a relational operator inside a function handed straight to a sort,
in source and tests alike.
Tests may still sort without a comparator:
a type-aware census
(TypeScript 5.9 from a scratch install,
since the package's TypeScript 7 has no compiler API)
found 153 such sorts in tests,
every one over text-typed values sorted to compare two sides,
which no order changes.
The red package case failed on 46 sites,
the set the census listed:
40 argument-less sorts in 31 production files,
relational comparators in `ledger-directory.ts`,
`probe-verify-sheet.ts`,
`damage-sample.ts` and `score-verify.ts`,
and two in the candidate-ledger and number-reads unit tests.

The fix added `textsInCodePointOrder` (`code-points.ts`),
which orders a list of text through `compareCodePoints` and keeps its element type,
and 39 of the 40 sorts go through it.
The comparators go through `compareCodePoints`.
`sample-draw.ts` held `compareKeys`,
a correct second copy of the comparison out of the scan's reach,
which the type-aware census found;
its three calls now use `compareCodePoints`.
The fortieth sort stays code-unit order on purpose:
`canonicalPromptValue` (`prompt-uniqueness-client.ts`) sorts record keys as canonical JSON (RFC 8785) does,
and its digest names a durable payload record another host reads,
so code-point order would be another canonical form.
It is the scan's one named exemption (`CODE_UNIT_ORDER_EXEMPTIONS`),
which the scan also fails once it no longer names a site.
The keys sorted are file names,
hex digests,
ids,
labels and model names,
so an order moves only where two keys first differ at an astral character against one in U+E000 to U+FFFF
(inference from the key formats,
with no replay),
and no cache version moves.
The full suite on `ae0a90f71` exited 0 with 1543 PASS lines.

Recurrence:
`mistake-prevention.md`,
"Text by code point".

## B186: one reading floor was measured in two counts

Red in `7f99edb6a`,
fixed in `17646f215`.

Found on 2026-10-05 (UTC)
by the same agent's report.
`readingMakesSense` (`image-reading-sense.ts`) compared a model's trimmed reading,
in UTF-16 units with its inner whitespace counted,
against `MIN_READING_CHARS`,
while `solidCharacters` counted non-whitespace code points against the same constant for the deterministic reader.
Fifteen astral characters,
30 units,
and fifteen letters spaced apart,
22 or 29 units,
were `usable` as a model's reading and `no-text` as the deterministic reader's
(the fixing agent's probe of its build before the fix).

The fix:
the reading screen asks `solidCharacters` of the trimmed reading,
and `solidCharacters` lives beside the constant in `image-reading-sense.ts`,
exported from there by `translate-barrel.ts`.
The rule is stricter only:
a reading that was `usable` can become `short`,
and none goes the other way.

No cache version moved.
The agent's reading of the code,
with no run made:
the picture store and every slice cache are stamped with the digest of the built files,
which this change alters,
so no reading stored under the old count is resumed.

Open to the owner's veto:
the count moved to the constant's file,
where the other way keeps it in `image-ocr.ts` behind a value import cycle.

Open when this entry was written,
closed in B252 (`df807a8b5`):
`image-reading-stage.ts` still logs a reading's length in UTF-16 units,
a number that can read larger than the count the verdict used.

Recurrence:
`mistake-prevention.md`,
"Text by code point".

## B252: a picture reading's length was logged in UTF-16 units beside a verdict in solid characters

Red in `3d4558d9c`,
fixed in `df807a8b5`.

Found on 2026-10-06 (UTC),
closing the item B186 left open.
`readImageAsset` logged `trimmed.length`,
so fifteen astral letters logged as thirty characters beside a verdict that counted fifteen.

The fix:
it logs `solidCharacters` of the trimmed reading.
The case is "LOGS A READING'S LENGTH IN THE SOLID CHARACTERS THE VERDICT COUNTED,
not in UTF-16 units:
fifteen astral letters are fifteen characters and a short reading,
twenty spaced ones are twenty and a read".

Recurrence:
`mistake-prevention.md`,
"Text by code point".

## B389: the settled audit counted cited characters in UTF-16 units, and its docs named the wrong caller

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers.
The CITED REFERENCES line of `withCitedReferences` printed `characters=` from the text's `length`,
in UTF-16 units,
so a character beyond the first plane counted twice.
By the agent's run against the base build,
the case
"COUNTS WHAT A PAGE CITES IN CHARACTERS,
a character outside the basic plane once,
as the line names its unit"
fails:
a page citing U+1F408 (cat),
a space and `naps.`
logs `characters=8`,
where it expects 7.
Five TSDoc paragraphs in `corpus-run/rendering-audit-settled-buy.ts` said `main` was their function's only caller,
where it is `runSettledAudit` (`corpus-run/rendering-audit-settled-drive.ts`);
one said `readCap` refuses a typed negative cap,
where `readAuditArguments` reads `--cap` through `wholeNumberFlag`;
and a comment in `corpus-run/rendering-audit-settled-args.ts` placed `capped` in `rendering-audit-settled.ts`.

The fix:
the line counts with `codePointLength` (`code-points.ts`),
as the package counts a page's characters,
and each doc names the code as it stands.

Recurrence:
`mistake-prevention.md`,
"Text by code point" and "Claims without their evidence".

## B432: the tense detector read letters by case mapping, one UTF-16 unit at a time

Red `dd4d201aa2`,
fixed in `9f936b2c2a`.

Found on 2026-10-12 (UTC) by an independent review of B419.
`present-tense-words.ts` took a letter to be a character whose lower and upper case differ,
read by `charAt`,
so a name in Han or another caseless script,
or beyond the first plane,
never opened a word:
14 of the 131 aliases the pinned translations declare could never match.
`wordsOf`,
`namesAt` and `isSEndingWord` now read whole code points by general category.
All 131 aliases and 92 names match;
the candidate count per page did not move on the corpus.

## B675: span agreement counted a quote inside a longer word that ends at a joint

Found on 2026-10-18 (UTC) by the sol review of the production diff of five landed rounds
(`s245/sol-review/answer-gpt61.md`,
finding 1),
and reproduced before the fix.

Outcome written on 2026-10-19 (UTC).
Red `a5fecc9bad`,
fixed in `f416ffa3a2`.

What a reader of a page would have seen:
judges who quoted `cat` and `cat's bowl` agreed on `cat`.
The shorter span was located,
and a splice then moved fewer words than one judge named.
This reaches real pages narrowly,
only where judges' quotes differ by a longer word that ends at a joint.

The cause:
`containsWords` (`consistency/src/consistency-contains-words.ts`) barred containment only where a letter,
a digit or a combining mark of a spaced script stood beside the match.
The locator also refuses a match at a joint of a longer word
(an apostrophe or a hyphen between letters or digits,
a full stop,
comma,
colon or slash between digits;
`consistency-word-joints.ts`),
so the span agreement counted a quote that the locator would not have located.
B645 had made the same test read a combining mark,
and a joint was the part left out.

Fixed:
`containsWords` also asks `cutsWord`,
the locator's own joint test,
about both edges of the match.
`cat` is no longer contained in `cat's bowl` or `cat-nap`,
nor `5` in `0.5`.
Both readers of containment follow:
the candidate spans of a place (`consistency-span-agreement.ts`)
and the slot words of a frame (`agreedSlotWordsOf`,
`consistency-majority.ts`).

Test:
a case in `consistency-span-agreement.unit.test.ts`.

Left as it is and documented in `consistency-contains-words.ts` (`fbc866e2f0`):
the fold writes a single corner bracket (`『cat』and`) as an apostrophe,
and a folded quote cannot tell it from one,
so `containsWords` differs from the locator there.
It withholds a candidate span and leaves a place unlocated,
and never locates one wrongly.

Stored artifacts and caches:
the stricter count would refuse a stored artifact that was right under the count its pass used,
so the recount of an artifact reads containment by the generation the artifact records (B682).
`CONSISTENCY_CACHE_VERSION` stays at 5:
its TSDoc says the counting of quotes is not stored,
and that a change to how quotes are located moves no key and needs no number.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Words inside words".
