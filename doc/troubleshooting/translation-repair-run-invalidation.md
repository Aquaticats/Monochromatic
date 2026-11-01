# What invalidates a corpus run, and what silently does not

Investigated 2026-08-13,
after a night of landing pipeline fixes while a pass
 was running.

## A running pass cannot see a rebuild

`corpus-pass.ts` has no dynamic imports,
so Node loads its whole module graph
 once at process start.
Rebuilding `dist/final/node/index.mjs` while a pass runs therefore changes
 nothing for that pass:
it executes the pipeline as of the moment it started,
 to completion.

This is good news and bad news in equal measure.
A pass is internally CONSISTENT no matter how much the repository moves under
 it,
so its artifacts are always one coherent population.
It is also silently STALE,
and nothing in its logs or artifacts says which
 pipeline it is running beyond the `tip` commit recorded per artifact.

Check it directly rather than assuming a rebuild took effect:

```bash
# no dynamic imports means the graph is frozen at startup
rg --count 'await import|import\(' src/corpus-run/corpus-pass.ts
stat --format='%y' dist/final/node/index.mjs   # compare against process start
```

## The slice cache has two guards and a hole between them

A cached slice is keyed on `SLICE_CACHE_VERSION`,
a `runShape` fold,
the slice
 index,
and both texts.
The two guards are documented and deliberate:

-   `SLICE_CACHE_VERSION` is bumped by hand when `ChunkRepairOutcome` changes
    shape or an existing field changes meaning.
-   `runShape` covers everything that changes what the models are ASKED:
    rosters,
    adjudication config,
    editor addendum,
    identity context.

Neither covers a change to what the code DOES with the answers.
A gate is exactly that.
The footnote-integrity gate added on 2026-08-13 left the prompts,
the roster
 and the texts identical,
so both guards matched,
while a candidate the old
 gate shipped may be one the new gate refuses.
A slice cached before it could be resumed after it,
and nothing would look
 wrong.

The bump was missed on the very commit that added the gate,
which is the useful
 part of this record:
the convention depends entirely on the author noticing,
 and it failed the first time it was tested.
`SLICE_CACHE_VERSION` now carries
 that case by name so the next gate change has a precedent to recognise itself
 in.

An automatic key over the hash of `src/` was considered and not built.
It would invalidate on comment and test changes too,
and a pass takes days,
so
 the cost of a spurious full recompute is high and the discipline is cheap
 where it is remembered.

## Passes at this pin

-   `pass10`,
    started 01:55 UTC,
    stopped 04:29 UTC with 3 settled entries.
    Ran the pipeline as of 01:55 UTC throughout:
    the invisible-line masker was in,
    the CRLF front-matter fix,
    the footnote-graph fix and the footnote gate
    were not.
    Artifacts kept under
    `node_modules/.monochromatic/translation-repair-runs-pass10`,
    and they are
    a consistent old-pipeline population rather than a mixed one.
-   `pass11`,
    started 04:27 UTC,
    stopped 04:36 UTC with nothing settled.
    Superseded before it had produced anything,
    by the typography fix.
-   `pass12`,
    started 04:35 UTC,
    stopped 05:14 UTC with nothing settled and six
    slices cached.
    Ran cache version 7:
    the typography fix landed 04:34 UTC (`846f9ff6d`),
    one
    minute before it started,
    so it carried that and nothing later.
-   `pass13`,
    started 05:12 UTC into
    `node_modules/.monochromatic/translation-repair-runs-pass13`,
    running cache
    version 9,
    so it carries the naturalness-eligibility fix `pass12` lacked
    plus the quote-anchoring telemetry.

Two restarts in ten minutes is cheap and a third would not be:
a restart costs
 whatever the current pass has settled,
so its price rises with every hour.
Batching is the reason `pass12` waited for a check that the other settled
 policies,
translator additions and declared names,
had actually reached the
 prompts.
