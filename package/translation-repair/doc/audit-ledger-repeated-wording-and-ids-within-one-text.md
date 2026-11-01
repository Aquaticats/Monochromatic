# Audit ledger: repeated wording and ids on a page

Titles,
links,
ids and wordings that stand more than once on a page,
settled by their first or last occurrence without saying so.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B129: a span checked against the first node of a repeated id

Red in `4185c097a`;
fixed in `e9ad88d97` by a throw that broke a caller's contract,
and corrected in `ca7677605`.
Found on 2026-10-05 (UTC) reading `validate-issue.ts`.
`validateSpanAnchor` found the node a span names with a search that stops at the first node carrying that id.
Handed a document with a second node under the id,
it checked a span built against the second node against the first:
a correct span over a copy of a parsed document in which every block carried the span's id
came back `stale-node-hash`,
"the base drifted",
for a base that had not
(probe of the unfixed build of `80e129926`).
A rejection is fed back to the model that wrote the claim,
so the wrong kind would have sent it a retry prompt about an anchor that was sound.

No run was harmed:
`parseDocument` ids its blocks `block/<index>` (`document-node.ts`),
and the one production caller of `validateIssueClaim`,
`resolveCriticIssue` (`critic-wire.ts`),
passes parsed documents.
The function is exported and its `AnchorTarget` parameter admits a hand-built document,
which is how the case builds one without a cast.

The first fix,
`e9ad88d97`,
gathered every node under the id and threw an `unreachable:` error naming the side and the id
before any rejection was decided.
It was committed on its own test file,
and the whole suite run on `9ff566830` printed 1564 [PASS] and 3 [FAIL] lines:
a `critic-wire.unit.test.ts` case,
"refuses a claim when caller-supplied documents list two nodes sharing one id whose hashes disagree",
hands `resolveCriticIssue` such a document and expects the claim refused with a reason.
That case had pinned the mislabel,
`anchor-validation (stale-node-hash)`.
It also showed the design the throw cut across:
this module's contract is that a defect of a claim's anchors comes back as data,
and `resolveCriticIssue` ends on a gate that turns any rejection into a named refusal of the claim.
A throw for one kind of document defect bypassed both.

The fix as it stands:
the repeated id is a rejection of its own kind,
`repeated-node-id`,
whose detail says the document lists the id more than once,
so the label is true,
nothing throws,
and the gate's refusal reads `anchor-validation (repeated-node-id)`.
Both cases were changed to expect that first and failed against the throwing build;
on `ca7677605` the whole suite printed 1564 [PASS] and 0 [FAIL] lines,
the lint read `Found 0 warnings and 0 errors.`
and the scans 35 [PASS] lines.
No production reader branches on the rejection kind:
`AnchorRejectionKind` appears outside `validate-issue.ts` only in the package index's export list and in tests.

The process lesson:
a fix that turns a module's rejection-as-data into a throw changes its contract,
and runs against every caller's cases,
by the whole suite,
before it is committed.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation" and "Tables keyed by text".

## B203: a title a voice answers twice was read by its first rendering without a word

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC)
by a census of every lookup in the package's production source that takes the first match of a key,
after B129.
The fixing agent read 403 lines of `find`,
`findIndex`,
`findLast`,
`indexOf`,
`new Map(` and `Object.fromEntries(` calls in 238 files,
each with its function,
and two shapes that pattern misses:
maps built from typed pairs,
8 of 142 typed constructions,
and maps filled by `set` in a loop,
161 sites in 111 files,
read once with no note kept for each.
By its decisions,
9 of the 403 lines were fixed,
117 stand with what makes each key unique,
11 are described and left,
15 sit in files that were another change's,
and 251 ask another question
(a scan of a string,
a membership test,
an ordered preference,
a merge of equal values).
The defects are this entry and B204 to B210,
two of them found in the shapes the pattern misses.
Out of the census's reach:
a first match written as a loop with an early return,
a record built by computed-key assignment or by spreading inside a fold,
and test files.

`settlePageTitles` (`page-title-lexicon-stage.ts`) searched each voice's reply
for the first entry numbered for a title.
`isPageTitleLexiconWire` admits any list of numbered renderings,
so nothing makes a number unique in a reply:
a voice answering one title twice had its second rendering passed over,
and a number the sheet never listed was passed over the same way,
neither with a finding.

