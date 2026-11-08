---
name: pi-as-subagent
description: Use when delegating a task to Pi as a subagent over RPC.
---

# Pi as a subagent

Pi is a second subagent runtime here,
beside Claude Code native subagents,
and a rule that says "subagent" without naming a runtime binds it too.

Drive `pi --mode rpc --provider neuralwatt --model deepseek-v4.1-flash` as a child process started in the repository,
where Pi loads `AGENTS.md` (from another folder rule `ATH` was not in its context).
In every prompt cite rule `ATH`,
because the model writes scratch to `/tmp` unprompted,
and quote the synthetic-load ban (rule `NLD`).
Leave `--tools` unstated:
it is an allowlist for every tool,
extension tools included,
so naming some removes the rest.
Pi saves the session by default and the owner wants that kept,
so a run can be read back or resumed with `--session <path|id>`.

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
2.  Make a worktree for the run with `git worktree add -b <branch> .claude/worktrees/<name> HEAD`,
    in the background,
    and go on only once it has succeeded:
    Pi starts in that folder.
3.  Copy the recorded driver and the read-only `git` shim into a fresh folder under `${HOME}/temp/agent`.
    A commit here auto-pushes and the model has a shell,
    so the shim keeps Pi from committing and the lead commits:

    ```sh
    # run from ~/Monochromatic
    doc=doc/agent/drive-pi-as-a-coding-subagent.md
    mkdir --parents <run>/shim
    awk '/^\/\/ pi-rpc-agent\.ts:/{p=1} p&&/^```/{exit} p' "$doc" > <run>/pi-rpc-agent.ts
    awk '/^#!\/usr\/bin\/env node/{p=1} p&&/^```/{exit} p' "$doc" > <run>/shim/git
    chmod 700 <run>/shim/git
    ```

4.  Write the prompt to a file.
    Beside rules `ATH` and `NLD`,
    give it:

    - the owner's words as the requirement,
      quoted;
    - your own reading as a suggestion it may reject;
    - the files it must not edit,
      including everything `file-enforcer` generates;
    - that the lead commits,
      because `AGENTS.md` tells agents to commit and the shim refuses;
    - a final report of each change,
      each disagreement,
      and anything it could not do,
      because it cannot ask you.

5.  Start it as its own capped unit,
    in the background.
    `--wait` makes this one command end when the unit ends,
    print the unit's result,
    and exit with its status;
    following `MainPID` with `tail --pid` waited out its whole timeout on a unit that had already finished.
    The driver defaults to another provider,
    so set both `PI_PROVIDER` and `PI_MODEL`:

    ```sh
    # run from ~/Monochromatic
    systemd-run --user --wait --unit=pi-<task> --collect \
      --property=RuntimeMaxSec=2500 --property=MemoryMax=3G --property=CPUQuota=200% --property=TasksMax=300 --property=IOWeight=10 \
      --setenv=PATH="<run>/shim:$(mise exec -- printenv PATH)" --setenv=HOME="$HOME" --setenv=REAL_GIT=/usr/bin/git \
      --setenv=PI_PROVIDER=neuralwatt --setenv=PI_MODEL=deepseek-v4.1-flash \
      --setenv=WALL_MS=2400000 --setenv=IDLE_MS=60000 --setenv=TURN_CAP=80 \
      "$(mise exec -- bash --noprofile --norc -c 'command -v node')" \
      <run>/pi-rpc-agent.ts <run>/out _<tag> <worktree> <prompt-file>
    ```

6.  Read progress from `<run>/out/feed_<tag>.jsonl` when asked.
    To steer,
    write a text file under a name starting with a dot in `<run>/out/inbox_<tag>/`,
    then rename it to one without the dot,
    because the driver reads any other name at once.
    Pi gets it as a steer while it runs and as a new prompt while it is idle,
    and `IDLE_MS` is how long the driver stays up for one after Pi settles.
    A message Pi refused stays there as `.failed-<name>`.
    To end the run early,
    create `<run>/out/stop_<tag>`.
7.  Read the result from `<run>/out/summary_<tag>.json`.
    `endReason` says how it ended:
    `idle` and `stop-file` exit 0,
    and `prompt-rejected`,
    `pi-exited`,
    `turn-cap` and `wall-clock` fail the unit.
    Its `lastAssistantText` is cut at 3000 characters and `tokens.costTotal` undercounts,
    so take the cost from `sessionStats` and the full report from the session file it names:

    ```sh
    # run from anywhere
    jq --raw-output 'select(.type == "message" and .message.role == "assistant") | .message.content[]? | select(.type == "text") | .text' <session file>
    ```

8.  For a follow-up after the driver has exited,
    start the same unit under a new name and tag,
    with a new prompt file and `--setenv=PI_SESSION=<session file>`.
9.  Review the diff as the lead,
    regenerate anything `file-enforcer` owns,
    and commit.

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
  and the recorded driver declined all of them.
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

Recorded experiments and protocol evidence:
`doc/agent/drive-pi-as-a-subagent.md`,
`doc/agent/drive-pi-as-a-coding-subagent.md`,
and `doc/agent/pi-goal-extension-driven-through-rpc.md`.
