# pi-shared-subagent-driver

Runs one Pi agent unattended over RPC.
It sends the prompt,
declines dialogs that would otherwise wait forever,
enforces the run's limits,
records a feed and a summary,
and delivers steering messages from an inbox folder.

The skill `.agents/skills/pi-as-subagent/SKILL.md` says how to start a run.
Evidence for how Pi behaves is in `doc/agent/drive-pi-as-a-coding-subagent.md`.

## Command line

`drive-pi-subagent` takes these options,
each written as `--name=value`:

- `--run-folder`: the folder this run writes to.
  Use a new one for each run.
- `--working-folder`: the folder Pi starts in.
- `--prompt-file`: the first prompt.
- `--provider` and `--model`: what Pi runs on.
- `--wall-seconds`: how long the run may last.
- `--idle-seconds`: how long the driver stays up for a steering message after Pi comes to rest.
- `--turn-cap`: how many model turns the run may start.
- `--session`: a session file to resume.
  It stays one argument,
  so a path holding spaces is safe.
- `--pi-argument`: one more word for Pi's command line.
  Repeat it for more words.

The first eight options are required,
so a run is always bounded.

`pi` must be on `PATH`.

## Files a run writes

- `feed.jsonl`: one line per event worth watching,
  as it happens.
- `summary.json`: appears whole and last,
  so its presence means Pi has gone and the feed is complete.
- `stderr.log`: Pi's standard error.
- `inbox/`: drop a message here to steer the run.
  Write it under a name starting with a dot,
  then rename it to one without the dot,
  because any other name is read at once.
  Pi gets it as a steer while it works and as a new prompt while it rests.
  A message Pi refused stays as `.failed-<name>`.
- `stop`: create this file to end the run early.

## How a run ends

`endReason` in the summary says how.
`idle` and `stop-file` exit 0.
`prompt-rejected`,
`pi-exited`,
`turn-cap`,
`wall-clock` and `driver-failed` exit non-zero after writing the summary.
`driver-failed` means the driver itself hit an error;
the summary's `driverError` says which.

A driver started by a Pi that a driver runs refuses at once,
so agents cannot start agents without end.
