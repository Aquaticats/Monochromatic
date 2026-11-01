# Audit ledger: readers that accept what no writer writes

Readers of stored records and directories that accepted a shape,
a name or a value their own writer never produces.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B64: four readers took any `*.json` name as an artifact

Found by the twelfth coverage batch.
The census (`readdirArtifacts`,
then in `artifact-placement.ts`) and the scheduler (`pass-settled.ts`) list only regular files named `*.json`,
after the drift `pass-settled.ts`'s region comment records,
where a directory or a symlink called `Mittens.json` marked Mittens settled.
Four readers of an artifacts directory never took that check:
`settled-carve.ts` `listSettledEntryIds`,
the rendering audit's run sets (`rendering-audit-settled-input.ts`,
whose loose layout did check),
`published-tree-listing.ts` `settledEntryIds` (read by `verify-published.ts` and `pass-republish.ts`)
and `editor-standing-read.ts`.
There a directory or a symlink named like an artifact became a settled entry,
stopped the archive reading with EISDIR,
or was read a second time;
run by hand over an archive holding one artifact,
a directory and a symlink,
the built `editor-standing-read` printed `artifacts=3`.
Five other readers took the census's lister and filtered its names by the suffix again
(`score-probe.ts`,
`attribution-read.ts`,
`damage-sample.ts`,
`draw-sample.ts`,
`pass-schema-census.ts`),
four more sliced the suffix off by hand
(`keepEligible`,
`assertArtifactProvenance`,
`loadEntry`,
`collectTwoLaneShippedRegions`),
and the artifacts directory's name was spelled out in ten files,
two of them as their own constants.

Fixed (`cc74ef7f5` and `7e1305ec6` red,
`daa5af787`,
then `8860dc52c` and `a60c1ad38` with no behaviour change):
`corpus-run/artifact-file-name.ts` owns what names an artifact and where it lives:
the `ArtifactFileName` type and its guard,
`entryIdOfArtifact`,
`artifactFileNameOf`,
`listArtifactFiles` (regular files only,
filtered by the suffix),
`artifactFilesIn` (the same,
reporting an absent directory),
`ARTIFACTS_DIR` and `artifactsDirOf`;
every reader lists through it,
and every name is built and read back through it.
`runs-layout.ts` owns the slice-cache and prompt-payload directory names,
which the cap census's skip list now reads,
and `prepareRunsLayout` has its first test.
Letting a non-regular file through the lister failed six test files
(`artifact-file-name`,
`settled-carve`,
`rendering-audit-settled-input`,
`published-tree-listing`,
`editor-standing-read`,
`pass-settled`).

Checked since (B65):
the ledger,
the published tree,
the probe runs and the slice cache each took an entry their writer never wrote,
and every listing now goes through one module.

Recurrence:
a rule about what a directory holds lives in one module every reader lists through,
and a new reader of that directory starts from that module;
a name spelled in two files becomes a constant one of them owns.

## B65: readers of the run's other directories took entries their writer never wrote

Found answering B64's open question,
by reading every directory listing in library source
(`rg 'readdir|opendir|Dirent|glob'`,
test files and fixtures aside).
Six readers took an entry of the wrong kind,
or a write still in flight,
as a record:

- `ledger-directory.ts` `readLedgerDirectory` attempted every name.
  The recorder writes each contest through `writeFileAtomic`,
  whose temporary name is `<name>.<pid>.partial` beside the target,
  so a report listing the ledger mid-write,
  or after a crash between write and rename,
  read that file as a contest
  (a whole one read as a round,
  a cut one counted among the refused contests,
  which the report calls contests the run recorded).
  A directory named like a contest was refused the same way,
  and a symlink to one counted that contest twice.
- `published-tree-listing.ts` `publishedEntryIds` listed the people directory by name and looked for each name's page;
  a regular file there raised ENOTDIR out of the listing,
  and so out of `verify-published`,
  whose owner rule is to print its findings and ship,
  and a symlink to an entry directory counted that page under a second name.
- `rendering-audit-settled-runs.ts` `newestRun` took a directory named like a later run as the newest run.
- `slice-cache-namespace.ts`:
  a directory named like one of a lane's slices raised EISDIR out of the cache open,
  stopping the entry,
  and the discard's `rm` raised on it too.
- `slice-cache-store.ts` `listResumableEntries` resumed an entry with progress a second time under a symlink's name.
- `entry-reattempt.ts` `countCachedSlices` counted a directory named like a slice as progress.

