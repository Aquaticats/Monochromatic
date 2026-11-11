# pi-shared-subagent-driver

Runs one Pi agent unattended over RPC.
It sends the prompt,
declines dialogs that would otherwise wait forever,
enforces the run's limits,
records a feed and a summary,
and delivers steering messages from an inbox folder.

Four more commands start those runs:
`start-pi-run-when-a-pool-has-room` counts the provider pools and chooses one,
`start-pi-run-in-a-worktree` makes the worktree and the unit,
`retry-pi-run-start-until-a-pool-has-room` waits for a slot,
and `start-pi-runs-from-a-batch-list` starts a whole list of runs.

Two more commands read the same runs folder:
`watch-overdue-pi-runs` ends when a run should have come back,
and `check-in-on-pi-runs` describes one run that has not.

The skill `.agents/skills/pi-as-subagent/SKILL.md` says how to start a run
and which command to run to watch for one that should have come back.
Evidence for how Pi behaves is in `doc/agent/drive-pi-as-a-coding-subagent.md`.

## Starting runs at all

A run is one Pi agent,
one worktree,
one unit and one run folder.
Four commands do that work,
and each offers the step it owns:

- `start-pi-run-when-a-pool-has-room`:
  counts the runs going now,
  chooses the pool one more run goes on,
  and hands the worktree launcher the words the run needs.
- `start-pi-run-in-a-worktree`:
  makes the worktree,
  links its dependencies,
  sizes the turn cap,
  starts the driver as a capped systemd unit,
  and removes the worktree once the run has finished with it.
- `retry-pi-run-start-until-a-pool-has-room`:
  asks the pool gate again and again until a slot frees or its own deadline passes.
- `start-pi-runs-from-a-batch-list`:
  starts each entry of a list through the retrying start,
  one at a time,
  and prints the line each entry ended with.

Each has a `mise` task of its own name under
`//package/pi-shared/subagent-driver:`.

### The provider pools

A pool is how many runs one provider account answers at once,
which is not the work a run does.
Every limit sits in `src/pool-limits.ts` beside the owner's words or the measured refusal it came from,
and the commands read them from there:

- `neuralwatt`:
  standard DeepSeek,
  20 runs.
- `neuralwatt-flex`:
  DeepSeek flex,
  22 runs.
  Flex answers about 100% slower,
  so a run moved there gets twice the wall clock it asked for.
- `opencode-go`:
  20 runs,
  taken only once both Neuralwatt pools are full.
- `neuralwatt-kimi-flex`:
  22 runs of Kimi K3.1 Pro flex,
  taken only by a run that names that model.

A run that names a model only one pool takes stays on that pool.
A run that resumes a session stays on the pool it named,
with no wall factor.
Every other run takes the first pool with room,
in the order above.

A run is counted from its launcher's own command line while the launcher is still running,
and from the unit's `ExecStart` once the launcher has gone,
so a launcher that has not made its unit yet still holds its slot.
An audit (`sol-`) and a Kimi run on a provider no pool holds are counted in no pool,
because neither competes for those slots.
A run whose pool cannot be read at all is counted in `opencode-go`,
which is what the launcher this replaced did.

### `start-pi-run-when-a-pool-has-room`

It takes the worktree launcher's options,
plus these:

- `--name`:
  the run's name.
  Required unless `--counts-only` asks for the counts alone.
- `--runs-folder`:
  the folder the runs live in,
  `${HOME}/temp/agent/pi-runs` by default.
- `--counts-only`:
  print the counts and start nothing.
  It needs no name and never refuses because a run is already going.
- `--session-of`:
  a finished run's folder,
  whose session this run resumes.

Every other word is the launcher's and is passed on untouched.

It answers one line,
and exits `0` when it started a run,
`3` when no pool has room,
and `4` when a run of that name is already going,
which is what the retrying start reads.

### `start-pi-run-in-a-worktree`

- `--name`,
  `--branch`,
  `--prompt`,
  `--cpu`,
  `--memory`,
  `--tasks`,
  and `--wall`,
  which are all required.
