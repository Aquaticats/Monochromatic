import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly/ts';

//region Repair slice key
// What makes two runs' repair slices the SAME slice, for cache purposes.
//
// Split from the driver rather than inlined there, for two reasons. The version
// constant carries the longest comment in this package, because every bump has
// to record why it was needed, and a reader of the driver does not want it. And
// the key is the one piece of the driver that can be tested without a client:
// inlined, nothing could check that a roster change moves it.
//
// Mirrors `translateRunShape` and `translateSliceKey`, deliberately. The two
// lanes cache into one directory, and a reader comparing what each folds into
// its key should be able to read them side by side.

/**
 Slice-cache schema version, mixed into every cache key.

 The cache stores serialized `ChunkRepairOutcome` values, so a run resuming
 across a change to that shape would splice yesterday's outcomes into today's
 report and silently answer a question they never recorded. Cached repair
 provenance is the live example: outcomes written before repairs existed carry
 none, and a resumed slice would contribute ungradable items to a precision
 sheet without anything looking wrong. Bump this whenever
 `ChunkRepairOutcome` changes shape OR an existing field changes meaning; the
 structural guard in the cache store catches only the first of those.

 A GATE change is the second kind and is the easiest to miss, so version 6 is
 recorded here as the example: the footnote-integrity gate left the prompts,
 the roster and the texts identical, so the structural guard and `runShape`
 both matched, while a candidate the old gate shipped may be one the new gate
 refuses. `runShape` cannot catch that by construction, because it covers what
 the models are ASKED and a gate changes only what the code does with their
 answers. Nothing enforces this bump; it was missed once already, on the very
 commit that added that gate.

 Version 7 is the same lesson applied straight away: widening typography
 restoration from the replaced region to the whole document changes the text
 that ships, with every prompt and every roster identical. Version 8 likewise:
 naturalness eligibility stopped counting a repaired parse as a degraded one,
 so slices that skipped the lane now enter it.

 Version 9 is the first bump for pure TELEMETRY, and it is still required:
 `quote-not-found` findings gained a suffix naming whether collapsing soft
 line breaks would have located the quote. No claim changes fate, but
 `findings` is part of the cached payload, so a resumed slice would report
 the old bare reason and understate the count the suffix exists to produce.

 Version 10 is telemetry again, and required for the same reason: the outcome
 gained `claimAttributions` and `heardCriticIds`. No claim changes fate and no
 text changes, but a slice resumed from a version-9 file would carry neither,
 so an entry would silently mix attributable and unattributable slices. That
 is precisely the population confusion the fields exist to prevent, so
 resolving it by tolerating the old shape would defeat them.

 Version 11 is the first bump for a BEHAVIOUR change since 8: accepted issues
 naming one defect in one place are now merged before envelopes are cut, so a
 chunk emits fewer issues and cuts fewer envelopes than it did. A slice
 resumed from a version-10 file would carry the duplicates the merge exists to
 remove, and would have spent the editor's budget on them, so the two cannot
 be mixed within one entry. Ratified in
 `doc/decision/translation-repair-duplicate-issue-emission.md`.

 Version 12 is behaviour again: the preservation gate now runs inside
 `applyPatchOperations` and rejects an operation that drops content no
 accepted issue quoted. A slice resumed from version 11 carries text an edit
 the gate would now refuse already changed, so the two cannot be mixed. (Ledger
 L4 later measured that the gate can see almost nothing, since envelopes are
 cut from the licensed quotes; the markup-atom check guards inside them.)

 Versions 13 through 16 were bumped without a paragraph here, which breaks
 the rule that every bump records why it was needed; recovered from the commits that moved the
 constant rather than reconstructed, so each is what its own change says it
 is. Version 13 (`4a4a8b6bc`) pairs sections only when the aligner is forced
 to, and never blocks on a refusal, so a slice resumed from 12 was paired by a
 different rule. Version 14 (`d1d1d874e`) stops the editor composing poetry
 over verse. Version 15 (`2193b5877`) computes line structure instead of
 asking the editor to notice it. Version 16 (`a14ea4f94`) records WHY the
 apply gate refused an operation, which is the findings payload rather than
 the text. The first three change what ships and the last changes the record,
 and all four are the kinds this note already names.

 Version 17 is behaviour by way of WHO WAS HEARD. The channel-marker stripper
 now matches the shape of a truncated `<|word|>` tail rather than the single
 exact string `|>`, so replies that reached version 16 as lost voices now
 parse. A slice resumed from version 16 was decided by a smaller panel than
 the same slice would convene now, and a chunk whose critic went unheard is
 not the same chunk as one whose critic spoke.

 Version 18 changes the RECORD rather than the text: every stage now emits a
 `stage-voice-lost` finding naming the models that never answered, including
 when quorum was met. Findings are read as a whole per entry, so an entry
 holding version-17 and version-18 slices would under-report voice loss on
 exactly half of itself while looking complete.

 Version 19 completes that record in two ways it was still incomplete. One
 finding is emitted PER MODEL rather than one naming a list, so counting
 findings counts voices lost rather than gathers that lost at least one. And
 the selection path carries its findings at all: `selectBestCandidate` built
 them and every caller discarded them, so a judge going silent left no trace.
 A version-18 slice under-counts on both.

 Version 20 finishes the audit that found version 19. The introduced-defect
 probe emitted findings and BOTH its live callers dropped them, on the
 accuracy path and in the naturalness lane, which is the one stage where a
 lost voice is least distinguishable from a clean result. Every producer of
 findings is now consumed except `derivability-probe.ts`, which is reached
 only by the recall benchmark and writes into no per-entry artifact.

 Version 21 widens what the naturalness lane may touch. Eligibility excluded
 every paragraph containing a newline, which rejected soft source wraps along
 with authored line breaks; it now excludes only authored line breaks. Measured over the
 92 entries at the pinned corpus commit, 811 of 2067 prose paragraphs carry an
 internal newline and 29 carry a hard break, so a version-20 slice was refined
 over a small fraction of the prose the lane could have reached. Those cached
 slices hold text the current lane would have been free to rewrite.

 Version 22 corrects the line-structure sentence the editor is handed. It
 opened `This region's CURRENT TEXT IS line-structured`, while the predicate
 behind it reads the SOURCE, and on the case it exists for the two disagree:
 `Toka_ls`'s verse chunk is 21 source blocks at median 22 against 18 target
 blocks at median 101. Version-21 slices were edited under a sentence that
 asserted something untrue about the text in front of the model and then asked
 for one output line per INPUT line, which on an already-merged translation
 asks for the merge to be preserved.

 Version 23 moves the line-structure decision from the slice to the enclosing
 CHUNK. Version 22 asked the predicate about each slice, and the predicate
 needs at least five blocks before it will answer anything but false, so
 subdivision destroyed the evidence it reads. Measured on the version-22
 `Toka_ls` run: the verse chunk is line-structured at 21 blocks, median 22,
 and subdivides into seven slices of which ONE still trips it, while four
 others sit at medians 20, 22, 23 and 29, inside the verse range, and fail
 only for want of a fifth block. Version-22 slices therefore carry the
 corrected sentence on a seventh of the verse it was written for.

 Version 24 stops the introduced-defect screen discarding over-deletions. A
 cached slice carries the whole `repairChunk` outcome, probe report included,
 so version-23 slices hold tallies screened by the old rule. That rule asked
 whether a claim restated an accepted issue by testing containment BOTH ways,
 which is right for added wording but wrong for removals: a removal claim
 quotes what DISAPPEARED, drawn from the same side the critic quoted, on a
 region that exists because the critic quoted something in it. Dropped wording
 CONTAINING the prior quote is the over-deletion signal itself, and it was
 being read as a restatement. Measured: removal-corroborated ran 159 across
 the original 56-entry run and 0 across every run after the reclassification
 landed, while corroborated held its per-region rate.

 Version 25 is behaviour by way of WHO WAS HEARD and WHO DECIDED, on two user
 decisions of 2026-08-15. The editor and refiner stages no longer wait for
 their whole roster, so a slice cached under version 24 was settled by a
 gather that could spend four deadlines recovering a voice this one stops
 asking for once quorum stands. And selection now seats producers, counting a
 ballot for the judge's own work at half weight, where version 24 removed them
 from the roster outright: the same candidates before the same models can
 reach a different winner. Neither change touches a prompt, which is exactly
 why the version has to move rather than the structural guard catching it.

 WHY 2026-08-15 DID NOT MOVE IT, recorded because a version that stays put
 needs the same account as one that moves, and this history's own rule says
 bumps have been missed exactly by nobody writing one.

 Two changes that day moved what `changed` MEANS. `winnerChangedText` stopped
 reading which candidate won and started reading the text, so a patch whose
 envelope operations cancel is now recorded as a slice nothing happened in.
 And the naturalness lane stopped stamping `changed` from the rewriter's
 verdict, which is measured against the accuracy text, and reads the archive
 text instead.

 NEITHER NEEDS A BUMP, for different reasons. Refinement outcomes are never
 persisted: the cache holds the accuracy outcome, and the lane runs again on
 every resume. And a version-25 accuracy record written before the first
 change can only be wrong in ONE direction, since the old rule answered
 `false` whenever the unchanged candidate won and that candidate carried the
 archive text: it can claim a change it did not make, never deny one it did.
 That is exactly the contradiction `sliceRecordAgrees` discards on resume, at
 a cost of one recomputed slice, and the run says so in its findings. Bumping
 would throw away every settled slice in the corpus to fix what the discard
 path already fixes one slice at a time.

 Version 26 takes the SLICE INDEX out of the key, on 2026-08-15, and this one
 does discard the corpus: every settled slice was keyed with its index in it.
 It is spent deliberately and once. Keeping the index meant that renumbering
 slices invalidated every slice after the change however untouched its text,
 and one-sided slicing renumbers by design, inserting a slice wherever a section has no
 translation. Paying it here means that change, and every slicing change after
 it, costs nothing IN RENUMBERING ALONE: moving a slice boundary, editing
 either text or changing its governance still misses, because each changes
 what the stages are asked. What a translator or an editor is asked is the source
 text, the incumbent, the governance flag and the run shape; where the slice
 sits is not part of the question, so it is not part of the key.

 Version 27 follows on 2026-08-15 for a change that touches no prompt and no
 text: `locateQuote` gained a pass that collapses soft line breaks, so a critic
 quote copied out of a wrapped paragraph now anchors where it used to be
 dropped. `critic-wire.ts` discards a claim it cannot anchor, which means the
 surviving issue set for a slice changed, and with it the patch and the settled
 text. The key holds the slice texts, the governance flag and the run shape,
 and all three are identical across the fix: the same key answers differently
 before and after it, which is precisely what a version is for.

 IT IS NOT THE ONE-DIRECTION CASE that let version 25 stand. That record could
 only overclaim a change, and `sliceRecordAgrees` catches an overclaim on
 resume at the cost of recomputing one slice. This one can differ in either
 direction and leaves no contradiction behind: a slice settled before the fix
 with a dropped wrapped quote reads as a clean settlement, indistinguishable
 from one where the critic found nothing. Nothing downstream can notice, so the
 version has to.

 `TRANSLATE_SLICE_CACHE_VERSION` deliberately does NOT move with it. Anchoring
 reaches the repair lane through `repair-stages.ts` alone; the translate lane
 never asks a critic to quote anything, so its settled slices still agree with
 what this code computes.

 TWO LATER ANCHORING CHANGES ALSO RODE INSIDE 27, checked rather than assumed:
 no slice-cache file had been written under it when either landed, so there was
 nothing on disk for them to disagree with. They are the uniqueness rule now
 judged over the broadest accepted form of a quote, and the lane literal this key now leads with.
 Once a run settles a slice under 27, the next behavioural change to anchoring
 takes 28.

 THE LANE LITERAL RODE ALONG WITH VERSION 27, which is the whole reason it
 landed on this day rather than another. `translateSliceKey` has always led
 with `'translate'` and this key led with nothing, so the isolation between the
 lanes rested on the store's file-name prefix alone. Folding the literal in
 changes every key, so it was recorded as waiting for a bump that was happening
 anyway rather than spending a corpus of its own. Version 27 is that bump, and
 no run had resumed under it when the literal went in, so it cost nothing
 beyond what was already spent.

 VERSION 27 STAYS PUT FOR THE REPAIR-LANE WINDOW, and this file's own rule
 is that a version holding still owes the same account as one that moves.

 The window is folded into the key itself rather than ridden on the version,
 and that is what makes the version unnecessary. A slice that has a neighbour
 now keys on its window, so it misses whatever it was worth and recomputes,
 which is correct: the stages are being asked a different question. A slice
 with NO neighbour, meaning a document of one slice, keys exactly as it did,
 and resuming it is also correct, because there is no window to have shown it
 and the question is byte-for-byte the one that was already answered.

 A VERSION BUMP WOULD BE STRICTLY WORSE HERE, not merely redundant. It would
 discard every settled slice in the corpus including the lone-slice ones whose
 question did not change, which is the cost version 26 paid deliberately and
 once. Folding the evidence into the key spends only the slices whose evidence
 actually moved.

 VERSION 28, on 2026-08-20, for the declared-name guard reaching this lane's
 acceptance. Unlike the window, this is not a question the key can carry: the
 declarations already ride in the run shape through `identityContext`, so a
 slice settled before the guard existed keys identically to one settled after
 and would resume with its refusal never asked. The guard is the reason to
 spend the corpus, and a guard any cache hit can walk past is not a guard.

 VERSION 29, the same day, for the opposite kind of change: the record grew
 rather than the question changing. `ChunkRepairOutcome` now carries `rounds`,
 every judged round with its ballots, and `droppedDeclaredNames` unconditionally
 rather than only on a refusal.

 A version-28 record is recomputed either way, because `isChunkRepairOutcome`
 now requires both arrays and a record written without them fails the check.
 The bump is here so the NUMBER stays honest: one version naming two record
 shapes is exactly what this file's versioning exists to prevent, and reading
 the structural check as a substitute would leave a reader of the history
 unable to tell when the shape moved. It costs nothing beyond version 28,
 which no run had resumed under.

 VERSION 30, on 2026-08-23, for the same kind of change again: the record grew.
 `ChunkRepairOutcome` now carries `authorship`, naming the models that wrote
 the repaired text, because the naturalness lane rechecks that text and may do
 so from cache, long after the stage that knew the answer returned.

 A version-29 record is recomputed either way, since `isChunkRepairOutcome`
 now requires the field. The bump keeps the NUMBER honest, on the same
 reasoning version 29 was spent on: reading the structural check as a
 substitute leaves a reader of the history unable to tell when the shape moved.
 Resuming a version-29 record instead would silently recheck refined text with
 nobody named as its author, which is exactly the discount this field exists
 to feed.

 VERSION 30 STAYS for complete-form contributor authority.
 Repair output cannot ship directly:
 differing lane text reaches lane-contest winner floor,
 while uncontested identical text matches translate lane already floored by
 `TRANSLATE_SLICE_CACHE_VERSION` 9.
 Main consolidation candidates are independently floored before settlement.

 Version 31 is telemetry, on the same reasoning as version 10: every
 adjudicated issue now carries `filedBy`, the critics that filed each member
 claim (owner, 2026-09-24: "we're not logging enough then. Refine how we
 log."). No claim changes fate and no text changes, but a slice resumed from
 a version-30 file would carry issues naming nobody, so an entry would mix
 attributed and unattributed issues, which is the confusion the field exists
 to end.

 VERSION 32, on 2026-09-27, because what `resolved` means moved: one cast
 checker ballot resolves nothing (`MIN_RESOLUTION_BALLOTS`). Candidate
 selection counts resolved issues, so a slice settled under version 31 may
 have shipped a patch whose only confirmation was one checker, and resuming
 it would carry that decision past the floor.

 VERSION 33, the same day, for both kinds of change at once. The QUESTION
 moved: the panel sheet asks each verdict's reason before its vote (owner,
 "Reason before vote"), and `repairRunShape` holds rosters and context but no
 sheet text, so a slice settled under the old sheet keys identically and
 would resume with votes cast without the reasoning the rule asks for. And the
 RECORD grew: every stored panel ballot carries its `reason`, so resuming
 would mix reasoned and unreasoned ballots in one entry, the confusion
 version 31 was spent to prevent. It costs nothing beyond version 32: the
 newest of the 14,596 slice-cache files under the agent runs was written at
 04:26 UTC that day, before version 32 existed.

 EVERY REPAIR-LANE CHANGE SINCE VERSION 32 RIDES INSIDE 33, checked rather
 than assumed, since none of them moved the number when it landed (ledger
 M25): the panel sheet reading the declared names (`a32739504`), the editors
 and their selections reading them and the references (`f16d5e8bf`), the
 introduced-defect probe reading them with the community renderings and the
 apparatus rule (`b6df6d5ee`), and the editor and checker removing an accepted
 addition rather than softening it (`d91b338d7`). The checkers seated in the
 benchmark's order (`f10de5198`) change the roster, which is key input. The
 house rules' "-re" line naming the meter that measures beside the metre
 (ledger K13, `403db3c6e`) reaches the critic, editor and panel sheets and
 rides inside too, checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 The glossary audit's corrections ride inside 33 too (ledger C3 to R16,
 `357f534b7`), checked on 2026-09-28 against the newest slice-cache file under
 the agent runs, still 04:26 UTC on 2026-09-27 (eighteen files after 04:20 UTC that
 day, none after 04:27 UTC): the COMMUNITY RENDERINGS block the editor and the
 introduced-defect probe read counts a rendering inflected and bounded, and
 the grammatical English house rule on the critic, editor and panel sheets
 states the doubled preposition. The terms, renderings and whys reach the
 identity context, which the key hashes.

 Rides inside 33 too: a panel ballot or checker report carrying no usable
 verdict on its sheet is no heard voice, so the gather re-asks the seat rather
 than closing the round on it (ledger L8, `abfc69393`); checked on 2026-09-28:
 still no slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 33 too: every gather now keeps a seat that answered unreadably
 out of the same-prompt retry rounds and re-asks each such seat, whichever
 round it came in, in the nudged recovery round (ledger P2, `005692e11`),
 which changes whose voices a round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 33 too: a reply whose complete JSON value more text follows is
 read rather than lost (ledger P8, `cac097368`), which changes whose voices
 every round hears; checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 Rides inside 33 too: a decision seat whose state the endpoint refuses
 as past its context, or that a stage cannot ask, now reads as out of reach
 for that ballot rather than as a lost voice (ledger P13, `a991ef1e1`), which
 changes the quorum a select round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 33 too: a winning patch sheds every edit whose issue the
 checkers did not confirm and at least one voted worse, and the reduced patch
 is proved once more (ledger L3, the owner's ruling of 2026-09-28,
 `69c149471`), which changes the text a chunk ships and its checker record;
 checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 33 too: the editor preservation gate refuses an edit that loses
 a footnote reference, link destination, MDX expression, inline code or tag
 no addition claim quoted, and the editor sheet names every such kind (ledger
 L4, the owner's ruling of 2026-09-28, `b5338610e` and `c4a4fbb62`), which
 changes the editor's question and which edits a chunk may apply; checked on
 2026-09-28: still no slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 33 too: the same gate refined for quality, letting an edit
 re-mark markup the translation authored and move an atom between envelopes,
 reading the whole source document, re-applied by the worse-voted strip, and
 stated on the editor sheet (ledger L4, `b07f8ac48` and `ab84cbfd7`), which
 changes the editor's question and which edits a chunk may apply; checked on
 2026-09-28: still no slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 33 too: a reply that could not be used is re-asked once,
 nudged, of the same model on another provider through the uniqueness
 wrapper's claims (ledger P9, `7011d72cc`), which changes whose voices every
 round hears; checked on 2026-09-28: still no slice-cache file newer than
 04:26 UTC on 2026-09-27.

 Rides inside 34 too: Bedrock reads dry with 1.33 USD still left, and its
 ledger holds every billed attempt at its bound (ledger P1, `2a108dfb3`,
 `107763dbb` and `7cfd5ae4b`), so a Gemma call moves to OpenRouter sooner
 as Bedrock nears its credit, which changes whose voices a round hears;
 checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 34 too: the recovery round re-asks a reply the length
 limit cut with a nudge naming the cut, apart from one off the shape
 (ledger P10, `ce0ef7b51`), which changes what such a seat is asked and
 whose voices a round hears; checked on 2026-09-28: still no slice-cache
 file newer than 04:26 UTC on 2026-09-27.

 THE PRE-LAUNCH CHECK OF 2026-09-28 (ledger M28): 33 was set
 in `f2cd70ece` at 02:41 UTC on 2026-09-28, after the newest slice-cache file under the
 agent runs (04:26 UTC on 2026-09-27), and no slice-cache file has been written
 since, so no answer cached under an earlier question can be served under
 this number. Every source commit since then rides inside it, those the
 accounts in this TSDoc name and those they do not (`corpus-run/cache-account-audit.ts`, ledger M28).

 VERSION 34, 2026-09-28, the first number spent while a run held the
 previous one. TianqiChen66621 ran the frozen dist of `9a3f28b30` and wrote
 28 slice-cache files under 33, from 10:26 to 10:49 UTC that day, so nothing may
 ride inside 33 from here on. Both kinds of change at once (ledger L5): the
 RECORD, because the tally holds an acceptance settled at neutral for a human
 (`e87e353ae`), so a slice settled under 33 would resume carrying accepted
 neutral issues, their envelopes and their edits; and the QUESTION, because
 the critic and panel sheets define the severity scale, tell the critic a
 claim naming nothing wrong is no issue, and ask a supporter to lift a real
 defect filed neutral (`severity-scale.ts`), none of which
 `repairRunShape` hashes. `REFINE_CACHE_VERSION` needs no move for the
 tally: its key hashes the adjudicated issues, status included. The pass
 retired those 28 files when TianqiChen66621's artifact landed, so the bump
 turned out to guard nothing on disk; it stays, since it was right when made.

 Rides inside 34: the declared-identity rule says a footnote marker carries
 its note, on the critic, panel, editor and introduced-defect probe sheets
 (ledger L5, `868e848d3`); checked on 2026-09-28 after TianqiChen66621: no
 slice-cache file newer than 04:26 UTC on 2026-09-27 remains.

 Rides inside 34 too: the critic and panel sheets share one block of
 obligatory differences, which replaces the panel's line on small words
 (ledger L5, `64a4bf63a`); same check, same result.

 Rides inside 34 too: patch application clamps each replacement line to the
 deepest quote its context allows (class one hundred eighty-seven,
 `58287ebe3` and `05a18ed02`), which changes the text an edit ships; same
 check, same result.

 Rides inside 34 too: the resolution checkers read the declared names with
 their rules, the cited references and each claim's quotes, on the proof and
 its worse-vote recheck (ledger L14, `resolution-sheet-evidence.ts`), which
 changes the ballots that decide resolution and strip edits; checked again on
 2026-09-28: the only slice-cache file not older than 04:26 UTC on 2026-09-27 is
 the consolidation entry written that minute.

 Rides inside 34 too: claim summaries fold onto one line on the panel,
 checker, editor and probe sheets, and a panel quote keeps its later lines
 indented under its evidence item (ledger L14(d), `3be658509`), which changes
 the sheets of any slice whose claims carry a line break; same check, same
 result.

 Rides inside 34 too: the chunk selection tells its judges that a decline
 counts as no vote and keeps the existing English only when every judge
 declines (ledger L14(c), `e7e530564`); same check, same result.

 Rides inside 34 too: the editor sheet and both editor selections say that
 saying something twice adds content unless the ORIGINAL says it again at
 that place (ledger L14(b), `49d3c285b`); same check, same result.

 Rides inside 34 too: the declared-name guard and the preservation gate's
 tokenizer read text by code point, so a handle written in letters beyond
 the first plane is checked rather than projected to nothing, and an
 ideograph beyond it is a token (ledger B21, `declared-name-survival.ts`,
 `preservation-tokens.ts`); over the pinned archives and settled pages no
 survival answer changes, and neither carries such an ideograph; checked on
 2026-09-29: still no slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 34 too: the declared-name guard a patched chunk passes carries
 a declared name only where it stands as a name, a letter meeting a digit and
 a small letter meeting a capital counting as edges (ledger B23,
 `name-projection.ts`, `11d029fde`); no stored patch was replayed, so the
 effect is read from the translate replay, where four wordings of two slices
 now drop a declared form; checked on 2026-09-29: still no slice-cache file
 newer than 04:26 UTC on 2026-09-27.

 Rides inside 34 too: the introduced-defect screen a patched chunk's damage
 evidence reads takes a prober's quote and an accepted issue's through the
 evidence fold (ledger B24, `introduced-defect-screen.ts`); of the 650 stored
 claims one turns from unanchored to corroborated; checked on 2026-09-29: the
 newest slice-cache file is still the one of 04:26 UTC on 2026-09-27.

 Rides inside 34 too: the introduced-defect probe whose claims a patched
 chunk's damage evidence reads carries the narrative bound beside the
 apparatus kinds it excuses a drop by (ledger B28, `introduced-defect-wire.ts`);
 checked on 2026-09-29: still no slice-cache file written after 04:27 UTC on
 2026-09-27, where a control from midnight finds 494.

 Version 35 is a TYPOGRAPHY change, the kind version 7 names: the quote style the repair restores into the replaced region
 (`restoreTypography`, called by `apply-patch.ts` before every gate) is read from the page parse now, so a straight mark in a link
 destination or title, an address, code, math, an expression or a tag is left alone where the old mask could curl it, a lone
 straight mark before markup (`'*cat*'`) opens a quotation, and a quoted number (`'42'`) pairs. Every prompt and every roster is
 identical, so the key is too, and a record the old restoration wrote holds replacement text the new one would ship differently.
 The agent runs hold no record this number could make stale as far as the checks that could be made show: the audit of
 2026-10-05 recorded the newest of 13,714 records as written before 34 was set, `cache-account-audit` finds none under this
 worktree's own runs directory, and a `find` for files newer than 34 over the one runs directory that could be listed found
 nothing, but its control since January found nothing either, so it proves no absence. The number moves because the one check
 that would let the change ride inside 34 could not be made with a control.

 Version 36 is a TYPOGRAPHY change of the same kind (n159): the restoration reads the context of a mark as the text a reader
 sees, so a mark beside emphasis, strike or code delimiters, link brackets, a tag or a comment stands beside the text on the
 other side of it (`<b>'hi'</b>` opens at the start, the `'n'` of `rock *'n'* roll` is an elision), a quoted number pairs by the
 reading that leaves fewer marks unpaired (`'the '90s'` reads `‘the ’90s’`), a fragment an edit rewrote takes its prose from the
 slice it lands in (`apply-patch.ts`), and the text of a shortcut reference or of a link whose label is an address is no prose.
 Every prompt and every roster is identical, so the key is too, and a record the old restoration wrote holds replacement text
 the new one would ship differently. The check that would let the change ride inside 35 (no slice-cache record written after
 the change, shown beside a control that finds records) was not made, since no runs directory is read in this round, so the
 number moves.

 Version 36 also covers the tense of quoted words (n161, TianqiChen666): the house rule every repair sheet carries and the tense
 lines of the judges, the polish gate and the selection sheets now say that words quoted as someone wrote or said them keep the
 speaker's tense, so a critic, editor or judge answer stored under 35 may hold a quotation moved into the past. The key hashes the
 version, the run shape and the texts and not the sheet, so a record written under the old sheet would be served under the new
 one; no record had been written at 36 when this was written: the numbers n159 set were first committed in `277145a862` together
 with R1, and the audit of that commit found 0 slice-cache records under the keyless worktree's runs directory
 (`s228/cache-audit-n161.txt`, line 992), so this rides inside the number n159 set.

 Version 37 covers the tense sentences moving again (n166, Z11): the house rule's sentence that puts the narrative of a life in the
 past and the judge tail's sentence that says no rule holds a life in the present now hold the quoted-words exception inside them,
 so a critic, editor or judge answer stored under 36 was written under the earlier wording of the same rule. The key hashes the
 version, the run shape and the texts and not the sheet, so such a record would be served under the new one. Whether one exists is
 not measured here: the audit of n161 found none, but the launch started since at `5f6c8593d7` runs at 36 and no runs directory is
 read in this round, so the number moves, as the consolidate, lane-contest, refine, translate and page title versions move for the
 same sheets. No record has been written at 37, which this change sets first.

 Version 38 covers the fixes of the XingZ6015 read (2026-10-20), which change what these sheets say and what their floors accept:
 reader protection reaches only the account of how a death or an attempt happened (owner, 2026-10-20,
 `house-rule/src/house-policy.ts`), which every sheet splicing the house rule restates; judges no longer see a note between
 translators (`document-structure/src/translators-note.ts`); the archive-block review says the page it publishes is complete; a
 consolidation challenge round seats a proposal two ballots named over a fallback the contest majority called flawed; and the
 declared-name floor meets a lost form of the subject with another declared form of the same person. The key does not hash the
 sheet or the floors, so a record written under the old ones would be served under the new. The XingZ6015 pass wrote records at the
 old number on `sha256-tree-v1:a92725968adf…`, so the change cannot ride inside it and the number moves, as every version whose
 sheets splice the house rule moves with it; the pairing version stays, since no pairing sheet carries the house rule. No record
 has been written at 38, which this change sets first.

 Version 39 covers the cut of a note between translators that stands alone on a quoted line
 (`document-structure/src/translators-note.ts`): the editor ensemble's candidate selection slate cut such a note with the spaces
 before it and left its quote markers, a bare `>   >` line that the judges read as a paragraph break inside the quote that the
 archive does not have (zheermao101, run zheermao15), and it now cuts the line. Of the 34 notes in the 92 pinned English archives
 one stands alone on a quoted line (zheermao101), so only a sheet that shows that archive's text changes, and the cut of every
 other note is what it was. The key hashes the version, the shape of the run and the texts, not the sheet, so a record written
 under the old cut would be served under the new. The check that would let the change ride inside 38 (no slice-cache record
 written after it, shown beside a control that finds records) could not be made with a working control: run zheermao15 started at
 02:12 UTC on 2026-10-20, after 38 was set, its slice-cache directory is empty, and a control from 2026-10-19 finds none either,
 so the number moves, as the consolidate, lane-contest, refine and translate versions move for the same cut. The consistency,
 page title and pairing versions stay, since none of their sheets cuts a note. No record has been written at 39, which this
 change sets first.

 Two later changes to the same cut ride inside 39. A second note on one line no longer takes the line
 break with it (5b75add81, d60b27a9e, 003983067), and a note ending a CRLF line keeps the line's carriage return
 (acf6ada60, e80301bf3). None of the 92 pinned English archives has two notes on one line or a carriage return, so no
 sheet text changes, and no record had been written at 39 when they landed: the only run since it was set,
 CuspariaKLSY18, runs a6a0ed7dc0, which predates it.

 Version 40 covers the judges of the repair lane no longer seeing a note between translators
 (`document-structure/src/translators-note.ts`): the critic, the panel, the introduced-defect prober and the resolution checker
 showed the archive's text with every note the archive's translators left each other, and now cut them against the original
 (12e147ae40, 5498ee860b); the coverage sheet that insertion admission reads cuts them from the translation and from the evidence of
 a prior verdict (89df3896d4); and the reference attestation sheets cut them from the archive rendering (fcd6d13fa3). The labels
 of the repair slate's evidence name the original only where they are the original's own, so the candidate selection slate
 the editor ensemble sends is what it was. The key hashes the version, the shape of the run and the texts, not the sheet, so a
 record written under the old sheets would be served under the new. Runs zheermao16 and CuspariaKLSY19 started on `a5dc23c5bb`, which holds 39 (set in
 `aa0ab6ff0`), so a record may stand at 39 and the check that would let the change ride inside it, no slice-cache record written
 after it shown beside a control that finds records, was not made, so the number moves, as the refine and translate versions
 move for the same sheets. Over the 92 pinned English archives 10 hold a note their original lacks (34 notes in all), so a sheet
 changes for at most those 10 entries. The consistency sheets cut them too (7ee1755b24), the pairing sheets (eaab1b1ed9) and the
 rendering audit (ebac709b03); their keys and generations are accounted for beside their own versions, and this one does
 not hold them. No record has been written at 40, which this change sets first.

 Rides inside 40 too: a quote the critic, the introduced-defect prober, a coverage voice or a reference attestation voice copied
 from one of those sheets across the place a note between translators was cut from used to stand in no stored text, and now
 anchors in the stored words on each side of the note (`document-structure/src/translators-note-view.ts`,
 `typography/src/locate-quote.ts`). `critic-wire.ts` drops a claim it cannot anchor, a prober claim it cannot place is
 `unanchored`, a coverage voice it cannot place counts as not anchored, and an attestation it cannot place is never verified, so
 the claims, the corroborated damage lines and the attested details of an entry holding such a note differ, in one direction:
 a quote that could not anchor does. The key hashes the version, the shape of the run and the texts, not the sheet, and not the
 attested details, so a record written before would be served after. Of the 12 run logs in the agent directory, none started on a
 tip that holds `43e735f7da`, where 40 was set: the newest, `a5dc23c5bb` of zheermao16 and CuspariaKLSY19, does not, and the
 others started on earlier ones, so none could write a record at 40. That is the evidence of the logs' `START tip=` lines and
 `git merge-base --is-ancestor`, shown beside a control that finds ancestry (`2eeef91fac` and `89ff91202e` of `HEAD`), and not a
 search of any slice-cache directory; a run on a worktree whose log is not among those 12 is not covered by it. Over the 92
 pinned English archives 10 hold a note their original lacks (34 notes in all), so a record changes for at most those 10 entries.
 The consistency version stays: its count reads the stored English and leaves a place unlocated when a quote crosses a note
 (`consistency/src/consistency-count.ts`), as it did, and its cache holds ballots only. The translate, refine, lane-contest and
 consolidate versions ride inside their own numbers, accounted beside each.

 Version 41 covers the selection ballot and the coverage judge giving their reasons before their votes (owner, 2026-10-21,
 "Reason first everywhere"; `e5bc27f9f0`): the editor ensemble's candidate selection (`adjudication/src/candidate-select-wire.ts`,
 now `reason` then `best`) and the coverage sheet that insertion admission reads (`archive/src/coverage-wire.ts`, now `reason`,
 `quote`, then `coverage`). The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. Runs zheermao21 and CuspariaKLSY21 started on `3637592bd4`,
 which holds the commit that set 40 (`git merge-base --is-ancestor`), so records may stand at 40 and the change
 cannot ride inside it; no search of any slice-cache directory was made, and the number moves. The consolidate, lane-contest, translate and refine versions move for the
 same ballots (29, 14, 23, 12). No record has been written at 41, which this change sets first.

 Version 42 covers the answer schemas of the repair lane declaring their properties in alphabetical order, and the keys that
 moved to make it so (owner decision of 2026-10-21 under the ruling "Reason first everywhere"; ledger B762): Charm Hyper
 returns a tool call's keys alphabetically, so only a declared order that is alphabetical gives every provider the same
 order. The panel verdict (`groups` before `verdicts`, and `severity` before `vote` in each), the critic (`sourceQuote` before
 `summary`), the introduced-defect check (`verdictReason`, the reason that stays last, in place of `reason`), the candidate
 selection and the coverage judge (`assessment`, `copiedSpan`), and the editor's rewrite (`newText` before `region`) changed
 their reply keys, instructions or both. The consolidate, lane-contest, translate and refine versions move for the same
 sheets (30, 15, 24, 13).
 The key hashes the version, the shape of the run and the texts, not the sheet and not the rule that reads the
 ballots, so a record written before would be served after. The previous number was set in `4b9fc9e645`, and this worktree's
 runs directory holds no slice-cache record (the audit `cache-account-audit` found 0, and no control that finds records was
 run, since the pass that writes them runs in another worktree this change does not read), so whether a record stands at
 41 is not known and the change cannot ride inside it: the number moves. No record has been written at 42, which this
 change sets first.

 Version 43 covers the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816): once the straggler grace
 ends, a short bench whose live seats are exactly its quorum closes one voice below it
 (`stage/src/stage-reachable-quorum.ts`), and a refusal inside a round sizes the need again
 (`stage/src/stage-round.ts`). Every round of the panel, the critic, the introduced-defect check, the candidate
 selection, the coverage judge and the editor gathers through that rule, so a record settled under the rule of
 2026-09-09 may hold an outcome the new rule reaches differently, on one voice fewer. The key hashes the version, the
 shape of the run and the texts, not the rule that closes a round. A build whose pipeline digest differs discards the
 namespace before it reads it (`openNamespacedCache`), and the digest holds the stage code, so that retires such a
 record; the number moves too, as the belt beside that brace, so no stored record stands under a number whose rule
 differs. Run zheermao23 is armed for 2026-10-22 19:16 UTC on `65e8325303`, which holds 42
 (`git merge-base --is-ancestor`), so records may stand at 42, and the change cannot ride inside it. The artifact generation
 moves to 20 for the same rule, and the consolidate (32), lane-contest (16), translate slice (25), refine (14),
 consistency (7), page title (6) versions move with this one; the pairing version stays, by its own account of
 2026-10-18 (`slice-reading/src/pairing-cache-version.ts`). No record has been written at 43, which this change sets
 first.

 Version 46 covers three moves that main and the Claude route each made at a number of their own: the settlement a
 cache may hold (`silentStages`, set as 44 on main), the system text and answer schemas of the route (set as 44
 there) and the reader taking a capital-only difference as the listed value (set as 45 there). Both lines held 44, so
 a merge that kept either 44 would serve a record written by the other line, and a build with the fold and without
 `silentStages` holds 45; 46 differs from every number either line wrote, so no record of either line stands at it.
 No record has been written at 46, which the merge sets first. The account of each move follows, each in the words
 of the line that made it.

 The settlement that may be cached (first set as 44 on main): the outcome gains `silentStages`, the stages that
 heard fewer voices than quorum as data, and the cache refuses to write or resume a settlement that names one
 (`repair-cache-gate.ts`, `slice-cache-store.ts`). Run XingZ6018 (2026-10-22, built on `8864316eb2`, which holds 43)
 wrote slice-cache records at 43, among them slice 85 of `XingZ60`: five panel packets heard nobody, the panel's
 findings began `panel-packet (cluster/...): ` and so no reader of where a finding begins saw them, and the settlement
 with its 12 needs-human issues was cached as examined. The key hashes the version, the shape of the run and the texts,
 not the rule that decides what may be stored, so a record written before would be served after: records stand at 43,
 and the change cannot ride inside it, so the number moves. The reader also refuses a record that lacks the field, so
 a record written at 43 is not resumed even by a run that kept the old number. The refine version stays at 14 (main's
 number then; the branch's number is in `REFINE_CACHE_VERSION`): its key and its question are unchanged, and a
 refinement record embeds the outcome, so the same reader refuses one
 that lacks the field; the refine write gate asks the same question as that reader (`refinementIsResumable`), so it
 writes nothing the reader would discard.

 THE OTHER VERSIONS STAY AT 44 ON THAT MOVE, each for its own reason (the numbers in this paragraph are those of main
 when it was made). The lane-contest (16), consolidate (32), consistency (7) and
 page-title (6) keys hash the texts and questions they decide over, and nothing here changes what any of them may store
 or how it decides: a slice the repair lane redoes under 44 reaches them as a changed key when its text changes and as
 the same answer when it does not. The translate slice version (25) stays because the translate lane never reads
 `silentStages` and its own gate is unchanged. The pairing version (3) stays by its own account of 2026-10-18
 (`slice-reading/src/pairing-cache-version.ts`). The artifact schema stays because no source under `artifact/src`
 names the field. (The sentence as main wrote it also gave a reason about artifacts that was false, corrected on
 2026-10-23: the scheduler counts every `*.json` as settled, so settled entries are not run again by a resume into the
 same directory, and the pipeline drift guard refuses that resume unless the launch opts in
 (`slice-cache/src/corpus-run/pass-generation-guard.ts:58-59, 360, 377`); see the artifact generation paragraphs of
 `provider/src/enum-case-fold.ts`.) The every-provider-dry stop of the same day
 (`bae1bb310e`) moves no version: it changes whether the next entry starts, not what a settlement may be cached
 as. No record has been written at 44, which this change sets first.

 The system text and answer schemas (first set as 44 on the Claude route): the system text of 2026-10-22 (R4 of the
 third review of the Claude route), where the sentence naming the
 length of the fence that encloses each block left the system text of the candidate selection
 (`adjudication/src/candidate-select-wire.ts`) and opens its user turn, because the length grows with the longest run
 of equals signs a shown text holds and so made the cached system prompt differ from call to call. The sentence is the
 same words with the same number, so what the models are told is unchanged and where they read it is not. The
 translate and refine versions move for the same slates (26, 15), and the consolidate and lane-contest versions for
 the same pair of moves (33, 17). The key hashes the version, the shape of the run and the texts, not the sheet
 (`source-scan/src/cache-key-sheet-blindness.unit.test.ts` reads the imports of the five key files), so a record
 written before would be served after. The previous number was set in `eb4db609ff`, and the tip `8864316eb2` of runs
 zheermao23 and XingZ6018 holds it (`git merge-base --is-ancestor`, with `2eeef91fac` as an ancestor of `HEAD` for the
 control that finds ancestry and the reverse pair for the control that does not); `find` under those two run
 directories finds 180 slice-cache files newer than that tip's commit time (2026-10-22T10:30:29Z), so records may
 stand at the previous number and the change cannot ride inside it. The consistency version stays because its key
 digests the exact sheet, and the page title and pairing versions stay because their sheets name no fence length, no
 break count and no marks. No record has been written at the new number, which this change sets first.

 The answer schemas of the range ride inside this number, checked on 2026-10-22 (F6 of the re-check of the second fix
 round) and not assumed: the critic (`adjudication/src/critic-wire.ts`: category, severity), the panel
 (`adjudication/src/adjudicate-wire.ts`: vote, severity), the resolution check
 (`adjudication/src/resolution-wire.ts`: verdict) and the introduced-defect check
 (`adjudication/src/introduced-defect-format.ts`: verdict) list their labels as an enum built from the constant their
 guard reads (P1, `3604fd00c3`). The derivability and restoration schemas moved the same way and feed
 `repair/src/repair-benchmark.ts` alone, whose output no slice record holds. The Claude body's description text for a
 stripped bound and its allowlist of keywords (R6 and R10, `66eb3f15e4`) change the request to one route only. The
 key hashes the version, the shape of the run and the texts, not the schema, so only the number retires a record.
 Both `3604fd00c3` and `66eb3f15e4` are ancestors of `33ca9974f2`, which set this number
 (`git merge-base --is-ancestor`), and neither is an ancestor of `8864316eb2`, the tip of runs zheermao23 and
 XingZ6018, so no record written by code without them can stand at this number.
 Measured at 2026-10-23T07:30Z: `git branch --all --contains 33ca9974f2` lists only the marker of a detached head
 (`* (no branch)`) and no branch name, and checking the heads of the 86 entries of `git worktree list` with
 `git merge-base --is-ancestor` finds the commit in `tr-w289`, `tr-w305` and `tr-w314`, all at detached heads (`tr-w314`
 is a review worktree at `c98d140b3f`). That set is a reading at that time and grows with each review worktree, so the
 argument that holds is the ancestry argument: no record written by code without the commit can stand at this number.
 The one bump covers both the system text and the schemas, and the number does not move twice for one release.

 The reader taking a capital-only difference as the listed value (first set as 45 on the Claude route): the reader of a
 model's reply takes a value that differs from a listed `enum` or `const` value
 only in capitalization as the listed value (`provider/src/enum-case-fold.ts`, run by `readJsonOutcome`). The panel,
 the critic, the introduced-defect check, the resolution check and the coverage judge list their labels as an enum
 (`adjudication/src/adjudicate-wire.ts`, `adjudication/src/critic-wire.ts`,
 `adjudication/src/introduced-defect-format.ts`, `adjudication/src/resolution-wire.ts`,
 `archive/src/coverage-wire.ts`), so a label that a reply spelled with another capital, which the guard refused and
 the round counted as no voice, now counts. The key hashes the version, the shape of the run and the texts, not the
 reader, so a settlement stored before would be served after, though it may rest on a round that lost such a voice.
 The previous number was set by `33ca9974f2`, and no slice-cache directory was searched for a record written by code
 that holds that commit and lacks the fold, so the change cannot ride inside it and the number moves. A build whose
 pipeline digest differs discards the namespace before it reads it (`openNamespacedCache`), and the number is the belt
 beside that brace. The account of every cache is in the header of `provider/src/enum-case-fold.ts`.
 */
