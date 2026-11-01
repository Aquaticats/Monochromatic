# Translation repair open decisions: what changed night by night, 2026-08-15 to 2026-08-19

The dated notes that told the owner what moved between readings of the questions,
newest first from 2026-08-19 back to the second night,
then the long list of what landed on 2026-08-15 and 2026-08-16 after the first writing.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## What `#121` added on 2026-08-19

WHAT `#121` ADDED,
08-19.
Two new questions,
9 and 10,
and one correction that needed no question
at all.

-   THE IDLE WINDOWS' STATED REASON WAS WRONG,
    THE VALUES WEREN'T.
    The 95.6 s median
    `STREAM_FIRST_BYTE_MS` and `STREAM_IDLE_MS` were disabled against is superseded 77 to 93 times
    by first production traffic,
    but the tail still holds completed calls at 183755 ms,
    124992 ms,
    and (three weeks earlier,
    a different roster) 347099 ms,
    the last within 3.6 percent of the
    360000 ms deadline.
    Both constants stay at 600000;
    only their TSDoc changed,
    in
    `doc/decision/translation-repair-runaway-call-termination.md` and commit `3893825b2`.
    This
    needed no question because the evidence pointed one way:
    nothing on record would have let a
    tighter window avoid killing a healthy call.
-   THE STRAGGLER GRACE'S CUT POPULATION IS COUNTED TO THE END OF THE PASS:
    19 cuts,
    not the
    audit's mid-flight 10,
    sixteen of them `hf:zai-org/GLM-5.2` and two newly `hf:Qwen/Qwen3.6-27B`.
    What to do about that concentration is Question 9.
-   WHETHER TO ACTIVELY RE-ARM THE IDLE WINDOWS BELOW 600000,
    trading the deadline's six-minute
    ceiling for something faster on a genuinely dead call,
    is Question 10.
    The measurement above
    settles the premise;
    it does not settle how much of the documented tail risk is worth paying
    against.

## What the fourth night changed, 2026-08-17

WHAT A FOURTH NIGHT CHANGED,
08-17.
One new question,
Question 8,
and one gate that is no longer a gate.
Nothing
below was answered in between and nothing changed its answer.

-   THE WINDOW TRIAL FINISHED,
    all 327 arms,
    zero refusals.
    Widening the judge's
    context did NOT move selection:
    over the 109 slices it read,
    the wide arm
    decided differently from a narrow arm 18 times and the two identically
    configured narrow arms differed 21 times,
    so the effect is smaller than the
    noise it was measured against.
    The result and the pre-registered reading it
    fired are in `doc/planning/translation-repair-window-trial.md`.
-   ITS NEGATIVE CONTROL MEASURED SOMETHING NOBODY ASKED FOR,
    and it constrains
    how any single pass may be read:
    the per-slice preserve-or-replace decision
    is about 19 percent unstable between identical runs.
    Rates over many slices
    survive that;
    per-slice claims and small comparisons do not.
-   THE LAUNCH GATE IS LIFTED.
    The corpus pass was held only because the trial
    was competing for the same six models,
    and it is done.
-   THE RENDERING AUDIT WORKS,
    and Question 8 asks where it should run.
    Both
    live arms ran three times:
    the planted defect was found in every run,
    by all
    three auditors in two of them and by two of three in the last,
    where the
    remaining voice's only claim failed to anchor.
    The faithful control drew no
    claims at all,
    in any run.
-   QUOTA WAS SPENT,
    deliberately and in small amounts:
    three runs of two audit
    arms against a three-model roster.
    Everything else was unit tests against
    the synthetic client.

## What the third night changed, 2026-08-16 into 2026-08-17

WHAT A THIRD NIGHT CHANGED,
08-16 into 08-17.
None of the questions below were answered in between and none of them changed
its answer:
the night went on the artifact reader,
two resume guards and the
last of the invariants review,
so this is a note about what is now built rather
than about anything you have to re-decide.

-   THE VERSION 2 ARTIFACT PARSER IS IN,
    and with it a refusal the adversarial
    list called for and I had dropped:
    a ledger naming one slice twice used to
    pass every relation,
    because they all join by POSITION,
    so two rows naming
    slice 5 agree with each other,
    with their evidence,
    and with the slice
    count.
-   TWO RESUME GUARDS,
    one of which corrects a claim this repository made in
    three places.
    The mixed-generation trap was recorded as unguarded;
    it was
    not.
    `assertResumableGeneration` already refuses a directory whose artifacts
    record another pipeline digest,
    and a build that writes version 1 cannot
    share a digest with one that writes version 2.
    What was genuinely open is
    the drift opt-in,
    which promises that naming a required commit keeps a mixed
    pool readable:
    true of build drift,
    false of schema drift.
    That hole is
    closed and the new guard is deliberately NOT overridable.
-   `#103` IS FULLY CLOSED,
    and two of its three remaining items turned out to
    be built already.
    Re-reading them cost less than rebuilding them.
