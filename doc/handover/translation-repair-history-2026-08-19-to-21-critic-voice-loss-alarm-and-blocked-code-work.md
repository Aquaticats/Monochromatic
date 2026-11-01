# Translation repair history 2026-08-19 to 2026-08-21: the critic voice-loss alarm and why code work was blocked

The early observation that the window seemed to cost critic voices,
the explanations that were measured and falsified,
the matched baseline that shrank the anomaly,
what was left as a question,
and the attempt to do code work in a separate worktree while the verify ran.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The window costing critic voices, and the code work blocked while the verify ran

### EARLY: the window is costing critic voices, and TWO of my explanations were wrong

Read 23 minutes into the verify,
on the first entry.
The observation is solid;
the cause is
not established,
and this section records both the falsified explanations rather than only
the surviving question,
because each was committed here before it was measured.

#### The observation

```text
GLaDOSister   baseline heard   new heard   baseline claims   new claims
chunk 0       6 of 6           6 of 6      4                 0
chunk 1       6 of 6           5 of 6      30                17
chunk 2       6 of 6           3 of 6      16                10
```

The baseline heard SIX OF SIX on every chunk of this entry.
The windowed run has dropped to
three.
Four critics were abandoned,
across four different models:

```text
critic <model>: abandoned 180000ms after quorum (cut-mid-reply after N delivered chars)
```

CLAIM COUNTS FALLING IS THEREFORE UNREADABLE AS IT STANDS.
Gate E watched for claim
INFLATION and claims went the other way,
but a stage that heard half its roster raises fewer
claims whatever it was shown.
Nothing yet separates "better-informed critics complain less"
from "fewer critics answered".

#### Not the sheet size

"The window roughly triples the sheet" was an estimate carried in and never measured.
Measured:

```text
critic sheet, no window   10485 chars
critic sheet, windowed    11997 chars     1.14x
of which fixed system prompt  9823
the varying half            662 -> 2174   3.28x
```

The varying half does triple;
it is small against a system prompt that dominates,
so the
whole prompt grows FOURTEEN PERCENT.
That cannot explain a reply still arriving when the
straggler window closes.

#### Not runaway generation either

The second explanation was that critics were echoing the two nearby blocks and running away,
which would fit "2,535,524 delivered chars".
It does not survive the conversion.

DELIVERED CHARS ARE RAW STREAM BYTES,
not produced text.
Over 110 completed streams in this
same run the raw-to-produced ratio has median 413:

```text
GLM-5.2         2535524 raw  ~=  6141 produced
GLM-4.7-Flash   1718876 raw  ~=  4163 produced
Nemotron         813057 raw  ~=  1969 produced
Qwen             260739 raw  ~=   632 produced
```

Those are ORDINARY REPLY SIZES.
In the same run GLM-5.2 COMPLETED a reply of 50,572 produced
characters,
eight times the largest abandoned one.
A model producing 6,141 characters is not
running away.

THE CONVERSION IS ROUGH AND SAYS SO:
the ratio spreads from 28 to 93,910 with p90 at 3,290,
so these are order-of-magnitude figures.
They are wide enough to rule out runaway and not
tight enough to rank the four against each other.

#### The matched baseline exists, and it SHRINKS the anomaly

`~/temp/agent/flagged-pass-2.log` covers exactly these five entries under the same rosters,
so it is the matched baseline.
Read properly it makes the signal much weaker than the first
reading of it here claimed.

```text
                        stages   abandoned   rate     stream retries
baseline (5 entries)       115          15   13.0%                52
this run (so far)           11           4   36.4%                 0
```

```text
critic stage, roster 6      stages   short   min heard
baseline                        42       3           5
this run                         5       2           3
```

THE 0.13 PERCENT BASE RATE PREVIOUSLY RECORDED HERE WAS WRONG and is withdrawn.
It came from
pooling all 242 logs in the agent scratch,
most of which are unit-test runs with small
rosters and no real fan-out.
Against the matched baseline the real figure is 13 percent of
stages,
a hundred times higher,
and 36 percent against it is elevated rather than
extraordinary.
At a 13 percent base rate,
four abandonments in eleven stages is roughly a one
in fifteen outcome:
uncommon,
not remarkable.

WHAT STILL STANDS is narrower and worth keeping:
the baseline's critic stage never went
below FIVE of six across 42 stages,
and this run reached THREE of six.
That single value is
outside the baseline's observed range.
One observation.

TWO THINGS ARGUE AGAINST THE WINDOW as the cause,
and both were missed on the first pass:

-   THE PANEL IS WINDOWED TOO and is clean:
    3 stages,
    none short,
    minimum six of six.
    If
    carrying two extra passages degraded a six-model fan-out,
    the panel should show it as
    well.
    It does not,
    though three stages is little to conclude from
-   STREAM RETRIES ARE ZERO this run against 52 in the baseline.
    Whatever else is true,
    the
    provider is not obviously having a worse night than it had on 08-18