export const SLICE_CACHE_VERSION = 46;
/**
 Everything about a repair run that changes what the models are ASKED, folded
 into every cache key.

 Without it a resumed slice could return an outcome produced under a different
 roster or a different editor addendum, and nothing would look wrong: the
 texts match, so the key matches. That is the failure a version constant
 cannot catch, because no shape changed. Identity context belongs here for the
 same reason, since it is front-matter-derived prompt content that varies per
 document pair.

 THE ADJUDICATION THRESHOLDS ARE NOT A RUN'S TO CHOOSE. They are
 `DEFAULT_ADJUDICATION_CONFIG`, which no run can replace since the
 per-run override went (T8 in the audit ledger): no production caller ever
 passed one, so every key this shape made held `null` in their slot. The slot
 stays `null`, so no settled slice's key moves, and a change to the thresholds
 themselves bumps {@link SLICE_CACHE_VERSION} rather than any key.

 THE PER-CALL DEADLINE IS DELIBERATELY OUT, decided when the two-lane driver
 began passing one explicitly rather than letting each lane keep its own
 default. It changes nothing any model is asked; it changes only how long this
 side waits before giving up on an answer. Folding it in would split the cache
 on a scheduling knob, so raising the deadline for a slow provider would
 discard every slice already bought under the old one, and lowering it again
 would resurrect them.

 The counter-argument worth stating is that a deadline changes what comes
 BACK, since a clipped call costs a voice. It does, and a key covering it
 would still promise something this pipeline never offered: a stage that loses
 a voice retries to a quorum and then proceeds, so the panel a slice was
 decided by already varies run to run under a FIXED deadline. What a resumed
 slice is owed is that the QUESTION was the same, and that is what this states.

 {@link translateRunShape} reached the same decision for the same reason. The
 two lanes now agree in writing, which is the point of recording it here: one
 lane keying on the deadline and the other not would mean a deadline change
 discarded half a document's cached work and kept the other half.

 @param models - every role roster this run seats

 @param identityContext - declared names travelling with every slice

 @param referenceContext - what the original's cited pages say, which the
 critic and panel sheets read (class thirty-five), so a slice settled before
 the references were shown keys apart from one settled after

 @returns Stable string for the key

 @example
 ```ts
 const runShape = repairRunShape({ models, identityContext, },);
 ```
 */