The fix:
`resolvePageTitleAnswers` (`page-title-lexicon-wire.ts`) reads a reply once,
keeps the first rendering for each title,
and returns `duplicate-title-rendering` and `title-index-out-of-range` findings,
which the stage reports under the voice's model id,
as `resolvePanelBallot` does for a panel ballot.
No settled title changes:
the first entry for a listed number is the one the search always took.

The whole suite ran on `21f2ec369` in a side checkout:
1,606 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.
No cache version moved,
for this entry or for B204 to B210:
no stored record exists under a current version (the audit named in B189),
and by the agent's measurements no decision moves at the pin.

Open to the owner's veto:
a voice's first rendering is kept and a finding written,
where the other way drops the voice or the title;
and the `title-index-out-of-range` finding,
one step past the family,
which fell out of the same loop.

Open,
described by the agent and left,
each a first match on a key nothing makes unique.
Two follow-ups were launched on 2026-10-05 (UTC).
One took the title link,
the pair bound and the revision's block,
the sites a page pass or a floor reads:
red in `51f6dd875`,
fixed in `65bd3a645`,
and recorded since as B228 to B230,
so the first three sites of this list are closed.
The other took the readers after them in this list
and had not handed back when this was written.
It has since:
red in `d00bc71b4`,
fixed in `8ce45ae86`,
and recorded as B238 to B245,
so every site of this list is closed:

- `corpus-run/title-reference-link.ts`:
  a title linked twice in one slice has its second link's rendering never located,
  so the pass leaves that link as written;
- `bilingual-pair-bound.ts`:
  the bound compares the blocks around the first line carrying a pair's wording,
  so a refrain's second block is never compared;
- `archive-revision-footnotes.ts`:
  a block is located by the first place its text stands on the page;
- `panel-stage.ts`:
  ballots are keyed by model id against a roster nothing holds distinct for the panel,
  the critics or the page title bench,
  as it is held for editors and checkers;
- `corpus-run/artifact-two-lane-read-naturalness-confirmation.ts`:
  a stored confirmation is bound to the first decisive round carrying its candidate;
- `corpus-run/probe-relabel-artifact.ts`:
  tallies are keyed by envelope id from a stored record the reader does not check;
- `corpus-run/probe-relabel-case.ts` and `corpus-run/probe-relabel-control.ts`:
  the first slice carrying a region's replaced text is taken;
- `corpus-run/window-trial-report.ts`:
  a later ledger row for an arm replaces an older one;
- `corpus-run/rendering-audit-settled-repeat.ts`:
  a second run's rows are keyed by subject from a file the reader casts and does not check;
- `grade-agreement.ts`,
  in a file that was another change's:
  a pre-grade file naming one position twice and carrying one row too many passes both checks,
  and the later verdict for that position is scored;
- `introduced-defect-screen.ts`,
  likewise:
  a prober answering one region twice with two verdicts is counted by the first,
  with no finding.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B204: every link to an href was given the archive's last text for it

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census,
reading the maps built from typed pairs.
`linkedTextPairs` (`page-name-glossary.ts`) keyed the archive's link texts by href,
and a map built from pairs keeps the last pair for a key.
An href names no one link:
a page can link one destination under a name in one sentence and under other words in another,
and every link of the original to that href then read the archive's last text for it as its rendering.