AND ONE READING WAS SIMPLY AN ARTEFACT:
`checker` and `probe` report a minimum heard of 3 in
both runs,
which looked like degradation next to the critic's 6 until the denominators were
checked.
Their roster IS three,
so three of three is a full house.

#### Two more readings, and both say the window is behaving

CLAIMS PER STAGE ARE UNCHANGED,
which is the null that matters most:

```text
              critic stages   claims raised   claims per stage
baseline                 42             429              10.21
this run                  5              51              10.20
```

Identical to two decimal places.
Whatever the window is doing to the roster,
it is not
changing how much critics find per stage.

ANCHORING DISCARDS ARE DOWN,
NOT UP,
which refutes the pre-registered worry directly.
The
expectation was that critics would quote from the NEARBY blocks,
that those claims would die
at anchoring because a quote from next door cannot anchor here,
and that the discard count
would spike:

```text
              claims raised   unanchored   rate
baseline                429           39   9.1%
this run                 51            3   5.9%
```

It did not spike.
On five stages that is weak,
but it is weak in the direction of the window
being harmless rather than in the direction of the worry.

#### Where this leaves the alarm, honestly

Raised loudly,
then dismantled by better measurement,
three times.
What survives is ONE
critic stage at three of six,
a value the matched baseline never produced in 42 stages.
Everything else that looked like corroboration has evaporated:

-   sheet size:
    1.14x,
    not 3x
-   runaway generation:
    abandoned replies convert to ordinary sizes,
    and a much larger reply
    completed in the same run
-   an extraordinary base rate:
    withdrawn,
    the matched baseline abandons 13 percent of stages
-   generic windowing damage:
    the panel is windowed too and is clean
-   provider having a bad night:
    stream retries are 0 against 52
-   critics quoting the neighbours:
    anchoring discards went down
-   claims suppressed:
    claims per stage identical to two decimals

THE RIGHT POSTURE IS TO FINISH THE RUN AND READ THE GATES.
The remaining four entries will
produce roughly forty more critic stages,
which is the sample that decides whether one 3-of-6
was a burst.
Nothing should change in the pipeline before then,
and the reason is now on the
record:
six of the seven things that would have justified a change turned out not to be true.

#### What is left, and it is a question rather than an answer

The abandoned replies were simply still arriving when the 180-second straggler window closed
after quorum,
which is the behaviour `#121` designed.
Whether the window made replies slower,
or this is ordinary run-to-run variance against a baseline that is itself one sample,
is not
established and cannot be from one entry.
`QNB` applies:
the run-to-run band was never
measured for heard-count,
so a single-run difference resolves nothing on its own.

WHAT WOULD SETTLE IT,
in preference order:

-   the remaining four entries of this verify.
    If heard-counts stay depressed across all
    five,
    variance is a poor explanation
-   MORE OF THE SAME COMPARISON.
    The matched baseline above is already read;
    what it cannot
    do is separate the window from run-to-run drift in the provider between 08-18 and today
-   a heard-count band measured over repeated runs of ONE unchanged entry,
    which is the only
    thing that makes a single-run comparison meaningful

WHAT NOT TO DO YET:
change the deadline,
change the straggler window,
or gate the window to
flagged slices.
All three are responses to a cause that has not been established,
and two of
the three would have been adopted on explanations that turned out to be false.

### The separate-worktree escape hatch does not work cheaply, so code work really is blocked

Tried 2026-08-19 while the verify ran,
because "do code work in another worktree" had been
floated as the way to keep moving without touching the running pool's digest.

`git worktree add --detach` succeeds instantly and the tree is correct,
but it carries no
`node_modules`,
at the root or in the package,
so `mise run //package/module/translation-repair:build` (moved on 2026-10-18 to
`//package/translation-repair/<package>:build`)
there needs a full workspace install first.
It was still going after two minutes and was
stopped.
The reasoning behind the idea holds:
a build in another worktree cannot move the
running worktree's `dist/final/node`,
and the main tree's `dist` timestamp was unchanged
afterwards.
It is the setup cost that makes it not worth it for a short wait.

SO CODE WORK IS GENUINELY BLOCKED until the run finishes,
rather than blocked by caution.
What front-loads well instead,
and did:

-   prepare the patch as a script that applies later,
    as `~/temp/agent/apply-126-key-labels.mjs` does
-   validate the instruments that will read the results,
    in BOTH directions
-   do the read-only audits,
    which need no build at all

The worktree was removed with `git worktree remove --force` and the running verify was
confirmed healthy immediately after.

ONE TRAP WORTH REPEATING,
since it fired twice today:
`pgrep --full '<pattern>'` matches its
OWN command line,
so a check for processes under `wt-126` reports the pgrep itself.
Anchor
the pattern to the binary,
as in `pgrep --full '^node .*corpus-pass\.mjs'`.
