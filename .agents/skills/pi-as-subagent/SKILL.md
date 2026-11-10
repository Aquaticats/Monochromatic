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
Also tell every agent,
in every prompt,
to run Git only as plain `git` and never to use real Git:
no `/usr/bin/git`,
no other absolute path to a Git binary,
and no other route around the wrapper that plain `git` reaches.
Real Git skips every guard against discarding work and every commit policy,
and agents here have committed and run `checkout --` through it.
The owner:
"Agents using real git is pretty bad.
As a duct tape patch,
put in the skill to always tell the agents to not use real git."
Also tell every agent never to print the environment (`env`,
`printenv`,
`set`),
even through a filter,
never to read another process's (`/proc/<pid>/environ`,
`ps e`),
and to name the one variable it wants:
an agent's `env | rg` put a real GitHub token in its session,
and all command output goes to the model provider.
The driver enforces that bound too:
it hands Pi its own environment with every variable whose name holds
`TOKEN`,
`SECRET`,
`KEY`,
`PASSWORD`,
`CREDENTIAL` or
`AUTH` removed
(`package/pi-shared/subagent-driver/README.md`),
so a dump shows the names a command may read and no value it may not.
Keep the sentence in every prompt anyway:
a command can still print a token it finds in a file.
Name the worktree in the prompt of an agent given one,
and tell it to run every command and make every edit inside it:
one agent ran its builds in the main checkout,
so they did not test its worktree's code,
and the uncommitted edits it left there blocked merges of other branches.
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
"Pi doesn't have a limit to number of concurrent subagents."
The practical limit is 20:
the owner said
"Try 20,
and if it doesn't blow up (in terms of api rate limits),
we're permanently raising the practical limit to 20,"
and a trial at 20 agents on 2026-11-09 met no rate-limit error.
On going higher the owner said:
"no,
that's definitely going to blow up."
A run on another provider's account is outside it.
The owner on 2026-11-10:
"Kimi trials don't count within 20 slots because 20 is neuralwatt's effective concurrency and kimi runs on Synthetic."
The owner the same day:
"You can use -flex postfix on neuralwatt models.
They have their own separate pool of 22 slots.
They're around 100% slower than standard in practice."
So `neuralwatt/deepseek-v4.1-flash-flex` is a pool of its own.
Picking all three pools,
standard Neuralwatt,
flex and `opencode-go`,
the owner wrote:
"In practice,
opencode-go may have worse inference quality and worse speed than neuralwatt's flex,
so prefer neuralwatt."
New runs fill standard Neuralwatt,
then flex with twice the wall clock,
then `opencode-go`.
Audits and reviews sit outside that limit and use another model.
The owner on 2026-11-09:
"Please use gpt-6.1-sol with the openai provider for audits and reviews.
Those don't count within the 20 limit."
They run through `codex`,
not through Pi.
The owner,
later that day:
"Use `codex` to run openAI reviews and audits instead.
`--dangerously-bypass-approvals-and-sandbox` has to be passed to codex because codex's auto mode has tons of issues which I'm not listing here."
and
"Still pass in xhigh explicitly for codex."
The command is rule `XVW` in `AGENTS.md`.
The owner then limited those runs:
"Keep GPT runs small.
GPT models cannot be trusted on any long horizon work."
and moved the long ones:
"Use Claude Sonnet on long horizon reviews and audits."
That holds in every harness,
Pi included:
rule `LHZ` in `AGENTS.md`.
Each run is its own driver process with its own run folder and unit name.
Runs in one checkout share its files,
so two in one crate build each other's half-finished edits.
Give a run its own worktree whenever anything else is working in the same packages.
Start a run for every independent queued item up to that number.
Resource limits that add up to more than the host has are ceilings,
not usage,
and are no reason to hold work back.