- `--branch=none` runs in the checkout itself and makes no worktree.
- `--run`:
  the run folder's number.
  Without it the next number no folder of that name holds is taken,
  so a restart never writes over an earlier run's summary.
- `--turn-cap`:
  how many model turns the run may start.
  Without it the cap is a quarter above the longest finished run of the same kind,
  and a runs folder with no finished run refuses the launch rather than guess.
- `--session`,
  `--provider`,
  `--model`,
  `--idle-seconds`,
  `--runs-folder`,
  and `--checkout-folder`.

The worktree is `<checkout-folder>/.claude/worktrees/pi-<name>`,
made from the checkout's `main` unless the run's branch already exists.
The unit's `RuntimeMaxSec` is the run's wall clock plus what the driver needs to put the run away,
which is the only bound that reaches a driver stuck in a file system call.

The wait for the unit is the launcher's own:
it starts the waiter,
and asks the manager for the unit's `ActiveState` every minute,
and ends the waiter once the unit has read as not running twice in a row,
or once the run's wall clock plus fifteen minutes has passed.
It used to wait on `systemd-run --wait` alone,
and while an impostor manager held the session bus name on 2026-11-11,
sixteen launchers never heard their units end and waited about ten hours.

When the run has ended the worktree is removed only if the run ended `idle` or on its stop file,
its HEAD is a branch,
and `git status` holds nothing.
It is never removed by force,
and every other ending keeps it and prints why.

### `retry-pi-run-start-until-a-pool-has-room`

`--deadline-seconds` and `--period-seconds`,
then `--` and the pool gate's words.
It ends when the gate starts the run,
when the gate answers something other than "no slot",
or at its deadline,
when it prints `no slot freed before the deadline:` and what it was asked for.

### `start-pi-runs-from-a-batch-list`

`--batch-file`,
`--runs-folder`,
and `--deadline-seconds`,
which is two hours by default.
The file holds a JSON array of entries,
each with `name` and `branch` required.
An entry may also name `prompt`,
`cpu`,
`memory`,
`tasks`,
`wall`,
`run`,
`sessionOf`,
`provider` and
`model`.
A `prompt` that is not a path is read under the runs folder,
and an entry with none reads `<name>-prompt.md` there.
Entries start twenty seconds apart,
and the command prints one line per entry once every run it started has ended,
keeping the end of both streams so a failure says why.

## Command line

`drive-pi-subagent` takes these options,
each written as `--name=value`:

- `--run-folder`:
  the folder this run writes to.
  Use a new one for each run.
- `--working-folder`:
  the folder Pi starts in.
- `--prompt-file`:
  the first prompt.
- `--provider` and `--model`:
  what Pi runs on.
- `--wall-seconds`:
  how long the run may last.
  It bounds Pi's start-up,
  every answer the driver waits for,
  and the watch,
  so a Pi that never answers still ends at this limit.
  A value whose milliseconds would be infinite is refused,
  because a deadline of `Infinity` is no deadline.
- `--idle-seconds`:
  how long the driver stays up for a steering message after Pi comes to rest.
- `--turn-cap`:
  how many model turns the run may start.
- `--session`:
  a session file to resume.
  It stays one argument,
  so a path holding spaces is safe.
- `--retry-cap`:
  how many attempts follow a model error another attempt may fix,
  three by default,
  which is the `retry.maxRetries` default Pi's own automatic retry uses.
  It is a whole number,
  and zero is allowed:
  a run with no attempts ends on its first error,
  as the driver did before it retried anything.
- `--retry-base-seconds`:
  the wait before the first attempt after a model error,
  two seconds by default,
  which is the `retry.baseDelayMs` default Pi's own automatic retry uses.
  The wait doubles with each attempt and never passes a minute.
- `--pi-argument`:
  one more word for Pi's command line.
  Repeat it for more words.

