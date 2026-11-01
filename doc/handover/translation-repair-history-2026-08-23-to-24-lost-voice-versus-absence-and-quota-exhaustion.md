# Translation repair history 2026-08-23 to 2026-08-24: a lost voice recorded as a genuine absence, and the quota behind it

The finding that a lost voice and a real absence land as the same gap,
the hourly time course that refuted the first reading,
the split of the absence reason,
and the correction that the cause was the account's own exhausted subscription limit,
with the owner's rulings on remedy.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## A lost voice and a genuine absence land as the same gap (`#198`)

FOUND 2026-08-24 by measurement,
while diagnosing why XIEPT2 was the only entry in the corpus producing unfilled slices.

`TranslateAbsenceReason`'s `no-candidate` says in its own TSDoc that it covers
"every translator was silent,
blank,
or lost its voice".
Those are three different facts.
A translator that declined is evidence about the passage;
a translator whose answer failed to parse is evidence about the hour.
Both land as the same reason,
the slice ships as a gap,
and the artifact records an absence a later reader cannot tell from a real one.

### The first reading was that XIEPT2 cannot fill its own gaps, and it was wrong

XIEPT2 had eight unfilled slices against zero across 108 slices in every other run,
which reads like a property of the hardest entry in the corpus.
It is not.

Voice loss is time-varying,
and it moved by an order of magnitude in one hour:

    xiept2-anchorfix   04:00 UTC    129 streams     4 lost     3%
                       05:00        229 streams     2 lost     1%
                       06:00       1896 streams   371 lost    20%

    recheck-ballot     05:00         14 streams     0 lost     0%
                       06:00       2023 streams   397 lost    20%

The unfilled slices track it exactly:

    XIEPT2 translate lane   05:00 hour    13 computed,   0 unfilled
                            06:00 hour     9 computed,  10 unfilled

### The cause is outside this build, and that part is proven

XIEPT2 has been running since 01:20 on a build loaded at launch,
and a rebuild does not change a running pass,
so none of the five commits landed today can have caused its 3 to 1 to 20 percent.
It is a frozen-code control that ran straight through the boundary.

Two passes on DIFFERENT builds and DIFFERENT entries
cross into 20 percent in the same wall-clock hour.
Loss is spread evenly across all six models,
59 to 62 each,
so no single model explains it either,
and for this window it supersedes `#77`'s finding that Kimi-K3 dominates.

WHAT IS NOT ESTABLISHED is which external thing it is.
Provider,
network and this host are all consistent with the evidence gathered.
The claim that stands is narrower and sufficient:
it is not our code.

### Why this is a release concern rather than a bad afternoon

An entry settled during a bad hour publishes a page with holes,
the artifact calls them absences,
`pageSilent` counts them,
and nothing anywhere says they were an artefact of the hour.
Ten of XIEPT2's slices are in exactly that state as this is written.

Three ways out are recorded on `#198`,
unranked pending a decision:
splitting the reason so the artifact records which of the three happened,
refusing to settle an entry whose gaps came from lost voices,
and re-asking a slate whose voices were all lost rather than declined.
Splitting the reason is a prerequisite for the other two,
since neither can act on a distinction the pipeline does not record.

### One operational consequence, taken immediately

`recheck-ballot-vub-2026-08-24` was stopped.
It had been launched to close `#192` and `#193`,
which `GLaDOSister` closed first,
so it was redundant before the degradation made its output untrustworthy as well.
`xiept2-anchorfix` was left running:
its purpose is the `#194` live boundary check,
which is about whether the publisher handles a silent anchor correctly,
and a degraded hour gives it MORE anchors to handle rather than fewer.
Its output is a test fixture,
not a deliverable.

Any quality measurement taken in this window is contaminated,
including anything `#196` would conclude about the per-entry cap.

### The reason is split, and the fact it splits on was already there

Landed 2026-08-24 in `5a482a74f`.

