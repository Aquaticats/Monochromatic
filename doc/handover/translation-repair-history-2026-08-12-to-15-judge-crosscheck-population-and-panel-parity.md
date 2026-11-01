# Translation repair history 2026-08-12 to 2026-08-15: the judge crosscheck, its population and the panel-parity ceiling

Enumerating the claims the judge crosscheck could use,
the join and attribution details the measurement settled,
the bound on how many verdicts a single removed vote can flip,
and the finding that panel parity caps the whole measurement at about three percent,
which is why it was stopped.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The judge crosscheck has a population, and its control arm survives contact

Session of 2026-08-13,
late.
`#31` had a design and a seating primitive and no
 idea whether the run could support the measurement.
It can,
and enumerating it
 cost no quota.

### What the enumeration measured

Over the 20 entries settled in `translation-repair-runs-pass13` at the time:

```text
judgeable claims 371   accepted arm 189   control arm 182
control by status  rejected 97   needs-human 85
entries carrying attribution 6 of 20
join failures 0
```

Per author,
claims proposed and which arms can carry a rate against the
 provisional `MIN_JUDGED_CLAIMS` floor of 30:

```text
Kimi-K3          accepted 44  control 26   accepted only
Nemotron-3-Super accepted 41  control 41   both
GLM-5.2          accepted 41  control 39   both
Qwen3.6-27B      accepted 32  control 22   accepted only
gpt-oss-120b     accepted 21  control 31   control only
GLM-4.7-Flash    accepted 11  control 23   neither
```

Run it with
 `mise run //package/module/translation-repair:score-crosscheck` (moved on 2026-10-18 to
 `//package/translation-repair/probe:score-crosscheck`),
and note that
 the task reads `resolveRunsDir()`,
which defaults to
 `translation-repair-runs`,
NOT the current pass.
Point it at the right run with
 `TRANSLATION_REPAIR_RUNS_DIR`.
Run bare against the default it reported 56
 entries,
none carrying attribution,
and every count zero.
That output is not
 wrong,
it is a reading of a different run,
and it looks identical to a run
 with nothing to say.

### Three things the measurement settled that guessing would have got wrong

Rejected claims DO carry proposers.
Attribution is collected at the critic
 stage,
and `retainAttributions` in `chunk-critic-phase.ts` only drops claims
 the deterministic screen killed,
never claims the panel later refused.
So both
 arms seat judges by the identical rule and the control needs no fallback.
Had
 it needed one,
the control would have been seated from the full roster while
 the accepted arm was seated from five of six,
and the two arms would not have
 been comparable at all.

Sole authorship covers essentially every claim,
298 of 299 at the first
 snapshot.
That is not critics failing to agree;
it is what the claim id is.
 `computeIssueClaimId` hashes category,
severity,
summary and every span offset
 and quoted string,
so two critics who spot the same defect in the same words
 still produce different ids unless their summaries match character for
 character.
The practical consequence is good for seating,
since five of six
 models are free to judge almost anything,
and bad for any reading of
 "corroboration" measured at id level.

Only 6 of 20 entries carry attribution,
because it landed partway through the
 pass.
That is why the census covers 371 claims against a far larger issue
 population,
and it will fix itself as entries settle.

### The join key is computed, not stored

The artifact nests claim ids at `issues[].issue.claims[].claimId`,
and the
 issue record itself carries no claim id.
An issue GROUPS several claims,
 because deduplication merges claims naming one defect,
and `status` lives on
 the issue while attribution is per claim.
A first attempt joined on
 `issue.claimId` and `issue.id`,
both absent,
and reported 0 of 299 attributed
 claims matching an accepted issue.
That reads exactly like a broken pipeline
 and was a broken query.
`QRY` covers this:
a search result claims the search
 ran,
and a join result claims the key existed.

### Unattributed claims are two different things and were one number

The first version counted every claim an issue named that attribution did not
 cover,
and reported 1368.
Nearly all of those sit on the 14 entries that
 predate attribution,
which is expected absence.
A claim missing on an entry
 that DOES carry attribution is something else entirely:
the two records
 disagreeing about claim identity.
Folded into one number,
a broken join would
 have been invisible inside an expected 1368.
Split,
the join failures read
 zero,
which is now evidence rather than silence.

### What the crosscheck can and cannot establish, restated

It can bar a claim's authors.
It cannot bar its adjudicators,
because
 `RUN_MODELS` seats the same six models as critics,
panel and judges and the
 provider serves no seventh.
It measures whether a verdict survives being
 re-asked with the author removed.
It is not precision.
The report prints that
 sentence itself,
so a reader who never opens this document still gets it.

The accepted arm alone would mean nothing:
a roster answering `supported` to
 everything scores identically to one reading carefully.
The finding is the gap
 between the arms.
Whether that gap is confounded,
since a claim is in the
 accepted arm precisely because the panel accepted it,
is the open question a
 sol review is currently chewing on.

### Still open

The judging pass itself.
It needs quota,
and the corpus pass contends for the
 same per-model slots,
so it waits.
`MIN_JUDGED_CLAIMS = 30` is now documented
 as a provisional guard rather than a calibrated threshold,
unlike
 `LOSS_FRACTION_LIMIT`,
which was fitted on 50 human-graded repairs.

### The crosscheck's headroom is bounded at 10%, computed without a single call

