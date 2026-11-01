# Audit ledger: follow-ups on emphasis definitions, cache accounts and the Synthetic quota shape

Paths in this record are the layout after 2026-10-18,
under `package/translation-repair`.

Four items were left after the page-definitions work of B665 to B667,
and the lead had them closed on 2026-10-18 on top of `7c21a56fc2`
(a clock-zone slip in the account of the third,
B672,
and a request to read the words of a place with recorded definitions that was not built,
B673,
followed):
the emphasis readers that still parsed without the page's definitions (B668),
the contradictions in the TSDoc of the consistency cache version (B669),
the pairing cache version,
which no account had read between `d614a0c1d` and `29d606470` (B670),
and the Synthetic meter reader that refused a body on 2026-10-18 (B671).

## B668: emphasis was read without the page's definitions in three places

Found on 2026-10-18 by the lead,
who read the claim of round n173 (every caller of the text fold receives the page's link and footnote definitions)
against `emphasisDelimitersOf`,
which takes `definitions` and was called without them in three places.

Outcome written on 2026-10-18.
Red `074599761e`,
fixed in `1a40579be6`.

Why it matters:
in CommonMark a reference definition changes how emphasis delimiters pair,
because an unresolved `[b*]` is plain text and its `*` can pair with an outer one.
`*a [b*] c*` closes at offset 5 without the definition of `[b*]` and at offset 9 with it.

What the three calls were,
read from the callers:

- `joinedMarksOf` and `marksPastMarkupOf` called it only when the caller passed no `emphasis`.
  Their one production caller,
  `readProof` in `consistency-fold-apostrophes.ts`,
  always passed the `emphasis` of
  `proseViewOf({ english })`,
  which was parsed without definitions,
  so the claim failed one step earlier than the three calls the brief named.
- `endsReadingRun` in `consistency-loose-marks.ts` parses `points.join('')`,
  the words of a span,
  which is always a fragment of a page.

What changed:
every reader takes an optional `definitions` and hands it to the parse.
That is `joinedMarksOf` (also to `foldedLinksOf`,
so the links and the delimiters agree),
`marksPastMarkupOf`,
`readProof`,
`proofViewOf` (whose remembered view is now found by the English and the definitions,
since a view read without them would otherwise be served to a read with them),
`provenApostrophesInEnglish`,
`markReadingsInEnglish`,
`readingsChangeOutside`,
`looseMarksOf`,
`holdsEdgeApostrophe`,
`apostropheMarksOf`,
`withApostropheForm` and `withLooseMarksOf`.
`planConsistencyEdit` takes `definitions` and reads the English around a splice with them,
and the decision of a place (`plannedPlace`) and the edge refusal (`edgeRefusalOf`) hand it the definitions of the page
the receiving slice stands in,
so the refusal is named for the reading the planner made.
`edgeClassOf` and `sameEdgeClass` moved to `consistency-edge-class.ts`
to keep `consistency-loose-marks.ts` under the line limit.

What stays read alone,
and why:
the words of the places.
The planner,
the apply stage and the reader of a record share `bringsAtEdge` and the loose-mark readers,
and the reader of a record holds no page,
so a definition handed to two of them and not the third would let the pass plan a splice that the reader refuses.
The pass would have to record the definitions the way generation 18 records those that move a slot (B665)
before the words of a place could be read with them.
This is open.
The proof of an apostrophe over a whole page (`withProvenApostrophes`,
`pageApostropheFormOf`,
the gains check) reads the page text,
which holds the page's definitions,
and a locator given no page (`locateWords` with the recorded `definitions`) still proves over the haystack alone.
That is open too,
and `withProvenApostrophes` takes no `definitions`.

Guard demonstration,
run on the fix commit with the lines that hand the definitions on taken out
(the `definitions` handed to the parse in the three readers,
to `readProof`,
to the remembered view,
to `readingsChangeOutside`,
to `planConsistencyEdit` and to the two planner calls of the edge refusal):
all eleven cases of `consistency-emphasis-definitions.unit.test.ts` failed
(`joinedMarksOf`,
`marksPastMarkupOf`,
`holdsEdgeApostrophe`,
`looseMarksOf`,
`apostropheMarksOf`,
`withApostropheForm`,
`proofViewOf`,
`provenApostrophesInEnglish`,
`markReadingsInEnglish`,
`readingsChangeOutside` and `planConsistencyEdit`),
and both cases of `pass-consistency-edge-definitions.unit.test.ts` failed (`edgeRefusalOf` and `plannedPlace`).
All thirteen passed again once the lines were restored and the packages rebuilt.

Each case reads one English three ways:
without the definition,
with it given,
and with it written into the text.
The second must equal the third and differ from the first.

Prevention:
a claim that every caller of a function receives a value is made after listing the callers with `rg`
and reading what each one hands it,
and a parameter with a default (`definitions = ''`) is the shape that lets a caller leave it out unseen.

## B669: the TSDoc of the consistency cache version contradicted itself

Found on 2026-10-18 by the lead,
reading the paragraph that B657 added.

Outcome written on 2026-10-18.
Fixed in `895259a704`.

The TSDoc said,
in the paragraph on version five and again in its general rule,
that a change to a sheet's wording moves the number,
and recorded in the paragraph of B657 a sheet change that did not.
The key digests the exact sheet
(`consistencyKey` hashes the version,
the stage,
the hash of the messages and the roster),
so a change of wording moves the key and not the number.
The number moves for what the key does not digest,
the reply shape (the response format is no part of the messages) and what a ballot must contain to be usable.
The file now says that once.

The paragraph of B657 also said that only the `choice-run-off` stage was affected,
as if the new paragraph reached the run-off among every candidate alone.
In `buildCarryChoiceMessages` the `runOff` flag adds the paragraph to the rules of every carry run-off,
and the run-off is asked for a lone leader with the labels behind it as well as for a tie,
so every `choice-run-off` key moved.
The sentence that a run-off among the tied labels keeps its key holds for the run-off among places alone.

## B670: the pairing cache version has no account for the commits from `d614a0c1d` to `29d606470`

Found on 2026-10-18 by the lead,
who noted that the last account covered the commits after `29d606470` only,
and that `PAIRING_CACHE_VERSION = 3` was set in `d614a0c1d`.

Outcome written on 2026-10-18.
Verdict:
3 need not move.
Test `665d64c7a3`,
account in the TSDoc of `pairing-cache-version.ts` in `5da176cf4f`.

Evidence:

- The range holds 1,205 commits under `package/module/translation-repair/src/*`
  and `package/translation-repair/*/src/*`,
  all under the first path since the split came after `29d606470`,
  and 27 of them touch the files of the pairing stage
  (`block-pairing-protocol.ts`,
  `block-pairing-question.ts`,
  `section-pairing.ts`,
  `prepare-section-round.ts`,
  `prepare-block-pairing.ts` and their neighbours).
- The key is `pairingQuestionKey`:
  the version,
  the question,
  the source and target texts,
  the pictures and the roster.
  The sheet a seat was shown and the rules that close a round are in no key,
  so no number could have retired a record when they changed.
- What retires a stored pairing is the generation of its namespace.
  `openPairingCache` and `openSectionPairingCache` are opened with the digest of the built pipeline the pass runs
  (`pass-prepare.ts` hands `pipelineDigest`),
  and `openNamespacedCache` discards the namespace when the stored generation differs.
  The probe on a throwaway directory:
  a record filled under one digest was resumed by that digest,
  found by none under another,
  and gone for the first once the other had opened the directory
  (the case in `slice-cache-pairing.unit.test.ts`).
- Every commit of the range that changed a byte the pipeline runs changed the digest,
  and one that changed none left the record as the same code settled it.
- The account of 2026-09-28 said no slice-cache file had been written since 04:26 UTC on 2026-09-27.
  That no longer holds:
  `corpus-pass-TianqiChen66626` wrote three block pairings under 3 and the section marker between 20:58 and 21:00 UTC
  on 2026-10-18
  (the first account here printed the files' local times,
  16:58 to 17:00 EDT,
  and labelled them UTC;
  B672).
  They resume only for the build that wrote them.
  A search of the slice-cache files under `~/temp/agent` (the throwaway checkouts left out,
  and three directories it could not list) found those and no other file newer than the bump;
  the first four searches of this account used a time the host's `find` refused and discarded the error,
  as M57 records,
  and were repeated with an accepted time and the error kept.
- What the number still holds is what the digest does not:
  a dependency,
  the Node binary,
  and the environment the straggler windows are read from.
  `477bbd6601` moved where the windows are read (the entry hands them down)
  and not whether they come from the environment.
  This account reads the sources of the range and not those three.

## B671: the Synthetic meter reader refused a body on 2026-10-18, and no public source gives another shape

Reported on 2026-10-18 by a pre-launch reading:
`[warn] [meterRecordOf] synthetic: budget unreadable, treating as spendable
(Synthetic /quotas body violated expectations: rollingFiveHourLimit is not an object)`.
The brief said the provider changed its `/quotas` response shape.

Outcome written on 2026-10-18.
No code changed,
and a note in the head of `provider-stream/src/synthetic-quota.ts` records what was found (`5da176cf4f`).

What was searched,
with no call to the endpoint:

- The reader is `parseQuotaSnapshot` in `synthetic-quota.ts`,
  called by the Synthetic client (`provider/src/synthetic-client.ts`)
  and caught in `meterRecordOf` (`provider-meters.ts`).
  It reads `rollingFiveHourLimit` and `weeklyTokenLimit` as the shape verified on 2026-07-16.
- The repository holds that shape in its tests,
  in the handover of provider facts and in a statusline package that parses the `x-synthetic-quotas` header;
  none holds another.
  `rg --files-with-matches rollingFiveHourLimit` found the reader,
  its tests,
  the statusline package and its tests.
- Synthetic's documentation of the endpoint (`dev.synthetic.new/docs/synthetic/quotas`) still shows the older
  `subscription` block only.
- The maintainer of Synthetic answered in `slkiser/opencode-quota` issue 276 on 2026-09-15 that a bearer key returns
  the full quota JSON with `weeklyTokenLimit` and `rollingFiveHourLimit`,
  and that `subscription` and `freeToolCalls` are legacy keys replaced by named limits.
  The same issue records a key on an account the reporter took to be on the free tier,
  answered with status 200 and the body `{}`;
  Synthetic did not say whether that is intended.
  A body of `{}` is refused by this reader with the message of 2026-10-18.
- A third-party schema (`aliou/pi-synthetic`) adds fields to `weeklyTokenLimit`
  and names no replacement for the five-hour block.

So the shape change is not shown by a public source:
the warning says a block was missing or not an object,
and the body was not recorded.
No second shape was added,
and the unknown-shape path stays as it was (a warning and a budget treated as spendable).

Follow-up written on 2026-10-18.
The refusal now says what arrived,
so the next reading settles the open point without the body being kept.
`QuotaShapeError` ends its message with the received top-level key names and the JSON type of each
(`received subscription:object, freeToolCalls:object`,
or `received {}` for an empty object),
or the type of a body that is no object (`received array`),
and never a value.
Key names are bounded to twenty listed,
forty characters each,
printable ASCII only,
since a key is provider text that reaches a log.
The meter warning carries the message whole.
Red tests:
`provider-stream/src/synthetic-quota.unit.test.ts` (five bodies)
and `provider/src/provider-meter-state.unit.test.ts`
(an empty body and the older shape,
with values that must not appear).
Guard demo:
with the line that adds the received description removed,
both cases fail.

Open:
the first reading after this change says what the provider returns,
and no second shape is read until it does.

Seen again on 2026-10-18 (UTC) in the pass TianqiChen66626,
which launched at 20:58 UTC at `7c21a56fc2`,
before the change that names the received keys.
The meter read `unreadable` in 126 of 126 readings,
with the warning `Synthetic /quotas body violated expectations: rollingFiveHourLimit is not an object`
and no key names (log line 204).
The pass treats the call as spendable,
so 651 Synthetic calls (298 of Qwen3.8-27B and 353 of Kimi-K3) ran with no reading of the allowance,
and the `JUDGE SEATS` lines printed `synthetic=wet` beside `METERS synthetic=unreadable`.
The lead's notes after the split say the reader is fixed in `w186`;
the outcome of this entry records that `w186` changed the warning and added no second shape,
so the open point stands.

Closed on 2026-10-19 (UTC) by B691:
the first reading that named the keys showed no five-hour block,
and Synthetic's rate-limit page no longer describes one.
A fix round for the pipeline defects of that pass runs in `w191`,
and its result is not recorded here.
[The pass log for 2026-10-18](../../../doc/planning/translation-repair-openrouter-2026-09-03-2026-10-18.md)
carries the reading.

Written on 2026-10-19 (UTC):
the round `w191` fixed the pipeline defects of that pass (B685 to B687).
Its second defect is the one named here:
the `JUDGE SEATS` lines printed `synthetic=wet` for a meter that said nothing,
and now print `unreadable-treated-as-spendable` (B686).
The refusal of the quotas body by the reader,
and the open point of this entry,
are unchanged.

## B672: local file times written as UTC in the account of the pairing cache version

Found on 2026-10-18 by the lead,
who read the host's zone (`date` gave 17:36 EDT where `date --utc` gave 21:36 UTC)
and the launch time of the pass (20:59 UTC) against the account.

Outcome written on 2026-10-18.
Fixed in the commit that carries this entry.

The account of B670 gave the writes of `corpus-pass-TianqiChen66626` as 16:58 to 17:00 UTC.
They were printed by `find -printf '%T…'`,
which prints the local zone,
and the label was added by hand.
`stat --format=%y` on the same files prints `-0400` beside the same digits,
and `TZ=UTC stat` prints the pairings at 20:59:08 UTC and 21:00:15 UTC and the generation markers at 20:58:54 UTC,
so the span is 20:58 to 21:00 UTC.
The TSDoc of `pairing-cache-version.ts` and B670 are corrected.
The only other copies were in those two files (`rg '16:5|17:00'` over the package).

The rule was already in `mistake-prevention-docs-and-references.md` (`find` and `stat` run under `TZ=UTC`),
and the zone scan (`clock-time-zones.unit.test.ts`) read the wrong time as it was meant to read a right one:
it passes any time followed by `UTC`.
No guard can catch it,
since the text does not say which tool printed the digits;
the prevention doc says so and names the check on the source.

## B673: reading the words of a place with recorded definitions was not built, because it reads the page the other way

Raised on 2026-10-18 by the lead as generation 19 of the artifact:
the pass records the smallest set of the page's definitions that change how the words of a place read,
the planner,
the apply stage and the reader of a record read the words with them,
and the proof of an apostrophe takes them too.

Outcome written on 2026-10-18.
Red tests for the words were committed and withdrawn (`920ab9bf80`,
`6a7daffc75`).
What was built is the proof of the apostrophe taking the definitions a caller gives
(red `262b62fae2`,
fixed in `aababac6fe`),
which the lead named and which stands on its own.
No generation was added,
and `ARTIFACT_SCHEMA_VERSION_V18` is still what the pass writes.

What was tried,
and what it showed:

- The mechanism was built in `consistency` and not committed:
  a counter that records,
  for any located place,
  the cut of the page's definitions that gives the span its slot and its word reading;
  the same cut joined for the two places of a splice
  and handed to `looseMarksOf`,
  `withLooseMarksOf`,
  `inReceivingForm`,
  `bringsAtEdge` and `gainsBesideKeptMark`;
  and a `definitions` carried on the plan so the check of the gains reads the replaced words with them.
- The mark at the end of words reads as an apostrophe of its word after closing markup
  only when the parse of the words closes something,
  and the parse of a fragment cannot see a delimiter outside it.
  In `Take *[*]a*’ now.` with `[*]` defined in another slice,
  the page reads the mark as `trailing` (an apostrophe),
  the counter locates `[*]a*’` at offset 6 and records `definitions: '[*]: u'`,
  the span read alone holds an edge apostrophe (as the page reads it),
  and the span read with that definition holds none and one loose mark.
  The planner then said `replace` for the words read alone
  and `unavailable` for the words read with the recorded definition,
  and a reader that took the record's words with that definition
  would find the edge mark of the span to be no apostrophe.
  So recording the definitions turns a place the pass carries correctly into one it leaves,
  and a record that reads back into one that does not.
- The other direction does not occur in a located place.
  Of the 205,740 strings of three to seven characters over `* _ ~ [ ] ( ) a b ’` and a space
  that start or end in `’` and hold a bracket with a label,
  84 pass `carriesOrCutsMarkup` and read another way with the definitions of their brackets,
  and all 84 are attached alone and loose with the definition.
  A span that reads attached only with the definition (`*a [b*] c*’`) is refused as cutting markup,
  because `carriesOrCutsMarkup` counts delimiters by parity and reads the span alone,
  and the counter leaves such a place unlocated with the reason `markup`.

What the search did not show:
a random search of 1,799,159 pages and quotes found no located place that records definitions,
and the hand-built page for `Take *[*]a*’ now.` found one at the first try.
The null was a sample that rarely held an opener,
a reference and a closer in that order,
and it is not evidence that the case cannot occur
(mistake-prevention,
searches and censuses that miss).

No difference between the pass and the reader exists today for the words:
both read them alone,
so a place whose words read differently with the page's definitions is not planned one way and read back another.
The limit is the one that the head of `consistency-loose-marks.ts` names,
and its reason there now includes the case of `Take *[*]a*’ now.`.

Options for the lead,
none taken:

- Leave the words read alone.
  Pros:
  the reachable case is read as the page reads it,
  nothing changes in a record,
  no generation moves,
  and a running pass is not cut off from the run directories of generation 18
  (`assertResumableSchemaGeneration` refuses another generation).
  Cons:
  a span that only a definition balances stays unlocated.
- Record the page's reading of the edge marks of a located span
  (which edge holds an apostrophe of its word) in place of definitions.
  Pros:
  the reader takes the page's reading as the counter had it,
  in every case,
  with no parse of a fragment.
  Cons:
  a new key and so a generation,
  and the pass-schema guard cuts a build off from older run directories.
- Make the markup test read the span with the definitions and by pairing,
  then record the definitions.
  Pros:
  `*a [b*] c*’` is located and read as the page reads it.
  Cons:
  the largest change,
  it unlocates places that are carried today (the span `[*]a*’` of `Take *[*]a*’ now.` cuts the page's emphasis),
  and it needs a generation.

Ranking:
leaving the words read alone,
then recording the edge readings,
then the definitions with the markup test,
because the first changes nothing the reachable case needs changed,
the second is faithful in every case at the price of a generation,
and the third carries the second's price and unlocates places the pass carries correctly today.
The attempt is kept outside the repository by the agent that ran it.

Written on 2026-10-19 (UTC):
the generation 19 that this entry raised was withdrawn and never landed.
The number 19 now belongs to the gate of B682,
which is a different change and records no definitions.

## B686: the judge seats line printed `wet` for a Synthetic meter that said nothing

Found in the read of the pass `TianqiChen66626`,
which launched at 20:58 UTC on 2026-10-18,
where the `JUDGE SEATS` lines printed `synthetic=wet` beside `METERS synthetic=unreadable` (B671).
It is the second of the three defects of that pass that the round `w191` fixed (B685 and B687 are the others).

Outcome written on 2026-10-19 (UTC).
Red `0c33424dd0` (provider) and `c3ac0080fd` (run),
fixed in `5235acfdee`,
with the shared fixture of the meter states in `0013c6f3d3` and `e9d720b816`.

The cause:
the line took `dry[provider] ? 'dry' : 'wet'` over the `BudgetView`,
a boolean record in which `routesAsDry` has already folded `unreadable` into not dry
(`run/src/corpus-run/run-seats-read.ts`).
The policy is unchanged:
an unreadable meter reads as wet and not dry
(`provider/src/provider-budget.ts`,
`MeterState` and `routesAsDry` in `provider/src/provider-meters.ts`),
and the warning `budget unreadable, treating as spendable` says so.
What was wrong is only that the line said the meter had answered.

Fixed:
`createProviderBudgets` returns `readStates` beside the view
(the type `ProviderBudgetsWithStates`,
while `ProviderBudgets`,
which the router takes,
is unchanged),
and `RunClient` gains `providerMeterStates`.
The line prints `dry` for a provider the router holds out,
`wet` for a meter that answered,
and `unreadable-treated-as-spendable` for a meter that said nothing.
It is one token,
so each `name=state` pair still reads on its own.
`doc/configuration.md` describes the three words.

What it does not change:
the meter reader still refuses the body of the quotas endpoint that B671 describes,
and the open point of B671 stands.

Cache keys and artifacts:
none.
The tally,
the meter states and the lines are never stored,
and a search of `stage/src`,
`provider/src` and `run/src` for `CACHE_VERSION` or `ARTIFACT_SCHEMA_VERSION` finds nothing.

Recurrence:
`mistake-prevention-messages-labels-and-counts.md`,
"Log lines that report nothing done".

## B691: Synthetic removed its five-hour window and its concurrency limit, and the meter read nothing for a day

Reported by the owner on 2026-10-19:
Synthetic removed its five-hour limit and its parallelism limits.
It is issue 1779 and closes the open point of B671.

What was read first,
on 2026-10-19 (UTC):

- Synthetic's rate-limit page (`synthetic.new/rate-limits`) describes a weekly credit limit alone,
  of which 2% comes back every 202 minutes,
  and names no five-hour request limit and no concurrency limit.
- One meter reading with the keys,
  approved by the owner for this issue
  (`run:budget-sample` at `9a8e1aa0cd`,
  14:35 UTC),
  printed the refusal B671 asked for,
  `rollingFiveHourLimit is not an object`,
  followed by `received subscription:object, search:object, freeToolCalls:object, weeklyTokenLimit:object`.
  The body has no five-hour block at all.
  The reading spent no generation;
  the other three meters read `wet`.

Outcome written on 2026-10-19 (UTC).
Red `a76e76e62a`,
fixed in `859954aa47` and `c642190ca4`,
docs `5cd3ec2a6e` and `23381065e0`.

- `parseQuotaSnapshot` reads a body with no `rollingFiveHourLimit` key as a snapshot with no five-hour window
  (`QuotaSnapshot.fiveHour` is optional,
  and absent rather than undefined).
  A key that is present and not an object,
  `null` included,
  is still refused,
  so a change of shape is still seen rather than read as no window.
  `weeklyTokenLimit` stays required.
- A refusal inside a block now also lists that block's keys and JSON types
  (`; weeklyTokenLimit holds percentRemaining:number, nextRegenAt:string`),
  under the bounds the top-level list keeps,
  and never a value.
  The fields inside `weeklyTokenLimit` were not seen on 2026-10-19,
  since the reader stopped at the five-hour block;
  if they changed,
  the next reading names them.
- `syntheticIsDry` decides on the weekly budget and reads the five-hour window only when the reply carries one.
  `EveryProviderDryError` says Synthetic has no weekly credit left.
  The meter line prints `syntheticFiveHour=none syntheticThrottled=none` when there is no window;
  `none` and not `no`,
  since nothing was read that says the account is not throttled.
  The readers of those fields,
  `meter-sample-read.ts` and `meter-report-level.ts`,
  carry the value as written,
  and their tests now hold both forms.
- `SYNTHETIC_PER_MODEL_CONCURRENCY` is positive infinity,
  as `HYPER_PER_MODEL_CONCURRENCY` is,
  and both the client's default and the router's slot limits read it.
  The ceiling of the slot ledger is still proven,
  through fixture limits.
  `doc/troubleshooting/translation-repair-provider-concurrency.md` keeps the measurements of 2026-08-30 as a record
  and states the change;
  the width 5 that D24 of `audit-ledger-docs-against-code.md` states was true when written.

What was lost:
from the first refusal on 2026-10-18 to this fix,
every reading of the Synthetic meter was `unreadable`,
and the router treated Synthetic as spendable,
which the weekly budget allowed;
TianqiChen66626 ran 651 Synthetic calls that way (B671).

Cache keys and artifacts:
none moves.
No cache version is declared in the provider,
provider-stream or shared-contract code this changes,
and the stored records are keyed by what a seat was asked.
Routing changes whose voices a round hears,
which the accounts of the versions record as riding inside them (ledger P1);
on 2026-10-19 the cache-account audit counts 827 of 865 source commits since `d614a0c1d` named by no account,
so these five go to the next pre-launch account with the rest.

Confirmed on 2026-10-19 (UTC).
After the change landed on `main` at `bcf2545b8e`,
a second keyed `run:budget-sample` reading,
approved by the owner,
at 15:23 UTC read the Synthetic meter:
`synthetic=wet`,
`syntheticWeekly=86.76063906122448%`,
`syntheticFiveHour=none`
and `syntheticThrottled=none`.
So `weeklyTokenLimit` still holds `percentRemaining` and `nextRegenAt` as a number and a string,
and issue 1779 was closed with that reading.

## B814: the cache-account audit read a version set in a merge resolution as in no commit

Found on 2026-10-22 (UTC) by the lead,
running the audit `cache-account-audit` at `c5bf2368d3` before the launch of zheermao23.
It refused `CONSOLIDATE_CACHE_VERSION = 31 in package/translation-repair/artifact/src/consolidate-key.ts is not in any commit`,
although 31 was committed.

Outcome written on 2026-10-22 (UTC).
Red `2464ee7f7`,
fixed in `c1cbb3384`.

The cause:
the translation-repair branch moved the number from 29 to 30 in `a40de882b` (B762)
while main moved it from 29 to 30 for B805 in `27db2c256`,
and the merge that met the two (`ba30e7ac6`) wrote 31 in its resolution.
No ordinary commit holds 31.
`cacheVersionSetting` asked `git log -S` for the commits that change how often the declaration appears,
and without `-m` git lists no merge at all.
Had it listed the merge,
`declarationLineCounts` would still have read nothing:
`git show` writes a merge as a combined diff,
with one mark column for each parent (`++export const ...`),
and the reader took one column and read the rest of the line as the declaration,
which then did not read.

Probed with git 2.56.0 on a throwaway repository before the fix:
`git log -S` without `-m` lists no merge;
with `-m` it lists a merge once for each parent its diff against differs,
so a hash can repeat;
`git show` on a merge prints a combined diff whose hunk header opens with one `@` more than it has columns,
and a line the resolution wrote is marked `+` in every column.
A merge that only brings in a side branch's change prints no line of its own.

Fixed:
`cacheVersionSetting` passes `-m` and keeps each hash once,
where it first stands,
and `declarationLineCounts` reads as many mark columns as each hunk header says.
A line marked `+` in every column counts as added,
one marked `-` in every column as removed,
and one that only some parents lacked as neither,
since the merge brought it in and did not write it.
So a merge that only brings in a side commit sets nothing,
and the side commit stays the setting.
The cases:
`cache-account-setting.unit.test.ts` finds the merge that set a value in its resolution,
and finds the side commit and not the merge where the merge only brought the value in;
`cache-account-read.unit.test.ts` counts a combined diff's added,
removed and brought-in lines.

A second reading of the fix found one more column fault,
fixed before it landed.
The reader set aside every line opening with `+++` or `---` as a file header,
but a merge of three parents marks a line its resolution wrote `+++`
and one it removed `---`,
so an octopus merge that set a version would have been refused the same way.
A file header never reads as a declaration,
so the reader now sets no line aside,
and a case counts an octopus merge's `+++` and `---` lines
(red on the first fix,
where both counted nothing).

After the fix the audit names `ba30e7ac6` as the setting of 31,
and the other seven versions as before.
The account of 31 said how 30 was reached but not how 31 was:
it gave the reason the number left 29 and said main set 30 "while this change was written".
It now names both commits that set 30,
the merge that set 31,
and the evidence that no pass ran on a build holding either 30:
the last runs,
zheermao22 and XingZ6017,
log `START tip=2a336603bf`,
which holds neither `27db2c256` nor `a40de882b` (`git merge-base --is-ancestor`).

Cache keys and artifacts:
none moves.
The audit reads git and the source and writes nothing a pass reads.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Cached decisions",
the paragraph on history readers and merges.
