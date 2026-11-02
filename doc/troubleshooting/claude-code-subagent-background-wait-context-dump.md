# Claude Code 2.1.316: a subagent left waiting on background work dumps its context into the main agent

## Symptom

A subagent that hands a long run (a mutation campaign,
a container gate) to a background command,
then has nothing
to do in the foreground,
costs an abnormal amount of compute.
The owner's words,
2026-11-01:

- "Because of how a few Claude Code quirks work together,
  long-running mutation runs when handed to subagents would
  burn abnormally amount of compute,
  so they'd have to be done by the main agent."
- "The core Claude Code problem:
  subagent context would be auto dumped into main agent's context (expensive!)
  if the
  subagent doesn't have any foreground work.
  Foreground bash is limited to max 3 min by Claude Code and the tuner
  doesn't work."

So a subagent cannot simply wait in the foreground for a run longer than three minutes,
and waiting in the background
ends its turn,
which delivers its context to the main agent.

## Root cause

Owner-reported;
not source-traced.
Claude Code ships as a bundled package (version under test `2.1.316 (Claude Code)`,
from `claude --version`),
and the
code path that limits a foreground Bash call and the one that forwards a stopped subagent's context were not read.
Queued:
trace both in the installed package and record the excerpts here.

What this session observed,
consistent with the report but not proving it:

- The harness's notice on every subagent completion reads:
  "A task-notification fires each time this agent stops with no live background children of its own.
  The user can
  send it another message and resume it,
  so the same task-id may notify more than once."
  A subagent whose only remaining work is a background command therefore stops,
  and notifies,
  while that command runs.
- In the coordinating session,
  Bash calls made without `run_in_background` returned
  "Command running in background with ID:
  ..." instead of their output,
  including short calls such as
  `git log --oneline -3`.
  Whether that comes from the three-minute foreground limit or from another rule was not
  established.

## Verification

- Version:
  `2.1.316 (Claude Code)`.
- Setting in force:
  `~/.claude/settings.json:13` sets `"BASH_MAX_TIMEOUT_MS": "86400000"`.
  The owner reports that this
  setting ("the tuner") does not lift the three-minute foreground limit;
  this session did not measure a foreground
  call's cut-off.
- Not run:
  a controlled pair of subagents,
  one waiting in the foreground and one in the background,
  with the main
  agent's context size compared.
  Until then the cost claim rests on the owner's observation.

## Verified workarounds

- The main agent starts every long run itself as its own systemd user unit
  (`systemd-run --user --unit=<name> --collect` with CPU,
  memory,
  task and disk-weight caps),
  waits on it with one
  background Bash command that ends at a terminal state,
  and reads the result when that command's notification
  arrives.
  Tradeoff:
  the main agent's own context carries the waiting and the result instead of a subagent's,
  and only one
  session can follow the run.
- Subagents keep only bounded,
  foreground-sized work:
  edits,
  focused test runs,
  short builds.
  A subagent that needs a long run's result is resumed with it after the main agent has it.
  Tradeoff:
  more hand-offs,
  each needing a brief.
- Where this repository applies it:
  mutation campaigns
  (`package/mutation-test/cli-cargo-mutants/README.md`,
  "Who runs a campaign") and the git-policy wrapper's handover
  (`doc/handover/cli-git-rust-implementation.md`,
  "Authority").

## What does not work

- Raising `BASH_MAX_TIMEOUT_MS` (set to `86400000` here):
  per the owner,
  foreground Bash is still limited to three
  minutes.
- Letting the subagent background the run and end its turn:
  per the owner,
  that is what dumps its context into the
  main agent.

## Upstream filing decision

Not filed.
`.out-of-scope/claude-code-upstream-bugs.md` exempts Claude Code defects from upstream filing and tracking
("This project does not file or track Claude Code bugs as GitHub issues");
this file is the canonical record instead.
The six-constraint check was not walked because the exemption applies,
and no draft is kept.