`no-voice-heard` joins `TranslateAbsenceReason` and `TranslateDecision`.
`SLATE_TERMINALS` gives it the same terminal as `no-candidate`,
because what the two tell a reader differs
while what they tell the consolidation does not:
neither reached a judge.
That record is typed `Record<TranslateDecision, ConsolidationTerminal>`
precisely so a new decision fails to typecheck instead of falling into whichever branch is last,
and it worked:
the choice had to be made rather than defaulted.

THE DISTINGUISHING FACT WAS ALREADY AT THE EXIT THAT COLLAPSED IT.
`ProducedSlate.heardTranslators` has been carried since the produce and judge halves were split,
for the reason its own TSDoc gives:
a decision taken over a thin slate is not the same decision,
and the judging half has no other way to know.
Zero heard is the whole test.
This is the fourth defect this session
where the information needed was present and discarded one line before it could be recorded.

### A test was pinning the conflation

The document case for an unfillable passage
builds its fixture with `silentForSource`,
which the harness documents as
"original whose slice every translator fails on",
"standing in for a provider that is down while the signal stays live".
It asserted `no-candidate`.

So the codebase's own fixture for a dead provider
was landing on the reason that means the passage is hard.
It now asserts `no-voice-heard` and its name says why.

### GFP, in both directions, because one direction would prove nothing

    always no-candidate    (the old behaviour)   exit 1, 4 cases fail
    always no-voice-heard  (the opposite)        exit 1, 2 cases fail
    restored                                     exit 0, 575 PASS

The new judge-level cases are a PAIR on purpose.
A single reason covering both states satisfies either case alone,
so only both together show a distinction is drawn rather than a constant returned,
and the two mutations are what demonstrate that.
Their client throws if anything asks it a question,
since an empty slate must buy no judging round.

### What is left on `#198`

Splitting the reason was the prerequisite.
Refusing to settle an entry whose gaps came from lost voices,
and re-asking a slate whose voices were all lost,
both remain open,
and both need a measurement that CANNOT be taken in the current window:
anything measured about re-ask effectiveness during a 20 percent hour
measures the hour rather than the remedy.

### Correction: the cause WAS established, and it was our own quota

Two claims recorded earlier under `#198` are wrong and are retracted here.

RETRACTED:
"WHAT IS NOT ESTABLISHED is which external thing it is.
Provider,
network and this host are all consistent with the evidence gathered."

The cause was established by reading the response bodies.
866 of 875 lost voices carried:

```text
SyntheticHttpError HTTP 429: {"error":"You've exceeded your subscription rate limits. Upgrade, or try again later..."}
```

`SyntheticHttpError` in that output is the class renamed `ProviderHttpError` on 2026-10-19.

Three were content-based and three were nginx 503.
It was not provider degradation and not the network.
It was this account's own Synthetic budget running out,
and the owner corrected the reading of it further:

> "own subscription rate limit being hit due to concurrency" - it's not concurrency.
> It's "remaining weekly credits".

That distinction decides the remedy.
A longer backoff cannot help,
because retrying an exhausted weekly budget never succeeds.
Failover to a second provider can.

RETRACTED:
that the two remaining `#198` items are blocked on a clean measurement window.
The owner rejected the premise outright:

> "are blocked on a clean window" - no,
> provider issues are normal and expected,
> our pipeline should be resilient enough to not fail even when the provider is having issues.

Refusing to settle is therefore itself a failure mode rather than a way out,
and waiting for quiet weather is not a plan.
The remedy is `#199`:
a second provider,
and the quota reader that has been built and unwired since 2026-07-16.

DIAGNOSIS DISCIPLINE,
since this went wrong twice in one session.
The first reading was that XIEPT2 could not fill its own gaps,
refuted by the hourly time course.
The second was that the provider was degrading,
refuted by the 429 body quoted here.
Both were stated before anyone read a response body.
The body was one tool call away the entire time.
