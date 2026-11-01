# Audit ledger: probes that read one editor card

A card the editor calibration could not bench,
the seats that flag leaves unconfined,
a relative share that moves with the run,
the probe that reads one card's edit against a known fix,
the frozen pool of forty trials it now reads as a seeded draw,
the guard that holds an arm to its amount call by call and to the bench's amount,
the sheet and the headline that no cheap reply passes,
the read step that checks the logs of the four arms,
the guard that admits each attempt of a call and counts what can have been billed,
the decision rule held as code with a floor taken from an exact enumeration,
and the dry run that prints what the arm will do before it calls anything.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
Entries from B894 on are in [the paired rule and shared trials file][paired-rule] and [its multi-pass arms follow-up](audit-ledger-editor-bench-multi-pass-arms-and-repeat-agreement.md),
because this file stood at 982 lines.

## B870: the editor calibration had no flag naming a card, so an unmeasured card could not be benched as an editor

Status:
fixed on 2026-10-23 (UTC),
found by the lead's brief for the effort-level bench of the Sonnet 5.5 cards.
Red `005b05672c8cd8e4649737834b555a0a104a6d9c`,
fixed in `f5e3745d6c8b4be784f0ec0e6a80e6819e7d028c`.

`editor-calibrate` seated the editor chair from the measured roster,
and the Sonnet effort cards carry `judge-unmeasured`,
so no command line could seat one of them as an editor.
The fix gives the command the `--candidates` and `--candidates-alone` flags the judge fidelity probe reads,
with the same readers and the same refusal wording
(`probe/src/corpus-run/probe-candidates.ts`).

No cache version moves:
the editor roster is key material of the slice key (`repair/src/repair-slice-key.ts`),
so a card arm has a different key by content,
and the calibration reads no payload cache.

## B871: `--candidates-alone` on the editor calibration confines only the editor chair

Status:
found on 2026-10-23 (UTC) while reading the fix of B870,
documented in `f5e3745d6c8b4be784f0ec0e6a80e6819e7d028c`,
not changed.

The critics,
the panel and the judges of a calibration arm stay the whole roster,
so every arm needs every provider's key and calls every provider.
The flag means "the editors are these cards",
not "nothing else is asked".
The bench package README says so beside the flag ("the flag confines the editing and not the spend").

## B872: a lone editor's slate ships unjudged, so a calibration with `--candidates-alone` has no ballots

Status:
found on 2026-10-23 (UTC),
the run does not refuse,
and the bench package README states it.

With one editor seated there is one candidate,
and the stage ships it without asking a judge.
The per-model vote share the calibration reports has nothing to count,
so the alone mode reads edits but not votes.
`EDITORS SHIPPED` is the only reading in that mode.

## B873: the usage assertion of the editor calibration's test moved with the flag declaration

Status:
fixed on 2026-10-23 (UTC) in `decd0d0836057e51e594965ae15c87c1d43a3359`.

The existing `editor-calibrate.unit.test.ts` compared the usage text to a string the declaration no longer produced.
The case now names the new flags.
The failure was found by the package's own `test:unit`,
and no behaviour changed.

## B874: the editor calibration's relative share moves too far between identical runs to separate effort levels

Status:
recorded on 2026-10-23 (UTC),
no code changed,
so no later round benches an effort level on it.

The calibration reads a card's disinterested vote share against the writers seated beside it.
That is a relative reading with no ceiling.
The bench package records how far it moves:
the TSDoc of `sliceStandingLines` in `bench/src/corpus-run/editor-calibrate-standing.ts` states that
a four-slice standing moved one model from 52.2 to 26.7 to 10.0 percent across identical runs.
Critics are asked fresh in each run,
so the accepted issues differ between arms.
Every arm also calls every provider (B871),
so a bench of one card at three effort levels costs the whole roster three times and still cannot tell the levels apart.

Decided by the lead:
keep the flags of B870,
and measure an editor card with the absolute probe of B875.

## B875: no instrument read one editor card's edit against a known fix

Status:
fixed on 2026-10-23 (UTC).
Red `246f55480b4b2c6a10b46549a6d8d151a41ee15f`,
`00cb16908aaada55b9f01a26cf6ea332637a190c`;
fixed in `7e94b96fa3161d18361dd1b0df8970830f4a5e11`,
`0f2b490642a33a9d35a74a67f18957585d795251`.

The new command `editor-fidelity-probe` (`probe/src/corpus-run/editor-fidelity-probe.ts`)
mirrors the judge fidelity probe's flags:
`--only`,
`--cap`,
`--damage`,
`--candidates`,
`--candidates-alone`,
and `--max-usd` (default 2).
Each trial is a reviewed reference's damaged twin with a known fix.
The probe seats only the named card as editor,
through the sheet and the patch format of the real editor stage,
and scores the patched text without a model:
fixed (exact,
or equal to the clean text once whitespace runs are collapsed),
collateral (text outside the defect changed),
and refused or unusable replies counted apart.
It asks no critic,
panel,
judge or checker.

The tally per card reads fixed,
collateral,
unusable,
out of the trials.

What the probe reads is bounded:
the reviewed references give three insertion twins and one alteration twin,
so a card meets four trials in a run.
A card that fixes all four reads the same as a card that fixes them with nothing to spare,
and the probe cannot see a ceiling effect.
A reading is a floor on a card's fidelity,
not a ranking between cards that all pass.