The owner on runs that do not come back:
"Regularly,
check in on subagents that should have finished their jobs (based on time estimations 90%+ percentile) but haven't returned."
`watch-overdue-pi-runs` is that check,
and it is one command in the shape rule `MWK` asks for:
it ends when a run is past the 90th percentile of the finished runs in `--runs-folder`,
or when its `--deadline-seconds` pass,
and prints one line per run to act on and nothing else.
It computes the limit itself from those runs,
so nothing here needs to be remembered.
When a line names a unit,
`check-in-on-pi-runs --runs-folder=<folder> --unit=<that unit>` prints its open tool call and the processes under the agent
with their age,
state and kernel wait.
Both commands and their options are in `package/pi-shared/subagent-driver/README.md`.
A quiet feed is usually a long test run.
A message in `inbox/` reaches Pi only between tool calls,
so a tool call that will never return gets the message first,
and then has its hung process ended by process number.

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
      --property=RuntimeMaxSec=2500 --property=MemoryMax=3G --property=CPUQuota=200% --property=TasksMax=300 --property=IOWeight=1 \
      --setenv=PATH="$(mise exec -- printenv PATH)" --setenv=HOME="$HOME" \
      "$(mise exec -- bash --noprofile --norc -c 'command -v node')" \
      "$PWD/package/pi-shared/subagent-driver/src/drive-pi-subagent.ts" \
      --run-folder=<run> --working-folder=<its worktree, or "$PWD"> --prompt-file=<prompt-file> \
      --provider=neuralwatt --model=deepseek-v4.1-flash \
      --wall-seconds=2400 --idle-seconds=60 --turn-cap=<sized from past runs>
    ```

    Size `--turn-cap` to the task from past runs at every launch.
    The owner,
    2026-11-09:
    "Always Size turn limits to the task based on past data."
    Read what the finished runs of the same kind used,
    narrowing by the run folders' names:

    ```sh
    # run from anywhere; <runs> holds the run folders, and the pattern picks the runs like this one
    jq --slurp '[.[] | select(.endReason == "idle" and (.turns | type == "number")) | .turns] | sort
      | {runs: length, median: .[length / 2 | floor], p95: .[length * 0.95 | floor], highest: .[-1]}' \
      <runs>/*-findings-fixed-*/summary.json
    ```

    Set the cap a quarter above `highest`,
    so a run as long as the longest that finished still has room.
    When too few runs of that kind have finished to say anything,
    read every run instead.
    A cap exists to end a run that stopped getting anywhere;
    one that fires on a working run costs a resume and leaves its work uncommitted.
    Measured at 2026-11-09T07:42:00-05:00 over 254 runs:
    finished fix rounds from a review or audit used a median of 126 turns,
    a 95th percentile of 329 and at most 358,
    every finished run at most 377,
    and a fixed cap of 400 ended a run with uncommitted work after 154 of its 180 minutes.

5.  Read progress from `<run>/feed.jsonl` when asked.
    To steer,
    write a text file under a name starting with a dot in `<run>/inbox/`,
    then rename it to one without the dot,
    because the driver reads any other name at once.
    Pi gets it as a steer while it runs and as a new prompt while it is idle,
    and `--idle-seconds` is how long the driver stays up for one after Pi settles.
    An entry must be an ordinary file:
    a named pipe is set aside rather than waited on.
    A message Pi refused,
    or an entry that cannot be read,
    stays there as `.failed-<name>`,
    and a later message of the same name that is also refused gets a number after it,
    so nothing replaces an earlier one.
    To end the run early,
    create `<run>/stop`.
6.  Read the result from `<run>/summary.json`,
    which appears last,
    so its presence means the driver has finished.
    `piExit` is there only when Pi's exit was seen,
    so a summary naming `pi-not-stopped` claims nothing about Pi having gone:
    the unit's own cleanup ends that process,
    and its `RuntimeMaxSec` is what bounds the driver when a file system call or a named pipe keeps it alive past the
    summary.
    `endReason` says how it ended:
    `idle` and `stop-file` exit 0,
    and `prompt-rejected`,
    `pi-exited`,
    `turn-cap`,
    `wall-clock`,
    `driver-failed`,
    `model-error` and `pi-not-stopped` fail the unit,
    and a Pi that left with a failing status after the run ended as `idle` fails it as `pi-exited`.
    `lastAssistantText` holds the whole final report,
    `sessionStats` holds the cost and names the session file,
    `modelError` holds what the model said when it settled on an error nothing recovered from,
    `modelErrorAttempts` says how many attempts that took,
    `retries` counts the attempts after a model error,
    `rateLimitErrors` counts the model errors that named a rate limit or a quota,
    and `feedProblem` names why the feed is incomplete when it is.
    The driver answers a transient model error with another attempt on its own:
    a provider server error,
    a request timeout,
    a dropped connection,
    or a rate limit is retried up to `--retry-cap` times with a growing wait,
    and anything else ends the run on its first error as before.
    Each attempt is a `retry` line on `feed.jsonl`,
    and `summary.json` says what they were,
    so a run that needed them reads apart from one that did not.
    A run that used up its attempts still ends `model-error`,
    which is when to start it again from the same session file as step 7 says.
7.  For a follow-up after the driver has exited,
    start the same unit under a new name,
    with a new run folder,
    a new prompt file,
    and `--session="<session file>"`.
8.  Read what it committed with `git log` and `git show`,
    fix anything wrong with further commits,
    regenerate anything `file-enforcer` owns,
    and merge a worktree's branch into `main`.
    The owner wrote "I don't think you'd need to rerun the same lints or tests another agent did, do you?":
    read which suites the agent's report names instead of running them again.
    After merging `main` into the branch,
    run only the compile or type check,
    which is what the agent could not have run on the combined tree,
    and skip that too when `main` was already in the branch.

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