They had,
so it started with nothing else pending.

`pass12` was NOT restarted for the naturalness-eligibility fix landed after it
 (cache version 8),
and the asymmetry was deliberate rather than fatigue.
The typography fix touched every repaired region of every entry,
99 curly
 characters lost corpus-wide;
the eligibility fix touches only slices holding an
 invisible-only line,
which is 3 lines in `Toka_ls` and nowhere else.
So `pass12` would have refined `Toka_ls` on fewer slices than the current code,
 and nothing else differed.
Restarting was worth doing but not urgent.

The quote-anchoring telemetry then made it worth doing at once,
on a different
 argument:
that telemetry is measurable ONLY on a pass that runs it,
so leaving
 `pass12` alone would have spent days of provider capacity producing a
 population that cannot answer the question the suffix exists to answer.
`pass12` was two hours from having settled anything,
which is the cheapest a
 restart ever gets,
and the cost only rises from here.

Restarting rather than continuing follows the standing instruction to land
 certainly-good pipeline changes immediately and restart runs as needed.
The ordering held every time:
`pass13` was confirmed streaming completions,
 applying an editor patch and hearing 3 of 3 checkers BEFORE `pass12` was
 signalled,
so no window existed with nothing running.

## First artifact under the new code, and what it does and does not show

`pass13` settled `AmbeR_the_anpa` first,
and the same entry exists in the
 56-entry population,
so the two can be put side by side.
They are NOT a
 controlled comparison:
the runs differ by many commits,
not only by the
 night's,
and this is one entry.

```text
                     pass13      old run
  status             repaired    repaired
  duration           27 min      79 min
  issues filed       36          53
  accepted           19          35
  rejected            6          13
  needs-human        11           5
  findings           39          42
```

What it rules out is useful even at one entry.
The finding profile is nearly
 identical,
kind for kind:
same `quote-not-found` count,
same
 `refine-candidates`,
`refine-selected` and `refine-declined` counts,
same
 editor-stage counts within one.
If the night's stricter gates were refusing
 patches in bulk,
the editor and refine findings would move and they did not.

The difference sits UPSTREAM of any patch gate.
Fewer claims were filed and a
 larger share of them landed on `needs-human`,
31% against 9%.
That is critic
 and adjudication behaviour,
which the footnote-integrity gate and the
 typography change do not touch.

Worth watching rather than acting on.
If the `needs-human` share stays near a
 third as `pass13` accumulates,
the composition of any sheet drawn from it
 differs from the sheets drawn so far,
and that matters for comparing precision
 figures across passes.

### Watched, and it was one entry

At 4 settled entries the share does not stay near a third:

```text
  pass13, 4 entries          14.1% needs-human
  the same 4 in the old run  10.2%
  the old run, all 56        10.1%

  pass13 excluding AmbeR     10.6%
  AmbeR_the_anpa alone       30.6%
```

`Acheron` is 5%,
`Anilovr` 13%,
`Arita` 10%.
Drop the one outlier and pass13
 sits at 10.6% against a 10.1% baseline,
which is no difference at all.

So the flag was raised by a single entry and is withdrawn.
Sheet composition
 across passes is not shifting.

The general caution is the one this file keeps earning:
a share computed from
 one entry describes that entry.
It took three more to tell a population change
 from an outlier,
and the first reading looked like a threefold shift.

## The acceptance RATE has moved between passes, which invalidates cross-pass precision

Measured at 8 settled entries,
all 8 present in the 56-entry population,
so this
 is the same-entry comparison rather than a sample against a population.

```text
  entry              pass13 filed/acc    old filed/acc
  Acheron               20/ 11              21/ 11
  AkiraComplex           9/  1              12/  1
  AmbeR_the_anpa        36/ 19              53/ 35
  Anilovr               79/ 46              77/ 61
  Arita                 71/ 46              74/ 45
  Chinatsu_Suzuki       42/ 24              41/ 27
  Considerate_cat       24/  8              26/  8
  Dethelly             242/132             282/198

  TOTAL                523/287             586/386

  accept rate        54.9%               65.9%
```