The first eight options are required,
so a run is always bounded.
Putting the run away has its own bound after the wall limit:
the last two answers,
Pi's stages of leaving,
and the feed's last write,
each with a limit of its own,
so a run ends even when Pi answers nothing at the end.

`pi` must be on `PATH`.

## Files a run writes

- `feed.jsonl`:
  one line per event worth watching,
  as it happens.
- `summary.json`:
  appears whole and last,
  so its presence means the driver has finished.
  `piExit` is there only when Pi's exit was seen,
  so a summary naming `pi-not-stopped` omits it and claims nothing about Pi having gone.
  `feedProblem` names why the feed is incomplete when it is,
  `modelError` carries what the model said when the run ended on it,
  `compactionProblem` carries what Pi reported of a compaction that kept the prompt waiting,
  and `retries` and `rateLimitErrors` count what the run retried and what it met of the failures the
  coordinator measures apart.
- `stderr.log`:
  Pi's standard error.
- `inbox/`:
  drop a message here to steer the run.
  Write it under a name starting with a dot,
  then rename it to one without the dot,
  because any other name is read at once.
  Pi gets it as a steer while it works and as a new prompt while it rests.
  An entry must be an ordinary file:
  a named pipe is set aside instead of waited on.
  A message Pi refused stays as `.failed-<name>`,
  and a later message of the same name that is also refused gets `.failed-<name>.<n>`,
  so no kept message replaces another.
  While a message is being delivered it is held under a dot name of its own,
  so a newer message published under the same name is neither deleted nor kept in its place.
- `stop`:
  create this file to end the run early.

## What Pi's environment holds

Pi hands every command of its `bash` tool the whole of its own environment,
so one `env` dump prints whatever the driver inherited,
and everything a command prints reaches a model provider.
The driver therefore hands Pi its own environment with every variable whose name holds
`TOKEN`,
`SECRET`,
`KEY`,
`PASSWORD`,
`CREDENTIAL` or
`AUTH` removed,
and notes the names it removed as the feed's `environment` line,
which carries names and never a value.

The rule fails closed:
a credential nobody listed yet is removed too,
and the only way back is the allow list in `src/environment-without-credentials.ts`,
where each entry states what breaks without it.
The list holds `XAUTHORITY`:
the X server cookie file a browser check reads,
where `xrandr --query` exits 0 with the variable and exits 1 with
`Authorization required, but no authorization protocol specified` without it.

The rule costs the names whose own tools were shown to manage without them:
`SSH_AUTH_SOCK`,
`ICEAUTHORITY`,
`GIT_AUTHOR_NAME` and
`STARSHIP_SESSION_KEY` all match and all go.
`gh` reads its token from the keyring,
Git's configured `gh auth git-credential` helper answers `git credential fill` with a password
under the stripped environment,
commit signing here signs with the key file (a throwaway commit signed with and without `SSH_AUTH_SOCK`),
and `mise` runs.

`mise` can read a GitHub token from its own store
(`github.credential_command` or `mise token github`),
which this machine does not configure yet,
so inside a subagent `mise` makes its GitHub API calls anonymously:
measured at 60 requests an hour,
where an authenticated call gets 5,000.
A machine that needs the higher limit configures that store rather than exporting the variable.

What this bounds is inheritance,
not access.
The stores stay where they are:
`auth.json`,
`gh`'s keyring and
`mise`'s own.
An ancestor process still holds the driver's own environment,
and a command can still read a secret out of a file.
A plugin that adds a variable to a command after Pi starts is outside the rule too.

The `mise` session carriers `__MISE_DIFF` and `__MISE_SESSION` stay,
so they are worth one check before trusting the rule:
in the environment that carried 38 credential-shaped variables,
neither carrier held any of those values,
compared rather than printed.

A provider credential still reaches Pi,
which is why no provider key is on the allow list:
a stored credential owns the provider,
so pi-ai reads an environment variable only when `auth.json` holds nothing
(`dist/auth/resolve.js` in Pi 1.5.0),
and the neuralwatt plugin reads its encrypted store before the environment
(`package/pi-plugin/neuralwatt/src/credential.ts`).
That holds for this machine's configuration,
not for providers in general:
a provider whose key exists only in the environment now fails to authenticate,
so store it with `/login`,
which writes `auth.json`,
or in the provider's own store.