No cache version moves:
the probe reads no cache,
and the gather it shares with the stage gives the stage the same output (B876).

## B876: the part of the editor stage before selection could not be asked without a judge roster

Status:
fixed on 2026-10-23 (UTC) in `7e94b96fa3161d18361dd1b0df8970830f4a5e11`,
red `246f55480b4b2c6a10b46549a6d8d151a41ee15f`.

`runEditorStage` built the sheet,
asked the editors,
passed what they said through the apply gate,
and then asked the judges.
It refuses a roster too short to judge,
so a probe could not seat one card through it.
The part before selection is now `gatherEditorProposals` (`repair/src/editor-gather.ts`),
and the stage calls it.
The probe asks the same function,
so the sheet is byte-identical to the stage's and no judge is in the call.
A test sends both the same inputs and compares the messages.

`sentenceRegionAround` (`repair/src/sentence-region.ts`) widens a defect span to the whole sentences it touches,
which is the region an accepted issue's critic would quote and an editor may rewrite.

No cache version moves:
the stage's output for the same inputs is unchanged,
and the existing stage tests pass as written.

## B877: a deletion twin has a fix the editor must write, which no deterministic test can score

Status:
decided on 2026-10-23 (UTC),
open to the owner's veto.

For an inserted sentence the fix is its removal,
and for an altered number the fix is the clean digits,
and both can be read from the patched text.
A deleted sentence is restored only if the editor writes the sentence again,
and the wording that counts as the same sentence is a judgement.
`--damage deletion` is refused with that reason,
and the default is the two families the probe can score.

## B878: the probe prints a per-arm cost ceiling and gates on the first-pass figure

Status:
fixed on 2026-10-23 (UTC) in `0f2b490642a33a9d35a74a67f18957585d795251`,
open to the owner's veto on the gate.

The ceiling counts one input token to the character of the sheet,
and the whole completion cap of the Claude cards
(`CLAUDE_COMPLETION_CAP`,
32000 tokens,
thinking counted in it) at the output price,
for each ask.
The first-pass figure is one ask a trial,
summed over the cards and trials.
The run is refused when it passes `--max-usd`,
before any call
(replaced by the guard of B881,
which refuses a call and not a run).
The worst case,
which asks again in each retry round and once more in a recovery round (five asks a trial),
is printed and not gated,
because the five-ask figure for four Sonnet trials passes the default of 2 USD that the first pass stays under.
A card with no price table in `claude-card-sides.ts` is refused rather than priced at zero.

## B879: the probe's client holds the Claude key alone and refuses any model that is not a named card

Status:
fixed on 2026-10-23 (UTC) in `0f2b490642a33a9d35a74a67f18957585d795251`.

`editorProbeEnv` hands the client builder only `TRANSLATION_REPAIR_CLAUDE_API_KEY`,
so a launch that holds every provider's key reaches one route.
When the key is unset,
the refusal names the Claude route alone.
`cardsOnlyClient` wraps the client and refuses a request for any model id the run did not seat.
A keyed launch still logs the unconfigured warnings for the other providers,
which the entry does not ask for.

## B880: four trials could not separate effort levels, so the probe reads a frozen pool of forty-one

Status:
fixed on 2026-10-23 (UTC) in `56ff293b924b06fb8de0ce0b670ec969712ea52b`,
red `88e995ca38fb9418cd0029c41e68632077ac2400`,
open to the owner's veto (the lead's ruling of at least forty scoreable trials per card).
Superseded by B888 on the count of trials (forty in place of forty-one),
the rule (a seeded draw run by a command)
and the replay.

B875 recorded the limit:
a card that fixes all four reviewed twins reads the same as a stronger one.
The pool (`probe/src/editor-fidelity-pool-manifest.ts`) is the three reviewed references,
first and unchanged,
and thirty-four heading sections drawn beside them from the same corpus pin
(`probe/src/editor-fidelity-pool-drawn-1.ts` to `probe/src/editor-fidelity-pool-drawn-4.ts`).
Together they stand for forty-one trials over twenty-four entries:
twenty-three insertions and eighteen alterations,
none of the entries holding more than four.
The twins come from the machinery the judge probe and `damage-sample` use
(`insertBorrowedSentence` and `alterSharedNumber` in `repair/src/fidelity-damage.ts`),
and every one is replayed through `buildReviewedFidelityReference` against the pinned clone,
so a range or a damaged twin that drifted from its hash refuses the run.
Only ids,
offsets and hashes are committed;
the corpus is unlicensed.

The machinery yields far more than forty.
Over the 92 readable entries,
90 split into the same heading sections in both languages,
and 221 of their sections hold a reference of at least 400 characters,
183 of which admit an insertion and 123 an alteration.
The draw stops at forty-one because that is the ruling's floor,
not what the machinery can give.
It is deterministic:
a second run gave the same manifest.

The rule is in the header of the manifest.
Entries in id order,
the reviewed entries left out,
the production aligner's equal-shape entries only,
sections of 400 to 2200 English characters,
the first section that builds an insertion twin and passes the scoreability check,
and the first other section that builds an alteration twin.
A trial is scoreable when it builds with one envelope,
the clean text scores exact,
the damaged text scores not fixed,
and a scripted editor applying the known fix through the real stage path
(its sheet,
its parse and its apply gate)
reads fixed with nothing else touched.