Eleven points,
z = 3.74,
far beyond chance at these counts.
Filing dropped 11%
 and acceptance dropped 26%,
so the change is in what the panel ACCEPTS rather
 than only in what the critics raise.

It is not uniform.
Four entries are essentially unchanged,
including one where
 `pass13` accepts slightly more,
and four dropped substantially.

WHAT THIS DOES NOT SAY.
The direction of quality is unknown.
A lower acceptance
 rate is an improvement if the panel is rejecting false positives it used to
 accept,
and a regression if it is rejecting real defects.
Nothing here
 distinguishes those,
and the graded sheets that could were drawn from the OLD
 population.

WHAT IT DOES SAY,
and it matters for every figure downstream:
a precision number
 measured on the old run describes a pipeline that accepted 65.9% of filed
 issues.
`pass13` accepts 54.9%.
Those are different populations,
so precision
 figures cannot be carried across without saying which pass produced them.

CAUSE UNKNOWN AND PROBABLY NOT ATTRIBUTABLE.
The two runs differ by many
 commits,
not only by the night's work,
and model sampling is not fixed.
This is
 a fact about the artifacts rather than a diagnosis,
and chasing it to a single
 commit would need a bisect over passes that each cost days.

### The drop is concentrated in the most SUBJECTIVE category, which hints at direction

"The acceptance RATE has moved between passes,
which invalidates cross-pass precision"
says the direction of quality is unknown.
It can be narrowed
 without human grading,
by asking WHICH issues the panel stopped accepting.

Taking every issue the old run ACCEPTED whose target span `pass13` also
 produced,
132 of them,
and asking what `pass13` did with the same span:

```text
  category      dropped  kept   drop rate
    style           7      3     70%
    accuracy       21     78     21%
    policy          1      9     10%
    fluency         0      9      0%
    terminology     1      1     50%
    extension       1      1     50%

  overall          31    101     23%
```

Style is dropped at 70% where the overall rate predicts 23%.
Seven of ten,
 against an expectation of 2.3,
has a 0.24% chance of arising from the overall
 rate,
so the concentration is real rather than small-sample noise.

WHAT THAT SUGGESTS,
carefully.
Style is the most SUBJECTIVE class the taxonomy
 has:
a claim that a rendering reads awkwardly is the one most likely to be a
 defensible difference of opinion rather than a defect.
A panel that rejects
 style claims far harder while leaving fluency untouched at 0% and policy at
 10% looks like it became more DISCRIMINATING,
not merely quieter.

WHAT IT DOES NOT ESTABLISH.
That is a hint from where the losses fall,
not a
 verdict on whether any individual rejection was correct.
Real style defects
 being missed would produce the same table.
Only human grading separates those,
 and that is what the probe calibration asked for.

So the honest statement is:
the acceptance rate fell 11 points,
the fall is
 concentrated in the class where false positives are most likely,
and that is
 weak evidence for improvement rather than regression.
Weak evidence is still
 better than the "unknown" this section started with.

## What one pass actually covers, and when `pass13` will stop

A pass is bounded by wall time,
not by the corpus.
`corpus-pass.ts` carries a
 SOFT budget of 720 minutes,
at which it stops STARTING entries and logs
 `SOFT budget reached`,
and a HARD per-entry cap of 180 minutes.
The in-flight
 entry when the soft budget hits runs on to its own ceiling,
so a run ends
 somewhat after twelve hours rather than exactly at it.

Projected from `pass13`'s own rate:

```text
  soft budget        720 min
  elapsed            415 min at 10 settled
  rate                42 min per entry
  remaining          305 min  ->  about 7 more entries

  projected total    about 17 of 92 = 18% coverage in ONE run
  runs for the corpus  about 6 at this rate
```