## How a run ends

`endReason` in the summary says how.
`idle` and `stop-file` exit 0.
`prompt-rejected`,
`compaction-failed`,
`pi-exited`,
`turn-cap`,
`wall-clock`,
`driver-failed`,
`model-error` and `pi-not-stopped` exit non-zero after writing the summary.

`driver-failed` means the driver itself hit an error:
`driverError` names it,
or `feedProblem` does when the feed could not be written.

`model-error` means Pi settled on a last assistant message that ended in an error nothing recovered from,
and `modelError` holds what the model said.
`modelErrorAttempts` says how many attempts the run made at it,
and the driver's own message names that count too.
An error an automatic retry recovers from is not one:
the run still ends `idle`,
because the reply after it succeeded.

`compaction-failed` means Pi reported a compaction that failed and then answered nothing,
so the run ended on the session Pi could not put in order rather than on a prompt Pi refused.
`compactionProblem` holds what Pi said of such a compaction,
or names the compaction that was still running when the run ended,
and it is there however the run ended:
a run that ended at its wall clock or on a model error after a compaction Pi could not finish carries it too.

## When Pi compacts before it takes the prompt

A resumed session larger than its model's window makes Pi compact before the turn the prompt asked for exists,
and the prompt's answer waits for that compaction:
`_prepareRun` runs `_checkCompaction` before Pi writes the answer
(`src/core/agent-session.ts` in Pi 1.5.0).
A resumed run of 905 483 tokens on a model with a 524 288-token window reported `compaction_start` four seconds in,
and a driver that gave every prompt thirty seconds recorded `prompt-rejected` with `no answer in 30000 ms`
while Pi was still compacting.

So the bound measures Pi's silence,
not the clock.
`compaction_start` stops the count and `compaction_end` starts it again,
which means a compaction Pi is running never spends the silence a prompt is allowed.
The run's own deadline is the outer bound of every wait,
so a compaction that never ends ends the run as `wall-clock`
with `compactionProblem` naming it,
and a wait can never be held past the run.
A compaction Pi reports failing is kept in the answer of the wait that spanned it,
so silence after one ends the run as `compaction-failed`.

A Pi that reports no compaction and answers nothing is unchanged:
the wait ends after its silence bound and the run ends as `prompt-rejected`.

### `save-the-final-message-of-a-review-that-wrote-no-review-file`

`--review` (repeatable),
`--runs-folder`,
and `--review-folder`.
A Kimi review that ends without writing `review.md` leaves its findings only in its last message,
and a review's worktree is kept until that file exists.
This reads the last assistant message whole from the Pi session file the newest finished
`kimi-review-<review>-<n>` run names,
and writes it as `<review-folder>/<review>/review.md` with a note saying which run it came from.
The summary's own `lastAssistantText` cannot be the source:
it is cut to 900 characters.
A review that wrote its own file is left alone,
and the command exits non-zero when any review it was asked for was not saved.

## The three answers a run waits for while Pi starts

A run's first three commands are `get_state`,
`get_commands`,
and the prompt,
and Pi is loading its session and its extensions while they wait.
Their silence is bounded by a measured figure of its own,
not by the bound an answer owes mid-run.
That figure is `START_UP_ANSWER_MILLISECONDS`,
two minutes,
and `src/driver-constants.ts` states it with the measurement it came from:
over 1,214 runs in a runs folder,
the longest answer Pi gave took 76 322 ms,
and four runs were still waiting 90 to 91 seconds after Pi started when the old thirty-second bound cut them off as
`prompt-rejected`.
Each of the three still stops counting silence while Pi reports a compaction,
and the run's wall clock still ends them.

## What the driver retries, and what it leaves alone

A run whose provider fails once ends as it would have before,
unless the failure is one another attempt can fix.
The driver reads that from the error Pi reports,
in the shape the providers write it:

- another attempt is made for a provider server error (a `5xx` status),
  a request timeout,
  a dropped connection,
  or a rate limit such as HTTP 429;
- the run ends on the first error for everything else,
  because another attempt reaches the same answer:
  a missing key or credential,
  an unknown provider or model,
  a context overflow,
  an abort the coordinator asked for,
  and a quota or plan limit nothing refills.

A status is read only where the error text starts with one,
which is where a response writes it:
`400 Bad Request: max_tokens must not exceed 500` states a limit on the request,
and the `500` in it is no server status.
A rate limit is decided before the words about credentials,
because a rate limit names what it limited:
`429 Too many requests for this API key` is about the key being asked too often,
not about a key that is wrong.

A shape the driver does not recognise is left alone,
which is how such a run ended before this existed:
only a failure that is known to pass is waited out.

That rule comes from the error texts the runs in `${HOME}/temp/agent/pi-runs` recorded on 2026-11-09.
Six summaries ended `model-error`,
and every assistant message that ended in an error carried one of four texts:
a Cloudflare `520` body twice,
`Request timed out.` once,
`Provider is not configured: neuralwatt` three times,
and `This operation was aborted` five times.

An attempt is a continuation prompt to the Pi that is already running,
which Pi answers as a new turn of the same session:
Pi drops an assistant message that ended in an error when it builds the request for the model
(`packages/ai/src/api/transform-messages.ts` in `earendil-works/pi`),
so the attempt reads the last state the model produced successfully,
which is what resuming the session by hand reads.
The wait grows from `--retry-base-seconds`,
doubling with each attempt,
raised to the wait a failure names when it names one
(the `520` body above says `"retry_after":60`),
and never longer than a minute,
which is the `retry.maxAgentDelayMs` default Pi's own automatic retry uses.
A failure that asks for a longer wait than that ends the run instead:
an attempt made early in the window it named reaches the same answer,
so the feed notes the wait it asked for and the run ends on the error.
The wait is cut to what the run's wall clock has left,
so no attempt is waited for past the run's own bound,
and a run that ends on its error names how many attempts it made at that error,
counted from the last reply that succeeded.

An attempt waits for Pi to rest as well as for its own delay:
a prompt handed to a Pi that is at work is queued behind that run or refused,
and the error the attempt would answer may still be recovered by that run's own reply.
A failure stays in the summary until a reply succeeds:
a prompt Pi consumes without starting a run and without replying recovers nothing,
so the run ends on the failure it began with rather than as a run that came back.

A wait is kept only while the failure it was scheduled for is the one standing:
a reply that recovers it,
or a different failure in its place,
drops the wait and whatever stands then is read afresh.
One case is left as it is:
a failure that is recovered and comes back word for word inside the same wait
inherits the wait already running,
so that attempt can go sooner than the provider asked.
Telling the two apart takes a success and an identical failure inside one wait,
between two looks,
and the worst it costs is one attempt sent early.
The attempts a run may make stay bounded either way,
because every attempt is counted and the cap ends the run.

A person reading the run afterwards sees each attempt twice:
a `retry` line on `feed.jsonl` carrying its number,
the bound it sits under,
its wait,
what kind of failure it answers and that failure's text;
and `retries` in `summary.json`,
which is zero for a run that never needed one.
The `retry` line is written when the attempt goes,
immediately before its `retry_prompt` line,
so an attempt is counted only once it was sent.
A wait the stop file,
the wall clock or Pi's own exit cuts short leaves the failure standing with no attempt behind it:
the run ends for that reason,
and `modelError` still names what the model said.
That standing failure is why a run the stop file ends reports `model-error` and exits non-zero rather than `stop-file`:
the task the run was given was not answered,
so a stop file cannot make it read as finished.
`rateLimitErrors` counts the model errors that named a rate limit or a quota,
whether or not another attempt could fix them,
because those are measured apart from other failures.