What the pool is not:
it is not source-reviewed.
A drawn section was chosen by a rule and never read for meaning by a person.
That suffices because the score compares the patched text with the text before the defect was planted,
so what the reference says is not what is measured.
`reviewedOn` on a drawn specification dates its freeze and says nothing of review.
The first entries by id are not a random sample,
two trials an entry is a thin spread,
and forty-one trials still bound the claims an arm can make:
a difference of a few trials between two efforts is inside what the spread arm shows.
`--cap` defaults to 64,
so an arm run without it asks every trial,
and a run prints its pool on the line `pool: N trials over M entries (insertion i, alteration a)`.

No cache version moves:
the probe reads no cache and the pool touches no stage key.

## B881: the gate on the first-pass total refused every arm of a forty-trial pool, so a per-call guard replaced it

Status:
fixed on 2026-10-23 (UTC) in `4434c4257a2a7462fce3700fcff3e7591dd2830e`,
red `2219151053267b904ca3a0dc84650e4768ecb998`,
`27b17b9065a6d91fbc77fe2d9a1536708bdd9741`,
open to the owner's veto.

B878 refused a run whose first-pass ceiling passed `--max-usd`.
The ceiling takes the whole completion cap on every ask,
so forty-one trials price at 14.97 USD for one Sonnet card against the default of 2.
`spendGuardedClient` (`probe/src/corpus-run/editor-fidelity-spend-guard.ts`) asks the question at each call instead:
the spend measured so far plus that call's ceiling must not pass `--max-usd`,
or the call is refused and the arm ends there,
with the rows it finished in their files.
The one-ask and worst-case ceilings are still printed,
and a line after them says the guard is on.

Spend is read from the replies.
A reply reports the tokens it used,
thinking counted in the completion,
and the card's prices turn them into dollars.
The report folds cached prompt tokens into one prompt count,
so the prompt is priced at the higher of the input and cache-write rates:
the figure does not fall under the bill and an overcount only stops the arm sooner.
A reply that reports no usage is counted at its call's ceiling.
A call that throws counts nothing,
and the attempts a client retries inside a call are its own to report and are invisible to the guard.
So the guard is a runaway stop on the replies it sees,
not an exact ledger,
and the process meter (`SPEND` lines) remains the record of what was billed.

Follow-up of 2026-10-24 (UTC),
round 5:
the sentence "a call that throws counts nothing" no longer holds,
as B885 had already changed it.
A throw now counts what can have been billed.
A call that never left counts nothing:
a layer above the client rejected it (a retained prompt claim,
a withheld seat),
so the client told the guard of no attempt and sent none.
A call whose last attempt wrote its own `SPEND` line counts that line once,
and no ceiling beside it.
A call whose last attempt wrote no line counts at the call's ceiling,
except an HTTP error status that carried no body,
which proves no billable body was delivered.
The attempts a client retries inside a call are no longer invisible:
the guard is told of each (B885).

## B882: a refusal thrown inside the client is a lost voice, so the guard ends the arm through the arm's signal

Status:
fixed on 2026-10-23 (UTC) in `4434c4257a2a7462fce3700fcff3e7591dd2830e`.

`attemptStageCall` (`stage/src/stage-call.ts`) turns a call that throws into a lost voice and asks again,
except when the signal has aborted,
the request is a fault of the caller's own building,
or every provider is dry.
A guard that only threw would have read as a lost voice,
been re-asked up to five times a trial,
and let the next trial begin.
The guard aborts the arm's signal with the refusal as its reason before it throws,
so every layer carries the refusal out,
and every later call is refused with the same one without reaching the client.
A test runs the whole probe with a fake client whose replies cost 0.205 USD against `--max-usd 0.5`:
the second call is never made and the finished row stays in its file.

## B883: an insertion at the end of a section is not scoreable, because removing it leaves the separator before it

Status:
recorded on 2026-10-23 (UTC),
no code changed.
Superseded by B887 and B888:
the equivalence now ignores trailing whitespace,
so these insertions are scoreable and the draw takes them.

The insertion splice leads the borrowed sentence with a space.
Mid-section,
removing the sentence leaves two spaces,
and the whitespace-collapse equivalence reads that as the clean text.
At the end of a section it leaves one trailing space,
which the equivalence does not cover,
so a perfect edit scores defect only with collateral.
The draw's scoreability check found two such insertions
and passed over them.
They are not in the pool,
and a later round that wants them has to extend the equivalence to trailing whitespace,
with a red test,
before it draws again.

## B884: the process slips of this round and of round 2, for the commit comments

Status:
recorded on 2026-10-23 (UTC).

Round 2 wrote `editor-fidelity-probe-read.unit.test.ts` after the code with no red run.
It is shown red here by mutating each of its four helpers in the working tree and rebuilding:
`seatedCardsOf` no longer requiring `--candidates-alone`,
`sheetCharsOf` counting nothing,
`pairName` swapping its handles,
and `readingWord` shortening the refusal wording.
Each of the four suites failed on its own assertion,
and the tree was restored from the committed text.
Round 2 also revised the red test files in the fix commit
(`0f2b490642a33a9d35a74a67f18957585d795251` against the red `00cb16908aaada55b9f01a26cf6ea332637a190c`):
formatting and structure for the source scans.
This round the red tests stayed byte-identical to their fix commits
(`git diff` between each red commit and its fix commit lists no test file of the probe).
The one exception is a layout change of a single statement,
made in its own commit (`27b17b9065a6d91fbc77fe2d9a1536708bdd9741`) between the red commit and the fix,
because the red commit left a `no-await-in-loop` warning that `lint:oxlint` reads as a failure.