So `pass13` will settle roughly 17 entries and stop.
That is the design working
 rather than a fault:
slice-level resumability means a capped entry resumes in
 the next run,
and `listResumableEntries` prefers in-flight documents,
so
 successive runs accumulate coverage rather than restarting it.
The 56-entry
 population was built the same way.

Worth stating because "a corpus pass" reads like it covers the corpus.
It does
 not,
and a reader expecting 92 settled entries from `pass13` would conclude
 something had gone wrong when it stops near 17.

## The timeout batching is already documented, and this run reproduces it

`RUN_EXCHANGE_TIMEOUT_MS` carries a note that timeouts "arrive in correlated
 batches,
about five per retry round",
explained by the provider slowing every
 concurrent call together under load rather than by hangs.

`pass13` reproduces exactly that.
Its GLM-5.2 deadline timeouts went 1,
then 3,
 then 9 across the early entries,
which read like a growing problem,
and then
 added ONE across the next two entries while schema mismatches added twelve.
Throughput improved over the same stretch,
from 46 to 42 minutes per entry.

A batch during congestion,
then quiet.
The existing note predicted the shape
 before it was observed,
which is worth knowing before treating a timeout
 cluster as a new signal.

## The needle telemetry was landed WITHOUT restarting, and why that differs from `pass12`

Landed `b8c678e0a` at 14:10 UTC while `pass13` was 9 hours into a 720-minute
 soft budget.
The standing instruction is to land certainly-good pipeline
 changes immediately and restart runs as needed,
and this one was landed
 immediately.
It was NOT restarted,
and the asymmetry with `pass12` is the
 point worth recording.

`pass12` WAS restarted for the quote-anchoring suffix,
on the argument that
 telemetry is measurable only on a pass that runs it,
so leaving it alone would
 spend days producing a population that cannot answer the question.
That
 argument does not transfer here,
for three reasons.

-   THE QUESTION IS ALREADY UNANSWERABLE BY RESTART.
    The needle preview exists
    to explain why `Futajuhuacha` misses at 35% where every other entry is near
    zero.
    That entry has SETTLED.
    Re-running it would mean discarding its
    artifact,
    which is a different and larger action than restarting a pass.
-   THE YIELD IS THIN.
    About 3 hours of soft budget remain,
    roughly 4 entries,
    and most settled entries carry ZERO wrap-explained misses.
    The expected
    gain is a handful of findings on entries that probably have none.
-   A RESTART BUYS NO COVERAGE HERE.
    `pass12` was restarted when it had settled
    nothing,
    so the restart cost nothing.
    `pass13` will end on its own within
    hours,
    and starting `pass14` then carries the new telemetry from its first
    slice while costing no in-flight work at all.
    Restarting now would discard
    the entry currently in flight to gain the same thing a few hours earlier.

So the rule this pair of decisions actually encodes is not "always restart" and
 not "never restart mid-pass".
It is:
restart when the pass would otherwise
 produce a population that CANNOT answer a live question,
and wait when the
 same telemetry arrives free at the next pass boundary.

`pass14` should start from the current tip once `pass13` stops.
The run monitor
 emits `PROCESS GONE` at that point,
which is the cue.

NO CACHE BUMP was needed.
`SLICE_CACHE_VERSION` covers changes to what
 `ChunkRepairOutcome` means and `runShape` covers what the models are ASKED.
The needle preview changes neither:
it appends diagnostic text to a finding for
 a claim that was already being discarded,
so a slice cached before it and
 resumed after it produces the same repair decisions.

### The next run is armed by a detached watcher, not by a note

SUPERSEDES the `pass14` framing of "Superseded:
`pass14` is armed by a detached watcher,
not by a note",
which was written before the target
 changed.
The watcher RESUMES `translation-repair-runs-pass13` rather than
 creating a fresh `pass14` directory,
