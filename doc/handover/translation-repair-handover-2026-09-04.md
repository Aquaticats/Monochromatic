# Translation repair handover snapshot: 2026-09-04

Part of the [current translation repair handover](translation-repair.md).

The day's work was making the legacy slice pipeline production ready with OpenRouter as the paid per-token fallback,
under the owner's direction decision of 2026-09-01 and the OpenRouter order of 2026-09-03.
Three entries shipped and were read.
Four defect classes were found and fixed,
all four in the pipeline's own screens rather than in the corpus.
The pipeline is still not production ready,
and the reason is recorded in
[`translation-repair-readiness-signal.md`](../planning/translation-repair-readiness-signal.md).

## Where the work stopped

The OpenRouter balance ran out at 16:15 UTC,
part way through the last entry's consolidation.
It reads about 0.36 USD.
The owner said a top-up was attempted,
did not go through,
and will be retried,
then stopped for the day.
No run is in flight.
Nothing in the working tree is uncommitted.

The full unit suite emitted 935 `PASS` lines and zero `FAIL` lines on the final build.
Lint and the type check are clean.

## Repository state

- Worktree:
  `/var/home/user/worktrees/translation-repair`.
- Branch:
  `translation-repair-rebased`,
  auto-push on.
- Tip:
  `1ff4cb31d`.
- Corpus pinned at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379` in `~/one-among-us/data`.
- The finite-prototype branch `prototype/translation-repair-finite-pipelines` stays unmerged.

## What landed today

The commits are listed newest first,
with the class each answers.

-   `1ff4cb31d` names each excluded lane and its deterministic finding on the lane contest's
    eligibility-floor log line.
    The Uekawakuyuurei run logged only `lane-contest-eligibility-floor (inadmissible choices excluded)`,
    and the reason lived in the slice cache.
-   `d6ffc4812` confirms a textless picture from the readers instead of stopping the entry,
    with `20ea56cd0` logging the opening of a refused reading so the wording is on record.
-   `d5e4d1ea0` names 那些秋叶 and the corner brackets in the house rules.
-   `d98e656cb` and `cd5288fa9` render the corpus's neutral pronoun `TA`,
    `Ta` or `ta` as singular they,
    and refuse a translation that leaves it standing.
-   `1c9663666` and `8d7b151ef` let the publisher accept the archive's rendering of a source
    destination,
    which is the document-level half of the either-rendering rule.
-   `82888d43b` owes one rendering where the archive rendered a reference another way,
    which is the per-slice half.
-   `ae1d2b55f` and `6bfe6da56` stop an entry on an ineligible standing rather than reattempting it,
    and accept an alias that carries the name among other renderings.
-   `6a1b4262e` stops starting entries once a run has spent its OpenRouter allowance.
-   `d55d83082` ignores measured-degraded OpenRouter endpoints,
    spelled as the gateway lists them.

Every fix above carries a guard shown to fail with its rule neutralised and to pass with it restored.

## The four classes the readings found

Each was found by reading a shipped page or a stopped run,
not by a test.

1.  **The publisher read the source alone.**
    The slice rule accepted the archive's `x.com` where the source carries `twitter.com`,
    and `DroppedDestinationError` then refused the assembled page 56 minutes and 4.24 USD into the run.
    The decision record's clause saying the publisher was unchanged was the defect.
    Fixed by giving the whole page the same pool the slices use.
2.  **The neutral pronoun reached a page untranslated.**
    `SS3B_0016` shipped "a small room for Ta" twice on a page that says they everywhere else,
    endorsed by three of five contest ballots as "the original's neutral Ta".
    The counter read only `TA` while the corpus writes `Ta` in seven sources and `ta` in eight,
    and the house rule never said what English makes of it.
    Sixteen of the seventeen archives that face it write they.
3.  **A textless picture stopped its entry.**
    A painting's canvas noise clears the deterministic reader's presence gate,
    every model then reports truthfully that the picture carries no text,
    and each such reply was screened as a refusal and re-asked four times.
    Three entries were unshippable on any roster for this reason.
    An absence report is now told apart from an inability,
    and two of them confirm the picture textless.
4.  **The site's own name was decided from the archive alone.**
    `SS3B_0016`'s consolidation gate kept "One Among Us" for 那些秋叶 by four ballots to three,
    arguing from the archive rather than from a rule,
    and shipped it inside the source's corner brackets.
    Both halves are house rules now.

## The three pages that shipped

All three verify with `verify-published`,
and all three were read line by line.

-   `luxuanwen3`,
    60 minutes,
    4.65 USD,
    764 calls.
    Two destinations carried in the original's rendering,
    front matter alias reading `鲵鲵, Nini`,
    the `PhotoScroll` line and the Camus attribution intact.
-   `SS3B_0016`,
    56 minutes,
    5.12 USD,
    887 calls.
    Three destinations,
    the archive's English Wikipedia rendering accepted and named by `destinations-archive-rendering`.
    Read before the pronoun and house-rule fixes,
    so its page carries the bare `Ta` that found class two.
-   `Uekawakuyuurei`,
    53 minutes,
    3.84 USD,
    524 calls.
    Four destinations,
    three pictures confirmed textless on the first ask,
    they or them or their twenty-one times for the subject the source calls `ta`,
    no bare pronoun left.
    Its consolidation ran starved for the last ten minutes.

The readings are in
[`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md),
one section per run.

## What the every-provider-dry path did, live