Pi has a per-run switch of its own,
the `set_auto_retry` RPC command,
and this driver does not use it:
it is not per-run.
`session.setAutoRetryEnabled` writes `retry.enabled` into the agent's `settings.json` and saves it,
measured with a real `pi --mode rpc` against a throwaway `PI_CODING_AGENT_DIR` on 2026-11-09,
so a run that used it would change the setting of every later run,
and a Pi killed before the driver could put the setting back would leave it changed.

`pi-not-stopped` means Pi was still alive when every bound on ending it ran out,
so the summary claims nothing about Pi having gone.
A run started as a systemd unit gets its processes cleaned up by the unit,
which is what ends a Pi that outlived these signals.

A run that ended on its own terms still fails as `pi-exited` when Pi left afterwards with a status other than zero,
such as while the driver was asking it for the run's statistics.
A signal the driver itself sent is not a failure.

A driver started by a Pi that a driver runs refuses at once,
so agents cannot start agents without end.

## Watching for runs that should have come back

The owner's rule:
"Regularly,
check in on subagents that should have finished their jobs (based on time estimations 90%+ percentile) but haven't returned."

`watch-overdue-pi-runs` is that check.
It is one command in the shape rule `MWK` asks for:
it ends at a terminal state and prints only lines to act on,
never progress.

```sh
# from anywhere, with the runs folder the driver wrote into
watch-overdue-pi-runs --runs-folder=$HOME/temp/agent/pi-runs --deadline-seconds=10800
```

It takes these options,
each written as `--name=value`:

- `--runs-folder`:
  the folder the driver wrote one run folder per run into.
  Required,
  because no look is taken without knowing where the runs are.
- `--deadline-seconds`:
  how long the command may keep looking.
  Required,
  so every wait it takes is bounded by a decision rather than by a default.
- `--interval-seconds`:
  how long to wait between looks,
  five minutes by default.
- `--recheck-seconds`:
  how long a run it reported is left alone before it is reported again,
  half an hour by default.
- `--limit-minutes`:
  a limit of your own that replaces both computed limits.
  It also passes the recheck interval,
  because a person asking now is not a rotation to keep quiet.
- `--state-file`:
  where what was reported is kept,
  `pi-overdue-watch/reported-runs.json` under `XDG_STATE_HOME` by default,
  or under the home directory's `.local/state` when that variable is not set.
- `--once`:
  look once,
  print what it found,
  and end.

It ends with one of these answers:

- one line per run past a limit,
  and one per call open past what finished calls take:

  ```text
  # --runs-folder=$HOME/temp/agent/pi-runs
  OVERDUE pi-copy-route.service: up 96 min, and 90% of 120 finished runs were done within 85 min; feed quiet 4 min, bash open 12 min; $HOME/temp/agent/pi-runs/copy-route-4
  STUCK? pi-copy-route.service: bash open 25 min, and 99.9% of 24,728 finished calls ended within 19 min; $HOME/temp/agent/pi-runs/copy-route-4
  ```

- `no Pi agent is running`,
  so a caller that starts runs later starts the command again.
- `no Pi agent is past a limit:` followed by what it compared against,
  which is what `--once` answers with.
- `the watch reached its deadline with no Pi agent past a limit:` and the same comparison.

A run is reported once and then left alone for `--recheck-seconds`,
so a coordinator that runs the command on every turn is told about one run about twice an hour.
What it reported is kept in the state file,
so a new process reads it too.

### Where the two limits come from

Both are computed from the runs folder on every look,
never written down here,
because the work decides them rather than the day they were measured:

- a run is overdue once it has lasted longer than the 90th percentile of `totalMilliseconds`
  in the finished runs,
  leaving out the runs whose length is no evidence of how long work takes:
  `model-error`,
  `prompt-rejected`,
  `compaction-failed`
  and `driver-failed`.
- a tool call is stuck once it has been open longer than the 99.9th percentile of
  every ended call in those runs,
  read from the moments each feed records.