The fix:
`renderedLinks` (`page-name-link-rendering.ts`,
a module of its own with its own test file,
since the glossary file was near its line cap)
gives a link the archive's one text for its href where there is one,
the archive's link at the same place where both documents link the href equally often,
and no pairing where the counts differ,
the rule ledger B59 gave `corpus-run/title-reference-link.ts`.
A mutant putting the last text back failed the new module's place case and its count case
(the fixing agent's run).

No page changes
(the fixing agent's measurements at the pin):
none of 92 archive pages links one href under more than one text,
and the builds before and after the fix write the same 395 glossary lines over the 92 pairs.
A pin that adds such a page could move a cached decision.

Open to the owner's veto:
a link whose rendering cannot be read gets no glossary line,
and its title is left to the page title lexicon,
where the other way keeps the first or the last text.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B210: a tag attribute was restored from the first of two the archive's tag names

Red in `49e3b8dfd`,
fixed in `21f2ec369`.

Found on 2026-10-05 (UTC) by the same census.
`attributeRewrites` (`corpus-run/jsx-attribute-restore.ts`) took the archive tag's first attribute of a name
and wrote its value over the bench's.
The archive's tag is text a person wrote,
and where it names an attribute twice,
which value a reader sees depends on what renders the tag.

The fix:
such an attribute is left as the bench wrote it,
and the tag's other attributes are restored as before.

No page changes
(the fixing agent's count at the pin):
of 118 archive tags,
none names an attribute twice.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B228: a title linked twice in one slice had its second link's rendering never located

Red in `51f6dd875`,
fixed in `65bd3a645`.

Found on 2026-10-05 (UTC)
by the first-match census of B203,
which described the site and left it.
`titleLink` (`corpus-run/title-reference-link.ts`) took the first link of the original whose text is the title.
Where one slice links a title twice,
to one destination or to two,
only the first link's rendering was located,
and the second stayed as the bench wrote it,
so a page could keep one title in two renderings.

The fix:
`titleLinks` gathers every closed link of the title,
and `locateLinks` locates each by the rule the file already had (ledger B59),
the page's link at the same place among the links to one destination,
and returns the located renderings in page order.
Where any link of the title is ambiguous the slice is,
since rewriting the located links and leaving the other would leave the title said two ways;
a link whose destination the page does not link at all is left as the page has it.
Three cases failed on the lead's red tree:
the two that begin "READS EVERY LINK OF THE TITLE",
for one destination and for two,
and the one that begins
"STANDS ASIDE WHERE ONE OF THE TITLE'S TWO DESTINATIONS IS LINKED A DIFFERENT NUMBER OF TIMES on the page".
A fourth,
for a page that dropped one of the title's two destinations,
passed before the fix and stays as a guard.

Not measured:
how many slices of the pinned corpus link one title twice.
The agent's read of the corpus clone was refused by its permission check,
and the lead did not run the count in its place (M118);
the count stays open for the owner to authorize.

The whole suite ran on `65bd3a645` in a side checkout,
which holds the fixes of B228 to B230:
1,621 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.
No cache version moved,
for this entry or for B229 and B230:
no stored record exists under a current version (the audit named in B189).

Open to the owner's veto:
a slice is ambiguous when any link of the title is;
and the located renderings are sorted into page order.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B229: the bilingual pair bound compared only the first place a refrain's wording stands

Red in `51f6dd875`,
fixed in `65bd3a645`.

Found on 2026-10-05 (UTC)
by the first-match census of B203.
`lineAt` (`bilingual-pair-bound.ts`) took the first line carrying a pair's English wording,
in the page and in the rendering.
Where the wording stands on more than one line,
a refrain,
only the first blocks were compared,
so a rendering that widened the refrain's second block was not refused;
and a rendering that repeated the line before the pair was measured against a block it could not be paired with
(the agent's probes before the fix).

The fix:
`linesAt` returns every place.
Where the page and the rendering carry the wording equally often,
the places are compared in order;
where they do not,
the bound says nothing for that pair and the repeated-line check speaks.
Two cases failed on the lead's red tree:
"REFUSES a rendering that widens the SECOND block of a refrain the page carries in two places"
and "STAYS SILENT ON THE PAIR where the rendering carries the wording on more lines than the page does,
leaving the repeat to the repeated-line check".

The agent's first patch kept a set of the wordings already compared,
keyed by the English wording alone,
so the second of two pairs sharing one English wording was skipped where the page kept the first pair's Han line only.
The lead's review found it.
The agent dropped the set,
each pair is read on its own,
and a set of the rendering blocks already named keeps one block from being named twice.
The case for it,
"REFUSES a widened block through the second of two pairs sharing one English wording,
where the page kept the first pair's Han line only",
was red against that first patch (the agent's run).

Not measured:
how often a wording stands on more than one line of a slice,
and what the bound says there where the counts differ,
for the reason B228 gives.
The choice of silence where the counts differ therefore rests on the file's own rule,
that nothing is refused where a block cannot be paired,
and on fixtures,
and is the first thing to read again once the count is taken.
The whole suite's run on `65bd3a645` is recorded in B228.

Open to the owner's veto:
silence where the two texts carry a wording a different number of times.

Open when this entry was written,
closed in B255 (`51ccd8680`):
the repeated-line finding prints a number before a counted noun (the agent's note),
handed to a follow-up launched on 2026-10-06 (UTC).

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B230: an archive block's revision was checked where the block's text first stands on the page

Fixed in `65bd3a645`.
Its cases need a parameter the fix adds,
so they were committed with it,
and their red runs are the agent's alone.

Found on 2026-10-05 (UTC)
by the first-match census of B203.
`revisionFootnoteFindings` (`archive-revision-footnotes.ts`) found the block again
with `indexOf` of its text in the page.
A block whose text stands on the page twice had its revision tried at the first place.
With the first copy inside a code fence,
where a marker references nothing,
and the second in the body,
where it references a note,
a revision dropping the marker from the body's block left no finding
(the agent's probe before the fix).

The fix:
the caller's offset comes through `runArchiveBlockReviewStage` and `replacementCandidates`
as a required `blockOffset`,
which `corpus-run/archive-block-repair.ts`,
the stage's one production caller,
fills from the block it cut out;
and the floor throws `BlockOutsideArchiveError` where the page does not hold the block at that offset.
The cases:
"READS THE REVISION AT THE OFFSET THE CALLER GIVES:
a block that stands twice,
the first inside a code fence,
loses the marker the note hangs on at its second place",
"REFUSES an offset at which the page does not carry the block,
though it carries the block elsewhere",
and the same block read at its first place,
which passed before the fix.
The whole suite's run on `65bd3a645` is recorded in B228.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text":
a caller that cut a block out passes where it stands,
and the callee does not find it again by its text.

## B263: the page-name glossary gave the first of two renderings as the page's

Red in `76c75dce9`,
fixed in `a6d1fe4f7`.

Found on 2026-10-06 (UTC) by a second census of lookups keyed by text,
over the surfaces the census of B203 did not read:
map writes,
loops that return at a first match,
sets used to drop repeats,
and keys of parsed objects.
`pageNamePairs` (`page-name-glossary.ts`) pairs links,
signatures and headings by the original's text,
and kept the first pair for an original text with a filter that remembered the texts it had met.
The archive renders one original text two ways where a title repeats as two headings
or one name is linked to two addresses,
and the sheet then gave the first rendering as the page's and said nothing of the other.

The fix:
an original text with more than one distinct rendering is left out of the sheet's lines and of `pairedPageNames`,
so the stage that settles repeated titles reads it;
a text every pairing renders alike keeps its first pair.
The lead's edit:
the fix walks the groups once,
where the agent's filter rebuilt a group's renderings for every pair,
with the same result and order.
On the lead's red tree `page-name-glossary.unit.test.ts` failed by an assertion on the sheet's lines.

The agent's count,
not rerun by the lead:
at the pinned corpus,
312 glossary pairs over 92 page pairs hold 310 distinct original texts and none under more than one rendering,
so no sheet changes there.
Measured by the lead for B263 to B265:
26 named test files pass,
with the 38 source scans and a clean type check.
The whole suite ran on `a6d1fe4f7` in a side checkout:
1,634 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 38 source scans passing.

Open:
the contributor-name authorities keep the first signature of a name signed twice
(none at the pin,
by the agent's count).

Open to the owner's veto:
a conflicted text is left out,
where keeping every pairing as its own line or a finding are the alternatives
(the module has no findings channel).

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".

## B264: one voice naming a footnote-definition pair twice counted as two voices

Red in `76c75dce9`,
fixed in `a6d1fe4f7`.

Found by the census of B263.
`readBlockPairing` (`pair-blocks-wire.ts`) refused a pair repeated next to itself,
and a body pair named again after others steps backwards,
but a footnote-definition pair is outside the order rule,
so a reply could name one again after another pair.
`agreePairs` counts each naming across the voices' pairings as a vote,
so one voice naming a definition pair twice agreed with itself at two votes needed
(the agent's run of the unfixed build).

The fix:
the reader refuses a correspondence named twice anywhere in a reply,
in the words it already used for an adjacent repeat.
Its TSDoc,
which said a translation block may not appear twice against the merge it accepts,
says what it does.
On the lead's red tree `pair-blocks-wire.unit.test.ts` failed because the expected refusal did not come.

Not measured:
whether a live model does this.

Open:
a doubled key in parsed JSON keeps the later value unseen,
which needs a reader aware of repeated keys and is a decision of its own.

Open to the owner's veto:
the repeat is refused in the reader,
which costs a voice that repeats a definition pair,
where counting a pair once per voice is the alternative.

Recurrence:
`mistake-prevention.md`,
"Tables keyed by text".
