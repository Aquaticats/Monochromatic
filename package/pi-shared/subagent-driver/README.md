# pi-shared-subagent-driver

Runs one Pi agent unattended over RPC.
It sends the prompt,
declines dialogs that would otherwise wait forever,
enforces the run's limits,
records a feed and a summary,
and delivers steering messages from an inbox folder.

Two more commands read the same runs folder:
`watch-overdue-pi-runs` ends when a run should have come back,
and `check-in-on-pi-runs` describes one run that has not.

The skill `.agents/skills/pi-as-subagent/SKILL.md` says how to start a run
and which command to run to watch for one that should have come back.
Evidence for how Pi behaves is in `doc/agent/drive-pi-as-a-coding-subagent.md`.

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
  and `modelError` carries what the model said when the run ended on it.
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
An error an automatic retry recovers from is not one:
the run still ends `idle`,
because the reply after it succeeded.

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
  such as `OVERDUE pi-copy-route.service: up 96 min, and 90% of 120 finished runs were done within 85 min;
  feed quiet 4 min, bash open 12 min; <run folder>`.
- one line per call open past what finished calls take,
  such as `STUCK? pi-copy-route.service: bash open 25 min, and 99.9% of 24,728 finished calls ended within 19 min;
  <run folder>`.
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
  `prompt-rejected`
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
each command,
the watch,
and putting the run away.
Each command keeps its own limit as well,
so one silent answer cannot spend the whole run,
and the run's wall limit shortens it rather than extending it.

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