and its script is
 `~/temp/agent/resume-run.sh`.
The commit that introduced it,
 `a63c44a94`,
says "arm pass14" and is inaccurate for that reason;
it is not
 amended,
per the no-amend rule.

The script was `~/temp/agent/continue-pass13.sh` when first armed.
It was
 renamed on the rewrite that added the smoke check,
because bash reads a script
 INCREMENTALLY as it executes,
so editing the file a running watcher is
 mid-read can make it resume at a byte offset that no longer means what it did.
 Every rewrite since stops the watcher by explicit pid first,
then edits,
then
 relaunches.

WHY RESUME rather than start fresh.
A new runs directory has no artifacts and
 no slice cache,
so `listResumableEntries` would find nothing in flight and the
 stratified ranking would work from the top,
re-settling the same early entries
 `pass13` already holds.
That is hours of provider capacity spent reproducing
 existing artifacts,
unattended,
overnight.
Resuming carries the settled
 entries forward,
takes a fresh 720-minute budget,
and pushes corpus coverage
 past what one pass reaches.

The separate-directory discipline that produced `pass10` through `pass13` was
 for BEHAVIOURAL changes,
where population purity decides whether figures can
 be compared.
The needle preview is diagnostic only,
so purity buys nothing
 here and coverage costs real capacity.

TWO HAZARDS were found while arming it,
both worth keeping:

-   `pgrep --full 'corpus-run/corpus-pass.ts'` matches any SHELL whose command
    line contains that string,
    including the run monitor's own.
    Measured:
    the
    loose pattern matched 4 processes where only one was the pass.
    A watcher
    using it would see the pass alive forever and never fire.
    The fix is
    `pgrep --full --exact 'node src/corpus-run/corpus-pass.ts'`,
    which matched
    exactly the real process.
-   sops decryption had to be proven NON-INTERACTIVE before trusting an
    unattended launch,
    since `createRunClient` refuses to build a client with
    no key and the run would die at an unattended overnight launch with only a log line.
    Verified from a `setsid` shell with stdin closed:
    the key resolves.

THE PATTERN HAZARD RECURRED TWICE MORE after being written down,
which is the
 useful part of this record:
knowing it was not enough to avoid it.
 `pgrep --full 'continue-pass13.sh'` listed the watcher AND the shell running
 the pgrep,
because the pattern appears in that shell's own command line.
The
 second time it was `pkill --full`,
which killed the shell issuing it,
and the
 command tool reported exit 144 with no other explanation.
The rule that
 survives:
never match a watcher by a pattern that appears in the command line
 doing the matching.
Kill by explicit pid,
and match with `--exact` against the
 full command line.

### The watcher smoke-checks the source before committing to a long run

The pass executes `node src/corpus-run/corpus-pass.ts` straight from SOURCE,
so
 an agent session that stops mid-edit leaves a broken module graph and the
 unattended resume dies in seconds.
The watcher now runs the documented
 zero-quota `--plan` check first and refuses to start the real run if it fails.

WHAT THE CHECK ACTUALLY BUYS,
stated honestly because the first version of this
 section overclaimed it.
Trace both paths against a broken tree:
without the
 check the resume dies in seconds and the log says `SyntaxError`;
with it the
 resume refuses and the log says `--plan failed`.
BOTH LOSE THE NIGHT.
The check
 buys legibility,
not survival,
and legibility is worth having,
since a run that
 died minutes after launch and a run that never started read identically in a settled-entry
 count.

What can genuinely save a night is the RETRY,
which is why the check runs up to
 three times with four-minute gaps before giving up.
A permanently broken tree
 is not repairable by anything in this script and retrying will not save it.
The
 transient failures will clear on their own:
git contention against the corpus,
 a network failure while constructing the client,
a file caught half-written.
 Those cost a whole night otherwise,
and a resume twenty minutes late is still a
 resume.

