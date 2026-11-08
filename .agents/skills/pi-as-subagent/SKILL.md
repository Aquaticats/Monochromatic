---
name: pi-as-subagent
description: Use when delegating a task to Pi (DeepSeek or another model) as a subagent over RPC.
---

# Pi as a subagent

Pi is a capable coding agent that runs as a child process you feed prompts and read events from.
Give it the full toolset and a real task,
then handle its quirks as they appear.
The protocol,
the full driver script,
and every quirk with its evidence live in two docs;
read the one a step points at when you reach that step.

- `doc/agent/drive-pi-as-a-subagent.md`:
  RPC protocol,
  client script,
  traps,
  model choice.
- `doc/agent/drive-pi-as-a-coding-subagent.md`:
  the `pi-rpc-agent.ts` driver,
  watching,
  quirks,
  the git shim.

## Run order

1.  Work where the task lives.
    A worktree (`git worktree add -b <branch> .claude/worktrees/<name> HEAD`) is an option when you want to diff or
    discard the result;
    the current checkout works too.
    Run the area's type check and unit tests first,
    so an old failure is never blamed on the subagent's first edit.
2.  List every matching provider and model pair from `pi --list-models <search>` and ask the owner which one;
    the owner prefers the `-1017` variants of DeepSeek V4.1 Flash.
3.  Start the child as `pi --mode rpc --provider <p> --model <m>`
    in a capped `systemd-run --user --unit=<name> --collect --setenv=PATH` unit,
    with CPU,
    memory,
    task,
    and `IOWeight` caps (never above 10).
    Leave `--tools` off so the model keeps every tool.
    Pi uses the owner's real home by default.
    Pi saves the session by default and the owner wants that kept,
    so the run can be read back or resumed with `--session <path|id>`.
4.  Write the brief.
    Quote the synthetic-load ban from `AGENTS.md`.
    To learn whether the model works unassisted,
    state the problem,
    the evidence,
    and the owner's intent.
    To get a known change,
    name the files,
    the exact sentences,
    and the tests.
    Fix the acceptance criteria before the run and judge the result twice:
    against the letter of each criterion,
    and by what the end user receives.
5.  Watch the feed through `jq` in the Monitor tool (`tail -n 0 -f`,
    one monitor per feed).
    Wait for the end with `timeout <seconds> tail --pid=<main pid> --follow /dev/null` in a background command,
    the PID from `systemctl --user show --property=MainPID --value <unit>`.
6.  Verify the result yourself,
    commit as the lead,
    and clean up what the run created:
    no `pi --mode rpc` process or unit left,
    leftovers in `/tmp` and the working directory deleted by exact name,
    a worktree you added removed after merge.

## Quirks and what to do

- Dialogs block.
  Pi can raise `select`,
  `confirm`,
  `input`,
  or `editor` requests,
  and the source leaves an unanswered one pending forever.
  Answer each with `{type:'extension_ui_response', id, cancelled:true}`.
  `agent_settled` marks a settled turn.
- Soft misses look like success.
  A `find` given a path as its pattern returns "No files found" with `isError` false;
  read the result text in the feed.
  The model recovers on its own,
  as it does from a guessed file extension or a rejected multi-block `edit`.
- It verifies more than the change needs:
  reads,
  edits,
  re-reads,
  greps other uses.
  A one-line task took 34 seconds;
  budget for it.
- Silence of 20 seconds or more during extended thinking is not a hang.
  Compare the feed's timestamp and the process state before calling a run stuck.
- It reads this repository's docs,
  including ones that describe the problem it was given.
  Record that as a confound when judging an unassisted attempt.
- With a shell it compiles and runs long jobs (one build used OpenMP) and writes into `/tmp` and its working directory.
  The capped unit bounds the load,
  and a cleanup by exact name handles the files.
  A throwaway working directory keeps them out of a checkout when that matters.
- A commit auto-pushes in this repository.
  Say in the brief that the lead commits.
  For a hard stop,
  put the read-only git shim from the coding doc first in the unit's `PATH`;
  it guards mistakes only.
- Tests that import a built artifact fail on a correct edit until the package is built,
  with errors that look like logic bugs.
  Build first,
  read each step's exit status,
  and check that the new test names appear in the output.
- A systemd user unit does not inherit the shell's `PATH`;
  without `--setenv=PATH` every arm fails in seconds with `spawn pi ENOENT`.
- `git worktree add` returns after minutes although the directory is usable within seconds;
  start it in the background with its output in a file.
- In a worktree with uncommitted changes the git-policy guard refuses `git checkout <ref> -- <path>` and `git reset`.
  To show a new test fails without the change,
  write the base content with `git show <ref>:<path>` and restore the same way.
- The owner's real home has a `thinking-default` plugin that overrides `--thinking`.
  `PI_CODING_AGENT_DIR=<dir>` points Pi at a directory holding `settings.json` (its `packages` list) and a copy of
  `auth.json` (mode 600 in a mode 700 directory) when a run needs different providers or packages;
  `--thinking` then applies as given.
  A provider that comes from a package needs that package in whichever home the child uses.
  Delete the `auth.json` copy afterwards.
- A project's `.pi` settings are ignored without a message unless the child gets `--approve`.
- `--tools` is an allowlist for every tool,
  so passing it removes whatever it does not name;
  a run with it says nothing about the tools it left out.
- The driver's feed cuts long results to 500 characters and drops `structuredContent` (a `bash` result's `exit_code`),
  so run the checks yourself.
- Two monitors on one feed deliver every event twice;
  stop the old one first.
- A prompt Pi rejects comes back as a response,
  not as an event.
  One Pi process runs one session.
- Timings on the `synthetic` provider are that provider's speed;
  tool-call and turn counts do not depend on it.