export function repairRunShape(
  {
    models,
    identityContext,
    referenceContext,
  }: {
    readonly models: RepairModels;
    readonly identityContext?: string;
    readonly referenceContext?: string;
  },
): string {
  return JSON.stringify([
    models.criticModelIds,
    models.panelModelIds,
    models.editorModelIds,
    models.judgeModelIds,
    models.refinerModelIds ?? [],
    models.checkerModelIds,
    models.editorRuleAddendum ?? '',
    // THE THRESHOLDS' SLOT, `null` in every key a run ever made, kept so none
    // moves (see the summary).
    null,
    identityContext ?? '',
    // ONLY WHERE THE ORIGINAL LINKS SOMEWHERE, so the 33 entries that cite
    // nothing keep the keys their slices were settled under.
    ...(((referenceContext ?? '') === '') ? [] : [referenceContext,]),
    // NOT `checkerSelfCertificationPermitted` (ledger X11, M37). Every reader
    // hands it to `assertCheckerIndependence`, which only refuses a roster
    // before anything is bought; a roster it admits is asked and weighed the
    // same either way, and the checkers themselves are keyed by `models.checkerModelIds`.
  ],);
}

/**
 Cross-run key for one slice under the repair lane.

 The schema version, this run's shape and both texts, so a content change, a
 roster change or an outcome-shape change all miss the cache and recompute.

 THE SLICE INDEX IS NOT IN IT, since version 26. Where a slice sits is not
 part of what the stages are asked, and keeping it there meant a renumbering
 discarded every slice below the change however untouched its text. A resumed
 record is stamped with the index it was asked under, and the mirror of this
 reasoning is in `translateSliceKey`, including the corpus measurement that
 says two slices carrying identical text within one document do not occur.

 @param runShape - what this run asks, from {@link repairRunShape}

 @param sourceText - slice original

 @param targetText - translation already there

 @param lineStructured - whether the enclosing chunk is line-structured

 @param neighbouringSourceText - original of the passages either side, shown to
 the critic, panel and editor as context they may not edit. In the key because
 a slice judged against its neighbours was asked a different question from the
 same slice judged alone, and nothing else records which it was

 @param neighbouringIncumbentText - archive English of those same two, which is
 the half that shows a relocation

 @param documentSourceText - complete source evidence, whose changes must invalidate slice and twin reuse

 @returns Hash keying this slice's outcome

 @example
 ```ts
 const key = repairSliceKey({ runShape, sourceText, targetText, lineStructured, },);
 ```
 */