-   NOTHING SPENT YOUR QUOTA.
    The only runs were plan-mode,
    which makes no model
    call,
    and unit tests against the synthetic client.
-   `#105`'S FIRST ITEM,
    the decline-rate measurement that gates its decision,
    still cannot run:
    it wants the real roster,
    and the launch gate holds while
    the window trial is live.

## What the second night changed, 2026-08-16

WHAT A SECOND NIGHT CHANGED,
08-16,
since none of these questions were answered
in between and you should not re-read the whole document to find it:

-   QUESTION 5'S OPTION E GOT DEARER,
    and it is the top-ranked option.
    The
    production re-run it calls for cannot be a two-arm comparison;
    that design
    measures the window and a resampled candidate slate at once.
    It is three
    arms now,
    with the slate held fixed,
    which needs a refactor first.
    Priced in
    option E.
    The RANKING did not move.
-   NOTHING ELSE HERE CHANGED ITS ANSWER.
    The rest of the night went on making
    the instrument honest rather than on new findings:
    a window index that
    silently returned "no window" now throws,
    the cache key that would have
    made both arms of that comparison identical is fixed and pinned,
    and the
    hop that carries the window to the judges is now a test that was shown to
    fail without it.
-   ONE THING I GOT WRONG AND FIXED,
    recorded because it touched your
    repository.
    Six comments about this work were posted onto unrelated GitHub
    issues:
    the task-tracker ids in this document collide with GitHub issue
    numbers,
    and `#84`,
    `#106`,
    `#107` and `#108` are real tickets about other
    things.
    The comments have been rewritten in place as misfiled notes,
    no
    content was lost,
    and `AGENTS.md` now carries the rule that would have
    prevented it.

## What landed after the first writing, 2026-08-15 to 2026-08-16

WHAT LANDED AFTER THIS DOCUMENT WAS FIRST WRITTEN,
so you are not reading
yesterday's state:

-   Both lanes now run from one driver that arbitrates nothing,
    and each lane
    result names the slices it shipped and withdrew rather than only counting
    them.
    Decisions 16 and 17,
    both open to veto.
-   The bench prices sending and answering separately now.
    Decision 18.
-   QUESTION 1'S COST BULLET CHANGED,
    because the same bench rows were re-read
    per stage at no new cost and the previous reading was half wrong.
    Read that
    bullet again if you answer Question 1 from cost.
-   THE ASSEMBLY CONTRACT WAS REBUILT,
    in decisions 20,
    21 and 22.
    Three things
    changed that a reader of a count needs to know.
    A contradictory cached slice
    is now discarded and recomputed rather than reaching assembly,
    so one bad
    file costs one slice.
    An assembly that changes no byte now ships nothing,
    whatever its slices decided.
    And the shipped set is DERIVED from the
    surviving replacements and checked against the document's own bytes rather
    than accepted as a separate list.
-   WHAT `withdrawn` MEANS IS WIDER THAN IT WAS.
    It used to be an integrity veto
    and nothing else.
    It now says the document does not carry that slice's
    change,
    for any of three reasons,
    and only the findings say which.
    If you
    were going to read `withdrawnSliceCount` as footnote damage,
    do not;
    it
    would over-count.
-   ONE SUSPECTED DEFECT WAS REFUTED RATHER THAN FIXED,
    `#97`.
    The claim was
    that a checker verdict recorded as `resolved` might describe pre-refinement
    text.
    A checker DOES run after refinement,
    and any issue it fails to
    re-confirm rolls back the whole slice.
    Measured over the 56 settled
    artifacts:
    32 resolved-and-refined slices against 32 re-check findings,
    zero
    violations,
    with the probe validated first.
    Nothing to build.
-   THE EMPTY-ROSTER REFUSAL WAS BUILT for every role except the critics.
    A lane
    configured with nobody in a required role now refuses before buying
    anything,
    at all three depths a caller can enter at.
    The quiet path is still
    right for OUTAGES and is untouched;
    what is refused is the deterministic
    case,
    before any work is done.
    CRITICS ARE STILL UNGUARDED,
    and that is
    Question 3's doing rather than an oversight:
    if the critic stage survives,
    an empty critic roster is a misconfiguration and belongs in the same check,
    and if it does not survive,
    an empty critic roster is the intended
    configuration.
    A second review sharpened the cost of leaving it open:
    the
    stage RUNS today,
    so an empty critic roster today produces exactly the quiet
    vacuous pass this check exists to refuse.
    Task `#93` carries the remainder.
