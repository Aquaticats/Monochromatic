import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { ConsolidationPolishConfig, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { LaneContestBallot, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidate key
// What makes two runs' consolidations the SAME consolidation, for cache
// purposes.
//
// Split from the driver for the reason `lane-contest-key.ts` gives: the key is
// the one piece of a driver testable without a client, and a reader of the
// driver does not want the cache reasoning in the middle of it.

/**
 Generation of the consolidation cache.

 MOVES WHEN THE QUESTION MOVES: either sheet, either schema, the structural
 guard, the validity floor, the gate's settling rule, or the wrap. Every one
 of those changes what a voice is asked or how its answer is read.

 IT MOVES FOR THE WRAP TOO, which is the case worth naming. The wrap runs
 after both rounds and looks like a recording change, but it can demote a
 consolidation to the standing text, which is a different settlement and not a
 different way of writing the same one.

 VERSION 3 adds target-authoritative contributor spelling to producer,
 selector, validity floor and gate. Version 2 settlements did not answer it.

 VERSION 4 adds body naturalness polish after fidelity gate. Earlier
 settlements never bought that stage and cannot resume as though they did.

 VERSION 5 gives final polish every structurally eligible body paragraph and
 makes target contributor public identities explicit in producer rule.

 VERSION 6 asks final polish to remove source-grammar calques and finish every
 clear issue in a changed paragraph. Version 5 asked a weaker question.

 VERSION 7 adds candidate-bound absolute review and one bounded correction.
 Earlier settlements could approve relative improvement with defects remaining.

 VERSION 8 makes absolute-review correction a distinct non-fallback mode and
 gives selectors its required findings. Version 7 selectors answered generic polish question.

 VERSION 9 permits a second candidate-bound correction when first correction
 exposes further material defects. Version 8 exhausted after one generation.

 VERSION 10 binds revised absolute-review and required-correction questions:
 Markdown soft breaks render as spaces, rejected base is not approved fallback,
 and findings are a minimum rather than an edit whitelist.

 VERSION 11 requires absolute reviewers and correction rewriters to perform
 separate local and whole-passage scans, makes publication-quality
 naturalness a hard selector eligibility floor rather than relative ranking,
 and requires a second acceptance before publication approval.

 VERSION 12 starts every direct roster round and absolute acceptance at
 exact-half quorum rather than requiring every provider seat.

 VERSION 13 replaces two-correction terminal refusal with continuous
 latest-rejection correction and distinct acceptance-challenge review.

 VERSION 14 threads every reviewed rejection into failed-strategy evidence
 and adds durable raw-payload replay across interrupted invocations.

 VERSION 15 floors every candidate on target-authoritative contributor identity.
 Earlier settlements could resume wording final publication guard rejects.

 VERSION 16 continuously replaces unendorsed standing wording using prior
 consolidation slate, ballots, gate ballots, terminal, and findings.
 Version 15 could resume settlement final publication guard rejects.

 VERSION 17 sends the gate's objections over an ineligible standing to the
 polish as an objection correction (owner, 2026-09-27). Version 16
 settlements polished comparatively and never saw the objections.

 VERSION 18 ships a slate declined twice over an ineligible standing by
 preference and sends its reasons to the polish beside the gate's, and tells
 the slate judges so (owner, 2026-09-27, "Preference + polish"). Version 17
 raised on that slate and stopped the entry, and its polish sheet carried
 one set of objections.

 VERSION 19 refuses a disputed slice's archive wording, and the repair
 lane's text where the checkers confirmed no repair of the disputed reading,
 as standing, incumbent, lane offer and proposal, and keys the dispute note
 (owner, 2026-09-27, "No eligible standing"). Version 18 stood the repair
 lane's text in for the archive whatever the checkers said, and keyed a
 disputed slice like an undisputed one.

 VERSION 20, the same day, for every change to what this stage asks or
 accepts after version 19 landed at 06:34 UTC, none of which moved the number
 (ledger M25): the floors a proposal is held to (F-1 to F-11, Han left in
 prose, a Han signer left in Han or romanized with no meaning), the gate and
 the polish gate reading the page and line rules, the refiner and writer no
 longer keeping what the house rules render, the naturalness review reading
 the house rules, the windowed gates and the naturalness review sizing their
 quorum on the reachable bench (ledger E3 and X8), and a slice with no valid
 wording keeping the archive, which is never persisted. A settlement written
 before any of these could resume past a floor it now fails. It costs
 nothing: no slice-cache file under the agent runs was written after 04:26 UTC
 that day, before version 19 existed.

 Two later changes ride inside 20 on the same reasoning, checked on
 2026-09-28 against that newest file: the house rules' "-re" line names the
 meter that measures beside the metre (ledger K13, `403db3c6e`), on every
 sheet this stage shows; and the prose ranges the Han-residue floor reads,
 which a proposal is held to, now look past a JSX comment, a stray quote mark
 and a lone backtick (ledger K7, `1873b23dc` and `dc325d847`).

 VERSION 21 (owner answers of 2026-10-11, "Majority flawed yes", "Run off
 the tied finalists" and "Majority, like the run-off") changes three things a
 settlement records. A tied slate over an eligible standing is run off once
 over its tied finalists when the standing drew no ballot, where version 20
 kept the standing unchallenged. The run-off over an eligible standing is
 triggered by a strict majority of the contest ballots calling the archive
 flawed, where version 20 required every ballot. And the same majority forfeits
 that standing past an undecided gate, where version 20 again required every
 ballot. A settlement written under version 20 kept a standing no judge chose,
 and kept an archive a majority condemned, so it cannot resume as though it
 still answered the question. The forfeit's move landed after 21 was set and
 before any pass ran on it, so no settlement was stored under 21 without it.

 The glossary audit's corrections ride inside 20 too (ledger C3 to R16,
 `357f534b7`), checked on 2026-09-28 against the newest slice-cache file under
 the agent runs, still 04:26 UTC on 2026-09-27 (eighteen files after 04:20 UTC that
 day, none after 04:27 UTC): the source-carry floor a proposal is held to refuses
 and passes other forms, the COMMUNITY RENDERINGS block on the gate and the
 polish gate counts a rendering inflected and bounded, and the grammatical
 English house rule states the doubled preposition. The terms, renderings and
 whys reach the identity context, which the key hashes.

 Rides inside 20 too: every gather now keeps a seat that answered unreadably
 out of the same-prompt retry rounds and re-asks each such seat, whichever
 round it came in, in the nudged recovery round (ledger P2, `005692e11`),
 which changes whose voices a round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 20 too: a reply whose complete JSON value more text follows is
 read rather than lost (ledger P8, `cac097368`), which changes whose voices
 every round hears; checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 Rides inside 20 too: a reply that could not be used is re-asked once,
 nudged, of the same model on another provider through the uniqueness
 wrapper's claims (ledger P9, `7011d72cc`), which changes whose voices every
 round hears; checked on 2026-09-28: still no slice-cache file newer than
 04:26 UTC on 2026-09-27.

 Rides inside 20 too: a disputed slice's repair text never stands in when it
 is the archive's own wording, and the refused-wording finding says why the
 repair lane's text is refused (ledger L12, `f66e96f06` and `3ffbcec10`). No
 settled slice moves: 0 of 1,132 fully resolved disputes over every artifact
 had unchanged text. Checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 Rides inside 20 too: Bedrock reads dry with 1.33 USD still left, and its
 ledger holds every billed attempt at its bound (ledger P1, `2a108dfb3`,
 `107763dbb` and `7cfd5ae4b`), so a Gemma call moves to OpenRouter sooner
 as Bedrock nears its credit, which changes whose voices a round hears;
 checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 20 too: the recovery round re-asks a reply the length
 limit cut with a nudge naming the cut, apart from one off the shape
 (ledger P10, `ce0ef7b51`), which changes what such a seat is asked and
 whose voices a round hears; checked on 2026-09-28: still no slice-cache
 file newer than 04:26 UTC on 2026-09-27.

 Rides inside 20 too: the run shape carries the slate judges beside the
 writers (ledger X11), which left it when `9a7d48354` split them from the
 writers on 2026-09-02, so a settlement another judging bench reached never
 resumes; every key moves, and none was written under 20. Checked on
 2026-09-28: still no slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 20 too: a slice the per-slice hook re-seats under a hold runs
 on the writers, slate judges and naturalness roles it hands over and is
 keyed by them (ledger H5); a slice nobody re-seated keys as before, so no
 key moves. Same check, same result.

 Rides inside 20 too: the strict parse reads a formula as the site does
 (`5ce9370ac`), and a proposal forming a formula its original lacks is
 refused before any judge (ledger X22, `translate-formula.ts`), which
 changes what the consolidation accepts; checked on 2026-09-29: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 20 too: the declared-name guard and the Han residue floor
 read text by code point, so a handle written in letters beyond the first
 plane is checked rather than projected to nothing, and an ideograph beyond
 it left in prose is refused (ledger B21, `declared-name-survival.ts`,
 `translate-han-residue.ts`); over the pinned archives and settled pages no
 survival answer changes, and neither they nor any stored artifact carries
 such an ideograph; same check, same result.

 Rides inside 20 too: the validator's link-name floor and contributor
 authority, which judge the standing and every lane text offered to the
 slate, carry a declared name only where it stands as a name, and the link
 floor reads a Latin source form, a mention and a page handle at handle edges
 (ledger B23, `name-projection.ts`, `11d029fde`); over the would-ship slices
 of every settled artifact neither answer moves that this change explains;
 checked on 2026-09-29: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 20 too: the consolidation slate collapses a rendering apart
 from the standing or a lane text only in straight against curly prose quotes
 into it (ledger B24, `collapseKey`, `straightenProseQuotes`); over the 2,905
 stored translate and consolidation slates, 305 carried such a twin; checked
 on 2026-09-29: the newest slice-cache file is still the one of 04:26 UTC on
 2026-09-27.

 Rides inside 20 too: the validator that judges the standing, every lane
 text offered and each consolidation candidate refuses prose guillemets and
 the editor sheet's «REGION marker and line heads (ledger B24,
 `translate-guillemets.ts`, `translate-sheet-leak.ts`); one comparison row
 of 6,285 across the stored artifacts carries a guillemet its incumbent
 lacks; checked on 2026-09-29: the newest slice-cache file is still the one
 of 04:26 UTC on 2026-09-27.

 Rides inside 20 too: whether a proposal changes anything is read as the page
 shows it (ledger B26, `sameWording`). The consolidation slate collapses a
 rendering apart from the standing or a lane text only in where its soft line
 breaks fall; a lane text that is the standing in all but layout is not
 offered back; the shipping wrap demotes a consolidation that is the standing
 in all but layout, and on a line-structured slice one that is the standing
 but for a trailing newline; the polish round demotes a refinement that is
 the base in all but layout. Over the 2,905 stored slates, 490 carried such a
 twin; checked on 2026-09-29: no slice-cache file was written after 04:27 UTC
 on 2026-09-27, where a control from midnight finds 494.

 Rides inside 20 too: the final polish refuses a rewrite that is one of the
 slice's disputed wordings, as the standing verdict, the lane offer and the
 producers' floor already did (ledger B29, `consolidation-polish-apply.ts`);
 the find over the three slice-cache roots, rerun for this change, finds no
 file written after 04:27 UTC on 2026-09-27 against a control of 494.

 Rides inside 20 too: where the original has explicit breaks and no archive
 wording, the writer's sheet states the break counts of the lane candidates
 it displays, as the gates after it state theirs (ledger B29,
 `consolidate-wire.ts`), which changes what such a writer is asked; the same
 find, rerun on 2026-09-29, again finds no file written after 04:27 UTC on
 2026-09-27 against a control of 494.

 Rides inside 20 too: the validator that judges the standing, every lane
 text offered and each consolidation candidate no longer lists the PRIOR
 FAILED CONSOLIDATION STRATEGY heading among the sheet labels it refuses
 unfenced (ledger B30, `translate-sheet-leak.ts`, `32e07afb7`), since no
 sheet has carried it since the single consolidation attempt (`1ba8f713a`);
 a fenced copy is still refused. An rg for the heading over the run
 directories, the agent runs and the user cache finds it only in logs,
 source copies and build output, in no stored slate, artifact or cache
 record; checked on 2026-09-29: no slice-cache file was written after 04:27
 UTC on 2026-09-27, where a control from midnight finds 494 and an earlier
 check's own output file.

 Rides inside 20 too: a consolidate-gate or polish-gate ballot whose findings
 key is left out keeps its choice, read as no findings, where either gate
 refused it whole (ledger B46, `findingsOrNone` in `contest-ballot-wire.ts`,
 `consolidate-gate-wire.ts`, `consolidation-polish-gate-wire.ts`), which
 changes whose voices a gate round hears; checked on 2026-09-30: no
 slice-cache file was written after 04:27 UTC on 2026-09-27, where a control
 from midnight that day finds 494.

 Rides inside 20 too: a polish round whose original or base no grammar reads
 keeps the base with a `consolidation-polish-unfloored` finding and asks
 nobody, where it paid its refiners and judges for a polish the structural
 check then left unvalidated (ledger B48, `consolidation-polish-round.ts`);
 checked on 2026-09-30: no slice-cache file was written after 04:27 UTC on
 2026-09-27, where a control from midnight that day finds 494.

 Rides inside 20 too: an absence the slate judging raises over a withheld
 standing, which no judging reaches, throws `WithheldSlateAbsenceError`
 where the settlement kept the archive on it (ledger B51,
 `consolidate-settle-judge.ts`); checked on 2026-09-30: no slice-cache file
 was written after 04:27 UTC on 2026-09-27, where a control from midnight
 that day finds 494.

 Rides inside 20 too: the floor every consolidation validity check calls
 compares quoted passages at every depth and refuses a wording carrying
 fewer than its page (ledger B110, `translate-validate-blocks.ts`), which
 changes which candidates and standings a slate holds; checked on
 2026-10-01: the newest of 13,714 slice-cache records under 391 runs
 directories is still the one of 04:26 UTC on 2026-09-27.

 THE PRE-LAUNCH CHECK OF 2026-09-28 (ledger M28): 20 was set
 in `66703994a` at 02:56 UTC on 2026-09-28, after the newest slice-cache file under the
 agent runs (04:26 UTC on 2026-09-27), and no slice-cache file has been written
 since, so no answer cached under an earlier question can be served under
 this number. Every source commit since then rides inside it, those the
 accounts in this TSDoc name and those they do not (`corpus-run/cache-account-audit.ts`, ledger M28).

 Rides inside 21 too: the slate judges and the gate of an entry that records a
 death are shown the tense candidates of each text on offer
 (`tense-candidate-block.ts`, `consolidate-gate-wire.ts`,
 `translate-slate-evidence.ts`), and the identity context carries the recorded
 date of death (`identity-context.ts`). No answer cached under the earlier
 question can be served under the new one: the run shape of such an entry names
 the recorded death and the subject names, and its identity context differs, so
 its key differs. An entry that records no death asks the question it asked
 before and keeps its key, so the version stays where it is and the answers
 already bought for those entries stay usable.

 22: the slate's twin collapse (`collapseKey`, `straightenProseQuotes`) and the
 validator's prose guillemet floor (`translate-guillemets.ts`) read prose from
 the page parse (`typography-prose-mask.ts`), so two renderings that differ by a
 mark in a comment or a link address no longer collapse, and a text the grammar
 refuses has no prose. That changes which candidates a slate holds as one. The
 check that would let the change ride inside 21 (no slice-cache record written
 after the change, shown beside a control that finds records) could not be made
 with a working control, so the number moves, as the slice, refine and translate
 versions moved for the same reader.

 23: the same readers read more of the page (n159). The text of a shortcut or collapsed reference (`[dogs' bowl]`) and the
 text of a link whose label is an address are no prose to the mask (`typography-link-text.ts`), so the slate's twin collapse
 (`collapseKey`, `straightenProseQuotes`) keeps the marks in them and two renderings that differ by such a mark no longer
 collapse; and the validator's prose guillemet floor (`translate-guillemets.ts`) refuses a candidate the grammar refuses when
 it carries a guillemet, since no parse tells its prose from code. Both change which candidates a slate holds as one and
 which it accepts. The check that would let the change ride inside 22 (no slice-cache record written after the change, shown
 beside a control that finds records) was not made, since no runs directory is read in this round, so the number moves, as
 the lane-contest, slice, refine and translate versions move for the same readers.

 23 also covers the tense of quoted words (n161, TianqiChen666): the house rule, the producer's tense line and the judges' tense
 lines now say that words quoted as someone wrote or said them keep the speaker's tense, so a consolidation or a ballot stored
 under 22 may hold a quotation moved into the past. The key hashes the version, the shape of the run and the texts and not the
 sheet, so a record written under the old sheet would be served under the new one; no record had been written at 23 when this was
 written: the numbers n159 set were first committed in `277145a862` together with R1, and the audit of that commit found 0
 slice-cache records under the keyless worktree's runs directory (`s228/cache-audit-n161.txt`, line 992), so this rides inside the
 number n159 set.

 24: the tense sentences moved again (n166, Z11). The house tense rule's sentence that puts the narrative of a life in the past now
 holds the quoted-words exception inside it, and the judge tail's sentence that says no rule holds a life in the present holds it
 too, so the sheets the consolidation and its judges read word the rule differently from 23. The key hashes the version, the shape
 of the run and the texts and not the sheet, so a record written at 23 under the earlier wording of the same rule would be served
 under the new one. Whether such a record exists is not measured here: the audit of n161 found none, but the launch started since at
 `5f6c8593d7` runs at 23 and no runs directory is read in this round, so the number moves, as the lane-contest, slice, refine,
 translate and page title versions move for the same sheets. No record has been written at 24, which this change sets first.

 25 covers the tense candidates (review of 2026-10-18): the candidates a judge is shown for an entry that records a death
 left out the narration between an opening curly single mark that a straight mark closes and a straight mark that a curly one closes
 (`maskQuotedSpeech`, `translate-stage/src/present-tense-text.ts`), because the mask joined marks of different pairs into one
 quotation, and they list it now. The key hashes the version, the shape of the run and the texts and not the sheet, so a record
 written under the old candidates would be served under the new ones. The cache account audit of 2026-10-18 (`cache-account-audit`
 over the run directory of TianqiChen66626) read 74 slice-cache records, the newest written after every current version was set, so
 the change cannot ride inside the number and the number moves, as the other two versions whose judges are shown these candidates
 move with it. A search of those records found no opening curly single mark in a candidate text, so no stored record is known to
 differ.

 26 covers the fixes of the XingZ6015 read (2026-10-20), which change what these sheets say and what their floors accept: reader
 protection reaches only the account of how a death or an attempt happened (owner, 2026-10-20, `house-rule/src/house-policy.ts`),
 which every sheet splicing the house rule restates; judges no longer see a note between translators
 (`document-structure/src/translators-note.ts`); the archive-block review says the page it publishes is complete; a consolidation
 challenge round seats a proposal two ballots named over a fallback the contest majority called flawed; and the declared-name floor
 meets a lost form of the subject with another declared form of the same person. The key does not hash the sheet or the floors, so
 a record written under the old ones would be served under the new. The XingZ6015 pass wrote records at the old number on
 `sha256-tree-v1:a92725968adf…`, so the change cannot ride inside it and the number moves, as every version whose sheets splice the
 house rule moves with it; the pairing version stays, since no pairing sheet carries the house rule. No record has been written at
 26, which this change sets first.

 27 covers the cut of a note between translators that stands alone on a quoted line
 (`document-structure/src/translators-note.ts`): the consolidation gate and the polish gate cut such a note with the spaces
 before it and left its quote markers, a bare `>   >` line that the judges read as a paragraph break inside the quote that the
 archive does not have (zheermao101, run zheermao15), and it now cuts the line. Of the 34 notes in the 92 pinned English archives
 one stands alone on a quoted line (zheermao101), so only a sheet that shows that archive's text changes, and the cut of every
 other note is what it was. The key hashes the version, the shape of the run and the texts, not the sheet, so a record written
 under the old cut would be served under the new. The check that would let the change ride inside 26 (no slice-cache record
 written after it, shown beside a control that finds records) could not be made with a working control: run zheermao15 started at
 02:12 UTC on 2026-10-20, after 26 was set, its slice-cache directory is empty, and a control from 2026-10-19 finds none either,
 so the number moves, as the lane-contest, repair slice, refine and translate versions move for the same cut. The consistency,
 page title and pairing versions stay, since none of their sheets cuts a note. No record has been written at 27, which this
 change sets first.

 Two later changes to the same cut ride inside 27. A second note on one line no longer takes the line
 break with it (5b75add81, d60b27a9e, 003983067), and a note ending a CRLF line keeps the line's carriage return
 (acf6ada60, e80301bf3). None of the 92 pinned English archives has two notes on one line or a carriage return, so no
 sheet text changes, and no record had been written at 27 when they landed: the only run since it was set,
 CuspariaKLSY18, runs a6a0ed7dc0, which predates it.

 28 covers the sheet of the absolute naturalness review (`archive/src/absolute-naturalness-review-wire.ts`), which the
 account of 26, "judges no longer see a note between translators", did not reach: the review the final polish asks of its
 would-ship slice showed the candidate and each paragraph with every note, so a judge could raise a Chinese comment in an
 English text as a naturalness finding, and it cuts them now against the original, as the gates before it do. A paragraph
 of nothing but notes is left off that sheet with its number unused. The review's seats, verdicts and findings are part of
 the settlement this key addresses (`polish.review`), and the key hashes the version, the run shape and the texts, not the
 sheet, so a record written for a slice whose text holds a note under the old sheet would be served under the new. Run
 zheermao16 is live on build a5dc23c5bb and writes records at 27, so the change cannot ride inside it and the number
 moves. Over the 92 pinned English archives 10 hold a note their original lacks (34 notes in all), and 28 stretches of
 them between blank lines hold one, so a sheet changes for at most those 10 entries; no other entry's sheet moves. The
 archive block review has no cache version of its own, the block it reviews stands whatever it finds, and the other
 versions stay, since none of their sheets is this one. No record has been written at 28, which this change sets first.

 Rides inside 28 too: the publication rule the consolidation, its standing verdict and its polish round meet refuses a
 candidate that writes the letters of the archive's spelling of a Han name written inside a sentence under another spelling
 (`translate-floor/src/translate-prose-name.ts`), and the identity lines the gates read carry those spellings
 (`rendering-audit/src/page-name-glossary.ts`; zheermao101, run zheermao16, 2026-10-20). The key does not hash the floors, so a
 record written under the old rule would be served under the new, if one had been written at 28. None has: 28 was set in
 `6a28f54be`, at 11:24 UTC-4 on 2026-10-20, and the two runs begun since, zheermao16 and CuspariaKLSY19, both log
 `START tip=a5dc23c5bb`, which does not hold that commit and sets 27. Both read and write at 27, so neither can have stored
 an answer at 28. The lane-contest and translate slice versions move for the same rule (13 and 22).

 Rides inside 28 too: the sheets of the settlement's other judges cut the notes between translators as well
 (`document-structure/src/translators-note.ts`): the introduced-defect prober and the resolution checker that look over a
 polish (12e147ae40, 5498ee860b), the candidate selection slate of the objection correction, whose current English and
 objections were shown whole because a search of the whole label took them for the original (12e147ae40), and the coverage
 sheet (89df3896d4). The key hashes the version, the run shape and the texts, not the sheet. Of the 12 run logs in the agent
 directory, none started on a tip that holds `6a28f54be`, where 28 was set: the newest tip, `a5dc23c5bb` of zheermao16 and
 CuspariaKLSY19, does not, and the others started on earlier ones, so none could write a record at 28. That is the evidence of
 the logs' `START tip=` lines and `git merge-base --is-ancestor`, and not a search of any slice-cache directory.

 Rides inside 28 too: a quote the introduced-defect prober copied from its sheet across the place a note between translators was
 cut from now anchors in the stored words on each side of the note (`adjudication/src/introduced-defect-screen.ts`,
 `typography/src/locate-quote.ts`), so the damage claims over a polish can differ for a slice holding such a note. The key
 hashes the version, the run shape and the texts, not the sheet. The same 12 logs, read the same way, show none that started on
 a tip that holds `6a28f54beb`, where 28 was set, so none could write a record at 28, and the number does not move.

 29 covers three changes to the settlement of 2026-10-21. A polish gate that ties between the polished and the base text
 now ships the polished text (owner, 2026-10-21, "Polished text wins ties"; `45657407a6`): the gate counts a tie at or above
 the minimum ballot count as a win for the polished text, where it kept the base, so a stored settlement of a tied gate holds
 the opposite verdict. The polish gate's sheet shows the claims the repair panel accepted (`5419332cef`, CuspariaKLSY21 slice
 0), which the dispute note leaves out at minor severity; the key carries that note when a slice has one
 (`repair-claims`), so a slice with such claims asks a new question by itself, and a slice without them is unchanged by this
 part. Every ballot gives its reason before its choice (owner, 2026-10-21, "Reason first everywhere"; `e5bc27f9f0`): the sheets
 of the consolidation gate and the polish gate (`consolidate-gate-wire.ts`, `consolidation-polish-gate-wire.ts`), of the
 absolute naturalness review (`absolute-naturalness-review-wire.ts`) and of the coverage judge that insertion admission
 reads (`coverage-wire.ts`) declare and word the reason first. The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. Runs zheermao21 and CuspariaKLSY21 started on `3637592bd4`,
 which holds the commit that set 28 (`git merge-base --is-ancestor`), so records may stand at 28 and the change
 cannot ride inside it; no search of any slice-cache directory was made, and the number moves. The lane-contest, translate
 slice, repair slice and refine versions move for the same ballots (14, 23, 41, 12); the archive block review has no version
 of its own; the consistency, page title and pairing versions stay, since their sheets and schemas are what they were. No
 record has been written at 29, which this change sets first.

 30 covers the writers of the final polish and the judges of their rewrites: the writers are shown the claims the repair
 panel accepted for the slice and the note on a disputed archive rendering, in the blocks the polish gate reads
 (`refine-prompt.ts`, `consolidation-polish-round.ts`; ledger B805), where before only the gate was, and the judges of
 the writers' rewrites (`refine-selection-context.ts`) are shown the same two notes, with a criterion that lets a
 candidate correct what a supported claim names. A slice with either note asks its writers and its judges a new question,
 and its stored settlement holds the polish they wrote and chose without the notes. The key already carries both notes
 when a slice has them (`repair-claims`, the dispute note), so the key does not change when the question does, and a
 record written at 29 would be served under 30. The judges' change rides in 30 with the writers', since both are one
 landing and no record has been written at 30. Runs zheermao22 and XingZ6017 log
 `START tip=2a336603bf`, which holds the commit that set 29 (`4b9fc9e645`, `git merge-base --is-ancestor`), so records
 may stand at 29 and the change cannot ride inside it; no search of any slice-cache directory was made, and the number
 moves. A slice with no accepted claims asks its writers what it asked, but its settlement is keyed under the same
 number, so it is recomputed too. The refine version stays: the repair lane's own refinement passes no note and its
 sheet is what it was (`refine-prompt.unit.test.ts`, "LEAVES THE SHEET UNCHANGED WHEN NO CLAIMS WERE ACCEPTED"). The
 lane-contest, translate slice and repair slice versions stay, since none of their sheets is a polish writer's. No record
 has been written at 30, which this change sets first.

 31 covers the answer schemas of the settlement declaring their properties in alphabetical order, and the keys that moved
 to make it so (owner decision of 2026-10-21 under the ruling "Reason first everywhere"; ledger B762): the contest ballot of
 the consolidation gate and the polish gate (`assessment`, `dropped`, `unsupported`, `verdict`), the absolute naturalness
 review (`assessment`, `findings`, `isAcceptable`), the coverage judge (`assessment`, `copiedSpan`, `coverage`), the
 candidate selection (`assessment`, `best`) and the introduced-defect check over a polish (`verdictReason`) changed their
 reply keys, instructions or both. The archive block review (`assessment`, `disposition`, `replacementText`, `sourceQuote`)
 has no version of its own. The lane-contest, translate slice, repair slice and refine versions move for the same sheets
 (15, 24, 42, 13).
 The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. 29 was set in `4b9fc9e645`, and this worktree's runs
 directory holds no slice-cache record (the audit `cache-account-audit` found 0, and no control that finds records was
 run, since the pass that writes them runs in another worktree this change does not read), so whether a record stands at
 29 is not known and the change cannot ride inside it: the number moves. This change moved it to 30 in `a40de882b`
 while main moved it to 30 for B805 in `27db2c256`, so the merge that met the two (`ba30e7ac6`) set 31 in its
 resolution, and each 30 keeps one meaning. The audit names that merge as the setting since ledger B814; before, it
 refused 31 as in no commit. No pass ran on a build holding either 30: the last runs, zheermao22 and XingZ6017, log
 `START tip=2a336603bf`, which holds neither commit (`git merge-base --is-ancestor`). No record has been written at
 31, which this change sets first.

 Rides inside 31 too: `hf:Qwen/Qwen3.8-27B` leaves the slate judges and the gate of the consolidation (owner, 2026-10-22,
 "Remove from both benches"; ledger B815). The key hashes the judges' ids (`judgeModelIds`), so a settlement judged with
 Qwen seated is keyed apart from one judged without it, and no stored record is served to a bench it was not judged by.

 32 covers the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816): once the straggler grace ends, a
 short bench whose live seats are exactly its quorum closes one voice below it (`stage/src/stage-reachable-quorum.ts`),
 and a refusal inside a round sizes the need again (`stage/src/stage-round.ts`). Every round of the slate judges, the
 consolidation gate, the polish gate and the absolute naturalness review gathers through that rule, so a record settled
 under the rule of 2026-09-09 may hold an outcome the new rule reaches differently, on one voice fewer. The key hashes
 the version, the shape of the run and the texts, not the rule that closes a round. A build whose pipeline digest
 differs discards the namespace before it reads it (`openNamespacedCache`), and the digest holds the stage code, so
 that retires such a record; the number moves too, as the belt beside that brace, so no stored record stands under a
 number whose rule differs. Run zheermao23 is armed for 2026-10-22 19:16 UTC on `65e8325303`, which holds 31
 (`git merge-base --is-ancestor`), so records may stand at 31, and the change cannot ride inside it. The artifact generation
 moves to 20 for the same rule, and the lane-contest (16), translate slice (25), repair slice (43), refine (14),
 consistency (7), page title (6) versions move with this one; the pairing version stays, by its own account of
 2026-10-18 (`slice-reading/src/pairing-cache-version.ts`). No record has been written at 32, which this change sets
 first.

 Version 33 covers the system text of 2026-10-22 (R4 of the third review of the Claude route): the consolidation
 writer and its gate keep the rendered-break instruction in the system text and show the measured break counts in the
 user turn (`consolidate/src/consolidate-wire.ts`, `consolidate/src/consolidate-gate-wire.ts`), with the counts of the
 candidates it displays. The counts are the same facts in a different place. The key hashes the version, the shape of
 the run and the texts, not the sheet (`source-scan/src/cache-key-sheet-blindness.unit.test.ts` reads the imports of
 the five key files), so a record written before would be served after. The previous number was set in `eb4db609ff`,
 and the tip `8864316eb2` of runs zheermao23 and XingZ6018 holds it (`git merge-base --is-ancestor`, with `2eeef91fac`
 as an ancestor of `HEAD` for the control that finds ancestry and the reverse pair for the control that does not);
 `find` under those two run directories finds 180 slice-cache files newer than that tip's commit time
 (2026-10-22T10:30:29Z), so records may stand at the previous number and the change cannot ride inside it. The
 consistency version stays because its key digests the exact sheet, and the page title and pairing versions stay
 because their sheets name no fence length, no break count and no marks. No record has been written at the new number,
 which this change sets first.

 The answer schemas of the range ride inside this number, checked on 2026-10-22 (F6 of the re-check of the second fix
 round) and not assumed: the consolidation gate and the polish gate (`consolidate/src/consolidate-gate-stage.ts`,
 `consolidate/src/consolidation-polish-gate-stage.ts`, both through `contestResponseFormat`) list their verdicts as
 an enum built from the constant their guard reads (P1, `f9e4fc3097`). The key hashes the version, the shape of the
 run and the texts, not the schema, so only the number retires a record. `f9e4fc3097` is an ancestor of `33ca9974f2`,
 which set this number (`git merge-base --is-ancestor`), and it is not an ancestor of `8864316eb2`, the tip of runs
 zheermao23 and XingZ6018, so no record written by code without it can stand at this number.
 Measured at 2026-10-23T07:30Z: `git branch --all --contains 33ca9974f2` lists only the marker of a detached head
 (`* (no branch)`) and no branch name, and checking the heads of the 86 entries of `git worktree list` with
 `git merge-base --is-ancestor` finds the commit in `tr-w289`, `tr-w305` and `tr-w314`, all at detached heads (`tr-w314`
 is a review worktree at `c98d140b3f`). That set is a reading at that time and grows with each review worktree, so the
 argument that holds is the ancestry argument: no record written by code without the commit can stand at this number.
 The one bump covers both the system text and the schemas, and the number does not move twice for one release.

 Version 34 covers the reader of a model's reply taking a value that differs from a listed `enum` or `const` value
 only in capitalization as the listed value (`provider/src/enum-case-fold.ts`, run by `readJsonOutcome`). The
 consolidation gate and the polish gate list their verdicts as an enum (`lane-contest/src/contest-ballot-wire.ts`), so
 a verdict that a reply spelled with another capital, which the guard refused and the round counted as no voice, now
 counts. The key hashes the version, the shape of the run and the texts, not the reader, so a settlement stored before
 would be served after, though it may rest on a round that lost such a voice. The previous number was set by
 `33ca9974f2`, and no slice-cache directory was searched for a record written by code that holds that commit and lacks
 the fold, so the change cannot ride inside it and the number moves. A build whose pipeline digest differs discards
 the namespace before it reads it (`openNamespacedCache`), and the number is the belt beside that brace. The account
 of every cache is in the header of `provider/src/enum-case-fold.ts`.
 */
export const CONSOLIDATE_CACHE_VERSION = 34;

/**
 What a line-structured slice appends to its key material.

 APPENDED, NEVER INSERTED, and only when the rule governs. A prose slice
 therefore hashes exactly the material it hashed before this mark existed,
 so every prose consolidation already settled stays resumable. Only the
 governed slices, whose producer sheet genuinely gained a rule, are re-bought.

 NOT A CACHE VERSION BUMP, which would have been correct and wasteful: it
 would discard the prose settlements too, and they were bought under a sheet
 identical to the one they would be re-bought under.
 */
const LINE_STRUCTURED_KEY_MARK = 'line-structured';

/**
 What a slice carrying picture readings labels them with.

 LABELLED AND PAIRED, unlike the mark above it, because this append carries
 CONTENT rather than a fact. A bare append of the readings could collide with
 the mark for a reading whose words happen to be `line-structured`, and a
 label in front of them costs one array element to make that impossible.
 */
const PICTURE_KEY_LABEL = 'pictures';

/**
 What a slice carrying a window labels its ORIGINAL side with.

 TWO LABELS RATHER THAN ONE, because the two sides are independently absent:
 a slice can stand beside a section the archive never translated, and folding
 them under a single label would let a source-only window and an
 incumbent-only one collide.

 SPELLED AS `translateSliceKey` SPELLS IT, so a reader tracing one window
 through both keys meets the same two words.
 */
const NEIGHBOURING_SOURCE_KEY_LABEL = 'neighbouring';

/**
 What a slice carrying a window labels its ARCHIVE side with.
 */
const NEIGHBOURING_INCUMBENT_KEY_LABEL = 'neighbouring-incumbent';

/**
 What a slice whose lane texts were offered on the slate labels them with
 (class forty, 2026-09-17). Appended only when any were, so every slice
 whose standing was an endorsed eligible lane keys as it always did.
 */
const LANE_TEXTS_KEY_LABEL = 'lane-texts';

/**
 Everything about this run that changes what the voices are ASKED.

 Without it a resumed slice could return a settlement reached by a different
 roster and nothing would look wrong, since the texts match and so the key
 matches. Identity context belongs here for the same reason the contest gives:
 it is front-matter-derived prompt content that varies per pair and measurably
 changes the answer.

 `perCallTimeoutMs` is deliberately ABSENT, on the reasoning every other lane
 gives: it changes how long a voice has to answer, not what it is asked.

 @param modelIds - writers asked to produce, and the judges where none are named

 @param judgeModelIds - slate judges and gate, which decide what ships, so a
 settlement another bench reached never resumes (ledger X11: they left the key
 when `9a7d48354` split them from the writers on 2026-09-02)

 @param identityContext - names and handles both documents declare

 @param lifeEnded - whether the entry records a death, folded in only when it
 does, so an entry that records none keys as before

 @param subjectNames - names the translation declares for its subject, folded in
 with a recorded death, since the tense candidates read each as a subject

 @param referenceContext - what the pages the original cites say, folded in
 only when the original cites any

 @param polishConfig - naturalness roles and document guard facts

 @param declaredNamePairs - names the front matter declares, source beside
 rendering, which the standing and slate floors read; folded in only when
 there are any (ledger A13: they reached the key only through the identity
 context, which lists the same fields in other words)

 @returns Stable string for the key

 @example
 ```ts
 const runShape = consolidateRunShape({ modelIds, identityContext, declaredNamePairs: [], },);
 ```
 */
export function consolidateRunShape(
  {
    modelIds,
    judgeModelIds = modelIds,
    identityContext,
    lifeEnded,
    subjectNames,
    referenceContext,
    polishConfig,
    declaredNamePairs,
  }: {
    readonly modelIds: readonly RosterModelId[];
    readonly judgeModelIds?: readonly RosterModelId[];
    readonly identityContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly referenceContext?: string;
    readonly polishConfig?: ConsolidationPolishConfig;
    readonly declaredNamePairs: readonly DeclaredNamePair[];
  },
): string {
  return JSON.stringify([
    modelIds,
    judgeModelIds,
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
    ...((polishConfig === undefined)
      ? []
      : [
        polishConfig.refinerModelIds,
        polishConfig.judgeModelIds,
        polishConfig.gateModelIds,
        polishConfig.declaredNames,
        polishConfig.definitions,
      ]),
    ...((declaredNamePairs.length === 0) ? [] : [declaredNamePairs,]),
  ],);
}

/**
 Cross-run key for one consolidated slice.

 THE CONTEST BALLOTS ARE IN IT, which is what separates this key from the
 contest's own. The consolidation sheet shows the producers what the contest
 judges said about each lane, as claims rather than as verdicts, so two
 consolidations over identical candidates and different ballots are not the
 same question and must not resume into one another.

 THE STANDING TEXT IS IN IT SEPARATELY from the two lane renderings, because
 it is not derivable from them here: the contest may have declined, in which
 case nothing stands, and the deciding half behaves differently.

 THE SLICE INDEX IS NOT IN IT, matching every other lane. Where a slice sits
 changes nothing a voice is asked, and keeping the index would discard every
 settled consolidation after a renumbering.

 @param runShape - what this run asks, from {@link consolidateRunShape}

 @param sourceText - slice original, which is the standard

 @param incumbentText - archive rendering, which is the structural standard

 @param syntax - syntax role changing producer, judge and gate policies

 @param repairText - what the repair lane would ship

 @param translateText - what the translate lane would ship

 @param standingText - what ships today, empty where the contest declined

 @param ballots - what the contest judges said, shown to the producers

 @param lineStructured - whether the enclosing chunk is line-structured,
 which decides whether the producer sheet carries the rule against merging
 lines, and so decides what was bought

 @param pictureContext - what the pictures this slice and its neighbours
 show were read to say, absent where none were read

 @param neighbouringSourceText - original of the passages either side, absent
 where the slice stands alone, since the window is shown to the producers

 @param neighbouringIncumbentText - archive rendering of the passages either
 side, absent for the same reason

 @param laneTexts - lane texts offered on the slate beside the proposals,
 absent or empty where the standing was an endorsed eligible lane (class
 forty, 2026-09-17)

 @param archiveDisputeNote - the dispute note a disputed slice is settled
 under, absent on an undisputed one

 @param acceptedRepairClaimsNote - the claims the repair panel accepted for
 the slice, which the final polish gate, its writers and the judges of their
 rewrites read, absent where it accepted none

 @returns Hash keying this slice's settlement

 @example
 ```ts
 const key = consolidateSliceKey({ runShape, sourceText, incumbentText, repairText, translateText, standingText, ballots, lineStructured, },);
 ```
 */
export function consolidateSliceKey(
  {
    runShape,
    sourceText,
    incumbentText,
    syntax,
    repairText,
    translateText,
    standingText,
    ballots,
    lineStructured,
    pictureContext,
    neighbouringSourceText,
    neighbouringIncumbentText,
    laneTexts = [],
    archiveDisputeNote,
    acceptedRepairClaimsNote,
  }: {
    readonly runShape: string;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly syntax?: SliceSyntax;
    readonly repairText: string;
    readonly translateText: string;
    readonly standingText: string;
    readonly ballots: readonly LaneContestBallot[];
    readonly lineStructured: boolean;
    readonly pictureContext?: string;
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
    readonly laneTexts?: readonly LaneText[];
    readonly archiveDisputeNote?: string;
    readonly acceptedRepairClaimsNote?: string;
  },
): string {
  return hashContent({
    content: JSON.stringify([
      'consolidate',
      CONSOLIDATE_CACHE_VERSION,
      runShape,
      sourceText,
      incumbentText,
      ...((syntax === undefined)
        ? []
        : [
          'syntax',
          syntax,
        ]),
      repairText,
      translateText,
      standingText,
      ballots,
      ...(lineStructured ? [LINE_STRUCTURED_KEY_MARK,] : []),
      // THE READING'S WORDS RATHER THAN ITS OWN KEY, matching
      // `translate-slice-key.ts` exactly, and for its reason: what changes a
      // producer's answer is the words it was shown, so two runs that read one
      // picture into different words asked different questions however identical
      // their texts were.
      //
      // APPENDED AND ONLY WHEN PRESENT, so a slice near no readable picture
      // hashes the array it always hashed. Most slices carry no pictures, and
      // discarding their settled consolidations to record an absence would buy
      // the corpus nothing.
      ...(((pictureContext === undefined) || (pictureContext === ''))
        ? []
        : [
          PICTURE_KEY_LABEL,
          pictureContext,
        ]),
      // THE WINDOW, on the same terms and for the same reason: the judges of a
      // consolidation read the passages either side from 2026-08-22, and a
      // settlement decided without them answered a different question.
      //
      // AFTER THE PICTURES RATHER THAN BEFORE, which is where the sibling
      // `translateSliceKey` puts the same two fields. Appending is this file's
      // own convention and the labels make position irrelevant to correctness,
      // so the orders differ and neither is wrong; a reader comparing the two
      // keys side by side should not read the difference as a defect.
      //
      // UNLIKE THE PICTURE APPEND, THIS IS NOT CHEAP. Most slices carry no
      // picture, so that field left almost every settled consolidation
      // resumable. Nearly every slice of a multi-slice document HAS a window,
      // so nearly every settlement keyed before this is re-bought. That is the
      // correct outcome rather than a regrettable one: resuming them would
      // return answers to a question nobody asked.
      ...(((neighbouringSourceText === undefined) || (neighbouringSourceText === ''))
        ? []
        : [
          NEIGHBOURING_SOURCE_KEY_LABEL,
          neighbouringSourceText,
        ]),
      ...(((neighbouringIncumbentText === undefined) || (neighbouringIncumbentText === ''))
        ? []
        : [
          NEIGHBOURING_INCUMBENT_KEY_LABEL,
          neighbouringIncumbentText,
        ]),
      // THE LANE TEXTS OFFERED, appended only when any were, so a slate that
      // carries them is never served a settlement bought by one that did not
      // and every other slice keys as before (class forty, 2026-09-17).
      ...((laneTexts.length === 0)
        ? []
        : [
          LANE_TEXTS_KEY_LABEL,
          laneTexts,
        ]),
      // THE DISPUTE NOTE, appended only when present: a slice settled under
      // accepted claims against its archive, whose archive the rule may refuse,
      // asks a different question from the same texts undisputed (owner,
      // 2026-09-27, "No eligible standing").
      ...(((archiveDisputeNote === undefined) || (archiveDisputeNote === ''))
        ? []
        : [
          'dispute',
          archiveDisputeNote,
        ]),
      // THE ACCEPTED REPAIR CLAIMS, appended only when present, on the same
      // terms as the dispute note: the polish gate of a slice the repair panel
      // accepted claims on reads them (CuspariaKLSY21 slice 0, 2026-10-21), as
      // do its writers and the judges of their rewrites (ledger B805), so it
      // asks a different question from the same texts without them, and a
      // slice the panel accepted nothing on keys as before.
      ...(((acceptedRepairClaimsNote === undefined) || (acceptedRepairClaimsNote === ''))
        ? []
        : [
          'repair-claims',
          acceptedRepairClaimsNote,
        ]),
    ],),
  },);
}

//endregion Consolidate key