`--plan` is the right check rather than a build,
and the difference is not
 cosmetic.
The build only proves `dist/` compiles,
and the corpus pass never
 reads `dist/`;
the unit tests do.
`--plan` loads the exact source graph the
 real run loads,
resolves the corpus at the pinned tip,
and constructs the
 client,
so it also catches a corpus or sops-key failure that a build could not
 see.
Measured:
about ten seconds and exit 0 on healthy source.

Each link was verified rather than assumed:

-   Node exits 1 on both mid-edit break modes.
    Tested on throwaway files:
    a
    truncated arrow function,
    and an imported module deleted out from under its
    importer.
-   `mise` propagates a task's non-zero exit rather than swallowing it.
    Tested
    in a throwaway project with its own `mise.toml`,
    one failing task and one
    passing task:
    exit 1 and exit 0 respectively.
-   `--plan` writes only `artifacts/` into whatever runs directory it is given,
    so the check runs against a `mktemp` throwaway and cannot perturb the
    directory the resumed run is about to continue.

The check is BOUNDED at 300 seconds,
a thirty-fold margin over measured healthy
 time,
because a watcher that blocks forever on its own guard is worse than no
 watcher:
the night is lost with nothing in the log saying why.
A TIMEOUT is
 treated differently from a failure and PROCEEDS,
since timing out means the
 module graph loaded and only speed is in question,
which is not the fault the
 guard exists to catch.
Only a genuine non-zero exit aborts.

UNDIAGNOSED,
and recorded because it is a plausible real failure rather than
 only an artificial one.
Given a runs directory that cannot be created
 (`/proc/definitely/not/creatable`,
while probing exit codes),
`--plan` did not
 fail fast.
It printed nothing,
not even its `START` line,
and sat in state `R`
 burning CPU for two and a half minutes until killed.
A full disk would present
 the same way.
The cause was not chased,
since the smoke check uses a creatable
 `mktemp` directory and the 300-second bound covers it either way.

### Superseded: `pass14` is armed by a detached watcher, not by a note

The needle-telemetry section ("The needle telemetry was landed WITHOUT restarting,
and why that differs from `pass12`")
says `pass14` should start when `pass13` stops,
and the run
 monitor emits `PROCESS GONE` as the cue.
That is a note to whoever reads it,
 not a mechanism:
the monitor is session-scoped and dies when the agent session
 does,
and `pass13`'s budget expires overnight.

So the cue is now executed by a detached script at
 `~/temp/agent/start-pass14.sh`,
launched with `setsid nohup` and reparented to
 `systemd`,
which is what makes it independent of any agent session.
It polls
 for the corpus pass to exit,
then starts `pass14` from the current tip into
 `translation-repair-runs-pass14`.

Its guards,
because an unattended launcher is only safe with them:

-   A 6-hour bound on the wait,
    well past `pass13`'s remaining budget,
    so a
    wedged pass cannot leave the watcher alive indefinitely.
-   A re-check after a 30-second settle,
    so it aborts if another launcher won
    the race and a pass is running again.
-   A refusal to start if `translation-repair-runs-pass14` already exists,
    so it
    can never write into a directory holding another run's artifacts.

It logs to `~/temp/agent/pass14-watcher.log`.
To cancel it,
`pkill --full
 start-pass14.sh`.

### Resumed slices will carry needle-less findings, which is not a pass-identity clue

Findings are part of `ChunkRepairOutcome`,
so a slice cached by `pass13` and
 resumed by `pass14` keeps the finding text it was written with.
An entry can
 therefore hold `quote-not-found (target)` from a resumed slice beside
 `quote-not-found (target) needle="..."` from a fresh one.

That is harmless for every decision,
since the preview is diagnostic and the
 claim was discarded either way.
It is worth stating only so a reader does not
 treat a needle-less finding as evidence about which pipeline version produced
 the entry.
The `tip` field is what says that.