Every case but one needs an entry no pass writes,
placed by hand;
the ledger's partial file is the one an ordinary crash leaves,
and a report run mid-write can meet.
Reading the listings also found the copies B64's rule is about:
`artifactFilesIn` and its `ArtifactListing` union (B64's own) repeated `namesIn` and `DirectoryReading`;
three readers repeated a listing that reads an absent directory as empty
(`ledger-directory.ts`,
`slice-cache-dir-read.ts` `readDirectoryNames`,
`declined-entries.ts`);
`declined-entries.ts` and the rendering audit's layout listing each filtered a `withFileTypes` listing by hand;
the ledger directory's name was spelled in `candidate-ledger.ts` and `ledger-report.ts`;
and the slice suffix in `slice-cache-namespace.ts` and `entry-reattempt.ts`.
`declined-entries.ts` (ledger A9b),
`pipeline-digest.ts` and the rendering audit's layout listing already checked each entry's kind.

Fixed (`e987110a1` red,
`467ab6c48`,
then `6583c32c4` with no behaviour change):
`corpus-run/directory-listing.ts` decides what kind of entry a reader takes:
`namesOfKind` lists regular files or directories by the entry's own type,
so a link is neither and is never followed;
`presentNamesOfKind` reads an absent directory as holding none;
`readingOf` reports an unreadable directory by its filesystem code,
and `namesIn` and `artifactFilesIn` both read through it.
Each reader takes the kind its writer makes,
and the ledger takes only names `ledgerFileName` writes (`isLedgerFileName`).
The ledger directory's name and the slice suffix each have one owner
(`LEDGER_DIR`,
`isSliceFileName`).
`directory-listing-scan.unit.test.ts`,
now among the source scans,
fails on any directory-listing import from `node:fs` or `node:fs/promises`,
and any namespace,
default or dynamic import of them,
outside the listing module and the two walkers it names with why
(`pipeline-digest.ts`,
`cap-census.ts`).
Letting every kind through `namesOfKind` failed the B65 cases for an entry that is not the kind asked for in six test files
(the slice cache's open case unseen behind its file's first failing describe),
both kind cases of `directory-listing.unit.test.ts`,
and the older case for a plain file under the slice-cache root,
whose ENOTDIR catch the fix removed;
letting `.partial` names through the ledger failed the in-flight case alone;
a stray `readdir` import in `ledger-directory.ts` failed the scan's package case.

Recurrence:
a reader lists through `directory-listing.ts` and names the kind of entry its writer makes,
then filters by the name its writer gives a record,
so a write still under its temporary name is never read;
the scan refuses any other listing.

## B92: a JSON array read as a record

Found 2026-10-01 (UTC) by a census of `isJsonRecord`'s calls,
red in `ee6e5b1c5`,
fixed in the commit adding this entry.
`isJsonRecord` (`json-guard.ts`) answered true for an array,
and its TSDoc called that harmless:
"numeric-keyed probes simply miss".
The census read its 129 calls in 62 production files
and found 38 where an array let through changed what a reader did,
and none that needed an array to pass.
An array where a record belonged was read as
an Anthropic frame or delta naming no type and passed over uncounted,
a usage-only stream event,
a resumable slice or translate cache record and a settled consolidation round with no further check,
an issue view with an empty status,
decision answers keyed "0" and "1",
attempt counts keyed by index,
a found stream error with every field unnamed,
or a body that reached a later,
less precise refusal than its own.
Three modules refused arrays on their own beside the guard:
`artifact-guard.ts`'s `requireRecord`,
whose TSDoc gave as its reason that nineteen other modules used the guard
"for values where an array is legitimate",
which the census did not bear out;
`corpus-run/pass-schema-census.ts`;
and `artifact-exact-guard.ts`,
by testing for an array first.

The guard now refuses arrays,
and the three local refusals defer to it.
The red commit's fourteen cases,
one per kind of reader,
pass;
one of them never ran red,
since its file stopped at the first failing suite
(the multi-suite defect this audit fixes separately).
A fifteenth case,
for `isLookupRecord` (`lookup-cache.ts`),
built an array carrying the record's fields with `Object.assign`;
it was dropped,
since `isLookupRecord` now calls the guard,
whose own case covers it,
and no file read through `JSON.parse` yields such a value.

A second lapse sat beside the first.
Eighteen checks in thirteen files wrote the guard's old test inline
(`typeof value === 'object'` and `value !== null`)
rather than calling it,
so the fix to the guard reached none of them,
and the census of the guard's calls,
which searched by its name,
listed none.
Each now calls `isJsonRecord`,
among them `isLookupRecord` and `isLookupHit`,
the run lock's holder reader,
the slice cache's namespace reader,
the roster card's row check
(which already refused arrays),
and the three front matter readers in `corpus-run/directory-id-name.ts`,
which now read fields off the narrowed record without casts.
The first census of those inline checks searched only the `===` form,
and an edit anchored on one check matched two in `lookup-cache.ts`
before the `!==` form was searched.
One reader changes what it does:
the work-title search read an answer that is not an object as no results,
and now refuses it,
as the cited-reference fetch refuses one;
its refusal case gains that answer.
The attribution reader's part was superseded by B106:
an issue record whose `issue` is not a record,
read here as a view contributing nothing,
is now refused,
naming its path.

Reach:
not measured over stored replies or cache files.
The readers face provider streams,
the package's own cache and record files,
which it writes as objects,
and front matter;
an array reaches them only from a provider answering out of contract
or a file written by something else.
No cache version moves:
the change refuses only records whose named fields are arrays,
which the package's writers never produce.

What enforces it:
`src/record-checks.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/record-checks.unit.test.ts`),
among the source scans,
fails on any `typeof` compared with `object` in the package's source outside `isJsonRecord`
and the seven checks it names with why
(a caught error read for its code,
stderr or position,
a member a type says cannot exist,
a spend reading whose other members are text,
and the shape a message names),
and fails when a named check no longer stands.
Its fixture case runs first;
with one converted check in `index-pair-list.ts` written inline again,
its package case failed,
and passed once the check was restored.