export function repairSliceKey(
  {
    runShape,
    sourceText,
    targetText,
    lineStructured,
    neighbouringIncumbentText,
    neighbouringSourceText,
    documentSourceText,
  }: {
    readonly runShape: string;
    readonly sourceText: string;
    readonly targetText: string;
    readonly lineStructured: boolean;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
  },
): string {
  return hashContent({
    content: JSON.stringify([
      // Mirrors the literal `translateSliceKey` has always led with. The two
      // lanes are already isolated at the STORE, which prefixes translate files
      // and leaves repair files bare, so this is a second mechanism rather than
      // the only one. It exists because two mechanisms that must agree should
      // not be able to drift: a store that ever stopped prefixing would hand one
      // lane's record to the other, and a key that names its lane cannot.
      'repair',
      SLICE_CACHE_VERSION,
      runShape,
      sourceText,
      targetText,
      // Two slices can carry identical text and still be governed differently,
      // because the verdict belongs to the enclosing chunk. It has to sit in
      // the key rather than ride on the version alone.
      lineStructured,
      // ABSENT AND EMPTY KEY ALIKE, and deliberately so, mirroring
      // `translateSliceKey`. A slice with no neighbours has no window to be
      // shown, so it is asked the same question a caller that never had the
      // parameter asked, and it should resume rather than be recomputed to
      // reach the identical answer.
      // LABELLED, because the two sides are otherwise INDISTINGUISHABLE once
      // spread into a positional array: a source-only window and an
      // incumbent-only window carrying the same text would hash identically and
      // one cached answer would serve two different questions. Asymmetric
      // windows are real here, since a neighbour that is an insertion anchor has
      // source text and no target text. `translateSliceKey` never had this
      // because it spreads NAMED properties, and mirroring its shape rather than
      // its effect is what introduced it.
      ...(((neighbouringSourceText === undefined) || (neighbouringSourceText === ''))
        ? []
        : [
          'nearby-source',
          neighbouringSourceText,
        ]),
      ...(((neighbouringIncumbentText === undefined) || (neighbouringIncumbentText === ''))
        ? []
        : [
          'nearby-incumbent',
          neighbouringIncumbentText,
        ]),
      ...(((documentSourceText === undefined) || (documentSourceText === ''))
        ? []
        : [
          'document-source',
          documentSourceText,
        ]),
    ],),
  },);
}

//endregion Repair slice key
