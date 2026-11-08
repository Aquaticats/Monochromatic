---
name: pi-as-subagent
description: Use when delegating a task to Pi as a subagent over RPC.
---

# Pi as a subagent

Pi is a second subagent runtime here,
beside Claude Code native subagents,
and a rule that says "subagent" without naming a runtime binds it too.

Drive `pi --mode rpc --provider neuralwatt --model deepseek-v4.1-flash` through the driver package,
`package/pi-shared/subagent-driver`,
started in the repository,
where Pi loads `AGENTS.md` (from another folder rule `ATH` was not in its context).
Real coding tasks always go through the driver;
the owner said "Forget about --print on a real coding task."
In every prompt cite rule `ATH`,
because the model writes scratch to `/tmp` unprompted,
and quote the synthetic-load ban (rule `NLD`).
Leave `--tools` unstated:
it is an allowlist for every tool,
extension tools included,
so naming some removes the rest.
Pi saves the session by default and the owner wants that kept,
so a run can be read back or resumed with `--session <path|id>`.

Pi may commit directly on `main`,
which auto-pushes,
and it may equally work in a worktree.
The owner on committing:
"We have to be happy with Pi committing directly on main.
We are not building another git wrapper or shim when we're already so busy with one.
If it turns out this blows up we'll consider enhancing our own git wrapper."
And later:
"Feel free to use worktrees.
I said "be okay with pi committing directly to main",
not that they have to."

The owner on how many can run at once:
"Pi doesn't have a limit to number of concurrent subagents.
In practice it's 16."
Each run is its own driver process with its own run folder and unit name.
Runs in one checkout share its files,
so two in one crate build each other's half-finished edits.
Give a run its own worktree whenever anything else is working in the same packages.
Start a run for every independent queued item up to that number.
Resource limits that add up to more than the host has are ceilings,
not usage,
and are no reason to hold work back.

## Start a run

`pi` is in `~/Monochromatic`,
on `PATH` only through that folder's `mise` environment:
an agent's shell has not activated `mise`,
so `command -v pi` finds nothing there,
and `mise exec -- pi` run from `~/Monochromatic` works.
A systemd user unit does not inherit the shell's `PATH` either;
launches without `--setenv=PATH` failed in seconds with `spawn pi ENOENT`.

1.  From `~/Monochromatic`,
    check that `mise exec -- pi --version` prints a version.
2.  Make a new run folder under `${HOME}/temp/agent`,
    one per run.
    The driver writes everything there.
    Choose where Pi works:
    its own worktree (`git worktree add -b <branch> .claude/worktrees/<name> main`,
    then `mise exec -- pnpm install --offline --frozen-lockfile` inside it),
    or the main checkout for a small change with nothing else in those packages.
3.  Write the prompt to a file.
    Beside rules `ATH` and `NLD`,
    give it:

    - the owner's words as the requirement,
      quoted;
    - your own reading as a suggestion it may reject;
    - the files it must not edit,
      including everything `file-enforcer` generates;
    - that it commits its own work with explicit paths,
      as `AGENTS.md` says,
      on the branch checked out where it works;
    - a final report of each change,
      each commit,
      each disagreement,
      and anything it could not do,
      because it cannot ask you.

4.  Start it as its own capped unit,
    in the background.
    `--wait` makes this one command end when the unit ends,
    print the unit's result,
    and exit with its status;
    following `MainPID` with `tail --pid` waited out its whole timeout on a unit that had already finished.
    The entry file uses only Node built-ins,
    so it runs from source with no build and no `node_modules`:

    ```sh
    # run from ~/Monochromatic
    systemd-run --user --wait --unit=pi-<task> --collect \
      --property=RuntimeMaxSec=2500 --property=MemoryMax=3G --property=CPUQuota=200% --property=TasksMax=300 --property=IOWeight=10 \
      --setenv=PATH="$(mise exec -- printenv PATH)" --setenv=HOME="$HOME" \
      "$(mise exec -- bash --noprofile --norc -c 'command -v node')" \
      "$PWD/package/pi-shared/subagent-driver/src/drive-pi-subagent.ts" \
      --run-folder=<run> --working-folder=<its worktree, or "$PWD"> --prompt-file=<prompt-file> \
      --provider=neuralwatt --model=deepseek-v4.1-flash \
      --wall-seconds=2400 --idle-seconds=60 --turn-cap=80
    ```

5.  Read progress from `<run>/feed.jsonl` when asked.
    To steer,
    write a text file under a name starting with a dot in `<run>/inbox/`,
    then rename it to one without the dot,
    because the driver reads any other name at once.
    Pi gets it as a steer while it runs and as a new prompt while it is idle,
    and `--idle-seconds` is how long the driver stays up for one after Pi settles.
    A message Pi refused,
    or an entry that cannot be read,
    stays there as `.failed-<name>`.
    To end the run early,
    create `<run>/stop`.
6.  Read the result from `<run>/summary.json`,
    which appears last,
    so its presence means Pi has gone and the feed is complete.
    `endReason` says how it ended:
    `idle` and `stop-file` exit 0,
    and `prompt-rejected`,
    `pi-exited`,
    `turn-cap`,
    `wall-clock` and `driver-failed` fail the unit.
    `lastAssistantText` holds the whole final report,
    and `sessionStats` holds the cost and names the session file.