This was its first live exercise.
From 16:15 UTC OpenRouter refused calls while the credits meter still read 1.12 USD and `wet`,
so the meter is not the authority on whether a call will be sold.
`EveryProviderDryError` was raised 166 times through the gate and refiner rounds,
205 voices were lost,
and the run neither held nor errored:
every consolidation slice shipped its lane-contest winner unpolished,
and the entry settled.
Issue 474's hold branch remains unexercised,
since holding is what did not happen.

## How a pass is launched and read

The launch recipe is not repeated here.
[`translation-repair-corpus-pass.md`](../runbook/translation-repair-corpus-pass.md)
carries the exact invocation,
including `TRANSLATION_REPAIR_RUNS_DIR`,
the overlap and writer-grace overrides,
and the `env -u` form that leaves a provider out of a run.
Every pass today used that form with both Synthetic and Charm Hyper unset,
so OpenRouter served alone.

ALWAYS KILL AND RELAUNCH,
the owner's rule of 2026-09-06,
written after this snapshot:
when source changes while a pass is running,
kill the pass by pid,
build,
and relaunch the same entry into a fresh runs dir on the new build.
The relaunches of BI4PBV and Uekawakuyuurei recorded here were that practice before it was a rule;
a page finished on a superseded build is not readiness evidence.

READING A PAGE is the gate the owner set on 2026-08-26,
and it is not the same as the pass reporting `SETTLED`.
What the three readings of 2026-09-04 did,
in order:

1.  Run `verify-published` against the run directory.
    It answers whether the page carries every wording its artifact promised,
    and nothing about whether those wordings are right.
2.  Read the shipped page beside the source and the archive,
    all three open.
    The page is `<runs dir>/fixed/people/<id>/page.en.md`;
    the source and the archive are `page.md` and `page.en.md` in the pinned corpus.
3.  Check what the deterministic rules cover,
    which is where a defect is cheap to confirm:
    every link destination,
    the front matter's shape and its declared name and alias,
    and any component line such as `PhotoScroll` or `ChannelBackupButton`.
4.  Read the prose for what no gate sees.
    That is where all four of today's classes were found:
    a pronoun left untranslated,
    a name settled from the archive,
    punctuation carried across,
    a picture reported textless and refused.
5.  Grep the run log for the refusal vocabulary:
    `fails the deterministic`,
    `lacks contest endorsement`,
    `contest winner fails`,
    `must carry exactly`,
    `StandingIneligible`,
    `REATTEMPT`,
    `would drop`,
    `untranslated as`,
    and,
    added 2026-09-06 after a page the grammar could not parse shipped with the warning in the log,
    `mdx-downgraded` and `would ship a page`.
    Read the `TALLY` and `DESTINATIONS` lines,
    and the `SEAT` lines for how many calls each model threw.
6.  Sum the `cost=` fields for what the pass spent,
    and read the last `METERS` line for what is left.
    Added 2026-09-06 after four pages had shipped with it unmeasured:
    count the page's straight quotes after a letter against its curly apostrophes
    (`grep -c "[A-Za-z]'"` beside `grep -c "[A-Za-z]’[a-z]"`;
    the first pattern widened the same night from `[A-Za-z]'[a-z]` after a plural possessive slipped it),
    count its U+2026 ellipses against its three-dot ones
    (added 2026-09-07,
    for the same reason),
    and compare with the archive,
    since a page mixing the two conventions is a defect no reader notices and every reader can measure.
7.  Write the reading into
    [`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md)
    as its own section,
    and put the belief it supports into
    [`translation-repair-readiness-signal.md`](../planning/translation-repair-readiness-signal.md).
    A reading that is not written down did not happen.

## What to do next

The order is the readiness document's,
and each item is a pass and a reading rather than a build.

1.  Top up OpenRouter.
    Each pass costs about 5 USD on a fresh build,
    and a code change moves the cache generation,
    so a re-run after a rebuild pays full price.
2.  Run one entry with double-quoted `PhotoScroll` paths that has not been run yet:
    `yulianNyanner`,
    `Arita` or `MTF_0615`.
3.  Run the entry with the comma-shaped one-line element.
4.  Run one entry whose consolidation completes unstarved on this build,
    which no run has yet done.
5.  Read each page the way the three above were read,
    and record the reading in the planning document and the readiness document.

No smaller item stays open from earlier days.
The ModelRun seat is settled in the catalog:
`modelrun` and `parasail` are ignored for MiniMax M3 since `d55d83082`,
so CoreWeave serves it alone under zero data retention.
The picture gather's straggler wait was re-read tonight on the three-reader pictures today's runs produced,
and the reading is in the planning log under "The picture gather's wait":
the gather waits for every reader with no window,
which costs a median of 11.6 seconds and cost six minutes once,
when a reader hung after two readings had already corroborated.
Whether to cut a straggler there is the owner's call and no dial was added.

The documentation debt this snapshot first recorded is cleared.
[`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md)
was a running log written as wrapped prose,
reported by `mise run lint:markdown` at about 1200 `semantic-line-breaks` findings.
It is now at zero,
and the sweep is provably content-preserving:
the sorted word multiset is identical before and after,
11081 words either way,
with the same 33 headings.

## Standing constraints

- Never echo an API key value,
  and never read `/proc/<pid>/environ`.
  The keys are injected by the root `mise.toml` from the sops-encrypted,
  gitignored,
  per-worktree `.env.local.json`;
  `mise exec --` from the worktree root injects them.
- Never set `thinking`,
  `budget_tokens` or `reasoning_effort`.
- Never write `Closes #N`.
- Spent Candidate A through M prompts may never be redispatched.
- Credentials,
  API keys,
  image bytes,
  and raw provider requests and responses stay private.
  The owner has relaxed everything else until the project is finished.
