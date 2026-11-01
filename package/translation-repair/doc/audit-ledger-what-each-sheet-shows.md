# Audit ledger: what each sheet shows

Sheets that lacked a block their role needs to judge or write.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B28: sheets missing a block their role needs (B21, family six)

Status:
fixed,
2026-09-29.
The full suite passed on `b01129566`,
the entry's last commit,
with no failing case among 1347 passing groups.
The census rendered every model-facing sheet the rendered-sheets fixtures build (38)
and asked which shared blocks each carries
(`~/temp/agent/f6-sheets/`:
`constants.ts` over every exported string constant of 80 characters or more,
`shared-lines.ts` over every line of 60 or more on two sheets or more,
grouped by the sheets carrying it,
and `context-presence.ts` over the fixture's identity and reference lines).
The fixture decides what context a builder is given,
so every gap a parameterized block showed was traced to the production caller before it counted.

### Roles

Writers write English that may ship:
the translate writer,
the consolidation writer,
the editor,
the refiner,
the translate repair turn,
and the archive block review,
whose reviewers write replacement text.
Wording judges choose among wordings:
the lane contest,
both gates,
and every selection slate,
the typed decision included.
Claim judges file or weigh claims against a rendering:
the critic,
the panel,
the resolution judge,
coverage,
the rendering audit,
the naturalness review,
and the introduced-defect probe.
The page-title lexicon writes the English every passage uses for a title the original names,
under the house rules.
Readers extract or align and write nothing that ships:
the picture readers,
reference attestation and its confirmation,
and section and block pairing.
The restoration judge and the derivability probe are measurement probes.

### Findings fixed

- The archive block review lists the apparatus kinds without the shared narrative bound
  (`NARRATIVE_DETAIL_IS_NOT_APPARATUS`);
  its own sentence names biography,
  events and dialogue,
  and leaves out a method,
  a time,
  a cause and a characterization.
  S5's fix put the bound on every sheet that files,
  votes on or writes against the archive.
- The introduced-defect probe excuses dropped wording as page apparatus without the bound,
  the misreading the bound was written to stop (class one hundred eight).
- The archive block review and its correction slate carry neither the declared names nor the cited references,
  though both exist when `pass-prepare.ts` calls `repairArchiveBlocks`
  (the reference context is built before the call,
  and the declared-name lines are passed to preparation beside it).
  The review judges archive wording against the original,
  which `slice-context.md` says every such sheet does with the references beside it,
  and the house rules it carries tell it to read a pronoun line and footnote vocabulary in a DECLARED NAMES block it never sees.
  Of 381 stored run logs under `~/temp/agent/` in which the review ran,
  95 are entries citing at least one reference,
  and 73 of those reached a correction slate,
  the path on which a detail a cited page states could be removed.
  The review's outcome findings reach stored files too rarely
  (2 files hold any)
  to count what the slates chose.
- Coverage finds "a fact,
  a name,
  a number" of a Chinese passage in the English
  without the declared names its house rules point to,
  so a passage anchored mainly by a declared name can read as uncovered.
  Preparation holds the identity where `pass-insertion-admission.ts` calls the stage.
- The page-title lexicon writes shipped title renderings under house rules that say to use vocabulary a note line in the DECLARED NAMES block establishes,
  and to give a work its official English title,
  and gets no such block:
  it runs before preparation,
  which built the identity lines inline,
  so the web lookups bought for the works the original names never reached it.
  No stored production run records a lexicon round:
  the 27 files under `~/temp/agent/` that log one are this audit's own full-suite logs,
  and no slice-cache file has been written since the lexicon's cache version was set.
- The refine slates carry the house rules and none of the declared names,
  though `refine-stage.ts` holds them and hands them to the refiner,
  which is told a handle survives exactly:
  the judges choosing among its rewrites could prefer one that respells a declared handle or changes a declared pronoun.
  This one was read as sound in the first pass of this entry,
  which checked the refine slates for the apparatus bound only.
- The rendered-sheets fixtures rendered 13 of their 38 sheets naming the DECLARED NAMES block without it:
  the editor,
  the translate and repair slates,
  the typed decision and the rendering audit,
  which production gives the names,
  the three refine slates,
  which it did not,
  and the house rules and the restoration judge,
  which rightly carry none.
  A guard reading the rendered sheets could not tell a sheet that lacks the block from one the fixture left bare,
  which is how the coverage,
  lexicon and refine gaps went unseen.

The narrative bound:
red `f8f5ec907`
(every rendered sheet listing the kinds carries the bound),
fix `678b1d725`.
The archive review's editorial-context line now states the shared bound,
keeping its own sentence for biography and quoted dialogue,
and the probe's apparatus excuse carries the bound after it.
The probe runs inside the repair and refine slice caches,
so both key files record the change riding inside versions 34 and 5:
no slice-cache file was written after 04:27 UTC on 2026-09-27,
where a control from midnight that day finds 494.

The declared names and references:
red `590a98921`,
fix `06c6d22ad`.
`pass-prepare.ts` passes both through `repairArchiveBlocks` to the review,
whose sheet,
correction slate and naturalness read now carry them;
the slate's first criterion removes a claim supported neither by the original nor by a page it cites,
since earlier criteria outrank later ones.
A retention may anchor in one cited page's own text
(`isArchiveReferenceQuoteAnchored`,
the source anchor's fold and minimum);
`referencePageTexts` in `reference-line-head.ts` cuts each line's head and address
and skips the attested lines,
each of which quotes the archive back,
and the lookup's failure notes,
whose wording the lookup now writes from constants in the same module.
The mutation batch (`b28-mutants.json`,
fourteen mutants over the anchor,
the page reader,
every thread,
the criterion and both bounds)
caught all fourteen,
and its comment control survived.
The full suite passed on `06c6d22ad` with no failing case.

Coverage:
red `1176f4584`,
fix `628d8ff53`.
`pass-insertion-admission.ts` passes the identity preparation holds to the coverage stage,
whose sheet carries it as a DECLARED NAMES block ahead of the passage,
with the shared declared-identity rules and one line saying a name the English writes in its declared form states the name the passage writes.
The coverage probe and control keep asking without it:
they measure the sheet over seeded passages.

The page-title lexicon:
red `2516419a4`,
fix `7bf2db51a`.
`page-identity-lines.ts` assembles the identity lines in one place,
which preparation reads,
and so does the preparation pass for the lexicon,
with every context line but the lexicon's own:
the work-title lookups and the names other entries declare.
The lexicon sheet shows them with the declared-identity rules,
its cache key hashes the identity it showed,
and the page-title cache version's account records why the change rides inside version 1.

The three sheets' role lines,
the one line each adds after the shared declared-identity rules,
became named constants in `355aaf84f`,
matching the literals they replaced character for character,
so a case can require a line without copying its wording.
A mutation batch over the coverage and lexicon threads
(`b28-coverage-lexicon-mutants.json`,
23 mutants)
caught 15,
and the comment control survived with 7 real survivors:
the follow-up coverage sheet without the rules,
both sheets choosing a fence without the identity,
both role lines dropped,
the lexicon's identity without other entries' names or read off the original in place of the archive,
and the shared assembly without its notes or its pronoun line.
`d37228134` added a case aimed at each,
with `sheet-fence.test-fixture.ts` reading the fence a sheet opened a block with,
so the fence cases need not ask `selectFence` for the answer,
and cases on the archive review's role line and fence,
which the first batch never mutated.
The rerun
(`b28-survivor-mutants.json`,
15 mutants:
those 7,
the archive review's role line and both of its fence inputs,
and three more parts of the assembly and its order)
caught all 15,
and its comment control survived.

The refine slates:
red `d24ba1401`,
fix `02adee08f`.
`buildRefineSelectionContext` takes the identity and shows it in every mode,
ahead of the references,
and the stage passes it.
The entry comes from `declared-names-evidence.ts`,
the repair slates' labelled entry moved out of `repair-selection-evidence.ts`
(a probe over the built index finds the shared label equal to the repair literal it replaced),
which the archive correction slate now reads in place of the bare heading,
since its criteria never mention names.
The refine key already hashes the identity,
and refine version 5's account records the change riding inside it.
The mutation batch
(`b28-refine-mutants.json`,
7 mutants:
the stage's thread,
the names in the context,
the correction mode without them,
the shared entry returning nothing,
and the repair,
archive and translate slates without the names)
caught all 7,
the translate slate's on a rerun after its pattern was corrected for the move's indent,
and its comment control survived.

The fixture and a guard:
`8aaedecf5` moves the translate slate's evidence assembly unchanged into `translateSlateEvidence`
(the 78 lines at the previous commit,
dedented,
equal the new function's array body),
and `a271f6f8a` renders every sheet with the context production gives it:
the repair and translate slates and the typed decision through the evidence functions production calls,
and the editor,
the rendering audit and the refine slates with what their callers pass.
`rendered-sheets-context.unit.test.ts` then requires every rendered sheet naming the DECLARED NAMES block to carry the fixture's names,
and every sheet naming CITED REFERENCES the fixture's references,
with the house rules and the restoration judge exempt by name and reason,
and a case failing on a stale exemption.
With the editor's context taken out of the fixture,
the guard names the editor;
the fixture restored,
it passes.

The cache-account audit
(`mise run cache-account-audit`)
reports every source commit since the seven versions were set riding inside all seven,
the archive review and coverage changes among them,
since no slice-cache file has been written since.

### Read and left as they are

The editor and the resolution judge act on issues the critic and panel classed,
and both of those carry the kinds and the bound.
The refiner and the refine slates judge against the current text,
where dropping a gloss is "dropped",
so the apparatus bound adds nothing there;
what the refine slates lacked was the declared names,
fixed in `02adee08f`.
The translate slates label the names with the bare heading,
since their own criteria state the declared-name rules
(`translate-selection-sheet.ts`).
Production passes the translate slates,
the repair slates and the editor the declared names and references,
and the rendering audit the declared names
(`translate-judge.ts`,
`editor-ensemble.ts`,
`repair-editor-stage.ts`,
`corpus-run/rendering-audit-settled-buy.ts`);
the first pass of this entry cited `rendering-audit.ts` for the last,
which passes whatever subject its caller builds.
The readers,
the attestation and the pairing sheets carry no house rules,
and write nothing that ships.

## B283: the `issues-absent` arm of `probe-relabel` sent the production disclosure with an empty list

Fixed in `abb506b7b`,
with its case,
which the agent ran red in its worktree.

Found by the move B282 records.
The arm that claims no issue list is known sent the production disclosure with an empty list,
which under a rendered disclosure writes the heading for a list into the prompt.

The fix:
it sends the withheld disclosure.
Production's setting is withheld today,
so today's output is unchanged.

Recurrence:
`mistake-prevention.md`,
"What each sheet is shown".

## B419: no judge was shown which wordings narrate an ended life in the present tense

Red `a1f98e76d4`,
fixed in `afeac61dbb`.

Found on 2026-10-11 (UTC) in the read of TianqiChen66622,
which shipped the archive's present-tense narration of its subject
because the judges left the archive standing.
The census of that day over the 92 pinned archive pages
labelled 80 of 536 detector hits by hand:
about 68% precision when traits of the person count,
about 24% when only narrated acts count.
A detector that weak cannot refuse a wording or mark a text flawed,
so the owner chose to show its hits to the judges as candidates.

The fix:
`present-tense-candidates.ts`,
with the readings it stands on
(`present-tense-text.ts`,
`present-tense-words.ts`,
`present-tense-vocabulary.ts`),
returns each candidate with its sentence,
matched words and rule,
skips quotations,
blockquotes,
headings,
code and HTML through the shared slice grammar,
and reads nothing where the entry records no death.
The subject is every name the translation declares,
the front matter `name` and each alias in `info.alias`
(`subject-names.ts`;
a comma-joined string or a list),
the longest tried first.
`corpus-run/entry-info-record.ts` reads `people/<id>/info.yml` at the pin;
`identity-context.ts` reads `died` by the rule every identity field follows,
a bare year included,
and the identity block every stage reads names the date.
The lane contest,
the translate slate,
the consolidation slate,
its run-off over the tied finalists
and the consolidation gate
print a labelled tense block for each text with a candidate
(`tense-candidate-block.ts`,
each sentence written as a JSON string);
no verdict,
flawed-archive count or run-off reads it.
Artifact generation 15 records the candidates for the audit trail,
on a contest slice (`lane-contest-tense.ts`)
and,
for the wording that shipped,
on a consolidation slice (`consolidate-shipped-tense.ts`);
the readers refuse the key in generation 14 and earlier,
on a `sole-lane` slice,
which asks no judge,
and on a consolidation slice that ships no wording of its own.
Replays tell the death only for a generation 15 artifact,
whose models saw it,
so an earlier artifact replays with the block its models saw.
No cache version moved:
each affected stage's run shape names the death and the subject names,
so an entry that records a death asks a new question under a new key,
and the three pinned-key witnesses keep their strings.

The port onto the branch head found a defect in the first draft:
its reader allowed the key from generation 14,
while the generation 15 `sole-lane` path (B425) returned before the key was read,
and its writer wrote the key on every slice,
so a sole-lane record would have been read with the key dropped.
The writer now skips a sole-lane slice and the reader refuses the key there.
The same port first logged an absent `info.yml` to standard output in two built commands whose terminal output is pinned;
the replays now record an absence in silence.

Decided by the owner on 2026-10-11 (UTC):
every model sees the death date;
the record holds candidates on contest and consolidation slices;
the subject names are the front matter `name` and its declared aliases.
Decided by the lead under the owner's instruction of the same day:
generation 15 alone;
no cache version bump;
a bare year counts as a recorded death;
the replay readers take the death date,
`corpus-run/page-republish.ts` included,
whose page never depends on the identity block
but whose pass already fails on a damaged `info.yml` at prepare.

## B442: the window trial judged on sheets with no death and no tense block

Red `a08a6634b7`,
fixed in `62a07ceeb9`.

`runSliceArms` called `judgeTranslateSlate` with no ended life,
and `drawEntry` prepared the pair with no `sourceEntryInfo` and never read `info.yml`,
so the instrument measured sheets the pass does not ask.
`drawEntry` now reads the entry's `info.yml` after both pages
and returns the page facts of its preparation,
which `runWindowTrial`,
`runPick` and `runSliceArms` pass on as a required parameter;
the judges get all of them,
and the translators the identity block,
as the pass gives them.
`TRIAL_PROTOCOL_VERSION` is 2,
so a ledger bought on the older sheets is not resumed;
the trial reads no slice cache.

## B445: the consistency judges were not shown the tense candidates of a life that has ended

Red `54d53e8b46`,
fixed in `d4736ccb6c`.

The consistency judges choose one rendering and carry it to every place,
yet their sheet showed no tense candidate when the entry records a death.
The sheet now prints the labelled tense block of the English of each slice it shows,
once per slice under the first place's label,
built by the same `tenseCandidateGroups` and `tenseCandidateLines` the other sheets use
(`consistency-tense.ts`),
inside the fence the sheet chooses against all its texts.
`consistencyKey` adds the death flag and the subject names only when the entry records a death,
so every other key keeps its bytes,
and `CONSISTENCY_CACHE_VERSION` stays 1,
since no pass has run at it.

## B697: reader protection reached past the account of how a death or an attempt happened

Status:
fixed on 2026-10-20 (UTC) on the owner's ruling,
red `f497ca2a20`,
fix `49553a32bb`,
its scope comment and rule example amended by `7071a8eb5b` and restored in part by `abf7fdae89` (M283),
integrated in the tree at `191a8bc38d`.
`28f4b8fc75` moved seven cache versions for the fixes of the XingZ6015 read
and names this scope first among its reasons:
`TRANSLATE_SLICE_CACHE_VERSION` 19 to 20,
`PAGE_TITLE_CACHE_VERSION` 3 to 4,
`CONSISTENCY_CACHE_VERSION` 5 to 6,
`LANE_CONTEST_CACHE_VERSION` 10 to 11,
`REFINE_CACHE_VERSION` 8 to 9,
`SLICE_CACHE_VERSION` 37 to 38
and `CONSOLIDATE_CACHE_VERSION` 25 to 26.
`PAIRING_CACHE_VERSION` stays 3,
since no pairing sheet splices the house rule.

Found in the read of XingZ6015,
lane contest slice 0 of XingZ60.
The page opens with her own words about doing things with a potassium cyanide capsule in her mouth (含着氰化钾胶囊).
It is not an account of how she died,
and the archive keeps the capsule.
Two of three contest judges chose the repair wording,
"I can even do something knowing that it could cost me my life",
which drops the capsule,
and cited reader protection:
the house rule opened "when a death or an attempt was by suicide"
and set no limit on where on the page a means could stand,
so a means named anywhere fell under it.

The question went to the owner at about 00:36 UTC on 2026-10-20:
does reader protection cover a suicide means mentioned outside the account of the death or an attempt?
The owner chose "Only the death" and rejected "Anywhere on the page":
"Scope the rule to how she died or attempted,
so her quote keeps the cyanide capsule as the original and the archive have it."

The rule is now scoped to the account of how a death or an attempt happened.
A means named elsewhere (a person's own words about what they might do,
a hypothetical,
a quoted work,
a story)
is rendered as the original states it.
The replicability test,
medication without substance or dose,
method words,
euthanasia and the immediate medical cause stay,
inside that scope.
The scope reached every sheet that restates the rule:

- the house rule carries the scope sentence and the owner's ruling as a comment,
  and its judges' tail and measuring tail say that a candidate vaguer about a means named elsewhere has left out what
  the original states;
- the attested-details rule of the translate floor,
  the three cited-reference rules of the slice reading (critic and panel,
  candidate and audit variants),
  the editor's precedence sentence in the repair lane,
  and the resolution checker's not-fixed answer in the adjudication package each carry the same limit.

The red test reads the rendered sheets through a map.
Before the fix all 8 of its cases failed:
the house rule carried no scope sentence,
44 rendered sheets named the rule unscoped,
and each restatement still listed a method,
a substance and a dose with no limit.

Recurrence:
none of an earlier entry.
A rule written for one case (a death by suicide) is read for the places it is spliced into,
and a ruling of the owner is quoted whole,
with the option it rejected,
into the comment over the rule.

## B703: the cut of the notes between translators left a bare quote line, joined two lines and dropped a carriage return

Status:
three fixes of 2026-10-20 (UTC) to `withoutTranslatorsNotes` in document-structure,
the cut every judging sheet passes the archive and the candidates through.
The quoted line:
red `1f66cad0ec`,
fix `53d68d85dc`,
versions `aa0ab6ff0a`,
pinned keys `8d449d05fe`,
landed as `cca7711de7` after gate g122.
Two notes on one line:
red `5b75add81a`,
fix `d60b27a9ed`,
gap reading `0039830674`.
The carriage return:
red `acf6ada605`,
fix `e80301bf32`.
Version accounts `4f9f25ae24`;
the last two landed as `a5dc23c5bb` after gate g123
(2564 `[PASS]` lines,
no `[FAIL]` line).

### A note alone on a quoted line

The cut took a note's whole line only when nothing but whitespace stood before it,
so on a line inside a blockquote the markers stood before it and the cut kept them:
zheermao101's archive line `>   > {/* ... */}` became a bare `>   >` line,
which a judge reads as a paragraph break inside the quote that the archive does not have.
The whole line now goes when what stands before the note is quote markers and whitespace only
and only whitespace stands after it.
The test reads the characters before the note and never the note's own words,
and it needs a marker,
so a `>` that is text (`1 > 0 `) keeps the old cut.
Of the 34 notes in the 92 pinned English archives,
one stands alone on a quoted line (zheermao101).

The keys of the five stages whose sheets make the cut hash the version,
the run shape and the texts,
not the sheet,
so a record written under the old cut would be served under the new.
`CONSOLIDATE_CACHE_VERSION` moved from 26 to 27,
`LANE_CONTEST_CACHE_VERSION` from 11 to 12,
`REFINE_CACHE_VERSION` from 9 to 10,
`SLICE_CACHE_VERSION` from 38 to 39
and `TRANSLATE_SLICE_CACHE_VERSION` from 20 to 21.
The account of each move says the check that would let the change ride inside the old number could not be made,
since the run `zheermao15` had an empty slice-cache directory and so did a control from 2026-10-19.
That reading lacked its cause:
a settled entry discards its own slice cache (`retireSettledEntryCache` in
`corpus-pass/src/corpus-run/pass-entry-caches.ts`),
and both runs settled,
so their directories could never have shown a record.
The check reads the `START tip=` line of each run begun since a number was set instead
([stored files and cached decisions](mistake-prevention-stored-files-and-cached-decisions.md)).
Moving the numbers costs a cold cache and the pinned keys,
and no record was lost by it.

### Two notes on one line

The cut judged each note's line from the end of the previous cut,
so the second of two notes on one line read an empty head,
counted as alone on its line and took the line break:
`x <note> <note>\ny\n` gave `x y\n`,
and on a quoted line two lines became one.
The same reading left a space at the end of a text and two spaces between words.
Notes with only whitespace that is not a line feed between them are now cut as one stretch (`noteStretchesOf`),
judged against the original text.
The fix commit's message says spaces and tabs;
the stretch also joins across a carriage return,
and `0039830674` says so in its message and in the TSDoc,
and reads the gap with `isWhitespace` instead of the trimmed-text comparison the blank-text scan lists.

### The carriage return of a CRLF line

The cut of a note that ends its line ran to the line feed,
so in a CRLF text it took the carriage return and left a bare line feed among CRLF breaks.
It now stops before a carriage return that ends the line.

### What the last two changed

None of the 92 pinned archives has two notes on one line or a carriage return
(cut with the old and the new code,
no archive's text differs),
so no sheet text changed,
and no record had been written at the new numbers:
the only run begun since they were set,
`CuspariaKLSY18`,
started at tip `a6a0ed7dc0`,
which predates them.
Both fixes ride inside the numbers,
and each version's account names their commits,
so the cache-account audit counts them as accounted.
The audit still lists `db2cd279a3`,
a lint-only split of one test in the same file,
and `4f9f25ae24`,
the accounts themselves;
neither changes what any stage does.

Recurrence:
none of an earlier entry.
A cut that removes text from a line is tested on the line forms the corpus can hold
(quoted lines,
two marks on one line,
CRLF ends),
not only on the forms the pinned archives show,
and a check whose evidence is a cache directory first reads what empties that directory.

## B704: the absolute naturalness review showed its reviewers the notes between translators

Status:
fixed on 2026-10-20 (UTC),
red `984b1f030a`,
fix `b3e2ec47fa`,
`CONSOLIDATE_CACHE_VERSION` 27 to 28 in `6a28f54beb`,
pinned key `bfe7cee84f`,
gated with the fixes of B705 and B706 as g125
(2565 `[PASS]` lines,
no `[FAIL]` line)
and landed in `eaa250287d`.

`buildAbsoluteNaturalnessReviewMessages` (`archive/src/absolute-naturalness-review-wire.ts`)
showed the candidate and each numbered paragraph with every note,
where the account of `CONSOLIDATE_CACHE_VERSION` 26 said judges no longer see one.
Its two callers are the final polish of a consolidation (`polishConsolidation`)
and the archive block review (`recordArchiveBlockNaturalness`).
A reviewer could raise a Chinese comment in an English text as a naturalness finding.
No such finding could delete or alter a note:
the polish ships its initial text whatever the verdict
(`consolidation-polish.ts`,
`correctionCount: 0`),
the archive block review keeps the block it reviewed,
and the block repair passes no prior findings,
so the findings are evidence only.

The sheet now cuts the candidate and each paragraph with `withoutTranslatorsNotes` against the original,
where the sheet text is built;
the subject stays exact,
since the review digests `candidateText` and `paragraphs`
and the artifact reader recomputes those digests from the shipped text.
A paragraph of nothing but notes is left off the sheet and its number goes unused,
so the stage's paragraph-count check and the reader's recompute still agree;
a reviewer could cite the unused number and pass the check,
and since findings report only,
that stays.
Of the 92 pinned archives 10 hold a note their original lacks (34 notes),
and only 4 numbered paragraphs hold one (XingZ60 1,
shihai4h 2,
zheermao101 1);
the other 30 notes stood only in the candidate fence.

The review's verdicts are stored in the consolidation settlement,
keyed by `consolidateSliceKey`,
which hashes the version,
the run shape and the texts,
not the sheet,
and `zheermao16` was writing records at 27 when the change was made,
so the number moved.

Recurrence:
of B698 and B702.
The census those entries call for,
every sheet the rendered-sheets fixture renders read for what it shows,
had not been run;
it was started on 2026-10-20 (UTC) as its own piece of work,
and its entry records what it found.

## B707: a judged polish re-romanized a Han name the archive already spells in its sentence

Status:
fixed on 2026-10-20 (UTC),
red `b7ee52b99f` (rendering-audit) and `7d3de9bc2b` (translate-floor),
fix `999f9a5a07` (rendering-audit) and `4ab33543fd` (translate-floor),
versions `89ff91202e`,
source scans held to the new modules `b81c866f25`,
gated as g126b
(2578 `[PASS]` lines,
no `[FAIL]` line)
and landed in `e429659e0b`.

Found in the run `zheermao16`,
slice 13 of zheermao101.
The archive's sentence for the one that cites 寒涟漪 names its writer "Han Lianyi".
The page shipped "Han Lian Yi (Cold Ripples)".
The lane contest chose the translate lane four to zero,
naming the archive's established rendering,
but the consolidation select,
its refine and the polish gate never saw it as a name's rendering:
the page-name glossary pairs link texts,
signatures and headings only,
the declared pairs are the front matter's name and aliases,
and the signer floor reads signature lines.
A cited author stands in a sentence,
so the house rule for a handle with no declared,
archive or corpus rendering was read as if it applied,
and three of five polish ballots followed its regrouping and its meaning in parentheses.

The pairing goes from the archive's capitalised Latin words to the original's Han by reading letters
(`proseNamePairs` in rendering-audit).
The identity lines list each pair under a heading that says the archive's spelling is the rendering,
so every sheet that splices the lines shows it,
and the floor `proseNameFindings` refuses a candidate that writes the same letters under another spelling.
Over the 92 pinned English archives 69 hold a pair and 60 change their identity block
(the others pair a name a link,
signature,
heading or the front matter already carries),
none of the 92 archives is refused as a candidate against itself,
and 240 of 240 respelled controls are refused.
Of the ten shipped pages under the agent runs one is refused,
the incident's.
The rule text is not changed:
the house rule and its scope sentence already name the case,
and a change to it would move every version whose sheets splice it.

`TRANSLATE_SLICE_CACHE_VERSION` moved from 21 to 22 and `LANE_CONTEST_CACHE_VERSION` from 12 to 13,
since both runs begun since those numbers were set log a tip that holds them.
`CONSOLIDATE_CACHE_VERSION` stays at 28:
the same two runs log a tip that does not hold the commit that set it.
The repair slice,
refine,
consistency,
page title and pairing versions stay,
since their keys hash the identity lines or digest the sheet,
or their stage does not meet the publication rule.

Recurrence:
of the family this ledger records,
a sheet that lacked a block its role needs to judge.
A rule that excepts a name "the archive already renders" is tested on every place the archive can render a name,
sentences included,
and a sheet that carries the exception carries the rendering beside it.

## B708: the judges, critics, auditors and pairers saw the notes between translators

Status:
fixed on 2026-10-20 (UTC),
guard red `88b5187179` and `c655a96182`,
red and fix per package
(`2553c8f8d5` and `12e147ae40` adjudication,
`5498ee860b` its prober module,
`f947ebf950` and `89df3896d4` archive,
`d8b8247e11` and `eaab1b1ed9` stage,
`5283ee9d0b` and `fcd6d13fa3` reference,
`10e1641b51` and `ebac709b03` rendering-audit,
`5c0d5ba784` and `36c5b02884` repair,
`35e421aa8f` and `7ee1755b24` consistency and corpus-pass),
guard green `59b10c6db4`,
versions `43e735f7da`,
`f1571d5012` and `31c0c8d9ad`,
source scans `ff69a94586`,
merged onto main as `1e955f206e`,
gated as g127
(145 steps exited 0,
2605 `[PASS]` lines,
no `[FAIL]` line,
49 source scans pass)
and landed with this entry.

The census B698,
B702 and B704 called for:
every sheet the rendered-sheets fixture renders,
54 in all,
rendered with a note planted in every English text and in a neighbour passage,
and read for whether it shows the note,
cuts it,
or cannot reach it.
A witness string beside each planted note tells a sheet that cuts the note from one the text never reaches.

The sheets that showed a note to a model that judges,
audits,
pairs or chooses:
the critic (`critic-prompt.ts`),
the panel (`adjudicate-prompt.ts`),
the resolution checker (`resolution-wire.ts`),
the introduced-defect prober (`introduced-defect-wire.ts`),
the coverage sheet (`coverage-wire.ts`),
the section and block pairing sheets (`pair-sections-wire.ts`,
`pair-blocks-wire.ts`),
the reference attestation and its confirmation (`reference-attest-wire.ts`,
`reference-attest-confirm-wire.ts`),
the rendering audit (`rendering-audit-prompt.ts`),
the restoration judge (`restoration-judge-wire.ts`)
and the consistency judge,
run-off and carry choice (`consistency-sheet.ts`,
`consistency-run-off.ts`,
`consistency-carry-choice-sheet.ts`).
The critic weighed most:
its prompt names Han left in a translation as `accuracy/untranslated`,
and an accepted issue becomes an envelope the editor rewrites,
so a Chinese comment could be translated
and the carry would then put the note back beside its translation.

The candidate selection slate (`candidate-select-wire.ts`) had its own cause:
it took any evidence block whose heading held the word ORIGINAL for the original,
so two headings that only mention it
(the refine slate's current English and the translate slate's surrounding existing translation)
were shown whole,
and their notes counted as the original's own in the candidates beside them.
The words before a heading's first comma decide now;
the six real original headings stay whole.

Each sheet cuts the notes for display only,
with `withoutTranslatorsNotes` against the original:
stored text,
digests and keys stay on the exact text.
The pairing sheets keep each index and the list length,
listing a block of nothing but notes empty under its own number.
The consistency sheets cut the English of each place,
the chosen paragraph and each candidate,
and a new `sliceSource` on the occurrence view carries the slice's whole original,
so a comment the original holds in a paragraph the sheet does not show is kept.

The writers still show the notes:
the editor,
refiner,
translate writer,
translate repair,
consolidation writer and the three carry writers.
A writer that keeps a note verbatim keeps it,
and one that drops it has it put back by the carry (`archive-note-carry.ts`),
which runs last in page assembly.
The guard `artifact/src/sheet-translators-notes.unit.test.ts` fails on any sheet that shows the note outside its writers list,
on a stale writers entry,
and on any sheet neither reached by the planted witness nor listed as unreached
(house policy,
page title lexicon,
derivability and the four picture readings).

Over the 92 pinned English archives 10 hold a note their original lacks (34 notes),
so the slice-level sheets change for at most those 10 entries;
19 of 63 sections and 28 of 634 blank-line stretches change under the cut,
and 25 stretches are left empty.
9 entries hold a note and a link in the original,
the most the attestation sheets can change.
The consistency sheets were not measured;
their bound is the same 10 entries.
No provider was called,
so the effect of each cut on what a model answers is not measured.

`SLICE_CACHE_VERSION` moved from 39 to 40,
`REFINE_CACHE_VERSION` from 10 to 11,
and `TRANSLATE_SLICE_CACHE_VERSION` to 22,
the number the handle fix of B707 set first,
so the two changes share it:
their keys hash the version,
the run shape and the texts,
not the sheet,
and both runs begun since 39,
10 and 21 were set log a tip that holds them.
`CONSOLIDATE_CACHE_VERSION` rides inside 28 and `LANE_CONTEST_CACHE_VERSION` stays at 13,
whose sheet does not change.
`PAIRING_CACHE_VERSION` stays at 3 under the account of B670,
`CONSISTENCY_CACHE_VERSION` at 6,
since `consistencyKey` hashes the built messages and so asks afresh under the cut sheet,
and `PAGE_TITLE_CACHE_VERSION` is untouched.
The ride-inside checks read the 12 `START tip=` lines of the run logs,
since the cache-account audit read 0 slice-cache records.

Decided with the entry:

- A judge's quote that crosses a place where a note was cut finds nothing in the stored text.
  3 of the 34 notes stand where that can happen (zheermao101,
  XingZ60 and shihai4h),
  two of them in the readiness rotation,
  so the fix is its own piece of work,
  landed before the rotation restarts.
- The entry-notes identity channel (`rendering-audit/src/entry-notes.ts`) still puts each archive comment in the declared
  names context,
  by the owner's rule of 2026-09-02;
  it is a vocabulary channel,
  not the note in the candidate,
  and stays.
- A writer that rewords or translates a note,
  which the carry would not recognise and so would duplicate,
  was not observed;
  the cut critic and panel close the path by which a judge would ask for it,
  and the guard's writers list records the case.
- The version moves follow the convention that a key hashing texts and not the sheet moves its number,
  while pairing keeps the account of B670;
  one policy for all would move constants this change has just accounted for,
  so none is changed.

The body of `43e735f7da` says its pinned keys land "in the next one";
they landed two commits later,
in `31c0c8d9ad`,
after the lane-contest account,
and the commit carries a comment that says so.

Recurrence:
of B698,
B702 and B704.
A cut that keeps text from a model is checked on every sheet that model reads,
by a guard that renders all of them with a planted case and a witness that tells cut from unreached,
and a rule that finds a block by a word in its heading reads the heading's name,
not the whole heading.

## B716: the composite candidate reached the judges with no assessment

Status:
fixed on 2026-10-21 (UTC),
red `882899efa4`,
fix `b7c4a959d5`,
source scans `651ae7846d`.
Found in the review of the archive block review after B712 to B715 met in one tree.
No cache version moved.

What the code did:
`archiveBlockSelectionEvidence` files each reviewer assessment under the candidate whose text the reviewer proposed.
No reviewer proposed the composite of disjoint revisions that B712 added,
so the select sheet listed assessments for the other candidates and none for the composite.
The judges weighed the one candidate they could not check against candidates they could.
The red cases read the select sheet the judges are sent:
the composite stood as candidate 3 of 4 and the list held no entry for it
(two of three cases failed with an empty list;
the case with no composite passed before and after).

The fix:
`compositeOfDisjointRevisions` returns what each revision it carries contributed
(the candidate number and the zero-based lines it changed),
and the evidence copies each contributor's assessments onto the composite,
in slate order,
as the reviewer gave them:
the same disposition and finding,
marked `coversLines`
(the block's lines,
counted from one,
that reviewer changed)
and `fromCandidate`.
The contributors' own entries stay as they were.
The label of the assessments adds,
only when a composite stands,
that its entries are the contributors' own and that no reviewer assessed the composite as a whole.
Every reviewer of a byte-identical revision is carried,
because the copy follows the revision and not the model.
The line arithmetic moved to `archive-composite-lines.ts`,
the composite module being over its line budget.

Decisions taken in this session:
- The composite is found on the slate by object identity.
  A merged byte-identical revision also has the producer kind `composite`,
  which the first draft searched by and which numbered a merged revision as the composite.
  A composite absent from the slate throws `CompositeNotOnSlateError`.
- A fresh reviewer verdict on the composite was rejected:
  it invents an opinion the reviewers did not give and asks the bench again.
- A bare note saying the composite is made of candidates 1 and 3 was rejected:
  the judges would have to join the entries by hand,
  and the reasons are what they lacked.
- Duplicating a finding on the composite and on its contributor is the design.
  The mark `coversLines` is what stops a judge reading it as an opinion of the whole composite.

Source scans:
the first run of the scans after the changes of B716 and B717 failed five scans,
three of them on the new files and classes:
the own-unit-test ratchet (two modules),
the marked-message inventory (three classes)
and the duplicate-body scan (two identical lambdas in the new case).
The other two,
references by position and zone-less clock times,
failed on a planning doc of the repository that this branch does not edit.
`651ae7846d` gives the modules units of their own and inventories the classes.

Cache account:
none moves.
The archive review is not cached and its result is stored under no key (B714),
and the select sheet's evidence text is hashed by no slice-cache key.
`mise run //package/translation-repair/slice-cache:cache-account-audit` on this tree read the same eight versions as B714
(`CONSISTENCY_CACHE_VERSION` 6,
`TRANSLATE_SLICE_CACHE_VERSION` 22,
`REFINE_CACHE_VERSION` 11,
`PAIRING_CACHE_VERSION` 3,
`CONSOLIDATE_CACHE_VERSION` 28,
`SLICE_CACHE_VERSION` 40,
`PAGE_TITLE_CACHE_VERSION` 4,
`LANE_CONTEST_CACHE_VERSION` 13)
and found no slice-cache record in the worktree (0),
which is no evidence either way in a keyless worktree.

Not measured:
whether judges choose the composite more or less often with its assessments shown,
which needs paid calls.
The assessments on the composite are the reviewers' reasons for lines they changed;
a reason that argued about the whole block would read wider than the lines it covers.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"What each sheet is shown".
