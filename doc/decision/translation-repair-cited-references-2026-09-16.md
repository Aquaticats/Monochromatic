# Translation repair: the critic and panel sheets read the pages the original cites

## Status

Decided by the owner on 2026-09-16 and landed in `dff8c91d2` (class thirty-five).
The owner's words,
answering the four options put to them:
"3,
and we have Exa.
Such Exa fetched results need to be cached too to avoid hitting reference links too much."
And,
while the fix was being built:
"Cache Exa results semi-permanently on disk since the reference links rarely change their content."

## What was found

The Mio19 page (2026-09-16,
the pass log's "Mio19 read" heading) lost the archive's clause
"older sister who is also trans".
The Chinese page says only 有一个姐姐,
in every one of its seven revisions.
The blog the page cites,
`stblog.penclub.club/posts/InMemoryOfMio/`,
says 「Mio 的姐姐也是 MtF。她大我们八岁，在小学五年级时就第一次穿了女装」,
and the translation commit `c7cc3259` was written by someone who had talked to the sister.
Four critics filed the clause as `accuracy/addition` with the summary that it is
"not present in the source text";
the panel supported all four;
the editor deleted it.

Both sheets already said not to.
The critic sheet (`critic-prompt.ts`) reads
"ACCURATE detail a translator added is not an addition defect ...
Report such detail ONLY when it is WRONG",
and the adjudication sheet (`adjudicate-prompt.ts`) reads
"Vote unsupported on a claim whose whole case is that the ORIGINAL does not carry it;
vote supported only when the added detail is WRONG."
Neither rule had a mechanism,
and neither held.

Measured over the repair lane's shipped edits
(`lanes.repair.result.issues` in each run's artifact,
category `accuracy/addition`,
disposition `shipped`):

- Mio19 filed 24 addition claims and shipped 20 edits from them.
- Mio16 filed 24 and shipped 14.
- Mio12 filed 22 and shipped 9.
- hakureico23 filed 12 and shipped 11.
- noname3 filed 9 and shipped 7.

Among Mio19's twenty:
the sister,
the backpacks in `photo5`,
"no one was hurt",
"in mainland China",
"lucky",
the closing farewell.
Some deletions are right by any reading
("two days after the accident" contradicts the source's own date);
others are knowledge the human translator had and the Chinese page never wrote down.

## The options put to the owner

Ranked A > C > D > B.

- A.
  Only what the source contradicts:
  an addition claim against archive text must quote the original statement it conflicts with;
  absence alone is refused before adjudication.
  Pro:
  honours the translator's knowledge,
  matches both prompts,
  deterministic.
  Con:
  translator embellishments stand too.
- C.
  Facts stand,
  wording may go:
  a contradiction required only for clause-sized claims.
  Con:
  the fact-versus-wording line is drawn by the judges.
- D.
  Feed the cited reference to the critics.
  Pro:
  the sister case resolves on evidence.
  Con:
  external fetches,
  only entries that cite,
  no help for facts from pictures or private contact.
- B.
  Keep source-only authority.
  Con:
  keeps deleting true facts against both prompts.

The owner chose D,
with the cache.

## The mechanism

- `cited-reference-scan.ts`:
  every `http://` or `https://` link the original writes,
  once each,
  in order of citation,
  trailing sentence punctuation removed,
  at most eight.
  Left out:
  the corpus's own site,
  and people's profiles
  (a one-segment path on github,
  twitter,
  x or t.me;
  any `space.bilibili.com` page;
  `zhihu.com/people/`),
  because the Mio probe returned a GitHub user page as repository names and a bilibili space as page chrome.
  Measured over the pinned corpus:
  59 of 92 originals link somewhere,
  116 links,
  twitter 45 and github 22 the commonest hosts.
- `cited-reference-fetch.ts`:
  `POST https://api.exa.ai/contents` with `urls` and `text.maxCharacters` 4,000,
  raw `fetch`,
  the key from `TRANSLATION_REPAIR_EXA_API_KEY` in the `x-api-key` header and never logged;
  `results[0].title` and `text`,
  `statuses[0].error.tag` on failure.
- `reference-cache.ts`:
  one JSON record per url,
  named by the url's digest,
  under `~/.cache/translation-repair/lookup/reference/`
  (the `TRANSLATION_REPAIR_LOOKUP_CACHE_DIR` override carries over).
  Never expires;
  a failure is cached too,
  so a dead link is not hit on every pass.
  To refresh one page,
  delete its file.
- `cited-reference-lookup.ts`:
  one line per page,
  `- reference N <url> ("<title>"): <text on one line>`,
  or `could not be fetched (<tag>)`,
  joined into `referenceContext` on the prepared pair.
  The log prints `REFERENCE N <url>: success, <chars> chars, cached|bought` and
  `REFERENCES cited=N cached=K bought=M`.
- `cited-reference-rule.ts`:
  the block `CITED REFERENCES, EVIDENCE ONLY` with the rule:
  a detail the translation carries that the original does not state but a cited reference states
  is accurate detail the translator took from the original's own references,
  never an addition;
  the references never license adding,
  never outrank the original,
  never license a defect elsewhere.
  The critic sheet carries it after the pair,
  the panel sheet after the claims.
- `repair-slice-key.ts`:
  the block joins the run shape only where the original links somewhere,
  so the 33 entries that cite nothing keep the keys their slices were settled under.

The threading is `referenceContext?: string` from `pass-prepare.ts`
through `prepareDocumentPair` and the prepared pair into `repairChunk`,
the critic phase,
`runCriticStage` and `runPanelStage`.
It is a field of its own,
not a line of the identity context,
because the identity block's rule says it is evidence about naming only
and the refiner reads that block as names that must survive exactly.

## Verified

- Live on 2026-09-16 with the Mio source:
  the blog came back with 姐姐也是 in its 4,000 characters,
  its archived copy timed out (`CRAWL_LIVECRAWL_TIMEOUT`,
  cached as such),
  the two profile pages came back as chrome and are now excluded.
  Four records in the cache;
  a second ask bought nothing.
- Guards shown to fail:
  with the block disabled,
  `critic-prompt.unit.test.ts`,
  `adjudicate-prompt.unit.test.ts` and `cited-reference.unit.test.ts` fail,
  and pass restored.
- Mio20 (frozen `dff8c91d2`) is the first pass on it;
  the reading is whether the sister clause ships and how many addition claims shipped against Mio19's twenty.

## Mio20 and class thirty-six

Mio20 (frozen `dff8c91d2`,
2026-09-16 18:44 UTC) showed the mechanism working where it was placed and failing one stage later.
The panel rejected the three addition claims on the sister clause 3 to 2
(Mio19:
supported 4 to 1),
addition claims accepted fell from 30 of 36 to 11 of 21,
and the repair lane's slice kept `who is also trans`.
Then the lane contest,
shown only the original,
had three of five judges call the repair candidate unsupported for the clause,
the translate lane won the slice,
and the consolidation wrote from it.
`20a7272dc` (class thirty-six) threads `referenceContext` into the lane contest sheet,
the consolidation writer and gate sheets and the translate judge
(which judges consolidation slates too),
with `CITED_REFERENCE_CANDIDATE_RULE`:
a detail the archive rendering or a candidate carries that a cited page states
is never counted unsupported and never a reason to prefer the rendering that drops it.
The three run shapes fold it in only when non-empty.
Mio21 is the first pass on it.

## Mio21, class thirty-seven and class thirty-eight

Mio21 (frozen `20a7272dc`,
2026-09-16 20:40 UTC) showed the prose rule losing to the sheets' own addition category.
The references were on every sheet and the judges' replies cite them,
four select judges writing that the sister's trans identity is
"true per the cited reference but is NOT stated in the ORIGINAL passage,
so it is an addition";
the panel supported the addition claim 3 to 2
(Mio20 rejected it 3 to 2),
the repair lane dropped the clause,
and the page kept it only because slice 2's slate was declined and the archive stood.
`35e9a0785` and `887e5262c` (class thirty-seven) make the reading mechanical where it can be:
at preparation the bench answers one focused question,
which archive details absent from the original a reference states,
each as a verbatim archive quote plus a verbatim reference quote,
both checked as substrings with whitespace removed;
details at least half the heard voices gave become `- attested:` lines under the references on every sheet,
and an `accuracy/addition` claim whose quote overlaps one is rejected before the panel.
The archive block review's revision of the first chat block shipped straight quotes into a curly page;
`6b853e5ce` (class thirty-eight) restores the archive's quote style on every revision candidate.
Mio22 (frozen `35e9a0785`) attested nothing because one of three sister attestations
wrote the reference's Chinese with spaces around its Latin tokens;
the whitespace fix landed and Mio23 is the first pass on it.

## Mio23 and class thirty-nine

Mio23 (frozen `887e5262c`,
2026-09-16 22:40 UTC) attested the sister detail by five of six voices,
the critic filed no addition claim on it,
and the repair lane kept the clause.
The page still lost it:
the translate lane,
written from the original with the archive as evidence only,
rendered "and she has an older sister",
and the lane contest chose translate 3 to 2 on the 985,
the local university and "before she passed away" that the repair candidate dropped,
one ballot adding that repair "wrongly keeps the reference-attested 'also trans' sister".
The candidate-sheet rule says a rendering that drops an attested detail drops accurate detail,
and the judges weighed it against the rest of the slice and chose the lane that never had it.
The writer sheet never carried the references
(class thirty-six reached the translate judge only),
so no translate candidate could carry the detail.
`27109ca8b` (class thirty-nine) shows the translate writers the attested lines
as an ATTESTED DETAILS block with a rule to carry every one,
folded into the translate run shape only where there are any.
Mio24 is the first pass on it.

## Mio24 and class forty

Mio24 (frozen `27109ca8b`,
2026-09-17 00:15 UTC) is the first pass where the mechanism held end to end on slice 2:
the sister attested by four of five voices,
both addition claims on it rejected before the panel with the attestation as the reason,
the repair lane kept "who is also trans",
and the contest chose repair 3 to 2.
The translate writers,
shown the ATTESTED DETAILS block for the first time,
put the clause in one candidate of five;
the translate judges,
who have the candidate-sheet rule,
chose a dropper 3 of 5.
That is recorded and not fixed:
the page would have carried the clause through the repair lane.
The entry stopped instead at slice 17 on class forty,
unrelated to the references
(the pass log's "Class forty" heading dated 2026-09-17 00:35 UTC):
a declined contest left the archive's one-line rendering standing,
the deterministic rule withheld it,
and the slate never saw the two valid lane texts.
`1c38e4f34` offers them.
Mio25 is the first pass on it.

## Mio25 and class forty-one

Mio25 (frozen `1c38e4f34`,
2026-09-17 03:00 UTC) settled,
and the clause was lost at the last stage that touches the text.
The sister attested five of six,
the repair lane kept the clause,
the contest chose translate four of six because the repair lane had also written an unsupported
"at the time of her death",
the consolidation slate gate restored the clause four of five with three ballots naming the
attestation,
and the consolidation polish removed it:
the refiner's proposal dropped "who is also trans",
and the polish gate chose it four of six as "removing the unsupported 'also trans' detail".
The refine stage,
shared by the repair lane's refiners and the consolidation polish,
and the polish gate had never been given the references or the attested lines;
class thirty-six had closed the contest,
the slate and its gate,
not these.
`836f90687` puts the same block on the refiner sheet,
in the refine judges' evidence and on the polish gate sheet,
and folds the references into the refine cache key.
Every sheet that can add,
keep or remove a detail now carries the references:
the critics,
the adjudication panel,
the translate writers and judges,
the lane contest,
the consolidation producers,
the slate judges,
the slate gate,
the refiners,
the refine judges,
the polish gate.

## Mio26 and class forty-three

Mio26 (frozen `052f8eea0`,
2026-09-17 03:25 UTC) never attested the sister:
five voices heard,
four answered the open question with an empty list
(two of them in six completion tokens),
one attested it with both quotes verifying,
and the quorum of three dropped it.
The same bench had attested it five of six on Mio25.
An open extraction question is one a voice can answer without reading,
and an empty list is not a vote against the detail.
`f7aa7c675` adds a confirmation round:
every verified candidate goes back to the bench as a numbered yes-or-no question carrying both quotes,
and the candidates at least half the confirming voices name are attested.
The extraction quorum stands when nothing was extracted or nobody confirms.
Mio26 was killed for it and Mio27 is the first pass on it.

## Mio27: the clause ships

Mio27 (frozen `f7aa7c675`,
2026-09-17 06:05 UTC) is the first Mio page since class thirty-five was found that carries
"She has an older sister who is also trans."
through every stage:
four of five voices extracted the sister and the confirmation ratified it five of five,
the repair lane kept the clause,
the contest chose repair five of six with four ballots naming the attestation,
the consolidation gate kept the standing,
and the polish kept the clause with its gate five of five,
two ballots saying "retain the attested sister detail" where Mio25's had said "unsupported".
One contest ballot and one slate-gate ballot still called the clause unsupported on the
class thirty-seven reading;
they were outvoted.
The confirmation round's rescue path,
a detail one voice extracted and the bench confirmed,
has not yet been measured on a live entry.

## Cost

Exa bills per page read;
the response carries `costDollars` and the four Mio pages were bought once.
The sheet cost is prompt tokens:
at most eight references of 4,000 characters on every critic and panel call of an entry that cites,
placed after the pair so the cached prefix on OpenRouter still applies.

## Rejected

- A mechanical contradiction requirement (option A):
  the owner preferred evidence over a rule.
- Highlights instead of page text:
  the question the critics ask is not known when the page is bought.
- Fetching at adjudication only:
  a critic that reads the reference files fewer claims,
  and every claim it does not file saves a panel and an editor round.