Three checks after the enumeration landed,
each of which changed the design.

FIRST,
the panel does not bar proposers,
and that is deliberate.
 `adjudicate-model.ts` states it:
panelists judge each claim strictly on
 document evidence,
they never learn which model proposed what,
and the
 electorate is fixed up front,
because a variable electorate of non-proposers
 was found to shrink consensus.
So a claim's author DID vote on its own claim,
 blind.
That makes "does the verdict survive removing the author" a real
 question rather than a no-op,
which is what the crosscheck needed to be worth
 running at all.

It also means the crosscheck deliberately does the thing the architecture
 rejected.
That is fine for a measurement,
which changes no pipeline behaviour,
 but a reader must not take a crosscheck result as a recommendation to seat a
 non-proposer electorate.
The settled decision already weighed that and went the
 other way.

SECOND,
and decisively:
the recorded tallies bound what the whole measurement
 can find.
A claim's plurality can only change when a single removed vote closes
 the gap,
which needs a margin of one or less.

```text
accepted claims 1027   could flip if one vote is removed 105   10.2%
rejected claims  440   could flip                         73   16.6%
```

Roughly nine in ten accepted claims are decided by a margin of two or more and
 cannot move no matter how their author voted.
That is an upper bound and the
 worst case,
since it assumes the author voted with the plurality every time.

The consequence is a much better run than the one planned.
Rather than judging
 all 371 claims,
target the near-ties:
the 105 accepted and 73 rejected claims
 where the electorate actually decides anything.
They are also the closest thing
 to a MATCHED pair the data holds,
since near-tie claims on both sides of the
 accept line are similar in difficulty by construction and differ in outcome.
 That answers the confounding objection to the two arms without matching on
 severity or category,
and it costs half the calls.

THIRD,
arm assignment currently reads `issue.status`,
which is per ISSUE,
while
 the panel votes per CLAIM.
Deduplication merges claims naming one defect,
so
 201 of 1258 issue records hold several claims.
Measured,
the two disagree:

```text
accepted -> supported     1018
rejected -> unsupported    440
needs-human -> supported   228
needs-human -> unsupported  23
accepted -> unsupported      9
source-defect -> sourceDefect 1
```

Nine claims sit inside an accepted issue carrying a plurality the panel never
 gave them.
Small,
and free to fix by reading the arm from `tallies[claimId]`
 and keeping `issue.status` as a separate field.

The `needs-human` split is the bigger one.
Those 228 claims lean SUPPORTED,
and
 the current census files all 182 non-accepted claims into one control arm on
 the grounds that the arm wants claims the panel did not accept rather than a
 particular reason.
That is wrong:
`rejected` means the panel decided against,
 `needs-human` means it declined to decide,
and agreement is undefined against a
 verdict never given.
Score `rejected` as the control and report `needs-human`
 separately.

None of these three needed a model call.
Two needed a grep and one needed a
 fold over artifacts already on disk.

## Panel parity decides the crosscheck, and it decides against running it

The judge crosscheck of `#31` was re-scoped onto near-tie claims,
the ones where
 removing a single vote could change the plurality.
Counting them as entries
 accumulated produced a result that looked like slow progress and was actually a
 structural ceiling.

### The near-tie rate did not drift, it split by pipeline tip

```text
tip cf68fdd51   2 entries    18 near-ties of  66 claims   27%
tip 9cacc3f02   5 entries     5 near-ties of 253 claims    2%
```

An earlier projection of roughly four near-ties per entry came from averaging
 across that boundary.
It is not a rate;
it is two different regimes.

### The cause is the size of the panel, not its decisiveness

The first guess was that fuller panels produce bigger margins.
Measured,
that is
 false:
mean voters per claim rose from 5.54 to 6.00 while the mean margin FELL
 from 3.00 to 2.71.
What changed is parity.

```text
voters 5   claims  383   margins 0:5   1:169 2:3   3:115 5:91          flippable 45.4%
voters 6   claims 1372   margins 0:198 1:44  2:352 3:31  4:408 6:339   flippable 17.6%
```

With five voters a two-way split lands on an ODD margin,
and margin 1 is the
 single most common outcome,
169 of 383.
With six it lands on an EVEN margin,
so
 margin 1 falls to 44 of 1372,
and the margin-0 cases are exact ties that the
 panel files as `needs-human`,
which both scored arms already exclude.

So on scored claims under a full six-model panel,
the share a single removed
 vote could flip is about 3%.

### What that means for `#31`

The crosscheck asks whether a verdict survives re-asking with its author
 removed.
Under the current panel it cannot change more than roughly three
 verdicts in a hundred,
and reaching a reportable per-arm rate would need most
 of the remaining corpus.
Spending on the order of a thousand calls to bound a
 3% effect is a bad trade,
and the number it produced would be dominated by the
 parity of whichever panels happened to be full.

Recorded as the reason to stop rather than as a reason to wait.

### The finding worth keeping

Voice loss does not merely remove a voice.
It changes the PARITY of the
 electorate,
and parity changes the shape of the verdict distribution:
a
 five-voter panel produces margin-1 verdicts 44% of the time,
a six-voter panel
 3%.
Anything that reads margins,
including any future confidence weighting or
 gating,
is reading panel size as much as panel opinion.
That is a live property
 of the pipeline and nothing currently accounts for it.