Calls made here are open to veto:

- the guard itself refuses arrays,
  rather than a second predicate beside it;
- a work-title search answer that is not an object is refused,
  rather than read as no results;
- the seven exemptions;
- the `Object.assign` case dropped for the guard's own.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B97: a list-shaped alias skipped the identity rule

Found 2026-10-01 (UTC) in T8's twenty-third batch,
red in `7088bf67d`,
fixed in the commit adding this entry.
Where an original's `name` and `info.alias` are one identity,
a translation must carry its name among its alias renderings.
`visibleIdentityOf` (`front-matter-translation.ts`) read `info.alias` only as a string,
and took any other shape for another schema,
so where the original wrote its alias as a YAML list holding just its name,
the rule never ran,
and a candidate whose list dropped the name passed.
The package's other alias reader,
`aliasesOf` (`corpus-run/directory-id-name.ts`),
already read a list by joining its items on the comma;
`visibleIdentityOf` now reads a list of strings the same way.

Reach:
none today.
No pinned page or `info.yml` writes `alias` as a list
(`b23-front-shapes.mjs` in the audit's scratch folder:
92 entries,
254 `alias` keys across their pages,
none list-shaped),
and the site's build shows each `info` field as one value
(`scripts/build.ts`,
line 205,
in the data repository),
so a list is a shape the site does not expect either.
The refusal's wording still says "comma-separated" for a list.
The two readers still split on different separators:
the candidate side on the comma its sheet tells a model to use,
`aliasesOf` on the three the originals use;
each separator constant now says so and names the other.

The same batch found a second reading,
which stands:
a page whose front matter fence pair holds nothing parses to null,
and `pageFrontMatter?.data ?? sourceData` then judges a candidate by the original's shape,
as for an insertion.
The audit's reader called that conflating an empty archive with none;
an empty block has no established keys to keep,
and judging by the empty shape would refuse every candidate that translates the original's metadata,
so the behaviour stays,
its comment now says so,
and a case pins it.

Calls made here are open to veto:

- a list alias read as its items joined on the comma,
  rather than left as another schema;
- the original's shape governing a page whose front matter holds nothing.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B438: the consistency reader passed groups its writer never writes

Red `f134107681`,
fixed in `a7a2a62c28`.

`assertGroupFits` compared "is a frame" with "both slot offsets present",
so a place of a repeated run carrying one slot offset alone gave false against false and passed;
each offset is now held to the kind of repeat on its own.
It refused a group of two or more places under `too-few-placed`,
but not fewer than two under a verdict the writer gives only to two or more,
since `dispositionOf` answers `too-few-placed` first;
it now refuses fewer than two places under any verdict but `too-few-placed` and `failed`.
The writer sets both offsets or neither and never writes either shape,
so only a damaged file reached them.
(Changed on 2026-10-12 (UTC) by B448:
a `failed` group needs two places too,
since only a group put to the bench can fail and the bench is asked only with two.)

## B440: the reader of an unplaced occurrence took a place with no bounds

Red `f134107681`,
fixed in `a7a2a62c28`.

`parseConsistencyUnplaced` took any three counts,
where the reader of a placed occurrence refuses a place ending at or before its start,
or on a line before the first.
One `requirePlaceBounds` (`artifact-two-lane-read-consistency-parts.ts`) now states the rule for both.