## B885: the spend guard did not count a call that threw or a retried attempt, so an arm could overspend unseen

Status:
fixed on 2026-10-23 (UTC) in `9a8b0a4c1fd801c25a9ab03930b2e017800628a6`
and `96a6b701ec5d770b70581fb3a785c791232c6dd2`,
red `0bb5810d6e2616f7fee982857ec4caf39912c052`,
`744ab1a4722da0e2868a757b176302bcabcac6df`,
`3c1cd1e0657283cbd3a1312a1e147c79b5fbb6b3`,
still red `cd6f8380806321aff0d5632b6b689867e63189aa`,
`7c61c829b725ca88250b0ba0e0dfb2cc0b56ec48`,
open to the owner's veto.

B881 said a call that throws counts nothing and the attempts a client retries are invisible to the guard.
Both are money.
The Claude client asks again on a 529 or a stream that stopped,
and Anthropic bills an attempt that delivered a stream whether or not its usage arrived.
The Bedrock,
NeuralWatt and OpenRouter clients already wrote each abandoned attempt at its bound;
the Claude client did not,
so a call's `SPEND` line read its last attempt alone and the run meter missed the rest.

Two changes close it.
The Claude client now writes every abandoned attempt through `onAbandonedAttempt`
(`provider/src/claude-abandoned-spend.ts`),
priced at the most it could have been billed:
the request body's bytes as input tokens at the higher of the input and cache-write rates,
and the `max_tokens` it sent as completion.
The spend guard counts a call that throws at its ceiling by a `SPEND` line of its own,
except an HTTP error status that carried no body,
which is the one throw counted as free,
and the arm ends with a line `GUARD total=<dollars> calls=N atCeiling=N`.
So the `SPEND` lines of an arm add to the guard's total,
which is a check the read step makes (B889),
and the guard is a ledger of the bill and no longer a stop on the replies it saw.

A throw after an abandoned attempt reads one ceiling high,
so the guard stops an arm sooner and never later.
The one-way flag the client keeps moved to its own module (`provider/src/one-way-flag.ts`) with a unit test,
because the source scans want every module to own one.

Cache:
`git log --no-merges -G _CACHE_VERSION` over every commit of this branch since
`83194090a561fd1134e96c35b7164daf46c0cb3a` lists none.
The same search over `c265e8df9525f4513b8229482d3e8d932ea630df`,
a commit known to move a cache version,
lists it,
so the search can show a hit.
The Claude client change adds a log line and a meter increment per abandoned attempt;
it touches no request body,
no reply,
no key and no record a cache holds.
No cache version moves.

Follow-up of 2026-10-24 (UTC),
round 5:
the guard admitted a call,
and the Claude client sent up to five attempts for it.
An arm could therefore pass its amount by the retries of its last call,
since the guard asked whether the spend so far plus one call's ceiling fitted,
and the client then billed up to five attempts of that size
(the retries admitted once family,
recorded in `mistake-prevention-spend-guards-and-client-retries.md`).
Fixed on 2026-10-24 (UTC) in `952474ba266ced69ae7d3add4308cf858410cecf`
and `a52f16ab65ad7a1334eeec7526d4d957ac882ac0`,
red `8787ec3ae2eb3d26687474cd6aec50f14c8dcb54`
and `b15e2ed7c8c4245f92aca2c73ab13a6ea9cad70e`,
open to the owner's veto.

The Claude client now tells its caller before it sends each attempt,
the first and every retry,
with the card and the most the attempt can be billed
(`provider/src/transient-retry.ts`,
`provider/src/claude-attempt-admit.ts`),
and the run route hands the caller's admit down to it without changing a request body
(`configureProviders` in `run/src/corpus-run/run-providers.ts`).
The guard takes its client as a builder of the admit it offers,
and an attempt is admitted when the spend so far plus its bound stays within `--max-usd`.
Otherwise the attempt is never sent and the arm ends.
The proving cases:
`every attempt is told to the caller before it is sent` in the provider,
`TELLS THE CALLER OF EVERY ATTEMPT A ROUTED CALL MAKES` in the run package,
`HANDS THE CLIENT BUILDER THE GUARD'S ADMIT` in the probe's run test,
and `REFUSES A RETRY THROUGH THE WHOLE ROUTE THE PROBE BUILDS` in the guard test,
where a scripted transport answers a retryable status four times at a spent sum
at which the old guard admitted the call and the fifth attempt would cross the cap.