Neither sample is grouped by kind of job.
The scratch script this came from grouped by three substrings of its day's run names
(`-findings-on-main`,
`-against-main`
and `tokio`),
and no boundary the driver or Pi records matches those:
a run named outside that habit silently takes another kind's limit.
Every finished run whose length is evidence is one sample instead,
and a false alarm costs one look at the agent.
A run that hit its wall clock or its turn cap is part of the sample:
it did the work for as long as it was allowed.
The calls of every finished run count,
including the calls of a run whose length does not:
a run the model provider ended still finished the calls it finished.

How long a call has been open is the age systemd reports for the unit minus the moment the run's feed recorded for that
call's start.
The two clocks begin within the driver's own start-up of each other,
a few hundred milliseconds,
and a wall clock that steps between two reads shows up in that difference,
so a limit in minutes is not moved by either.

A sample too small for the percentile it is asked for answers its own largest value,
so a machine with four finished calls compares against its longest one rather than inventing a shorter limit.
A runs folder with no finished run at all sets no limit,
and the command says so instead of calling every run overdue.

### Checking in on one run

When a line names a run,
`check-in-on-pi-runs` says where it stands:

```sh
check-in-on-pi-runs --runs-folder=$HOME/temp/agent/pi-runs --unit=pi-copy-route.service
```

```text
=== pi-copy-route.service
up 48 min; main pid 3269231; /home/user/temp/agent/pi-runs/copy-route-4/feed.jsonl
feed quiet 6 min; last call open: bash, started 41 min into the run: mise run //package/x:test
processes under the agent: 6; the 6 oldest:
  pid 4300 ppid 4242 12m S wait=do_wait bash -c mise run //package/x:test
```

A quiet feed is usually a long test run,
so it reads the processes before it says anything:
each line carries the age,
the state,
and the kernel call the process is waiting in,
which tells a run waiting on a test suite from one waiting on a child of its own.

It reads each process's command line and never its environment.
The age of a unit is the difference between now and the start systemd stamped on the monotonic clock,
so no wall clock can move it.

A unit is `pi-<task>.service` and each attempt of that task writes `<task>-<number>`,
so the run going now is the highest number of that name in the runs folder.
A name the driver records itself would say this more directly;
until a run writes its own folder into its summary,
the two names are all there is to go on.

### What the watch cannot bound

Every wait the command takes is bounded by `--deadline-seconds`:
the wait between looks is cut to what the deadline leaves,
so the command ends even when no run ever passes a limit.

What no in-process bound reaches is a file system call on a folder that stops answering,
here as in a run of the driver,
and one look reads every finished feed.
Run the command as a systemd unit with its own `RuntimeMaxSec` when that matters,
or give it a smaller `--deadline-seconds`;
the driver README says why a call that cannot be cancelled also cannot be waited out.

### What the tests reach

`systemctl` and `ps` sit behind one seam,
a function that runs a command and answers with what it printed,
and the units a look judges are handed to it.
A test writes its own runs folder,
hands over its own units,
and needs neither a running unit nor a process table.

## What is bounded, and what is not

Every wait for Pi is bounded:
Pi's start-up,
its state and its commands,
each command after the prompt,
the watch,
and putting the run away.
Each command keeps its own limit as well,
so one silent answer cannot spend the whole run,
and the run's wall clock is the outer bound of every wait:
a command's silence stops counting while Pi reports a compaction,
and nothing holds a wait past the run's deadline.

What no in-process bound reaches:
a file system call on a folder that stops answering,
and a path where `feed.jsonl` or `stderr.log` belongs that is not an ordinary file.
A named pipe there can keep the driver's own process alive after it has written the summary,
because opening it cannot be cancelled once it has started.
The driver reports what it could not do and writes the summary,
and a run started as a systemd unit is ended by that unit's own deadline.
Its `RuntimeMaxSec` is what bounds the driver itself;
`--wall-seconds` bounds the run.

A failure to keep Pi's standard error is reported as a `problem` line and does not by itself change how the run ends,
because it is the diagnostic record and not the record the caller reads.