Found by the same work and changed by the lead's decision,
in work still open when this entry was written:
`reasonContradicted` (`artifact-two-lane-read-consistency-located.ts`) reads a place recorded
`not-found`,
`ambiguous`,
`slot-not-found` or `slot-ambiguous` as fine whatever the ballots quote;
and the region comment of `pass-consistency-apply.ts` says a replacement meant for a refused slice is recorded
`kept-floor`,
where one already `overlapped` stays `overlapped`.
Kept by the lead under the owner's instruction of 2026-10-11:
a `failed` group reads with any count of places;
the reader checks no slot order or containment,
which the writer cannot get wrong without failing its own splice.
(Both changed on 2026-10-12 (UTC):
the reader now refuses a `failed` group of fewer than two places (B448)
and a slot that does not start before it ends or lies outside its place (B444);
`reasonContradicted` was settled by B444 and the region comment by B448.)

## B443: the consistency pass's ballots, stored gathers and quotes took what no gather or slice holds

Red `54d53e8b46`,
fixed in `d4736ccb6c`.

Found on 2026-10-12 (UTC) by an adversarial review of B431,
each red at the pass's own commit.

- A reply key `seat` overwrote the roster seat in a stored ballot,
  and any extra key was stored;
  `voiceOf` now copies the ballot's own fields after the roster seat.
- A stored gather resumed with a quorum,
  a ballot count or a seat set no fresh gather makes;
  `recordFits` now checks it against the bench after the cache answers,
  and warns and asks again otherwise (B439 states the same family in the cache reader).
- A quote that began inside emphasis,
  or carried a footnote mark,
  reached a slice,
  since the floors read neither;
  `carriesOrCutsMarkup` (`consistency-markup.ts`) leaves a quote or slot with an unpaired emphasis,
  code,
  bracket or parenthesis run,
  or any footnote mark,
  unlocated with the reason `markup`.
- Two places of one slice whose quotes overlap were spliced against each other;
  both are now unlocated with the reason `overlaps-another`.
- A frame's slot that was its whole quote would have replaced the receiver's frame words with the slot alone;
  such a place is unlocated with the reason `slot-whole`.
- The slot majority was counted among every ballot,
  not among those that gave the agreed quote;
  `agreedSlotOf` now takes the quote,
  and the reader's recount passes it.
- `quoteCore` kept a leading space in the words and cut a variation selector off the character before it;
  `consistency-quote-bounds.ts` reads whole characters.
- The first-letter adaptation read `charAt(0)`,
  so a capital beyond the first plane was not adapted,
  and stopped at an opening mark;
  it now reads whole characters by general category past the opening marks.
- A chosen label the sheet never showed read as unlocated;
  it now throws `ConsistencyInvariantError`.

## B444: the consistency reader accepted relations no writer gives

Red `54d53e8b46`,
fixed in `d4736ccb6c`.

The same review found the reader took slot offsets outside their place or ending before they start,
a group of one place the bench considered,
`failed` included,
a seat heard twice,
places of one slice said to read alike,
a slice holding both a replaced place and one the floors refused,
and a reason for an unlocated place that no ballot or group could have given;
`reasonContradicted` now reads whether the group is a frame,
which settles the gap B440 left open.
The slice checks moved to `artifact-two-lane-read-consistency-slices.ts`,
with a test of its own.
A round trip from the real writer through the reader for every outcome and reason passed before and after.

## B680: the recount of a carry the bench was not asked about never read its run-off

Found on 2026-10-18 (UTC) by the sol review of the production diff of five landed rounds
(`s245/sol-review/answer-gpt61.md`,
finding 7),
and reproduced before the fix.

Outcome written on 2026-10-19 (UTC).
Red `bfca728da0`,
fixed in `9df04023d0`.

What the reader accepted:
`assertCarryRecounts`
(`artifact/src/corpus-run/artifact-two-lane-read-consistency-carry-recount.ts`)
checked the ballots,
the usable count,
the quorum and the candidates of a carry whose bench was not asked,
and returned before it read `carry.runOff`.
A run-off beside a decision of no alternative names finalists that nobody voted among,
and it was accepted.

What the writer can produce:
`carryWithoutChoice` (`corpus-pass/src/corpus-run/pass-consistency-carry-choose.ts`)
records `no-alternative` with no run-off,
and the pass attaches a run-off only after `askCarryChoice` has been asked with finalists
(`runOffAsked`,
in the same file).
So no pass writes the shape.

Fixed:
the branch for an unasked carry refuses a run-off at `...runOff`,
with the reason "no run-off,
since the bench was not asked among the candidates".
The refusal is not gated by generation:
no pass of any generation wrote the shape,
so no stored artifact is refused for it.

Reader check (read-only):
`readSettledArtifact` over every JSON file under the `artifacts` directories of `corpus-pass-TianqiChen66624`,
`corpus-pass-TianqiChen66625` and `corpus-pass-TianqiChen66626`.
At the time of the check the three held two files,
none of them in the third,
both of generation 17 and both accepted,
before and after all the fixes of this round.
They hold located renderings and carries,
so the recounts of B675,
B677 and this entry ran over them.
It changes no page.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Files a later run reads back".
