import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { IncumbentKind, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Lane contest key
// What makes two runs` contests the SAME contest, for cache purposes.
//
// Split from the driver for the same reasons `repair-slice-key.ts` and
// `translate-slice-key.ts` were: the key is the one piece of a driver that can
// be tested without a client, and a reader of the driver does not want the
// cache reasoning in the middle of it.

/**
 Generation of the contest cache.

 MOVES WHEN THE QUESTION MOVES: the prompt, the schema, the ballot reader, the
 quorum, or anything else that changes what a judge is asked or how its answer
 is read. It does NOT move for a change to how a settled outcome is RECORDED,
 since the ballots on disk still answer the question that bought them.

 VERSION 2 adds target-authoritative contributor spelling to metadata policy.
 Version 1 ballots never answered that question and cannot settle it now.

 VERSION 3 tells judges which syntax candidates deterministic publication
 guard rejects. Earlier ballots spent votes on candidates unable to ship.

 VERSION 4 starts straggler grace at exact-half participation even when
 deterministic eligibility excludes one lane.

 VERSION 5 excludes candidate that drops target-authoritative contributor form.

 VERSION 6 (2026-09-28): version 5 landed on 2026-08-29, and nothing moved it
 through a month of changes to what the judges are asked and how their
 rounds close (ledger M28): the rewrite probe's corroborated claims
 (`b2a39c09a`, `2530537a2`), the eligibility floor's named findings
 (`558b46e11`, `1ff4cb31d`), windowed rounds (`33a023445`) and rounds sized on
 the seats that could answer (`a107c7486`, `29baade8f`), the community
 glossary and rendered line structure (`b7a0b4f5f`, `1a6ebbf62`, `a0fd3cc7c`,
 `070db03dd`), cited references (`20a7272dc`), the dispute note (`85ed2881f`,
 `07170adf0`), apparatus (`7ad1b8ec7`, `2472ec48e`, `e7e3f9c17`), the size note
 (`d94f6787b`), declared names inside linked titles (`645ed9d62`), Canadian
 spelling (`b45000747`), every house rule the judging sheet carries, and the
 prose ranges the eligibility floor's Han-residue check reads (ledger K7).
 Contest ballots were cached under version 5 on fifteen days between
 2026-09-07 and 2026-09-27, each under that day's sheet, so a resumed run
 could read ballots cast on a question no longer asked.

 The glossary audit's corrections ride inside 6 too (ledger C3 to R16,
 `357f534b7`), checked on 2026-09-28 against the newest slice-cache file under
 the agent runs, still 04:26 UTC on 2026-09-27 (eighteen files after 04:20 UTC that
 day, none after 04:27 UTC): the eligibility floor refuses and passes other forms,
 the COMMUNITY RENDERINGS block on the contest sheet counts a rendering
 inflected and bounded, and the grammatical English house rule states the
 doubled preposition. The terms, renderings and whys reach the identity
 context, which the key hashes.

 Rides inside 6 too: the contest sheet no longer shows the accuracy probe's
 claims on a slice whose patch lost, since the repair candidate there never
 carried the patched text (ledger L7); checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 6 too: every gather now keeps a seat that answered unreadably
 out of the same-prompt retry rounds and re-asks each such seat, whichever
 round it came in, in the nudged recovery round (ledger P2, `005692e11`),
 which changes whose voices a round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 6 too: a reply whose complete JSON value more text follows is
 read rather than lost (ledger P8, `cac097368`), which changes whose voices
 every round hears; checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 Rides inside 6 too: a reply that could not be used is re-asked once,
 nudged, of the same model on another provider through the uniqueness
 wrapper's claims (ledger P9, `7011d72cc`), which changes whose voices every
 round hears; checked on 2026-09-28: still no slice-cache file newer than
 04:26 UTC on 2026-09-27.

 Rides inside 6 too: Bedrock reads dry with 1.33 USD still left, and its
 ledger holds every billed attempt at its bound (ledger P1, `2a108dfb3`,
 `107763dbb` and `7cfd5ae4b`), so a Gemma call moves to OpenRouter sooner
 as Bedrock nears its credit, which changes whose voices a round hears;
 checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 6 too: the recovery round re-asks a reply the length
 limit cut with a nudge naming the cut, apart from one off the shape
 (ledger P10, `ce0ef7b51`), which changes what such a seat is asked and
 whose voices a round hears; checked on 2026-09-28: still no slice-cache
 file newer than 04:26 UTC on 2026-09-27.

 Rides inside 6 too: a slice the per-slice hook re-seats under a hold is
 judged by the judges it hands over and keyed by them (ledger X12); a slice
 nobody re-seated keys as before, so no key moves. Same check, same result.

 Rides inside 6 too: the strict parse reads a formula as the site does
 (`5ce9370ac`), and a lane wording forming a formula its original lacks is
 no longer eligible for the contest (ledger X22, `translate-formula.ts`);
 checked on 2026-09-29: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 6 too: the declared-name guard and the Han residue floor
 read text by code point, so a handle written in letters beyond the first
 plane is checked rather than projected to nothing, and an ideograph beyond
 it left in prose is refused (ledger B21, `declared-name-survival.ts`,
 `translate-han-residue.ts`); over the pinned archives and settled pages no
 survival answer changes, and neither they nor any stored artifact carries
 such an ideograph; same check, same result.

 Rides inside 6 too: the validator's link-name floor and contributor
 authority, which judge the contest's winner, carry a declared name only
 where it stands as a name, and the link floor reads a Latin source form, a
 mention and a page handle at handle edges (ledger B23, `name-projection.ts`,
 `11d029fde`); over the would-ship slices of every settled artifact neither
 answer moves that this change explains; checked on 2026-09-29: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 6 too: the validator that decides each lane's eligibility
 refuses prose guillemets and the editor sheet's «REGION marker and line
 heads (ledger B24, `translate-guillemets.ts`, `translate-sheet-leak.ts`);
 one comparison row of 6,285 across the stored artifacts carries a guillemet
 its incumbent lacks; checked on 2026-09-29: the newest slice-cache file is
 still the one of 04:26 UTC on 2026-09-27.

 Rides inside 6 too: the validator that decides each lane's eligibility no
 longer lists the PRIOR FAILED CONSOLIDATION STRATEGY heading among the
 sheet labels it refuses unfenced (ledger B30, `translate-sheet-leak.ts`,
 `32e07afb7`), since no sheet has carried it since the single consolidation
 attempt (`1ba8f713a`); a fenced copy is still refused. An rg for the
 heading over the run directories, the agent runs and the user cache finds
 it only in logs, source copies and build output, in no stored slate,
 artifact or cache record; checked on 2026-09-29: no slice-cache file was
 written after 04:27 UTC on 2026-09-27.

 Rides inside 6 too: a ballot whose findings are left out or are not a list
 of strings keeps its choice, read as no findings, where the contest refused
 it whole (ledger B46, `findingsOrNone` in `contest-ballot-wire.ts`,
 `lane-contest-wire.ts`), which changes whose voices a round hears; checked
 on 2026-09-30: no slice-cache file was written after 04:27 UTC on
 2026-09-27, where a control from midnight that day finds 494.

 Rides inside 6 too: the validator that decides each lane's eligibility
 compares quoted passages at every depth and refuses a candidate carrying
 fewer than its page (ledger B110, `translate-validate-blocks.ts`), which
 changes which lanes reach a contest; checked on 2026-10-01: the newest of
 13,714 slice-cache records under 391 runs directories is still the one of
 04:26 UTC on 2026-09-27.

 THE PRE-LAUNCH CHECK OF 2026-09-28 (ledger M28): 6 was set
 in `d614a0c1d` at 04:30 UTC on 2026-09-28, after the newest slice-cache file under the
 agent runs (04:26 UTC on 2026-09-27), and no slice-cache file has been written
 since, so no answer cached under an earlier question can be served under
 this number. Every source commit since then rides inside it, those the
 accounts in this TSDoc name and those they do not (`corpus-run/cache-account-audit.ts`, ledger M28).

 Rides inside 6 too: the judges of an entry that records a death are shown the
 tense candidates of each text on offer (`tense-candidate-block.ts`,
 `lane-contest-wire.ts`), and the identity context carries the recorded date of
 death (`identity-context.ts`). No answer cached under the earlier question can
 be served under the new one: the run shape of such an entry names the recorded
 death and the subject names, and its identity context differs, so its key
 differs. An entry that records no death asks the question it asked before and
 keeps its key, so the version stays where it is and the answers already bought
 for those entries stay usable.

 7: the validator that decides each lane's eligibility reads prose guillemets
 through the prose mask built from the page parse (`translate-guillemets.ts`,
 `typography-prose-mask.ts`), so a guillemet in a comment, a link or code no
 longer counts and a candidate the grammar refuses has no prose. That changes
 which lanes reach a contest. The check that would let the change ride inside 6
 (no slice-cache record written after the change, shown beside a control that
 finds records) could not be made with a working control, so the number moves,
 as the slice, refine and translate versions moved for the same reader.

 8: the same validator refuses a candidate the grammar refuses when it carries a guillemet (n159, `translate-guillemets.ts`),
 since no parse tells its prose from code, where it read the guillemet as no prose and let the candidate through; that changes
 which lanes reach a contest. The check that would let the change ride inside 7 (no slice-cache record written after the
 change, shown beside a control that finds records) was not made, since no runs directory is read in this round, so the
 number moves, as the consolidate, slice, refine and translate versions move for the same readers.

 8 also covers the tense of quoted words (n161, TianqiChen666): the house rule and the judges' tense lines now say that words
 quoted as someone wrote or said them keep the speaker's tense, so a ballot stored under 7 may hold a candidate that moved a
 quotation into the past, or one that kept it, judged by the old rule. The key hashes the version, the shape of the run and the
 texts and not the sheet, so a record written under the old sheet would be served under the new one; no record had been written at
 8 when this was written: the numbers n159 set were first committed in `277145a862` together with R1, and the audit of that commit
 found 0 slice-cache records under the keyless worktree's runs directory (`s228/cache-audit-n161.txt`, line 992), so this rides
 inside the number n159 set.

 9 covers the tense sentences moving again (n166, Z11): the house rule's sentence that puts the narrative of a life in the past and
 the judge tail's sentence that says no rule holds a life in the present now hold the quoted-words exception inside them, so a
 ballot stored under 8 was judged under the earlier wording of the same rule. The key hashes the version, the shape of the run and
 the texts and not the sheet, so such a record would be served under the new one. Whether one exists is not measured here: the audit
 of n161 found none, but the launch started since at `5f6c8593d7` runs at 8 and no runs directory is read in this round, so the
 number moves, as the consolidate, repair slice, refine, translate and page title versions move for the same sheets. No record has
 been written at 9, which this change sets first.

 10 covers the tense candidates (review of 2026-10-18): the candidates a judge is shown for an entry that records a death
 left out the narration between an opening curly single mark that a straight mark closes and a straight mark that a curly one closes
 (`maskQuotedSpeech`, `translate-stage/src/present-tense-text.ts`), because the mask joined marks of different pairs into one
 quotation, and they list it now. The key hashes the version, the shape of the run and the texts and not the sheet, so a record
 written under the old candidates would be served under the new ones. The cache account audit of 2026-10-18 (`cache-account-audit`
 over the run directory of TianqiChen66626) read 74 slice-cache records, the newest written after every current version was set, so
 the change cannot ride inside the number and the number moves, as the other two versions whose judges are shown these candidates
 move with it. A search of those records found no opening curly single mark in a candidate text, so no stored record is known to
 differ.

 11 covers the fixes of the XingZ6015 read (2026-10-20), which change what these sheets say and what their floors accept: reader
 protection reaches only the account of how a death or an attempt happened (owner, 2026-10-20, `house-rule/src/house-policy.ts`),
 which every sheet splicing the house rule restates; judges no longer see a note between translators
 (`document-structure/src/translators-note.ts`); the archive-block review says the page it publishes is complete; a consolidation
 challenge round seats a proposal two ballots named over a fallback the contest majority called flawed; and the declared-name floor
 meets a lost form of the subject with another declared form of the same person. The key does not hash the sheet or the floors, so
 a record written under the old ones would be served under the new. The XingZ6015 pass wrote records at the old number on
 `sha256-tree-v1:a92725968adf…`, so the change cannot ride inside it and the number moves, as every version whose sheets splice the
 house rule moves with it; the pairing version stays, since no pairing sheet carries the house rule. No record has been written at
 11, which this change sets first.

 12 covers the cut of a note between translators that stands alone on a quoted line
 (`document-structure/src/translators-note.ts`): the contest judges' sheet cut such a note with the spaces before it and left its
 quote markers, a bare `>   >` line that the judges read as a paragraph break inside the quote that the archive does not have
 (zheermao101, run zheermao15), and it now cuts the line. Of the 34 notes in the 92 pinned English archives one stands alone on a
 quoted line (zheermao101), so only a sheet that shows that archive's text changes, and the cut of every other note is what it
 was. The key hashes the version, the shape of the run and the texts, not the sheet, so a record written under the old cut would
 be served under the new. The check that would let the change ride inside 11 (no slice-cache record written after it, shown
 beside a control that finds records) could not be made with a working control: run zheermao15 started at 02:12 UTC on
 2026-10-20, after 11 was set, its slice-cache directory is empty, and a control from 2026-10-19 finds none either, so the number
 moves, as the consolidate, repair slice, refine and translate versions move for the same cut. The consistency, page title and
 pairing versions stay, since none of their sheets cuts a note. No record has been written at 12, which this change sets first.

 Two later changes to the same cut ride inside 12. A second note on one line no longer takes the line
 break with it (5b75add81, d60b27a9e, 003983067), and a note ending a CRLF line keeps the line's carriage return
 (acf6ada60, e80301bf3). None of the 92 pinned English archives has two notes on one line or a carriage return, so no
 sheet text changes, and no record had been written at 12 when they landed: the only run since it was set,
 CuspariaKLSY18, runs a6a0ed7dc0, which predates it.

 13 covers the archive's spelling of a Han name written inside a sentence (zheermao101, run zheermao16, 2026-10-20): the
 publication rule that decides which candidates the contest seats now refuses a candidate that writes the letters of the
 archive's spelling under another spelling (`translate-floor/src/translate-prose-name.ts`), so a candidate seated under 12 may be
 one the rule refuses, and the identity lines carry the archive's spellings (`rendering-audit/src/page-name-glossary.ts`). The
 identity lines are part of the key, so an entry whose lines change asks a new question; the floor is not, so a record written
 under the old rule for an entry whose lines do not change would be served under the new. Runs zheermao16 and CuspariaKLSY19 start
 at `a5dc23c5bb`, which holds the commit that set 12 (`aa0ab6ff0`), so records at 12 are written there and the change cannot ride
 inside it; the number moves, as the translate slice version moves with it. Over the 92 pinned English archives no archive is
 refused as a candidate against itself, and one of the ten shipped pages under the agent runs is refused (zheermao16). The
 consolidate version rides inside 28, the repair slice, refine, consistency, page title and pairing versions stay (their
 sheets and floors are not this rule's, and the consistency floors run live). No record has been written at 13, which this
 change sets first.

 Rides inside 13 too: the cut of the notes between translators from the sheets of the other judges (12e147ae40, 5498ee860b,
 89df3896d4, eaab1b1ed9, fcd6d13fa3, ebac709b03, 7ee1755b24) and the label rule of the candidate selection slate, which this
 lane's sheet does not use, since it labels its own blocks (`lane-contest-wire.ts`). Its sheet is what it was, so the cut moves no
 number, as the page title version stays: the lexicon sheet shows source and Chinese titles only. The guard that renders every sheet with a planted note (88b5187179, c655a96182) and the restoration judge that
 only the repair benchmark runs (36c5b02884) change no sheet this lane asks.

 Rides inside 13 too: a quote the introduced-defect prober copied across the place a note between translators was cut from now
 anchors in the stored words on each side of the note (`typography/src/locate-quote.ts`), so the corroborated damage claims of
 the repair lane can differ for an entry holding such a note. The damage claims shown to the judges are part of this key
 (`damage`, in `laneContestSliceKey`), so a slice whose claims change asks a new question, and this lane's sheet is what it was. The
 coverage verdict is read by insertion admission and not by this lane, and is accounted beside the translate version. Of the 12 run logs in the agent directory, none started on a tip that holds `89ff91202e`, where 13 was set, so none could
 write a record at 13 (the evidence of the logs' `START tip=` lines and `git merge-base --is-ancestor`, as the repair slice
 version gives it), and the number does not move.

 14 covers the contest ballot giving its reason before its choice (owner, 2026-10-21, "Reason first everywhere";
 `e5bc27f9f0`): the schema and the instruction of the lane contest's sheet (`contest-ballot-wire.ts`, `lane-contest-wire.ts`)
 declare `reason`, then the lists of unsupported and dropped content, then the choice, where the choice came first. The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. Runs zheermao21 and CuspariaKLSY21 started on `3637592bd4`,
 which holds the commit that set 13 (`git merge-base --is-ancestor`), so records may stand at 13 and the change
 cannot ride inside it; no search of any slice-cache directory was made, and the number moves. The
 consolidate version moves for the same ballot and for the polish gate tie rule and sheet (29); the polish gate tie rule and
 the accepted claims block do not touch this lane. No record has been written at 14, which this change sets first.

 15 covers the contest ballot's reply keys declared in alphabetical order (owner decision of 2026-10-21 under the ruling
 "Reason first everywhere"; ledger B762): the lane contest's sheet (`contest-ballot-wire.ts`, `lane-contest-wire.ts`) asks
 for `assessment`, `dropped`, `unsupported` and `verdict`, where it asked for `reason`, `unsupported`, `dropped` and
 `choice`. The ballot a reader builds from the reply keeps the stored names, so a stored ballot reads as before. The
 consolidate version moves for the same ballot (30).
 The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. The previous number was set in `4b9fc9e645`, and this worktree's
 runs directory holds no slice-cache record (the audit `cache-account-audit` found 0, and no control that finds records was
 run, since the pass that writes them runs in another worktree this change does not read), so whether a record stands at
 14 is not known and the change cannot ride inside it: the number moves. No record has been written at 15, which this
 change sets first.

 Rides inside 15 too: `hf:Qwen/Qwen3.8-27B` leaves the lane contest's judges (owner, 2026-10-22, "Remove from both
 benches"; ledger B815). The key hashes the judges' ids (`modelIds`), so a contest judged with Qwen seated is keyed apart
 from one judged without it, and no stored record is served to a bench it was not judged by.

 16 covers the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816): once the straggler grace ends, a
 short bench whose live seats are exactly its quorum closes one voice below it (`stage/src/stage-reachable-quorum.ts`),
 and a refusal inside a round sizes the need again (`stage/src/stage-round.ts`). Every round of the lane contest
 gathers through that rule, so a record settled under the rule of 2026-09-09 may hold an outcome the new rule reaches
 differently, on one voice fewer. The key hashes the version, the shape of the run and the texts, not the rule that
 closes a round. A build whose pipeline digest differs discards the namespace before it reads it
 (`openNamespacedCache`), and the digest holds the stage code, so that retires such a record; the number moves too, as
 the belt beside that brace, so no stored record stands under a number whose rule differs. Run zheermao23 is armed for
 2026-10-22 19:16 UTC on `65e8325303`, which holds 15 (`git merge-base --is-ancestor`), so records may stand at 15, and
 the change cannot ride inside it. The artifact generation moves to 20 for the same rule, and the consolidate (32),
 translate slice (25), repair slice (43), refine (14), consistency (7), page title (6) versions move with this one; the
 pairing version stays, by its own account of 2026-10-18 (`slice-reading/src/pairing-cache-version.ts`). No record has
 been written at 16, which this change sets first.

 Version 17 covers the system text of 2026-10-22 (R4 of the third review of the Claude route): the lane contest keeps
 the rendered-break instruction in the system text and shows the measured break counts of the original and of both
 candidates in the user turn (`lane-contest/src/lane-contest-wire.ts`). The counts are the same facts in a different
 place. The key hashes the version, the shape of the run and the texts, not the sheet
 (`source-scan/src/cache-key-sheet-blindness.unit.test.ts` reads the imports of the five key files), so a record
 written before would be served after. The previous number was set in `eb4db609ff`, and the tip `8864316eb2` of runs
 zheermao23 and XingZ6018 holds it (`git merge-base --is-ancestor`, with `2eeef91fac` as an ancestor of `HEAD` for the
 control that finds ancestry and the reverse pair for the control that does not); `find` under those two run
 directories finds 180 slice-cache files newer than that tip's commit time (2026-10-22T10:30:29Z), so records may
 stand at the previous number and the change cannot ride inside it. The consistency version stays because its key
 digests the exact sheet, and the page title and pairing versions stay because their sheets name no fence length, no
 break count and no marks. No record has been written at the new number, which this change sets first.

 The answer schemas of the range ride inside this number, checked on 2026-10-22 (F6 of the re-check of the second fix
 round) and not assumed: the lane contest's ballot (`lane-contest/src/contest-ballot-wire.ts`, through
 `contestResponseFormat`) lists its verdicts as an enum built from the constant its guard reads (P1, `f9e4fc3097`). The
 key hashes the version, the shape of the run and the texts, not the schema, so only the number retires a record.
 `f9e4fc3097` is an ancestor of `33ca9974f2`, which set this number (`git merge-base --is-ancestor`), and it is not an
 ancestor of `8864316eb2`, the tip of runs zheermao23 and XingZ6018, so no record written by code without it can stand
 at this number. Measured at 2026-10-23T07:30Z: `git branch --all --contains 33ca9974f2` lists only the marker of a
 detached head (`* (no branch)`) and no branch name, and checking the heads of the 86 entries of `git worktree list`
 with `git merge-base --is-ancestor` finds the commit in `tr-w289`, `tr-w305` and `tr-w314`, all at detached heads
 (`tr-w314` is a review worktree at `c98d140b3f`). That set is a reading at that time and grows with each review
 worktree, so the argument that holds is the ancestry argument: no record written by code without the commit can stand
 at this number. The one bump covers both the system text and the schemas, and the number does not move twice
 for one release.

 Version 18 covers the reader of a model's reply taking a value that differs from a listed `enum` or `const` value
 only in capitalization as the listed value (`provider/src/enum-case-fold.ts`, run by `readJsonOutcome`). The lane
 contest's ballot lists its verdicts as an enum (`lane-contest/src/contest-ballot-wire.ts`), so a verdict that a reply
 spelled with another capital, which the guard refused and the round counted as no voice, now counts. The key hashes
 the version, the shape of the run and the texts, not the reader, so a ballot stored before would be served after,
 though it may rest on a round that lost such a voice. The previous number was set by `33ca9974f2`, and no slice-cache
 directory was searched for a record written by code that holds that commit and lacks the fold, so the change cannot
 ride inside it and the number moves. A build whose pipeline digest differs discards the namespace before it reads it
 (`openNamespacedCache`), and the number is the belt beside that brace. The account of every cache is in the
 header of `provider/src/enum-case-fold.ts`.
 */
export const LANE_CONTEST_CACHE_VERSION = 18;

/**
 Everything about this run that changes what the judges are ASKED, folded into
 every contest key.

 Without it a resumed slice could return ballots cast by a different roster,
 and nothing would look wrong: the texts match, so the key matches. Identity
 context belongs here for the same reason, since it is front-matter-derived
 prompt content that varies per pair and measurably changes the answer.

 `perCallTimeoutMs` is deliberately ABSENT, on the same reasoning the other
 two lanes give: it changes how long a voice has to answer, not what it is
 asked, and including it would discard every settled contest on a deadline
 change.

 @param modelIds - roster asked to judge

 @param identityContext - names and handles both documents declare

 @param referenceContext - what the pages the original cites say, folded in
 only when the original cites any, so an entry that cites nothing keys as
 before

 @param lifeEnded - whether the entry records a death, folded in only when it
 does, so an entry that records none keys as before

 @param subjectNames - names the translation declares for its subject, folded in
 with a recorded death, since the tense candidates read each as a subject

 @returns Stable string for the key

 @example
 ```ts
 const runShape = laneContestRunShape({ modelIds, identityContext, },);
 ```
 */
export function laneContestRunShape(
  {
    modelIds,
    identityContext,
    referenceContext,
    lifeEnded,
    subjectNames,
  }: {
    readonly modelIds: readonly RosterModelId[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
  },
): string {
  return JSON.stringify([
    modelIds,
    identityContext ?? '',
    ...(((referenceContext ?? '') === '') ? [] : [referenceContext,]),
    ...((lifeEnded === true)
      ? [
        [
          'life-ended',
          subjectNames ?? [],
        ],
      ]
      : []),
  ],);
}

/**
 Cross-run key for one contested slice.

 THE SLICE INDEX IS NOT IN IT, matching both lanes. A key is what makes two
 runs` slices the same slice, and what a contest judge is asked is the
 original, the archive rendering and the two candidates. Where the slice
 happens to sit changes none of it, and keeping the index would discard every
 settled contest after any renumbering.

 THE ARCHIVE RENDERING IS IN IT even though the contest never ships it. The
 judge is shown it as evidence about what the passage has said before, so two
 contests over identical candidates and different archive wording are not the
 same question.

 @param runShape - what this run asks, from {@link laneContestRunShape}

 @param sourceText - slice original, which is the standard

 @param incumbentText - archive rendering shown as evidence

 @param incumbentKind - whether the archive has wording here at all

 @param syntax - syntax role changing judge policy

 @param repairText - what the repair lane would ship

 @param translateText - what the translate lane would ship

 @param repairDamageClaims - corroborated claims shown against the repair
 candidate, absent when none

 @param archiveDisputeNote - accepted additions shown against the archive
 rendering, absent on an undisputed slice (class one hundred eight)

 @param lineStructured - whether the line rule governs the slice, which
 puts the judge line clause on the sheet (ledger S15)

 @returns Hash keying this slice`s ballots

 @example
 ```ts
 const key = laneContestSliceKey({ runShape, sourceText, incumbentText, incumbentKind, repairText, translateText, lineStructured, },);
 ```
 */
export function laneContestSliceKey(
  {
    runShape,
    sourceText,
    incumbentText,
    incumbentKind,
    syntax,
    repairText,
    translateText,
    repairDamageClaims = [],
    archiveDisputeNote,
    lineStructured,
  }: {
    readonly runShape: string;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly incumbentKind: IncumbentKind;
    readonly syntax?: SliceSyntax;
    readonly repairText: string;
    readonly translateText: string;
    readonly repairDamageClaims?: readonly string[];
    readonly archiveDisputeNote?: string;
    readonly lineStructured: boolean;
  },
): string {
  return hashContent({
    content: JSON.stringify([
      'lane-contest',
      LANE_CONTEST_CACHE_VERSION,
      runShape,
      sourceText,
      incumbentKind,
      incumbentText,
      ...((syntax === undefined)
        ? []
        : [
          'syntax',
          syntax,
        ]),
      repairText,
      translateText,
      // Claims change what the judges were shown, so ballots bought without
      // them must not answer for a question asked with them; a slice with no
      // claim keys exactly as before.
      ...((repairDamageClaims.length === 0)
        ? []
        : [
          'damage',
          ...repairDamageClaims,
        ]),
      // A dispute note changes the question the same way (class one hundred
      // eight); an undisputed slice keys exactly as before.
      ...((archiveDisputeNote === undefined)
        ? []
        : [
          'dispute',
          archiveDisputeNote,
        ]),
      // The line clause changes the question the same way (ledger S15); a
      // prose slice keys exactly as before.
      ...(lineStructured ? ['line-structured',] : []),
    ],),
  },);
}

//endregion Lane contest key