The guard also charges only what can have been billed.
A call that never left costs nothing;
a call that threw after retries counts one line for each attempt the client wrote
and the call's ceiling only when its last attempt wrote no line;
a reply with usage but no line on the meter counts at its ceiling.
The cases are `COUNTS NOTHING FOR A CALL THAT NEVER LEFT`,
`COUNTS ONE ATTEMPT ONCE WHEN THE CALL THROWS AFTER RETRIES`
and `COUNTS A REPLY WHOSE OWN LINE IS MISSING EVEN WHEN AN EARLIER ATTEMPT WROTE ONE`.
The recheck's two surviving mutants of the guard
(the reading of the meter's change,
and the baseline of the meter at the guard's construction)
are killed by `COUNTS A REPLY WITH USAGE BUT NO LINE ON THE METER AT ITS CEILING`
and `READS THE SPEND OF THE ARM ALONE`:
each mutant was built and failed its case.
`KEEPS A STREAM ERROR EVENT SAFE TO CALL FREE` pins the one throw counted free
by reason of what the stream delivered,
a 200 with text followed by an error event,
which writes its bound line before it throws.

## B886: the bench amount of 20 USD was held by nothing but the sum of three hand-typed caps

Status:
fixed on 2026-10-23 (UTC) in `f60c7b102162663a03f973e62d18ec8ae6c63b81`,
red `5ab2a717844a2b036bdfc2b83f1637f1d9721c84`,
open to the owner's veto.

The owner ruled 20 USD for the whole bench,
and each arm held itself to its own `--max-usd` only.
Four arms with caps of 3,
4,
4 and 8 add to 19.
`--bench-spent-usd` states what the arms already run measured,
and an arm whose `--max-usd` plus that figure passes `EDITOR_BENCH_MAX_USD` (20) is refused before any call,
with both figures in the refusal.
The figure is typed by whoever launches the next arm,
from the `GUARD total=` line of the last,
so the read step checks the bench total again from the logs (B889).

Follow-up of 2026-10-24 (UTC),
round 5:
the flag had a default of 0,
so an arm launched without it passed every check with no figure for what the earlier arms had spent.
Fixed on 2026-10-24 (UTC) in `6a2677b200e1b3be12461b6996542f910b741bc6`,
red `a3a96ea1106193eab7d7983987a510b128d6ca31`,
open to the owner's veto.
`--bench-spent-usd` is now required:
a line without it is refused with one line at exit 6 before anything is read or built,
even with `--cap 0`,
and the guard line prints it.
The read step checks that it was given four distinct logs at the cards low,
medium,
medium and high,
and that each arm states an earlier spend at least the `GUARD total=` of the arms before it
(`checkEditorBench` in `probe/src/corpus-run/editor-fidelity-arm-bench.ts`).
Each check that fails exits 6.

## B887: a cheap reply could pass the probe, so the sheet carries page context and the headline counts only exact fixes

Status:
fixed on 2026-10-23 (UTC) in `84f697f3b46e6bb23cb7cb90e5c032215bbe5c3e`
and `ad33d9916abfbc9cf78f45ea6fb1fcbe49139e4f`,
red `a57dcc2e76dfa030b958ed3de0a3f1890337915d`,
`ebf264ba4d27d7a513a61f45a884cfeaa41548a6`,
`bbb6991f25ee8bf4207c3a37033a4e5f6e509500`,
still red `c4a09d7f6131e581520c00fac9f9debee95d4bea`,
`95f82c9dcbf0cd933b976d1d68d8da9ccf540a72`,
open to the owner's veto.

Two findings of the recheck made the old instrument weaker than the question.
The sheet held the passage and its issue alone,
where the editor stage is handed the sections either side,
the declared names and the whole original of the page.
And the headline counted a patched text that removed the defect and changed more,
and a reply that removed the planted sentence together with its neighbours.

The sheet is built by `buildEditorMessages` through `gatherEditorProposals`,
the stage's own builder and gather,
from context prepared by `prepareDocumentPair` and read by `neighbouringSource` and `neighbouringIncumbent`
(`probe/src/editor-fidelity-context.ts`).
The sheet of one trial of the headed fixture carries a declared-names block with the pronoun line,
the original and the translation of the section,
the heading and the next section as `NEARBY ORIGINAL, CONTEXT ONLY` and `NEARBY EXISTING TRANSLATION, CONTEXT ONLY`,
the instruction that the nearby blocks are context and must not be edited,
and one region with its issue,
its current text and a context window.

The headline is a clean fix:
a patched text equal to the clean text,
or equal once whitespace runs are collapsed and trailing whitespace is ignored.
Everything else that removes the defect is defect only,
has its own count and is never in the headline.
The issue of an insertion trial quotes the added sentence with its neighbours.
Production quotes the span a critic flagged,
which is the added sentence alone,
so the wider quoted region is a deliberate departure and is open to the owner's veto.

What the sheet still lacks,
stated as limits:
the reference context,
which production buys from keyed web lookups;
the entry's `info.yml`;
the line-structure rule addendum;
a per-defect issue summary,
for the issue is the generic one of its kind;
and the size of the unit,
for the unit is a heading section where production slices are at most 400 characters,
so the probe's editor reads more of the page than production's does.

Gaming controls,
run against the old and the new reading on the old pool of forty-one:
- Delete the quoted insertion region:
  equivalent in 22 of 23 trials before,
  defect only with collateral now,
  and not counted.
- Restore the number and nothing else:
  defect only with collateral in 17 of 18 and counted before,
  and not counted now.
- A perfect fix plus a trailing `!`:
  defect only in 18 of 18 and counted before,
  and not counted now.
- Echo the sheet:
  unusable.
- `edits: []`:
  refused,
  counted apart.
- A wrong number:
  not fixed.
- The ideal fixes:
  exact.

The decision rule is printed on the `RULE` line of every arm before its first call:
clean fix is exact plus equivalent,
read pooled and per kind of defect;
an effort differs from the default (medium) only when its pooled clean fix count is further from the mean of the two
medium arms than the larger of 3 and the two medium arms' own difference,
and neither kind moves against it;
any other difference,
and any tie,
keeps the default effort.

Follow-up of 2026-10-24 (UTC),
round 5:
this rule was prose,
and its threshold of 3 trials calls a difference in more than half of the benches with none (B891).
The rule is now code and its text is longer than the one quoted here.
The wider region of an insertion trial stays and is now labelled in that text:
it measures locating an added sentence among its neighbours and copying the neighbours verbatim,
and not the span-only view production gives an editor.
The read step prints the defect-only count of each kind beside the headline as a secondary count.
Fixed in `4f1cd29786853868de4d181e45709905af077f2f`,
red `a2f92a6063ba03682b17ae964125abe109ba3794`,
open to the owner's veto.

## B888: the pool of forty-one was a first-sections sample, so the pool is a seeded draw by a command in the repository

Status:
fixed on 2026-10-23 (UTC) in `fe1949dfaeed8c6f630155e505af8625d74b2504`
and `921e578fe8c2c958c6c0078d2c1de210983d09c2`,
red `1486b9491feff53ab24e5f9f078bb8232c49661e`,
`296648165e51c42ba2fa77dbaf9a443f7c8f38ab`,
still red `47d5f7edd8c3cc1e32665a647fe6ebda53bdd945`,
open to the owner's veto.

B880 drew the first entries by id and recorded the rule in a comment.
The rule is now code.
`editor-fidelity-pool-draw` (`probe/src/corpus-run/editor-fidelity-pool-selection.ts`) orders the readable entries
by a hash of a domain,
a seed and the entry id,
takes entries in that order until the pool holds forty trials,
and builds at most one insertion and one alteration trial from each.
The seed is `round-4`,
the domain is `editor-fidelity-pool/v2`,
and the draw is dated 2026-10-23.
`--check` compares the committed files with a fresh draw,
then reads every committed reference through the hash-verifying reader and runs the scoreability check on its trials.

Over the pinned clone `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
93 entries are listed,
50 are eligible,
and 21 were taken.
They gave 36 drawn trials,
19 of insertion and 17 of alteration,
beside the 4 reviewed trials,
so the probe prints `pool: 40 trials over 24 entries (insertion 22, alteration 18)`.
Sheets run from 25,644 to 32,094 characters,
median 27,839.
Entries left out and why:
not of equal shape 2,
unreadable 1,
no donor for an insertion 25,
no donor span 16,
no number shared with the Chinese 45,
and not scoreable 6
(5 where the scripted editor read unusable,
1 that scored defect only with collateral).
The draw ran twice and gave the same files,
and the replay read 36 references and found all 40 trials scoreable.

A drawn trial is scoreable only when the scripted ideal edit scores exact,
the damaged text scores not fixed,
and deleting the quoted region does not read as a fix.
That last condition,
and the end of a section no longer being a separator problem,
supersede B883:
the equivalence of B887 ignores trailing whitespace,
so an insertion at the end of a section is scoreable and is drawn when its section is.
B880's count of forty-one trials,
its first-by-id rule and its hand-run replay are superseded by this entry.
The pool is still not source-reviewed,
two trials an entry is still a thin spread,
and forty trials still bound what an arm can claim.

No cache version moves:
the draw reads the clone and the probe reads no cache.

## B889: nothing read an arm's log for a failure before its figure was read, so a code reads the logs

Status:
fixed on 2026-10-23 (UTC) in `3f4aa8cde943417d9de1c2e7fefde5c3923c8b04`,
red `ff706ab047255f356582738f7999560690251a33`,
open to the owner's veto.

A bench of four arms leaves four logs and a clean fix count in each.
A count read from a log that ended in an error,
a different pool,
or a spend that does not add up is not evidence.
`editor-fidelity-read` checks each arm for its exit status
(read from `<arm>.exit` beside `<arm>.log`),
the manifest line and its digest,
the pool line,
the ceiling line,
the guard line,
the `EDITOR` line,
the row count,
no `[error]` line,
the `SEAT` line
(`threw` stated,
nothing unreachable,
no seat withheld for a refused credential,
no dark seat),
and the identity of the `SPEND` lines' sum with the `GUARD total=`.
Across the arms it checks that every manifest digest is the same and that the bench total is under 20 USD.
It prints one `PASS` or `FAIL` line each,
then `CHECKS n pass, n fail`,
and exits 6 when any failed.
The probe now logs `manifest: N specifications, digest <hex>`.
The runbook for the four arms is in `probe/README.md`.

Follow-up of 2026-10-24 (UTC),
round 5:
an arm whose trials read unusable said only that they yielded nothing.
A trial that stopped at the output-token cap measured the cap,
and one whose exchange threw measured the wire,
and neither measured the effort.
Fixed in `4f1cd29786853868de4d181e45709905af077f2f`,
red `a2f92a6063ba03682b17ae964125abe109ba3794`,
open to the owner's veto.
Each row line now ends with `; stopped` and the end of each ask of its trial:
`ok`,
`refused`,
`output-cap`,
`unparseable`,
`rejected` or `threw`,
joined by `+` when a trial asked more than once,
and `none` when it asked nothing.
The read step fails an arm in which a row states none,
an ask ended at the cap,
or an ask threw,
and warns when its unusable and refused trials together pass 4.
When every check has passed it prints for each arm an `ARM` line and a `STOPS` line,
then the verdict of the decision rule (B891).
The bench checks of B886 are part of the same step.

## B890: the process slips of round 4, for the commit comments

Status:
recorded on 2026-10-23 (UTC).

Short hashes stand in the bodies of five commits of this round,
where a full hash belongs:
`fe1949dfaeed8c6f630155e505af8625d74b2504` names
`1486b9491feff53ab24e5f9f078bb8232c49661e`,
`296648165e51c42ba2fa77dbaf9a443f7c8f38ab` and
`47d5f7edd8c3cc1e32665a647fe6ebda53bdd945`;
`bbb6991f25ee8bf4207c3a37033a4e5f6e509500` and `95f82c9dcbf0cd933b976d1d68d8da9ccf540a72` name
`9a8b0a4c1fd801c25a9ab03930b2e017800628a6`;
`9a8b0a4c1fd801c25a9ab03930b2e017800628a6` names `7c61c829b725ca88250b0ba0e0dfb2cc0b56ec48`;
and `96a6b701ec5d770b70581fb3a785c791232c6dd2` names `cd6f8380806321aff0d5632b6b689867e63189aa`.

Three red commits needed a second commit that changes only tests for lint or scan reasons
(`cd6f8380806321aff0d5632b6b689867e63189aa`,
`7c61c829b725ca88250b0ba0e0dfb2cc0b56ec48` and
`95f82c9dcbf0cd933b976d1d68d8da9ccf540a72`),
and `47d5f7edd8c3cc1e32665a647fe6ebda53bdd945` rewrote the draw's tests for the repo scans.
The source scans ran late in each of those chains,
which is why.

The fix commit `3f4aa8cde943417d9de1c2e7fefde5c3923c8b04` does not leave its red tests byte-identical to
`ff706ab047255f356582738f7999560690251a33`.
It changes the manifest-line case of `editor-fidelity-probe-run.unit.test.ts`,
because the red case expected the digest of the whole pool where the run digests the specifications it was asked to read
(`--only` narrows them),
and it moves one trailing comment in `editor-fidelity-read.unit.test.ts`.
It also adds the unit tests of the arm facts and the arm answers,
which the red commit did not hold.

Follow-up of 2026-10-24 (UTC),
round 5:
the count of three in this entry is short by one.
Four commits followed a red commit and changed tests only:
the three it names and `c4a09d7f6131e581520c00fac9f9debee95d4bea`,
which B887 lists as still red and whose message hides that it changes the expectations of the red context cases.
`47d5f7edd8c3cc1e32665a647fe6ebda53bdd945` is a fifth rewrite and stays apart.
Two more commits changed tests with no red run in front of them:
`921e578fe8c2c958c6c0078d2c1de210983d09c2` adds a case to `editor-fidelity-pool-manifest.unit.test.ts`
in the data commit that regenerates the pool,
and `fe1949dfaeed8c6f630155e505af8625d74b2504` edits three listing tests of the source scans.
The same follow-up corrects the index of the audit ledger,
which called this entry the slips of a third round.

## B891: the decision rule was prose and its threshold called a difference in over half the benches with none

Status:
fixed on 2026-10-24 (UTC) in `4f1cd29786853868de4d181e45709905af077f2f`
and `0cd4f916b68c5dc6bcc58de7b09696a6c0c77598`,
red `a2f92a6063ba03682b17ae964125abe109ba3794`,
open to the owner's veto.
The fix made the rule code with a floor taken from an exact enumeration.
Superseded on 2026-10-24 by B894 in [the paired rule and shared trials file][paired-rule]:
the floor,
the kind clause and the enumeration fixture `editor-fidelity-decide-null.test-fixture.ts` were replaced by a paired
exact test,
and that fixture and its unit test were removed in
`6404700c3457eb51a60d63bc76bbe5685aff875d`.

The rule of B887 said an effort differs from the default when its pooled clean fix count is further from the mean of the
two medium arms than the larger of 3 trials and the two medium arms' own difference.
Nothing applied it:
no function took four arms and returned a verdict,
so the verdict was the reader's.
And a floor of 3 trials was never checked against what two identical arms do.

`decideEffort` (`probe/src/corpus-run/editor-fidelity-decide.ts`) takes the clean fix counts of the four arms for each
kind and returns the verdict with the clause that decided it:
`beats-medium`,
`tie`,
`fixes-fewer`,
`inside-threshold` or `kind-veto`.
The text of the rule is `decisionRuleText` in the same file,
printed on the `RULE` line before any call,
and the read step prints the verdict from `decideEffort`,
so the text and the verdict have one source.
The gap of an effort is its pooled clean fix count minus the mean of the two medium arms.
It counts when it reaches the threshold,
the larger of the floor (12 trials) and the two medium arms' own difference.
A kind clause keeps the default effort for a gain that a kind contradicts:
a kind more than 3 trials short of the mediums' mean for that kind
(about one standard deviation of a count out of twenty).
A tie,
a gap inside the threshold,
an effort that fixes fewer,
and a vetoed gain all keep medium,
and when both efforts beat the mediums by the same gap medium is kept as a tie.
The cases are in `editor-fidelity-decide.unit.test.ts`:
a tie,
a gap of 11 and a gap of 12,
a kind dip of 1,
3 and 4 trials,
equal mediums,
and mediums that differ by 14.

The floor comes from an exact enumeration,
not a simulation
(`probe/src/corpus-run/editor-fidelity-decide-null.test-fixture.ts`,
pinned by `editor-fidelity-decide-null.unit.test.ts`,
which also checks the enumeration against a plain loop over every outcome of a pool small enough to list).
The model:
the four arms fix a trial with one true chance,
the arms are independent,
and the trials of an entry are not.
The pool draws at most one insertion and one alteration from an entry,
so its 40 trials are 24 clusters:
16 entries with both kinds,
6 with an insertion only and 2 with an alteration only.
An entry with both kinds adds 0,
1 or 2 to an arm's count,
with a within-entry correlation from 0 to 1.
The chance that at least one effort is called different is summed over every pair of medium counts,
at true rates from 0.30 to 0.95 in steps of 0.01 and correlations of 0,
0.25,
0.5,
0.75 and 1.
The smallest floor that holds the worst chance to 0.05 at each correlation is 9,
10,
11,
12 and 12;
at a correlation of 1 and a rate of 0.5 the chance is 0.58208 at a floor of 3,
0.37859 at 6,
0.15801 at 9,
0.10755 at 10,
0.06987 at 11,
0.04346 at 12 and 0.02583 at 13.
The floor is 12 because the trials of an entry are likelier to read alike than not,
and a floor that holds only for independent trials would not hold for the pool.
The 0.04346 is an upper bound:
the arms share their trials,
which makes their counts agree more than independent arms would,
and the kind clause is left out of the enumeration,
which can only turn a call into a keep.
It is not a power calculation:
12 trials of 40 is a difference of 30 points,
and a smaller real difference is not separated by this bench.

`editor-fidelity-decide-null.ts` first sat among the production modules.
The source scans found it unreached,
since only a test calls it,
so it moved to a test fixture in `0cd4f916b68c5dc6bcc58de7b09696a6c0c77598`,
and the rule text names the fixture.

## B892: the dry run with a cap of zero returned before it printed the lines that say what the arm will do

Status:
fixed on 2026-10-24 (UTC) in `4f1cd29786853868de4d181e45709905af077f2f`,
red `a2f92a6063ba03682b17ae964125abe109ba3794`,
open to the owner's veto.

`--cap 0` returned before the manifest,
pool,
ceiling and guard lines,
so a dry run showed the owner nothing of the arm it prepared.
It now reads the corpus,
prints the manifest,
pool,
ceiling,
guard and `RULE` lines,
then a line saying that nothing was called,
and exits without building a client or making a directory
(`probe/src/corpus-run/editor-fidelity-opening.ts`).
The ceiling is priced over every trial of the pool.
Over a clone that holds no corpus the run stops at the read with a nonzero status and no such line.
`WRITES THE MANIFEST, POOL, CEILING, GUARD AND RULE LINES` pins the lines,
and `READS THE CORPUS AT A CAP OF ZERO` pins the failure.
The dry runs of this round,
one for each arm as the runbook launches it,
ran with an environment holding no key.

## B893: the process slips of round 5, and the cache statement

Status:
recorded on 2026-10-24 (UTC).

Three commit bodies of this round cite a commit by its short hash:
`6a2677b200e1b3be12461b6996542f910b741bc6` names `a3a96ea1106193eab7d7983987a510b128d6ca31`,
`76e2a4e4e33698088426dd180ad88964d1dca26a` names `a2f92a6063ba03682b17ae964125abe109ba3794`,
and `0cd4f916b68c5dc6bcc58de7b09696a6c0c77598` names `4f1cd29786853868de4d181e45709905af077f2f`.

The red commit `a2f92a6063ba03682b17ae964125abe109ba3794` was not lint clean,
so `76e2a4e4e33698088426dd180ad88964d1dca26a` changed its tests for lint only between the red commit and the fix:
the red tests are not byte-identical in the fix commit,
and no assertion changed.
The source scans ran after the fix commit and not before it,
and found eight findings in it,
which `0cd4f916b68c5dc6bcc58de7b09696a6c0c77598` fixed.
Its test edits are a renamed variable,
reworded comments,
and the unit test of the enumeration importing the fixture it now tests.

The guard cases of `b15e2ed7c8c4245f92aca2c73ab13a6ea9cad70e` were red by a missing export,
since the test file did not load,
and not by an assertion.
They are shown to fail by mutation instead,
each mutant built and the probe's unit tests run:
the guard admitting an attempt without the refusal (the whole route sent 5 exchanges where the case expects 1),
a call that never left counted at its ceiling,
and a last attempt that wrote a line counted again at the ceiling.
The two mutants of the recheck are shown the same way.

Cache:
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' -- '*.ts'` over
`b4179250d44032ba7d4e75ef3bc5929f50a36d38..HEAD`,
every commit of this round,
lists none.
The same search over `c265e8df9525f4513b8229482d3e8d932ea630df~1..c265e8df9525f4513b8229482d3e8d932ea630df` lists it,
a commit known to move a cache version,
so the search can show a hit.
The bare search,
`git log --no-merges -G _CACHE_VERSION`,
over the branch since `83194090a561fd1134e96c35b7164daf46c0cb3a`,
lists `47fbac3e0b78ca5fb80292303c28fccefa8cbdf6`
and `ce3fdcdaa63848131dfdbe8bfc8a77d58d8ba50b`,
and over this round alone it lists the commits that add this entry and its follow-ups.
A bare search matches any added line that holds the string,
and each of those commits adds a ledger line that quotes it:
B885 quotes the search it ran,
B863 of main does the same,
and this entry does.
None of them touches a constant.
This round's source changes reach the Claude client (a hook it calls before an attempt),
the run route that hands the hook down,
and the probe,
which reads no cache.
They change no request body,
no reply and no key,
and nothing under `slice-cache` or `stage`.
No cache version moves.
B898 names the file of the hook this statement calls "a hook it calls before an attempt".

[paired-rule]: audit-ledger-editor-bench-paired-rule-and-trials.md