-   THE ASSEMBLY AND SLICE-RECORD CONTRACTS WERE HARDENED AGAIN overnight on the
    15th,
    after a second review of the same code.
    Three of these change numbers
    you may read,
    so they are worth knowing before you read any:
    -   A NATURALNESS REFINEMENT THAT LANDS BACK ON THE ARCHIVE WORDING is now
        recorded as unchanged,
        and its slice's resolved-issue credit is dropped.
        The rewriter is measured against the accuracy text it rewrites,
        so it
        can move off that text right back onto the archive's own words;
        that was
        being recorded as a change,
        which would have named the slice in the
        shipped set and then failed the whole document at assembly.
    -   A FRESHLY SETTLED SLICE RECORD is now checked against its own text
        before it is cached,
        on both lanes,
        exactly as a resumed one is.
        Only
        one direction of that contradiction was ever caught downstream:
        a record
        claiming a change it did not make is refused at assembly,
        while one
        DENYING a change it made was dropped in silence.
    -   REFINEMENT NOW HONORS THE ABORT.
        It had none:
        a torn-down exchange
        surfaced as whichever stage happened to fail,
        and a phase that settled
        under an abort returned a document that read as a finished run.
        A fully
        cached document still finishes,
        because what a stopped run cannot do is
        buy what it is missing.
-   A REPO-WIDE LINT GAP was measured while doing that,
    and it is a question for
    you rather than a defect:
    `doc/planning/unused-import-lint-policy.md`.
    It
    blocks nothing.
-   THE EVENING OF THE 15TH went to the cache-key review and to the first two
    landings of one-sided slicing.
    NONE OF IT ADDS A QUESTION,
    which is why it
    is a list rather than a section:
    every choice in it was settled by a review,
    by your ruling on the cache keys,
    or by the design already recorded in
    `#100`.
    Decisions 23 to 25 carry the reasoning,
    and what changed is:
    -   A SLICE'S CACHE KEY NO LONGER NAMES ITS POSITION,
        on both lanes,
        which
        is the change you authorized to land while the window is free.
        A record
        is now keyed by what was asked,
        and the answer to that question does not
        depend on where in the document it was asked;
        a resumed record is
        re-stamped with the index it is resumed under.
        Both caches were measured
        empty first,
        so nothing was discarded.
    -   A REFUSAL THAT NAMES A SLICE BY NUMBER IS NO LONGER STORED.
        It was the
        one thing in a stored record that did depend on position,
        so it would
        have survived the key change and reported the wrong slice number after a
        resume.
        The number is now derived at the document level,
        where the
        positions are known.
    -   A COLD RUN AND A WARM ONE NOW BUY THE SAME THING.
        Both lanes memoized
        every settled slice in process,
        including ones the cache deliberately
        refused to persist,
        so a document with two identical sections bought one
        of them on a cold run and two on a resumed one,
        and no count said so.
    -   A CHUNK CAN NOW NAME A PLACE rather than only cover text,
        and assembly
        refuses every placement shape that would move or overwrite existing
        wording.
        Nothing produces such an anchor yet,
        deliberately:
        `#100` lands
        the producers last,
        so by the time a section with no translation is
        sliced,
        the assembly it flows into already refuses what it must.
    -   THE BLANK LINE BETWEEN TWO BLOCKS HAS AN OWNER,
        which is assembly rather
        than the prompt.
        This closes `#101`.
    -   THE LANES CAN NOW BE HANDED A SLICE THE ARCHIVE NEVER TRANSLATED,
        which
        is the third landing of `#100`.
        The translate lane refuses to settle one
        it could not fill rather than reporting a settled slice carrying the
        empty string,
        and the repair lane says the question does not apply
        rather than sending critics to complain about a blank.
        Decisions 26 and
        27 carry the two choices in it that are yours to veto;
        nothing produces
        such a slice yet,
        so no run changes today.
-   THE NIGHT OF THE 15TH INTO THE 16TH went to the quote anchoring chain and to
    building `#84`'s instrument.
    One item here changes a number you may read and
    one adds a question,
    so both are stated plainly:
    -   QUOTE ANCHORING COULD ANCHOR A CRITIC'S QUOTE TO THE WRONG OCCURRENCE.
        Ambiguity was judged inside each matching pass rather than over the
        broadest form the chain accepts,
        so a quote appearing twice under
        whitespace folding could still bind to the first.
        The three passes are
        now one,
        over one normalization.
        MEASURED COST:
        zero.
        Across three
        corpus passes,
        16,479 real critic quotes,
        not one became ambiguous under
        the stricter rule,
        and a positive control proved the probe could see
        ambiguity when it exists (566 hits when the same needles were searched
        over whole pages).
        The slice cache version moved for it,
        since the
        change alters which claims survive while every keyed input stays
        identical.
    -   `#84` NOW HAS AN INSTRUMENT AND A FIRST READING,
        which Question 5 carries
        in full.
        The short version:
        over four entries the judges chose the
        complete text in all sixteen trials,
        which rules out keeping what they
        were handed and preferring a ballot position,
        and CANNOT rule out
        preferring the longer text,
        because a deleted sentence makes the
        complete candidate the longer one every time.
        An insertion fixture,
        whose
        correct answer is the shorter candidate,
        is built and running.
    -   TWO OF SIX JUDGES ABSTAINED ON ELEVEN OF SIXTEEN of those trials,
        both
        refusing the pair because the archive's own handle romanisation violates
        the sheet's exact-names criterion.
        That is worth knowing before Question
        6,
        since it means the effective roster on such slices is four voices.