7.  For a follow-up after the driver has exited,
    start the same unit under a new name,
    with a new run folder,
    a new prompt file,
    and `--session="<session file>"`.
8.  Read what it committed with `git log` and `git show`,
    fix anything wrong with further commits,
    regenerate anything `file-enforcer` owns,
    and merge a worktree's branch into `main`.

## Protocol

- One JSON command per line on stdin,
  one JSON record per line on stdout,
  logs on stderr.
  Split stdout on LF only,
  because Unicode line separators are valid inside JSON strings.
  Closing stdin ends the process.
- Every command takes an optional `id` that its `response` repeats.
- `{"id":"c1","type":"prompt","message":"..."}` answers `disposition: "started"`:
  accepted,
  not finished.
  A prompt sent mid-run needs `"streamingBehavior":"steer"` and answers `queued`;
  without it Pi rejects the prompt (documented,
  not run).
  `{"type":"steer","message":"..."}` and `{"type":"follow_up","message":"..."}` also queue during a run.
  `{"type":"get_last_assistant_text"}` returns `data.text`,
  or an empty object when the session has no assistant message.
- Wait for `agent_settled` before reading the final text;
  `agent_end` ends only one low-level run.
  Register that waiter before sending `prompt` or `abort`,
  because `agent_settled` can arrive before the response does.
- A dialog request (`extension_ui_request` with method `select`,
  `confirm`,
  `input`,
  or `editor`) waits for an answer,
  and the source leaves it pending forever unless a timeout field or an abort resolves it.
  Answer each with an `extension_ui_response` carrying the request's `id`;
  `{"type":"extension_ui_response","id":<the request id>,"cancelled":true}` declines it,
  and the driver declines all of them.
  The other UI methods (`notify`,
  `setStatus`,
  `setWidget`,
  `setTitle`,
  `set_editor_text`) need no reply.
- A prompt that cannot start (a missing key,
  for example) comes back as a `response` with an error text and no events,
  so a client that waits only for `agent_settled` waits forever:
  give it a deadline and an exit handler.
- A process holds one active session at a time;
  `fork`,
  `clone`,
  `switch_session`,
  and `new_session` replace it.
  `switch_session` to a file that does not exist still reports success.
  Parallel agents need separate processes.

## Pi quirks

- Leave extensions loaded:
  some providers exist only because an extension registers them,
  and `--no-extensions` made `synthetic` unknown.
- `pi auth check --provider neuralwatt --model deepseek-v4.1-flash --json --no-refresh` printed `invalid_state`
  and the same model worked in a real run:
  trust a prompt that gets a reply.
- The repository's `ask-user-question` tool works only in the terminal UI and answers RPC with an unavailable-tool error,
  and `ui.custom()` returns `undefined` at once in RPC mode,
  so the subagent cannot put a question to the human.
- The owner's real home has a `thinking-default` plugin that sets the level to `xhigh` on every start and model change,
  whatever the command line says.
  Leave the thinking level alone:
  pass no `--thinking` and send no `set_thinking_level`.
- In a throwaway folder,
  project `.pi/settings.json` was ignored with no message unless the child got `--approve`;
  other project resources were not tested.
- A multi-block `edit` is rejected whole when any one `oldText` does not match,
  and a `find` given a path as its pattern returns "No files found" with `isError` false,
  so a soft miss and a success look alike until the result text is read.

## DeepSeek V4.1 Flash quirks

The owner rates it at about a low-level Sonnet 5.5 when run at `xhigh`,
and finds it works poorly at any other thinking level.
The owner also said:
"Expensive models are also not necessarily more trustworthy than cheap models."
It implemented and checked a multi-file change from a problem statement and evidence,
without step-by-step instructions,
though it also found the repository's existing diagnosis.

- Unprompted,
  it declines a hard or open objective:
  given "prove the Collatz conjecture" it answered that the problem is open and ended its turn.
  Told to attempt a proof,
  it did the work:
  a written proof attempt and compiled verifiers.
  Phrase a hard objective as attempts to make,
  and steer with one concrete next attempt when it stalls.
- It recovers from its own tool-argument slips without help.
- Long thinking at `xhigh` leaves the feed quiet.
  Check the process state and the time of the last event before calling a run stuck.
- It left scripts and compiled binaries in `/tmp`,
  and notes and sources in its working directory.
- It finds and uses repo docs that already describe the problem,
  including ones that name the cause,
  so a good result may come from the doc and not from its own work.
- It builds more than the problem needs.
  Unassisted,
  it wrote a prompt change plus a phrase-list post-filter and tests,
  a much larger change than the prompt-only change that measured the same.
  Read the diff's scope against the problem.
- Told to change only the words a task needs,
  it did:
  four one-line edits,
  after 35 turns and 56 tool calls of checking.
- Given the lead's reading as a suggestion to check,
  it disagreed with one item,
  cited the commit that added the sentence,
  and was right.
  Offer a reading as a suggestion,
  not an instruction.
- It wrote `.mjs` scratch files despite rule `NMJ`.

The driver's options and files:
`package/pi-shared/subagent-driver/README.md`.
Recorded experiments and protocol evidence:
`doc/agent/drive-pi-as-a-subagent.md`,
`doc/agent/drive-pi-as-a-coding-subagent.md`,
and `doc/agent/pi-goal-extension-driven-through-rpc.md`.
