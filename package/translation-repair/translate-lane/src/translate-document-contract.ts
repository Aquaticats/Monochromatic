import type {
  LaneSliceText,
  UnfilledReason,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { QuotedPassages, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SliceAlignmentAssessment, } from './translate-alignment.ts';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { SliceSelection, } from './slice-selection.ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Translate document contract
// What the translate lane stores per slice, and what it returns per document.
//
// SEPARATE FROM THE REPAIR CONTRACT on purpose. A repair outcome carries
// issues, repair regions, resolved issue ids and checker verdicts, none of
// which a translation has; a widened type would give every translate slice
// fields that can only be filled with lies, and a widened cache guard would let
// a repair-shape slice resume into a translate run, which nothing downstream
// could detect.

/**
 Schema of one stored translate slice.

 Starts at ONE rather than continuing the repair cache's numbering: the two
 lanes version independently, so a translate change cannot invalidate settled
 repair work and a repair change cannot invalidate settled translations.

 Bump for any change to what a record means: the record shape, the decision
 kinds, the alignment predicate, or what the lane asks the models.

 VERSION 2, on 2026-08-15, takes the SLICE INDEX out of the key. No record
 changed; what changed is which slices count as the same slice. Keeping the
 index meant any renumbering discarded every slice after it, and one-sided slicing
 renumbers by design, since inserting a slice for an untranslated section
 shifts everything below it. The index is now stamped onto a resumed record by
 whoever asked for it, and `translateSliceKey` carries the measurement saying
 identical-text slices inside one document do not occur in this corpus.

 VERSION 3, the same day, puts the INCUMBENT KIND into the key, because what a
 run ASKS about a slice with no translation is a different question: it must
 be filled or left as the gap it is, while a slice that has one may settle on
 what is already there. The bump discards nothing, measured before the change:
 no record had been settled under version 2 at all.

 VERSION 4, on 2026-08-20, for the declared-name guard. Every slice cached
 before it was settled without that check, so a resumed run would ship a
 replacement that dropped a declared name rather than re-deciding it. A guard
 any cache hit can walk past is not a guard.

 VERSION 5, the same day, because that guard now compares names on letters and
 digits alone. Measured over the pinned corpus, the comparison it replaces
 missed 12 of the 123 declared forms that are present in their own entry, so a
 slice settled under version 4 may have shipped the loss of one of those and
 would report itself decided. The widening loses none of the forms the old
 comparison found, so no cached slice can be re-decided the other way.

 VERSION 6, on 2026-08-22 (`973f7b47d`), because the verse rule reached the
 judges' sheet in both lanes. A slice judged before it could have ranked a
 rendering the rule places lower, so every slice settled under version 5 is
 re-judged; nothing about the record's shape or key changed.

 VERSION 7, on 2026-08-28, because metadata candidate admission now preserves
 contributor spelling established at same YAML comment path. A cached front
 matter candidate could retain source-script attribution and bypass guard.

 VERSION 8, same day, because body contributor lines now preserve public
 identity spellings established by existing English attribution. An earlier
 cached slice can replace them with literal source-side transliterations.

 VERSION 9 deterministically floors every candidate on complete target
 contributor survival rather than relying on prompt compliance alone,
 and excludes an unrepairable contributor-violating voice without fabricating
 incumbent authorship.

 VERSION 10 replaces identical second judging with prior-decline challenge.

 VERSION 11 continues absent passages from latest rejected slate and findings.

 VERSION 12 excludes archive wording that fails deterministic source floor
 and treats it as absent fallback until stage-local translation settles.

 VERSION 13 tells the judges of a slice whose archive wording is withheld
 that a declined slate still ships by preference, and ships the follow-up
 round's preferred candidate past a declined challenge round (owner,
 2026-09-27, "Preference + polish"); version 12 told them the passage had no
 translation and stopped the entry on that decline.

 VERSION 14 keeps a disputed archive off the slate as incumbent and as a
 copied candidate, stands the repair lane's text in only where the checkers
 confirmed every disputing issue resolved, and keys the dispute note (owner,
 2026-09-27, "No eligible standing").

 VERSION 15, the same day, for every change to what this lane asks or
 accepts after version 14 landed at 06:34 UTC, none of which moved the number
 (ledger M25). The floors: the address floor (F-1), the suicide floor (F-4),
 the floors that need no grammar on an unreadable original (F-5), the
 line-structure floor (F-7), the sheet-leak floor (F-8), the neutral-pronoun
 floor (F-9), glossary floors inside an accepted Han title (F-11), Han left
 standing in English prose (F-3, `078939ac7`) and a Han signer left in Han or
 romanized with no meaning (A17, `7ec9669bd`). The sheets: the declared names
 fenced on every sheet, the house rules and the apparatus kinds as one list,
 the line rule shown to the translate probe, and the contest reading line
 structure. The lane contest, a windowed stage, sizes its quorum on the
 reachable bench (ledger X8). A slice settled before any of these could resume
 with a candidate the floors now refuse. It costs nothing: the newest of the
 14,596 slice-cache files under the agent runs was written at 04:26 UTC that day,
 before version 14 existed.

 Two later changes ride inside 15 on the same reasoning, checked on
 2026-09-28 against that newest file: the house rules' "-re" line names the
 meter that measures beside the metre (ledger K13, `403db3c6e`), on every
 writing and judging sheet; and the prose ranges the Han-residue floor (F-3)
 reads now look past a JSX comment, a stray quote mark and a lone backtick
 (ledger K7, `1873b23dc` and `dc325d847`), so the floor sees Han it missed
 behind them.

 The glossary audit's corrections ride inside 15 too (ledger C3 to R16,
 `357f534b7`), checked on 2026-09-28 against the newest slice-cache file under
 the agent runs, still 04:26 UTC on 2026-09-27 (eighteen files after 04:20 UTC that
 day, none after 04:27 UTC). The source-carry floor refuses forms it passed and
 passes forms it refused (a slide onto a tier, "fossil-fuel car", "turned into
 in"), and no key reads a refused form; the COMMUNITY RENDERINGS block on the
 select sheets now counts a rendering inflected and bounded; and the
 grammatical English house rule states the doubled preposition. The terms,
 renderings and whys reach the identity context, which the key hashes.

 Rides inside 15 too: every gather now keeps a seat that answered unreadably
 out of the same-prompt retry rounds and re-asks each such seat, whichever
 round it came in, in the nudged recovery round (ledger P2, `005692e11`),
 which changes whose voices a round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: a reply whose complete JSON value more text follows is
 read rather than lost (ledger P8, `cac097368`), which changes whose voices
 every round hears; checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: a decision seat whose state the endpoint refuses
 as past its context, or that a stage cannot ask, now reads as out of reach
 for that ballot rather than as a lost voice (ledger P13, `a991ef1e1`), which
 changes the quorum a select round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: no alignment, quote-loss or declared-name refusal keeps
 an archive the publication rule refuses, and such a slice carries
 `translate-archive-ineligible` (ledger X6, `6445a2e35`), which changes a
 refused slice's text and findings; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: a reply that could not be used is re-asked once,
 nudged, of the same model on another provider through the uniqueness
 wrapper's claims (ledger P9, `7011d72cc`), which changes whose voices every
 round hears; checked on 2026-09-28: still no slice-cache file newer than
 04:26 UTC on 2026-09-27.

 Rides inside 15 too: a disputed slice's repair text never stands in when it
 is the archive's own wording, and the refused-wording finding a translate
 author reads says why the repair lane's text is refused (withdrawn at
 assembly, or unconfirmed by the checkers) rather than blaming the checkers
 for both (ledger L12, `f66e96f06` and `3ffbcec10`). No settled slice moves:
 0 of 1,132 fully resolved disputes over every artifact had unchanged text.
 Checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 15 too: Bedrock reads dry with 1.33 USD still left, and its
 ledger holds every billed attempt at its bound (ledger P1, `2a108dfb3`,
 `107763dbb` and `7cfd5ae4b`), so a Gemma call moves to OpenRouter sooner
 as Bedrock nears its credit, which changes whose voices a round hears;
 checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 15 too: the recovery round re-asks a reply the length
 limit cut with a nudge naming the cut, apart from one off the shape
 (ledger P10, `ce0ef7b51`), which changes what such a seat is asked and
 whose voices a round hears; checked on 2026-09-28: still no slice-cache
 file newer than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: the strict parse reads a formula as the site does
 (`5ce9370ac`), and a candidate forming a formula its original lacks is
 refused before any judge (ledger X22, `translate-formula.ts`), which
 changes what the lane accepts; checked on 2026-09-29: still no slice-cache
 file newer than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: the declared-name guard and the Han residue floor
 read text by code point, so a handle written in letters beyond the first
 plane is checked rather than projected to nothing, and an ideograph beyond
 it left in prose is refused (ledger B21, `declared-name-survival.ts`,
 `translate-han-residue.ts`); over the pinned archives and settled pages no
 survival answer changes, and neither they nor any stored artifact carries
 such an ideograph; same check, same result.

 Rides inside 15 too: the declared-name guard and the link-name floor carry a
 declared name only where it stands as a name, a letter meeting a digit and a
 small letter meeting a capital counting as edges, and the link floor reads a
 Latin source form, a mention and a page handle at handle edges (ledger B23,
 `name-projection.ts`, `11d029fde`); over the would-ship slices of every
 settled artifact, four wordings of two slices now drop a declared form the
 old containment took as kept, and no link-floor or contributor-authority
 answer moves that this change explains; checked on 2026-09-29: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 15 too: `collapseKey` reads prose quotes through the typography
 fold (ledger B24, `straightenProseQuotes`), so a rendering apart from the
 incumbent or another only in straight against curly prose quotes collapses
 into it on the slate, and the repair lane skips validating it as a copy of
 the incumbent. Over the 2,905 stored slates, 305 carried such a twin and in
 30 the chosen candidate was the incumbent's; checked on 2026-09-29: the
 newest slice-cache file is still the one of 04:26 UTC on 2026-09-27.

 Rides inside 15 too: the validator refuses a candidate whose prose sets a
 quotation in guillemets (ledger B24, `translate-guillemets.ts`), and the
 sheet-leak floor the editor sheet's «REGION marker and its CURRENT TEXT: and
 CONTEXT: ... heads; over the stored artifacts one comparison row of 6,285
 carries a guillemet its incumbent lacks, and none of the pinned originals
 or archives carries a guillemet or those heads; checked on 2026-09-29: the
 newest slice-cache file is still the one of 04:26 UTC on 2026-09-27.

 Rides inside 15 too: the slate's key reads a rendering as the page shows it
 (ledger B26, `wordingKey`), so on a slice the line-structure rule does not
 govern, a rendering apart from the incumbent or another only in where its
 soft line breaks fall collapses into it, and the repair turn's copy check
 reads the same key. Over the 2,905 stored slates, 490 carried such a twin
 and in 93 the chosen candidate was the incumbent's, line-structured slices
 included, which the key leaves apart; checked on 2026-09-29: no slice-cache
 file was written after 04:27 UTC on 2026-09-27, where a control from
 midnight finds 494.

 Rides inside 15 too: where the original has explicit breaks and no archive
 wording, the follow-up writer's sheet states the break counts of the
 candidates the judges declined, as the judges had them (ledger B29,
 `translate-wire.ts`), which changes what such a writer is asked; checked on
 2026-09-29: no slice-cache file was written after 04:27 UTC on 2026-09-27,
 where a control from midnight finds 494.

 Rides inside 15 too: the quote guard counts blockquotes off the parse the
 floor reads, at every depth, where it counted blank-line chunks opening with
 `>` (ledger B42, `quote-preservation.ts`), and a quote-loss refusal stores
 the counts it compared, so the record shape changes and a replacement is
 refused or kept differently; over the stored records one refusal,
 hulicaijia24 slice 2, kept the archive over a replacement carrying its
 quote. Checked on 2026-09-30: no slice-cache file was written after 04:27
 UTC on 2026-09-27, where a control from 00:00 UTC that day finds 450.

 Rides inside 15 too: a slice the floor can compare nothing on settles before
 any call, on the archive's own bytes with the decision `unfloored` where
 there is one and unfilled with that reason where there is none (ledger B43,
 `translate-unfloored.ts`), and the front-matter floor answers `unknown`
 rather than charging a refused side's YAML to the candidate (ledger B44),
 which changes such a slice's record; no pinned slice is one (0 of 1,351
 originals, 0 of 92 front-matter slices), and a record whose stage heard
 nobody is never cached. Checked on 2026-09-30: no slice-cache file was
 written after 04:27 UTC on 2026-09-27 under the agent runs or the package's
 runs, where a control from 00:00 UTC that day finds 494, once a throwaway
 checkout and a worktree-copy payload holding copies are set aside.

 Rides inside 15 too: the floor compares quoted passages at every depth
 against the page, or the original where there is none, and refuses a
 candidate carrying fewer (ledger B110, `quotedPassageFloorFindings` in
 `translate-validate-blocks.ts`), so a candidate keeping every top-level
 block while turning a blockquote nested inside a container tag into prose
 is refused before any judge, where it passed; over 1,277 pinned
 archive-against-original slices and 5,520 settled would-ship slices no
 verdict moves. Checked on 2026-10-01: the newest of 13,714 slice-cache
 records under 391 runs directories is still the one of 04:26 UTC on
 2026-09-27 (`cache-account-audit` with the agent runs added).

 THE PRE-LAUNCH CHECK OF 2026-09-28 (ledger M28): 15 was set
 in `66703994a` at 02:56 UTC on 2026-09-28, after the newest slice-cache file under the
 agent runs (04:26 UTC on 2026-09-27), and no slice-cache file has been written
 since, so no answer cached under an earlier question can be served under
 this number. Every source commit since then rides inside it, those the
 accounts in this TSDoc name and those they do not (`corpus-run/cache-account-audit.ts`, ledger M28).

 Rides inside 15 too: the translate judges of an entry that records a death are
 shown the tense candidates of each candidate text
 (`tense-candidate-block.ts`, `translate-slate-evidence.ts`), and the identity
 context carries the recorded date of death (`identity-context.ts`). No answer
 cached under the earlier question can be served under the new one: the run
 shape of such an entry names the recorded death and the subject names, and its
 identity context differs, so its key differs. An entry that records no death
 asks the question it asked before and keeps its key, so the version stays
 where it is and the answers already bought for those entries stay usable.

 Version 16 is a TYPOGRAPHY change: the prose mask the typography fold (`straightenProseQuotes`, which `collapseKey` reads) and
 the guillemet floor (`translate-guillemets.ts`) share is read from the page parse now, so a quote or a guillemet in a link
 destination or title, an address, code, math, an expression or a tag is no prose, and a text the grammar refuses has no prose
 at all. A slate collapsed or a candidate refused under 15 may be one the new reading keeps apart or accepts. The check that
 would let the change ride inside 15 could not be made with a control (see version 35 of the repair slice key), so the number
 moves.

 Version 17 reads more of the page with the same mask (n159): the text of a shortcut or collapsed reference and the text of a
 link whose label is an address are no prose, so `collapseKey` keeps the marks in them, and the guillemet floor refuses a
 candidate the grammar refuses when it carries a guillemet, since no parse tells its prose from code. A slate collapsed or a
 candidate accepted under 16 may be one the new reading keeps apart or refuses. The check that would let the change ride inside
 16 was not made, since no runs directory is read in this round, so the number moves.

 Version 17 also covers the tense of quoted words (n161, TianqiChen666): the house rule and the writer's tense line now say that
 words quoted as someone wrote or said them keep the speaker's tense, so a writer's answer stored under 16 may hold a quotation
 moved into the past. The key hashes the version, the roster and the texts and not the sheet, so a record written under the old
 sheet would be served under the new one; no record had been written at 17 when this was written: the numbers n159 set were first
 committed in `277145a862` together with R1, and the audit of that commit found 0 slice-cache records under the keyless
 worktree's runs directory (`s228/cache-audit-n161.txt`, line 992), so this rides inside the number n159 set.

 Version 18 covers the tense sentences moving again (n166, Z11): the house rule's sentence that puts the narrative of a life in the
 past, which the translate sheet carries, now holds the quoted-words exception inside it, so a writer's answer stored under 17 was
 written under the earlier wording of the same rule. The key hashes the version, the roster and the texts and not the sheet, so such
 a record would be served under the new one. Whether one exists is not measured here: the audit of n161 found none, but the launch
 started since at `5f6c8593d7` runs at 17 and no runs directory is read in this round, so the number moves, as the consolidate,
 lane-contest, repair slice, refine and page title versions move for the same sheets. No record has been written at 18, which this
 change sets first.

 Version 19 covers the tense candidates (review of 2026-10-18): the candidates a judge is shown for an entry that records a death
 left out the narration between an opening curly single mark that a straight mark closes and a straight mark that a curly one closes
 (`maskQuotedSpeech`, `translate-stage/src/present-tense-text.ts`), because the mask joined marks of different pairs into one
 quotation, and they list it now. The key hashes the version, the shape of the run and the texts and not the sheet, so a record
 written under the old candidates would be served under the new ones. The cache account audit of 2026-10-18 (`cache-account-audit`
 over the run directory of TianqiChen66626) read 74 slice-cache records, the newest written after every current version was set, so
 the change cannot ride inside the number and the number moves, as the other two versions whose judges are shown these candidates
 move with it. A search of those records found no opening curly single mark in a candidate text, so no stored record is known to
 differ.

 Version 20 covers the fixes of the XingZ6015 read (2026-10-20), which change what these sheets say and what their floors accept:
 reader protection reaches only the account of how a death or an attempt happened (owner, 2026-10-20,
 `house-rule/src/house-policy.ts`), which every sheet splicing the house rule restates; judges no longer see a note between
 translators (`document-structure/src/translators-note.ts`); the archive-block review says the page it publishes is complete; a
 consolidation challenge round seats a proposal two ballots named over a fallback the contest majority called flawed; and the
 declared-name floor meets a lost form of the subject with another declared form of the same person. The key does not hash the
 sheet or the floors, so a record written under the old ones would be served under the new. The XingZ6015 pass wrote records at the
 old number on `sha256-tree-v1:a92725968adf…`, so the change cannot ride inside it and the number moves, as every version whose
 sheets splice the house rule moves with it; the pairing version stays, since no pairing sheet carries the house rule. No record
 has been written at 20, which this change sets first.

 Version 21 covers the cut of a note between translators that stands alone on a quoted line
 (`document-structure/src/translators-note.ts`): the translate lane's candidate selection slate cut such a note with the spaces
 before it and left its quote markers, a bare `>   >` line that the judges read as a paragraph break inside the quote that the
 archive does not have (zheermao101, run zheermao15), and it now cuts the line. Of the 34 notes in the 92 pinned English archives
 one stands alone on a quoted line (zheermao101), so only a sheet that shows that archive's text changes, and the cut of every
 other note is what it was. The key hashes the version, the shape of the run and the texts, not the sheet, so a record written
 under the old cut would be served under the new. The check that would let the change ride inside 20 (no slice-cache record
 written after it, shown beside a control that finds records) could not be made with a working control: run zheermao15 started at
 02:12 UTC on 2026-10-20, after 20 was set, its slice-cache directory is empty, and a control from 2026-10-19 finds none either,
 so the number moves, as the consolidate, lane-contest, repair slice and refine versions move for the same cut. The consistency,
 page title and pairing versions stay, since none of their sheets cuts a note. No record has been written at 21, which this
 change sets first.

 Two later changes to the same cut ride inside 21. A second note on one line no longer takes the line
 break with it (5b75add81, d60b27a9e, 003983067), and a note ending a CRLF line keeps the line's carriage return
 (acf6ada60, e80301bf3). None of the 92 pinned English archives has two notes on one line or a carriage return, so no
 sheet text changes, and no record had been written at 21 when they landed: the only run since it was set,
 CuspariaKLSY18, runs a6a0ed7dc0, which predates it.

 Version 22 covers the archive's spelling of a Han name written inside a sentence (zheermao101, run zheermao16, 2026-10-20): the
 publication rule that every translate candidate meets now refuses a candidate that writes the letters of the archive's spelling
 under another spelling (`translate-floor/src/translate-prose-name.ts`), so a record stored under 21 may hold a candidate the
 rule refuses, and the identity lines the sheets read carry the archive's spellings (`rendering-audit/src/page-name-glossary.ts`).
 The key hashes the version, the shape of the run and the texts, not the sheet or the floors, so such a record would be served
 under the new rule. Runs zheermao16 and CuspariaKLSY19 start at `a5dc23c5bb`, which holds the commit that set 21 (`aa0ab6ff0`),
 so records at 21 are written there and the change cannot ride inside it; the number moves, as the lane-contest version moves with
 it. Over the 92 pinned English archives no archive is refused as a candidate against itself, and one of the ten shipped pages
 under the agent runs is refused (zheermao16). The consolidate version rides inside 28 and the repair slice, refine, consistency,
 page title and pairing versions stay, for the reasons the lane-contest version gives. No record has been written at 22, which
 this change sets first.

 Version 22 also covers the translate lane's judges no longer seeing a note between translators
 (`document-structure/src/translators-note.ts`): the candidate selection slate named its block of the surrounding existing
 translation "SURROUNDING EXISTING TRANSLATION (English), context only: ... the ORIGINAL of THIS passage ...", which a search
 of the whole label took for the original, so the neighbours' English was shown whole and every note in it counted as the
 original's own, the candidates beside it keeping theirs; the label's name decides now, the words before its first comma, and
 the block is cut (12e147ae40). The coverage sheet that insertion admission reads for this lane cuts the notes from the
 translation (89df3896d4). The key hashes the version, the shape of the run and the texts, not the sheet, so a record written
 under the old sheet would be served under the new. Runs zheermao16 and CuspariaKLSY19 started on `a5dc23c5bb`, which holds 21
 (set in `aa0ab6ff0`), so a record may stand at 21 and the check that would let the change ride inside it, no slice-cache record
 written after it shown beside a control that finds records, was not made, so the number moves, as the repair slice and refine
 versions move for the same sheets. Over the 92 pinned English archives 10 hold a note their original lacks (34 notes in all),
 so a sheet changes for at most those 10 entries. Both changes set 22 before any run started on a tip that holds it, so the two ride in one number.

 Rides inside 22 too: a quote a coverage voice copied from that sheet across the place a note between translators was cut from
 used to stand in no stored text and counted as not anchored, and now anchors in the stored words on each side of the note
 (`archive/src/coverage-verdict.ts`, `typography/src/locate-quote.ts`), so the verdict insertion admission reads for this lane
 can carry a passage it could not. The evidence the verdict keeps is still the stored slice. The translate lane asks no critic
 or prober to quote anything, and the attested lines its sheets show are part of its key, so nothing else of this change reaches
 it. Of the 12 run logs in the agent directory, none started on a tip that holds `43e735f7da`, where 22 was set, so none could
 write a record at 22 (the evidence of the logs' `START tip=` lines and `git merge-base --is-ancestor`, as the repair slice
 version gives it), and the number does not move.

 Version 23 covers the selection ballot and the coverage judge giving their reasons before their votes (owner, 2026-10-21,
 "Reason first everywhere"; `e5bc27f9f0`): the translate lane's candidate selection (`adjudication/src/candidate-select-wire.ts`,
 now `reason` then `best`) and the coverage sheet that insertion admission reads for this lane (`archive/src/coverage-wire.ts`,
 now `reason`, `quote`, then `coverage`). The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. Runs zheermao21 and CuspariaKLSY21 started on `3637592bd4`,
 which holds the commit that set 22 (`git merge-base --is-ancestor`), so records may stand at 22 and the change
 cannot ride inside it; no search of any slice-cache directory was made, and the number moves. The consolidate, lane-contest, repair slice and refine versions move for
 the same ballots (29, 14, 41, 12). No record has been written at 23, which this change sets first.

 Version 24 covers the answer schemas of the translate lane declaring their properties in alphabetical order, and the keys
 that moved to make it so (owner decision of 2026-10-21 under the ruling "Reason first everywhere"; ledger B762): the
 rewrite wire of the translate floor (`translatorNote` in place of `explanation`, still after the translation it explains),
 the candidate selection (`assessment`, `best`) and the coverage judge that insertion admission reads (`assessment`,
 `copiedSpan`, `coverage`). The consolidate, lane-contest, repair slice and refine versions move for the same sheets (30, 15,
 42, 13).
 The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. The previous number was set in `4b9fc9e645`, and this worktree's
 runs directory holds no slice-cache record (the audit `cache-account-audit` found 0, and no control that finds records was
 run, since the pass that writes them runs in another worktree this change does not read), so whether a record stands at
 23 is not known and the change cannot ride inside it: the number moves. No record has been written at 24, which this
 change sets first.

 Version 25 covers the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816): once the straggler grace
 ends, a short bench whose live seats are exactly its quorum closes one voice below it
 (`stage/src/stage-reachable-quorum.ts`), and a refusal inside a round sizes the need again
 (`stage/src/stage-round.ts`). Every round of the translate lane's producers, candidate selection and coverage judge
 gathers through that rule, so a record settled under the rule of 2026-09-09 may hold an outcome the new rule reaches
 differently, on one voice fewer. The key hashes the version, the shape of the run and the texts, not the rule that
 closes a round. A build whose pipeline digest differs discards the namespace before it reads it
 (`openNamespacedCache`), and the digest holds the stage code, so that retires such a record; the number moves too, as
 the belt beside that brace, so no stored record stands under a number whose rule differs. Run zheermao23 is armed for
 2026-10-22 19:16 UTC on `65e8325303`, which holds 24 (`git merge-base --is-ancestor`), so records may stand at 24, and
 the change cannot ride inside it. The artifact generation moves to 20 for the same rule, and the consolidate (32),
 lane-contest (16), repair slice (43), refine (14), consistency (7), page title (6) versions move with this one; the
 pairing version stays, by its own account of 2026-10-18 (`slice-reading/src/pairing-cache-version.ts`). No record has
 been written at 25, which this change sets first.

 Version 26 covers the system text of 2026-10-22 (R4 of the third review of the Claude route): the translate writer
 and its follow-up keep the rendered-break instruction in the system text and show the measured break counts of the
 original and of the rejected candidates in a user-turn block (`translate-floor/src/translate-wire.ts`), and the slate
 judges of the translation receive the same counts as an evidence block and the fence length at the head of the user
 turn (`translate-stage/src/translated-slate-criteria.ts`, `adjudication/src/candidate-select-wire.ts`). The counts
 and the length are the same facts in a different place. The key hashes the version, the shape of the run and the
 texts, not the sheet (`source-scan/src/cache-key-sheet-blindness.unit.test.ts` reads the imports of the five key
 files), so a record written before would be served after. The previous number was set in `eb4db609ff`, and the tip
 `8864316eb2` of runs zheermao23 and XingZ6018 holds it (`git merge-base --is-ancestor`, with `2eeef91fac` as an
 ancestor of `HEAD` for the control that finds ancestry and the reverse pair for the control that does not); `find`
 under those two run directories finds 180 slice-cache files newer than that tip's commit time (2026-10-22T10:30:29Z),
 so records may stand at the previous number and the change cannot ride inside it. The consistency version stays
 because its key digests the exact sheet, and the page title and pairing versions stay because their sheets name no
 fence length, no break count and no marks. No record has been written at the new number, which this change sets
 first.

 Version 27 covers the reader of a model's reply taking a value that differs from a listed `enum` or `const` value
 only in capitalization as the listed value (`provider/src/enum-case-fold.ts`, run by `readJsonOutcome`). The
 translate repair stage lists its resolutions as an enum (`translate-floor/src/translate-repair-wire.ts`), so a
 resolution that a reply spelled with another capital, which the guard refused and the round counted as no voice, now
 counts. The key hashes the version, the shape of the run and the texts, not the reader, so a slice stored before
 would be served after, though it may rest on a round that lost such a voice. The previous number was set by
 `33ca9974f2`, and no slice-cache directory was searched for a record written by code that holds that commit and lacks
 the fold, so the change cannot ride inside it and the number moves. A build whose pipeline digest differs discards
 the namespace before it reads it (`openNamespacedCache`), and the number is the belt beside that brace. The account
 of every cache is in the header of `provider/src/enum-case-fold.ts`.
 */
export const TRANSLATE_SLICE_CACHE_VERSION = 27;

/**
 Models the translate lane seats.

 @example
 ```ts
 const models: TranslateModels = { translatorModelIds, judgeModelIds, };
 ```
 */
export type TranslateModels = {
  /**
   Models producing independent translations of each slice.
   */
  readonly translatorModelIds: readonly RosterModelId[];

  /**
   Whole roster judging the slate, translators included; a ballot for a
   judge's own translation counts less rather than not at all.
   */
  readonly judgeModelIds: readonly RosterModelId[];
};

/**
 Roster a per-slice hook hands the translate lane, as the repair lane's
 hook hands it one (ledger H5).

 @example
 ```ts
 const seating: TranslateSliceSeating = { translateModels, };
 ```
 */
export type TranslateSliceSeating = {
  /**
   Roster read since the lane started, absent while the given one stands.
   */
  readonly translateModels?: TranslateModels;
};

/**
 What the driver did with one slice's stage result.

 @example
 ```ts
 const disposition: TranslateDisposition = 'refused-alignment';
 ```
 */
export type TranslateDisposition =
  /**
   Stage result taken as it stands.
   */
  | 'stage-result'
  /**
   Stage wanted to replace the incumbent and the alignment guard refused,
   because the source cannot account for the text being replaced.
   */
    | 'refused-alignment'
  /**
   Stage wanted to replace the incumbent and the quote guard refused, because
   the replacement carried fewer quoted passages than the archive does.
   */
    | 'refused-quote-loss'
  /**
   Stage wanted to replace the incumbent and the declared-name guard refused,
   because the replacement dropped a name the archive text carried and the
   documents declare.
   */
    | 'refused-declared-name';

/**
 Fields every settled translate slice record carries, whatever the driver
 did with the stage result.
 */
type TranslateSliceRecordFields = {
  /**
   Lane discriminator, checked when a cache file is read so a repair outcome
   can never be resumed as a translation.
   */
  readonly kind: 'translate-slice';

  /**
   Schema this record was written under.
   */
  readonly schemaVersion: number;

  /**
   Global slice index, as preparation stamped it.
   */
  readonly sliceIndex: number;

  /**
   Everything the stage decided, including the slate, every ballot and its
   findings.
   */
  readonly stageResult: TranslateStageResult;

  /**
   Text the driver accepted for assembly.
   */
  readonly outputText: string;

  /**
   Whether that text differs from the translation already in the archive.
   */
  readonly changed: boolean;

  /**
   Measurements behind the alignment decision, recorded on every slice rather
   than only refused ones: a rate needs its denominator.
   */
  readonly alignment: SliceAlignmentAssessment;

  /**
   Stage findings plus any refusal this driver added.
   */
  readonly findings: readonly string[];
};

/**
 Settled record for one translate slice.

 Carries the WHOLE stage result beside the driver's decision, so a reader can
 tell "judges preferred a replacement and the guard refused it" from "judges
 kept the incumbent". Those are opposite facts about the same lane and both
 ship the same text.

 THE DROPPED NAMES COME WITH THEIR DISPOSITION (ledger T8, sixth batch).
 They were an optional field on every record, so the reporter fell back to
 an empty list for a declared-name refusal that always carries them; the
 union now says which record carries them, and the cache reader refuses a
 declared-name refusal without them.

 THE QUOTE COUNTS COME WITH THEIRS (ledger B42), for the reason
 `QuotedPassages` gives: the reporter recounted the whole archive where the
 guard counted its judged part.

 @example
 ```ts
 const changed = record.outputText !== incumbentText;
 ```
 */
export type TranslateSliceRecord = TranslateSliceRecordFields & (
  | {
    /**
     The declared-name guard refused the replacement.
     */
    readonly disposition: 'refused-declared-name';

    /**
     Declared forms the replacement dropped, which are why it was refused.

     STORED, unlike the alignment refusal's sentence, because these forms
     name no slice index and so survive a record being resumed at a
     different position. The reporter has no preparation to recompute them
     from.
     */
    readonly droppedDeclaredNames: readonly string[];
  }
    | {
    /**
     The quote guard refused the replacement.
     */
      readonly disposition: 'refused-quote-loss';

    /**
     Quoted passages the guard compared, which are why it was refused.
     */
      readonly quotedPassages: QuotedPassages;
    }
    | {
    /**
     What the driver did with the stage result, where neither a declared name
     nor a quoted passage refused it.
     */
      readonly disposition: Exclude<TranslateDisposition, 'refused-declared-name' | 'refused-quote-loss'>;
    }
);

/**
 One passage this run left missing, with why and what it heard.

 @example
 ```ts
 const unfilled: UnfilledSlice = { sliceIndex: 4, reason: 'no-candidate', findings, };
 ```
 */
export type UnfilledSlice = {
  /**
   Slice the archive has no translation for.
   */
  readonly sliceIndex: number;

  /**
   Why this run produced none either.
   */
  readonly reason: UnfilledReason;

  /**
   What the stage gathered before it gave up: which translators were heard,
   what collapsed, what the judges counted.

   ALSO IN {@link TranslateDocumentResult.findings}, deliberately. The flat
   list is what a corpus-wide count reads, and this is what says which passage
   each finding belongs to; neither answers the other's question.
   */
  readonly findings: readonly string[];
};

/**
 Result of translating one whole document.

 Has no partial variant. A lane that ran out of time throws, leaving its
 settled slices in the cache for the next attempt, because a result reporting
 unvisited slices as unchanged is indistinguishable from a document that
 needed no translation.

 @example
 ```ts
 const { translatedText, changedSliceCount, } = await translateDocument({ ... },);
 ```
 */
export type TranslateDocumentResult = {
  /**
   Translation rebuilt from accepted per-slice text.
   */
  readonly translatedText: string;

  /**
   Slices preparation produced, which every other slice count of this result
   is out of.
   */
  readonly sliceCount: number;

  /**
   Slices whose accepted text SHIPPED, which is what the document carries.

   Counted after assembly rather than from the records, because the footnote
   guard can withdraw a replacement the judges chose: a record saying it
   changed and a document carrying the archive's text are both true, and this
   count belongs to the document.
   */
  readonly changedSliceCount: number;

  /**
   Slices where the alignment guard refused a replacement the judges chose.
   */
  readonly refusedSliceCount: number;

  /**
   Slices whose replacement was withdrawn at assembly.

   Three causes, and the findings are what tell them apart: a footnote the
   assembly would have left worse than the archive's, a structural regression
   no identifier names, and a surviving set that reassembles to the archive
   text exactly. The last one withdraws replacements nothing is wrong with,
   because a document identical to the archive carries no change to name.
   */
  readonly withdrawnSliceCount: number;

  /**
   Slices the returned document CARRIES a replacement for, in document order.

   Named rather than counted, because what this lane is measured against is
   per slice: which slices it replaced, and whether the repair lane touched
   the same ones. A count answers neither, and re-deriving the set from the
   records would re-derive it WRONG, since a record says what the slice chose
   rather than what the document carries.
   */
  readonly changedSliceIndices: readonly number[];

  /**
   Who won each slice and whether the document kept it, in document order.

   The index sets say WHICH slices moved; this says who the text came from
   and how the judges got there. Every question asked of this lane since it
   was built is per slice and per producer, and a count answers none of them.
   */
  readonly sliceSelections: readonly SliceSelection[];

  /**
   Slices whose replacement the assembly guard took back, in document order.

   Ordered by `orderedChangeSets` rather than left in the order the guard
   worked, so a reader joining two lanes slice by slice reads both sets by one
   rule. Disjoint from {@link TranslateDocumentResult.changedSliceIndices} by
   construction, and the same fact
   {@link TranslateDocumentResult.withdrawnSliceCount} counts.
   */
  readonly withdrawnSliceIndices: readonly number[];

  /**
   Shipped slices whose text the assembly guard trimmed, with the text the
   document carries: the decision with an orphan definition block cut.

   The delivery ledger reads a shipped row's text here before it reads the
   decision, so the rows say what the document carries. Empty when every
   shipped slice carries its decision whole.
   */
  readonly trimmedReplacements: readonly SliceReplacement[];

  /**
   Slices resumed from the cache rather than translated this run, so a cheap
   run is distinguishable from a lane that found nothing to do.
   */
  readonly resumedSliceCount: number;

  /**
   Whether this document is a whole translation.

   READ THIS BEFORE {@link TranslateDocumentResult.translatedText}. A result
   whose status is `unfilled` carries a document with passages the archive
   never translated and this run could not either, so publishing it or
   comparing it against a complete one measures something else. The field
   exists because the gaps were nameable and still missable: a consumer
   reading only the old fields would have seen an ordinary success.
   */
  readonly status: 'complete' | 'unfilled';

  /**
   Slices with NO translation in the archive that this run could not fill, in
   document order, each with the reason and the evidence.

   A different thing from every other set here, and the reason it is its own
   field. A slice that is unshipped, unwithdrawn and unchanged elsewhere in
   this result means the judges looked and kept the archive's wording; these
   slices have no archive wording to keep, so the document carries the gap it
   came with. They settle no record and cache nothing, so the next run asks
   again.

   STRUCTURED RATHER THAN A LIST OF INDICES, because the evidence has to have
   an owner: several unfilled slices flatten their stage findings into one
   document-level list, where nothing says which passage each belongs to.
   */
  readonly unfilled: readonly UnfilledSlice[];

  /**
   One settled record per slice, in document order.

   SHORTER THAN THE SLICE COUNT when {@link TranslateDocumentResult.unfilled}
   names any slice, since a slice that produced nothing settles no record.
   */
  readonly slices: readonly TranslateSliceRecord[];

  /**
   What this lane DECIDED for every prepared slice, beside the archive's own
   wording, in document order.

   Built at the document level rather than stored on
   {@link TranslateDocumentResult.slices}, which are CACHE records: an
   incumbent belongs to a preparation, and a slice resumed from an earlier run
   would otherwise serve the wording that preparation had then. Carries no
   shipped flag for the same reason, since whether a slice shipped is decided
   by an assembly guard reading the whole document and can differ between two
   runs of the same slice.
   */
  readonly sliceTexts: readonly LaneSliceText[];

  /**
   Every slice's findings, flattened for the artifact.
   */
  readonly findings: readonly string[];
};

//endregion Translate document contract
