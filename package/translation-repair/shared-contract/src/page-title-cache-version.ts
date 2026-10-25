//region Page title cache version

/**
 Generation of the page title lexicon's cached answers (ledger H16).

 Version one (2026-09-28): one rendering per title a page repeats and the
 archive leaves unpaired, the most voices' rendering, the earliest seat on the
 roster breaking a tie. A change to the sheet's wording or to the choice moves
 this number, since each changes what a cached answer means under the same key.
 The span detector does not: the key hashes every title it yields, with its
 count, in order (`pageTitleKey`), so a changed detection is a changed key.

 Rides inside (2026-09-29): `d9a306602` (titles listed by where each first
 stands) and `ee39e2ba5` (one shared heading reader, ledger X20) changed the
 span detector only. No slice-cache file has been written since this value was
 set, under the agent runs or the package's own runs directory, where a control
 time finds 494, so no answer is cached under it. Every later source commit
 rides inside it the same way, those an account names and those none does
 (`corpus-run/cache-account-audit.ts`, ledger M28). The choice changed once
 since (ledger B24, 2026-09-29): renderings apart only in apostrophe style or
 spacing of any kind count as one, and corner brackets and underscores come
 off as wrappers. That rides inside too: a find for slice-cache files written
 since 2026-09-28T19:38Z, when this value was set, finds none, and one from
 2026-09-27T00:00Z finds 494.

 The sheet changed once since (ledger B28, 2026-09-29): it shows the page's
 declared identity, web lookups and notes among it, with the rules for
 reading it. That rides inside too, on two grounds: the key now hashes the
 identity the sheet showed, so every key an earlier build wrote has moved;
 and the same find, rerun for this change, finds none since 2026-09-28T19:38Z
 against a control of 494.

 Version two (n161, 2026-10-15): the house rule the sheet carries gained the tense of quoted words (words quoted as someone wrote
 or said them keep the speaker's tense), so a rendering stored under one was chosen under the old rule and a title that quotes a
 line of someone's own words may hold it in the past. The key hashes the titles, the identity and the roster and not the sheet, so a
 record written under the old sheet would be served under the new one, and the find that would let the change ride inside one (no
 slice-cache file written since the change, beside a control that finds files) was not made, since no runs directory is read in this
 round. The pairing sheets carry no house rule, so the pairing version does not move.

 Version three (n166, 2026-10-15, Z11): the house rule's sentence that puts the narrative of a life in the past now holds the
 quoted-words exception inside it, so a rendering stored under two was chosen under the earlier wording of the same rule. The key
 hashes the titles, the identity and the roster and not the sheet, so such a record would be served under the new one. Whether one
 exists is not measured here: the audit of n161 found 0 slice-cache records under the keyless worktree's runs directory
 (`s228/cache-audit-n161.txt`, line 992), but the launch started since at `5f6c8593d7` runs at two and no runs directory is read in
 this round, so the number moves, as the consolidate, lane-contest, repair slice, refine and translate versions move for the same
 sheet. No record has been written at three, which this change sets first.

 Four covers the fixes of the XingZ6015 read (2026-10-20), which change what these sheets say and what their floors accept: reader
 protection reaches only the account of how a death or an attempt happened (owner, 2026-10-20, `house-rule/src/house-policy.ts`),
 which every sheet splicing the house rule restates; judges no longer see a note between translators
 (`document-structure/src/translators-note.ts`); the archive-block review says the page it publishes is complete; a consolidation
 challenge round seats a proposal two ballots named over a fallback the contest majority called flawed; and the declared-name floor
 meets a lost form of the subject with another declared form of the same person. The key does not hash the sheet or the floors, so
 a record written under the old ones would be served under the new. The XingZ6015 pass wrote records at the old number on
 `sha256-tree-v1:a92725968adf…`, so the change cannot ride inside it and the number moves, as every version whose sheets splice the
 house rule moves with it; the pairing version stays, since no pairing sheet carries the house rule. No record has been written at
 four, which this change sets first.

 Five covers the reply of the page title reading declaring its keys in alphabetical order (owner decision of 2026-10-21
 under the ruling "Reason first everywhere"; ledger B762): the lexicon sheet (`artifact/src/page-title-lexicon-wire.ts`) asks
 for `rendering` before `title`, where it asked for `title` first, and says so in its reply instruction.
 The key hashes the titles, the identity and the roster and not the sheet, so a record written under the old sheet would be
 served under the new one. The audit `cache-account-audit` found no slice-cache record under this worktree's runs
 directory and no control that finds records was run, so whether one stands at four is not known and the number moves, as
 the consolidate, lane-contest, repair slice, refine and translate versions move for the same ballots. The pairing sheets
 carry no schema of this change, so the pairing version stays. No record has been written at five, which this change sets first.

 Six covers the quorum rule of 2026-10-22 (owner, "Let the grace open"; ledger B816): once the straggler grace ends, a
 short bench whose live seats are exactly its quorum closes one voice below it (`stage/src/stage-reachable-quorum.ts`),
 and a refusal inside a round sizes the need again (`stage/src/stage-round.ts`). Every round of the page title lexicon
 gathers through that rule, so a record settled under the rule of 2026-09-09 may hold an outcome the new rule reaches
 differently, on one voice fewer. The key hashes the version, the shape of the run and the texts, not the rule that
 closes a round. A build whose pipeline digest differs discards the namespace before it reads it
 (`openNamespacedCache`), and the digest holds the stage code, so that retires such a record; the number moves too, as
 the belt beside that brace, so no stored record stands under a number whose rule differs. Run zheermao23 is armed for
 2026-10-22 19:16 UTC on `65e8325303`, which holds 5 (`git merge-base --is-ancestor`), so records may stand at 5, and
 the change cannot ride inside it. The artifact generation moves to 20 for the same rule, and the consolidate (32),
 lane-contest (16), translate slice (25), repair slice (43), refine (14), consistency (7) versions move with this one;
 the pairing version stays, by its own account of 2026-10-18 (`slice-reading/src/pairing-cache-version.ts`). No record
 has been written at 6, which this change sets first.

 Seven covers the rule of 2026-10-25 (the lead's ruling on the owner's "Two ballots decide"): a round that heard fewer than two
 voices settles no title and stores nothing (`artifact/src/page-title-lexicon-stage.ts`,
 `corpus-pass/src/corpus-run/pass-page-titles.ts`), where one heard voice used to settle every title it rendered and the round was
 stored. A record stored at six from a round of one voice holds a title no two voices settled, and the key hashes the version, the
 titles, the identity and the roster, not the rule, so such a record would be served under the new one. The number moves so that
 no stored record stands under a number whose rule differs. No record has been written at 7, which this change sets first.

 Eight covers the lead's ruling of 2026-10-25 on finding F1 of re-check 2 (settled by the owner's rule that one judge alone
 never decides): a title settles only on a rendering two or more voices gave (`artifact/src/page-title-lexicon-stage.ts`), where
 a title one voice alone rendered, or that tied at one voice apiece and went to the earliest seat, used to settle on a round of
 three or more voices. A record stored at seven may hold such a title, and the key hashes the version, the titles, the identity
 and the roster, not the rule, so it would be served under the new one. Whether a record stands at 7 is not known (the number
 was set a few commits before and no slice-cache directory was searched), so the number moves.

 @example
 ```ts
 const material = JSON.stringify({ version: PAGE_TITLE_CACHE_VERSION, source, titles, identity, roster, },);
 ```
 */
export const PAGE_TITLE_CACHE_VERSION = 8;

//endregion Page title cache version
