//region Pairing cache version

/**
 Generation shared by section and block roster-pairing question identities.

 Version two requires exact-half participation before straggler grace.
 Version one implicitly began grace after two usable voices.

 Version three (2026-09-28) moves with the rules that close a pairing round,
 which changed twice after version two landed on 2026-08-29 without moving
 it (ledger M28): the pairing stages read their round in windows with retry
 rounds since `33a023445` (2026-09-09), and size their quorum on the seats
 that could answer since `a107c7486` and `29baade8f` (2026-09-27). A pairing
 settled under the older rule could resume on a round the current rule would
 not have closed. The pictures a sheet was shown are keyed separately
 (`blockPairingQuestionKey`).

 Rides inside 3 too: every gather now keeps a seat that answered unreadably
 out of the same-prompt retry rounds and re-asks each such seat, whichever
 round it came in, in the nudged recovery round (ledger P2, `005692e11`),
 which changes whose voices a round closes on; checked on 2026-09-28: still no
 slice-cache file newer than 04:26 UTC on 2026-09-27.

 Rides inside 3 too: a reply whose complete JSON value more text follows is
 read rather than lost (ledger P8, `cac097368`), which changes whose voices
 every round hears; checked on 2026-09-28: still no slice-cache file newer
 than 04:26 UTC on 2026-09-27.

 Rides inside 3 too: a reply that could not be used is re-asked once,
 nudged, of the same model on another provider through the uniqueness
 wrapper's claims (ledger P9, `7011d72cc`), which changes whose voices every
 round hears; checked on 2026-09-28: still no slice-cache file newer than
 04:26 UTC on 2026-09-27.

 Rides inside 3 too: Bedrock reads dry with 1.33 USD still left, and its
 ledger holds every billed attempt at its bound (ledger P1, `2a108dfb3`,
 `107763dbb` and `7cfd5ae4b`), so a Gemma call moves to OpenRouter sooner
 as Bedrock nears its credit, which changes whose voices a round hears;
 checked on 2026-09-28: still no slice-cache file newer than 04:26 UTC on
 2026-09-27.

 Rides inside 3 too: the recovery round re-asks a reply the length
 limit cut with a nudge naming the cut, apart from one off the shape
 (ledger P10, `ce0ef7b51`), which changes what such a seat is asked and
 whose voices a round hears; checked on 2026-09-28: still no slice-cache
 file newer than 04:26 UTC on 2026-09-27.

 Rides inside 3 too: both keys fold the roster that answers (ledger X13),
 so a pairing one bench settled is never resumed for another; every key
 moves, and none was written under 3. Same check, same result.

 Rides inside 3 too: both keys serialize their material as one JSON value
 through `pairingQuestionKey` (ledger X15), so no two questions share a key,
 and every key moves again; checked on 2026-09-28: still no slice-cache file
 newer than 04:26 UTC on 2026-09-27.

 THE PRE-LAUNCH CHECK OF 2026-09-28 (ledger M28): 3 was set
 in `d614a0c1d` at 04:30 UTC on 2026-09-28, after the newest slice-cache file under the
 agent runs (04:26 UTC on 2026-09-27), and no slice-cache file has been written
 since, so no answer cached under an earlier question can be served under
 this number. Every source commit since then rides inside it, those the
 accounts in this TSDoc name and those they do not (`corpus-run/cache-account-audit.ts`, ledger M28).

 THE ACCOUNT OF 2026-10-18 (ledger B670) for the production commits from `d614a0c1d` to `29d606470`, which no account had read
 (1,205 commits under the sources of the family and of the one package before the split, 27 of them touching the files of the
 pairing stage): 3 need not move. The statement of the pre-launch check of 2026-09-28, that no slice-cache file had been written since, holds no
 longer: `corpus-pass-TianqiChen66626` wrote three block pairings under 3 between 20:58 and 21:00 UTC on 2026-10-18. What
 retires a stored pairing when the code that settled it changes is not this number. The key holds the version, the question, the
 texts, the pictures and the roster (`pairingQuestionKey`), so the sheet a seat was shown and the rules that closed the round
 are in no key and no number could have retired a record when they changed. What does is the namespace's generation: both
 pairing caches are opened under the digest of the built pipeline the pass runs (`openPairingCache` and
 `openSectionPairingCache`, `slice-cache-store.ts`), and a build whose digest differs discards the namespace before it reads
 it (`openNamespacedCache`; the case in `slice-cache-pairing.unit.test.ts` fills a directory under one build and finds it empty
 under another). Every commit of the range that changed a byte the pipeline runs changed that digest, and a commit that
 changed none left the record as the same code settled it, so no commit of the range lets a stored pairing outlive the code
 that settled it. The number stays the belt beside that brace, and it moves for what the digest does not hold: a dependency,
 the Node binary, or the environment, from which the straggler windows are read (`477bbd6601` moved where they are read, to the
 entry, and not whether they are read from the environment). This account reads the sources of the range and not those three.

 Rides inside 3 too: the section and block pairing sheets cut the notes between translators from the translation's sections and
 blocks, as the other judging sheets do (`document-structure/src/translators-note.ts`, `eaab1b1ed9`), keeping every index and the
 length of each list. The sheet is in no key, so this number could not retire a record when it changed, and the account of
 2026-10-18 above holds for it: the key holds the version, the question, the exact texts, the pictures and the roster, and a
 build whose pipeline digest differs discards the namespace before it reads it, so no pairing settled over the old sheet
 outlives the code that settled it. Runs zheermao16 and CuspariaKLSY19 started on `a5dc23c5bb`, which holds 3, and wrote or
 may have written pairings under it; they are on a different digest from any build holding the cut, so none of their records
 is served to it. The digest does not hold a dependency, the Node binary or the environment, and the cut changes none of them.
 Of the 92 pinned English archives 10 hold a note their original lacks (34 notes in all); 19 of the 63 sections and 28 of the
 634 stretches between blank lines of those archives hold one, and 25 of those stretches hold nothing else, so the sheet lists
 them empty under their own number.

 Rides inside 3 too: the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816), under which a short
 bench whose live seats are exactly its quorum closes one voice below it once the straggler grace ends
 (`stage/src/stage-reachable-quorum.ts`). The section and block pairing rounds gather through that rule, and the rule
 is in no key, so this number could not retire a record when it changed; the account of 2026-10-18 above holds for it:
 a build whose pipeline digest differs discards the namespace before it reads it, so no pairing settled under the old
 rule outlives the code that settled it. Run zheermao23, armed for 2026-10-22 19:16 UTC on `65e8325303`, may write
 pairings under 3 on a digest no build holding the rule has. The digest does not hold a dependency, the Node binary or
 the environment, and the rule changes none of them. The consolidate, lane-contest, translate slice, repair slice,
 refine, consistency and page title versions move for the same rule (32, 16, 25, 43, 14, 7, 6).

 @example
 ```ts
 const material = JSON.stringify({ version: PAIRING_CACHE_VERSION, question, source, target, pictures, roster, },);
 ```
 */
export const PAIRING_CACHE_VERSION = 3;

//endregion Pairing cache version
