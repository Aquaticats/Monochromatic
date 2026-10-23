//region Consistency cache version

/**
 Generation of the page consistency pass's cached answers (the owner's ruling
 of 2026-10-11).

 Version five (the lead's rulings of 2026-10-15 on the run of TianqiChen66625): what the
 judges are asked and what a reply must contain to count changed, after two passes
 (TianqiChen66624 and TianqiChen66625) wrote version four. Every sheet of the three judge asks (the first
 round, the run-off among places and the choice among a place's candidates) ends in a
 sentence that lists the exact labels the reply may use, and the response format of each
 ask lists them as an `enum`, so no stored answer of version four replies to the question
 asked now (the key digests the sheet, so the keys moved of themselves; the number moved
 as well, for the reply shape and for what a ballot must contain to be usable, which the key does not digest). A reply that names exactly one shown label in
 another form (its case, white space, a bare number, the word with the number) now counts,
 and is stored as the shown label: a record of version four holds only replies the
 stricter guard counted, each already in the shown form, so none could hold a value
 computed under the old rule that the new rule would count differently. A choice among
 candidates that no candidate won by a strict majority is now run off among the finalists,
 under a key of its own (`choice-run-off`); no record of version four is read after the bump,
 since the key hashes the number, so the choice and the run-off that follows it are both asked
 afresh. The counting of quotes, which locates a chosen place by the longest quote that stands
 once and that the quotes of more than half of all the usable ballots contain (a ballot that
 quoted nothing of the place counts against every candidate), is not stored and moves no key.

 The finalists of a run-off moved no number either (the lead's rule, open to the owner's veto, extending the owner's
 rule of 2026-10-11): a first round or a carry choice with no strict majority and a lone leader is now run off among
 the leader and the labels next behind it, where it was kept. Which labels are the finalists is counted from the stored
 ballots and is not stored, and the sheet of a run-off shows only its finalists under a key that digests the sheet, so
 a run-off among three labels asks a question under a key of its own, and a run-off among the tied labels keeps its
 key byte for byte. The wording of the first round's sheet and of the run-off among places is the same.

 The sheet of the run-off among a place's candidates changed its bytes (the lead's repair of 2026-10-18, ledger B657) and the
 number still did not move, since the key digests the exact sheet: the new bytes make a new key, and no stored answer is read under
 a key it was not written under. Its rules now say that a first round was split between these candidates and the judge is asked
 again, because a run-off whose finalists are every candidate (a split of two, or of three over three) sent the first ask's sheet
 byte for byte, and a client that answers a repeated model and prompt from its first payload replayed the first round's ballots.
 The paragraph goes into the rules of every carry run-off, whichever labels are its finalists (the `runOff` flag of
 `buildCarryChoiceMessages` adds it, and the run-off is asked for a lone leader with the labels behind it as well as for a tie),
 so every `choice-run-off` key moved, and the statement above that a run-off among the tied labels keeps its key holds for the
 run-off among places alone. The `choice` stage is not affected: the sheet of the first ask is byte for byte what version five has,
 so the key of every first ask is unchanged, and a carry run-off stored under its earlier key is asked afresh under the new one.

 Version four (the review of 2026-10-13): what the writers and the judges are
 shown changed again, before any pass wrote version three. The words of every
 place of every other group that stand in a block are listed to keep, not only
 those of the group's own places, and the writer sheet's rule about them names
 places of this repeat or of another; the choice sheet shows the judges the
 words each receiving place replaces (THE WORDS TO REPLACE, numbered as the
 writer sheet numbers them) and says so in its rules; a place in a table, a code
 block or a setext heading is carried by the code splice alone, so no writer is
 asked about it. The sheets of the carry changed, so no stored answer of version
 three is resumed. The stored shapes did not change: a stored answer is still
 the heard voices with the quorum the gather closed on.

 Version three: what the pass asks changed again, before any pass wrote version
 two. A quote locates only at the boundaries of words, so a quote or a slot that
 stood inside a longer word no longer locates and the spans carried change; a
 quote's quotation marks and brackets that its words do not need are read as
 edge punctuation, like a full stop; the writers are given the innermost block
 that holds the words, and every receiving place of one block together, with the
 words of the group's other places inside it to keep; the choice sheet shows the
 judges what the writers were shown (the chosen place's ORIGINAL and English
 paragraph, the receiving places' ORIGINAL with the blocks around it, and the
 chosen place's English slot for a frame); and the reply shapes of the three
 judges' sheets name no real label. Every sheet of the stage changed, so no
 stored answer of version two is resumed.

 Version two: what the pass asks changed in three ways. The consistency sheet
 asks each judge to quote only the words that render the repeated source wording
 and to name, in the reason, a word of the chosen rendering that renders source
 outside the repeat. The bench is asked once more, over a sheet that shows only
 the finalists, when the choices tie for first. And a rendering is carried to a
 receiving place by writers and a choice among candidates, so the cache holds
 four kinds of answer, each under a key that names its kind: the ballots of the
 first round, the run-off's ballots, the writers' paragraphs and the choice
 ballots. A stored answer is the heard voices with the quorum the gather closed
 on. The counting, which reads them against the English, is not stored: a change
 to how quotes are located, how a choice wins or how a place is kept moves no key
 and needs no number, since the ballots mean the same under the same sheet.

 A change to a reply shape or to what a ballot must contain to be usable moves this number, since each changes
 whether a stored answer counts and the key digests neither. A change to a sheet's wording moves the key and not the number,
 since the key digests the exact sheet: an answer stored under the old wording is never found under the new key. Version one was written by one pass (TianqiChen66623),
 over a sheet that let a quote carry words of the source outside the repeat,
 counted by the exact text of a quote, and is not resumed.

 The tense-candidate block (the lead's decision of 2026-10-12) moved no number:
 the key adds the recorded death and the subject names for an entry that records
 a death, so such an entry asks a new question under a new key, and an entry
 that records none keeps its key byte for byte, as the key of every other judged
 stage does (ledger B419).

 Version 6 covers the fixes of the XingZ6015 read (2026-10-20), which change what these sheets say and what their floors accept:
 reader protection reaches only the account of how a death or an attempt happened (owner, 2026-10-20,
 `house-rule/src/house-policy.ts`), which every sheet splicing the house rule restates; judges no longer see a note between
 translators (`document-structure/src/translators-note.ts`); the archive-block review says the page it publishes is complete; a
 consolidation challenge round seats a proposal two ballots named over a fallback the contest majority called flawed; and the
 declared-name floor meets a lost form of the subject with another declared form of the same person. This key hashes the sheet, so
 the new house rule asks anew on its own, but not the floors that read the answers, so a carry chosen under the old declared-name
 floor would be served under the new one. The XingZ6015 pass wrote records at the old number on `sha256-tree-v1:a92725968adf…`, so
 the change cannot ride inside it and the number moves, as every version whose sheets splice the house rule moves with it; the
 pairing version stays, since no pairing sheet carries the house rule. No record has been written at 6, which this change sets
 first.

 Rides inside 6 too: the three judging sheets (the repeated wording, the run-off and the choice among a block's carry candidates)
 cut the notes between translators from the English they show (`document-structure/src/translators-note.ts`, `7ee1755b24`). The
 key digests the exact sheet, so each judge asks anew under a key of its own and an answer stored under the old sheet is never
 found, which is the rule that a change to a sheet's wording moves the key and not the number; what a ballot must contain and what makes one usable
 are what they were, so no number moves. The counting still reads the quotes against the stored English. Runs zheermao16 and
 CuspariaKLSY19 started on `a5dc23c5bb`, which holds 6 (set in `28f4b8fc7`), and their records stand under keys no cut
 sheet can make. Of the 92 pinned English archives 10 hold a note their original lacks (34 notes in all), so a sheet changes for
 at most those 10 entries.

 Stays at 6 for the alphabetical reply keys (owner decision of 2026-10-21 under the ruling "Reason first everywhere"; ledger B762):
 the three judging sheets ask for `assessment`, `candidateQuotes`, `choice` and `keep`, and the choice-only sheets for
 `assessment` and `choice`, where they asked for `reason`, `quotes`, `choice` and `keep` and for `reason` and `choice`. Only the
 reply is renamed. A function in `consistency-wire.ts` and one in `consistency-choice.ts` map the reply to the ballot, and the
 ballot, its guard and the stored answer keep the names `reason` and `quotes`, so what a stored answer must contain and what
 makes it usable are what they were. The key digests the exact sheet, whose reply instruction changed, so each judge asks anew
 under a key of its own and an answer stored under the old sheet is never found: the rule that a change to a sheet's wording
 moves the key and not the number.

 Version 7 covers the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816): once the straggler grace
 ends, a short bench whose live seats are exactly its quorum closes one voice below it
 (`stage/src/stage-reachable-quorum.ts`), and a refusal inside a round sizes the need again
 (`stage/src/stage-round.ts`). Every round of the consistency pass's three judges gathers through that rule, so a
 record settled under the rule of 2026-09-09 may hold an outcome the new rule reaches differently, on one voice fewer.
 The key hashes the version, the shape of the run and the texts, not the rule that closes a round. A build whose
 pipeline digest differs discards the namespace before it reads it (`openNamespacedCache`), and the digest holds the
 stage code, so that retires such a record; the number moves too, as the belt beside that brace, so no stored record
 stands under a number whose rule differs. Run zheermao23 is armed for 2026-10-22 19:16 UTC on `65e8325303`, which
 holds 6 (`git merge-base --is-ancestor`), so records may stand at 6, and the change cannot ride inside it. The artifact
 generation moves to 20 for the same rule, and the consolidate (32), lane-contest (16), translate slice (25), repair
 slice (43), refine (14), page title (6) versions move with this one; the pairing version stays, by its own account of
 2026-10-18 (`slice-reading/src/pairing-cache-version.ts`). No record has been written at 7, which this change sets
 first.

 Version 8 covers the reader of a model's reply taking a value that differs from a listed `enum` or `const` value only
 in capitalization as the listed value (`provider/src/enum-case-fold.ts`, run by `readJsonOutcome`). The three judging
 sheets list their labels as an enum built per ask (`consistency/src/consistency-wire.ts`,
 `consistency/src/consistency-choice.ts`), so a label that a reply spelled with another capital, which the guard
 refused and the round counted as no voice, now counts. The cache holds ballots only, and the key digests the exact
 sheet, the roster and the version, not the reader, so a ballot stored before would be served after, though it may
 rest on a round that lost such a voice. The previous number was set in `eb4db609ff`, and a record written under it by
 a run that started before this change may stand, so the change cannot ride inside it and the number moves. A build
 whose pipeline digest differs discards the namespace before it reads it (`openNamespacedCache`), and the number is
 the belt beside that brace. The account of every cache is in the header of `provider/src/enum-case-fold.ts`.

 @example
 ```ts
 const material = JSON.stringify({ version: CONSISTENCY_CACHE_VERSION, sheet, roster, },);
 ```
 */
export const CONSISTENCY_CACHE_VERSION = 8;

//endregion Consistency cache version
