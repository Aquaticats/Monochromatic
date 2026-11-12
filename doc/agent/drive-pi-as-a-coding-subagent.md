# Driving Pi as a coding subagent and watching it work

Purpose:
record how to run Pi (DeepSeek V4.1 Flash) as an implementer that edits files in a worktree,
how to watch it live,
how to steer it between turns,
and the quirks seen while doing so.
It builds on `doc/agent/drive-pi-as-a-subagent.md`,
which covers the RPC protocol itself.
The runs were made on 2026-11-07 against Pi 1.5.0 with `synthetic` / `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`.
The task they served is in `doc/handover/pi-goal-reviewer-human-input-rubric.md`.

## Setup that worked

### A worktree with dependencies

`git worktree add -b <branch> .claude/worktrees/<name> HEAD` in this repository gives a worktree that already has
`node_modules` and the git-ignored `dist` folders,
because the git-policy tooling copies ignored files in
(`package/git-policy/cli/README.md`,
the paragraph on what happens after `git worktree add` has filled a worktree).
`mise run //package/pi-plugin/goal:lint:types` and `:test:unit` ran in it with no install step.

The `git worktree add` call itself returned only after minutes,
on both worktrees made this way,
although the directory,
the branch,
and the copied `node_modules` were usable within seconds.
Redirecting its output to a file did not change that,
so a held pipe is not the cause.
The same README paragraph says the wrapper starts each tool's offline repair in the new worktree after the copy
(`pnpm install --offline --frozen-lockfile`,
a Cargo build directory clone,
and `mpm-shadow install --frozen-lockfile`).
That the add waits for those repairs is inferred from the delay and is not stated in the README.
Run the add in the background with its output in a file,
and check the worktree directly.

### A baseline before the subagent edits anything

Run the type check and the unit tests in the worktree first.
Here the baseline caught a failure that predated the work:
`src/review-contract.ts` imported `TSchema` from `@earendil-works/pi-ai`,
and pi-ai 1.5.0 no longer exports it,
so `lint:types` failed with TS2305 in the main checkout too.
Without the baseline,
that failure would have been blamed on the subagent's first edit.

### An isolated Pi home

`PI_CODING_AGENT_DIR=<dir>` makes Pi use that directory instead of `~/.pi/agent`
(`packages/coding-agent/src/config.ts`,
`ENV_AGENT_DIR`).
A directory holding only `settings.json` and a copy of `auth.json` was enough:

- `settings.json` with `packages` listing the installed provider package by path,
  here `~/.pi/agent/npm/node_modules/@benvargas/pi-synthetic-provider`.
  `pi list` showed exactly that package.
- `pi --list-models DeepSeek-V4.1-Flash` then listed `synthetic hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`.
- The copy of `auth.json` needs mode 600 in a mode 700 directory,
  and must be deleted afterwards.

This drops every other extension of the owner's setup,
including `thinking-default`,
which overrides `--thinking`.
In the isolated home `--thinking xhigh` is therefore the level that applies.
The level was set on the command line and was not read back with `get_state` in these runs.

The `commands` list of the isolated home still contained many `skill:` entries.
Where they were loaded from was not determined,
so do not assume the isolated home carries no skills.

### Tools

`--tools` is an allowlist of tool names or patterns to enable,
built-in and extension tools alike
(`pi --help`:
"Comma-separated allowlist of tool names or patterns (*) to enable").
The implementer ran with `--tools read,edit,write,grep,find,ls`:
no shell,
so it cannot run tests or use git.
The lead ran the checks and committed.
That kept the model inside the worktree by construction,
not by instruction.

## The driver

The script `pi-rpc-agent.ts`,
which the runs in this doc used,
started one Pi process in RPC mode and recorded everything worth watching.
It differs from the client in `doc/agent/drive-pi-as-a-subagent.md` in four ways:

- It writes a compact feed of tool calls with their arguments,
  tool results,
  assistant text,
  and token cost.
- It reads steers from an inbox folder.
  A file dropped there is sent as `steer` while the agent runs and as a new `prompt` while it is idle.
- It answers any dialog request with `{ type: 'extension_ui_response', id, cancelled: true }` and records it,
  so a dialog cannot stall the session.
- It stays alive across tasks until an idle timeout,
  a stop file,
  a turn cap,
  or a hard wall time.

The runs recorded in this doc set `PI_TOOLS=read,edit,write,grep,find,ls`,
`PI_THINKING=xhigh`,
and `PI_EXTRA_ARGS=--no-session`,
and the version of the driver used in those runs hard-coded them.
None is needed:
unset,
Pi keeps its own thinking level and every tool,
and it saves the session,
which the owner wants kept.

The package `package/pi-shared/subagent-driver` replaced that script on 2026-11-08.
It kept this behaviour and added failure handling and tests.
Options are now command-line flags and the files have fixed names in one run folder,
so the `PI_*` variables,
`WALL_MS`,
`IDLE_MS`,
`TURN_CAP` and tagged file names in the records of this doc name the script as it was.
Its `README.md` lists the options and files,
and `.agents/skills/pi-as-subagent/SKILL.md` has the command to start a run.

## Watching it live

Tail the feed through `jq` and let the Monitor tool turn each line into a notification:

```sh
tail -n 0 -f <run>/feed.jsonl | jq --unbuffered --raw-output \
  'select(.kind=="tool_start" or .kind=="tool_end" or .kind=="assistant" or .kind=="settled" or .kind=="dialog") |
   "\(.t) \(.kind) \(.tool // "") \(.isError // "") \(.args // .result // .text // "" | tostring | .[0:220])"'
```

Use `tail -n 0 -f` when re-arming after an expiry.
`tail -n +1 -f` replays the whole feed.
Two monitors on the same feed each deliver every event,
so let the old one expire before judging the volume.

Record shapes the feed relies on:

- `tool_execution_start` carries `toolName` and `args`.
- `tool_execution_end` carries `isError` and `result.content[].text`.
  An `edit` result also carries `details.diff`,
  which shows the exact change as it happens.
- An assistant `message_end` carries `content` parts of type `text`,
  `thinking`,
  and `toolCall`,
  plus `usage` with a `cost.total`.

## Quirks seen

- The model passed a path as the `pattern` of `find` and got "No files found matching pattern" with `isError` false.
  `find` takes a glob pattern and a separate `path`.
  A soft miss and a success look alike in the feed unless the result text is read.
  The next `find` with a bad directory returned a real error from `fd` with `isError` true.
- After those misses the model listed directories one level at a time and reached the file.
  It did not stop or ask.
- The model issues independent calls together,
  such as a `read` and a `grep` in one assistant message.
- It verified more than a one-line import change needs:
  it confirmed the export in `typebox`,
  read pi-ai's generic `Tool` type,
  edited,
  re-read the file,
  and grepped for other uses.
  A small first task still took 34 seconds from prompt to settle.
- Gaps of 20 seconds or more between events occurred during thinking at `xhigh`.
  Silence in the feed is not a hang.
Every timing in this doc was measured on the `synthetic` provider on 2026-11-07,
which the owner describes as a relatively slow provider for this model,
with roughly three times the prefill and generation speed expected later.
Read the seconds as that provider's,
  not as the model's.
The counts of tool calls and turns do not depend on speed.
On 2026-11-08 the owner reported that `deepseek-v4.1-flash` is available through the `neuralwatt` provider.
`pi --list-models flash` lists `neuralwatt` with `deepseek-v4.1-flash`,
`deepseek-v4.1-flash-flex`,
and `deepseek-v4.1-flash-speed`,
each with a 1.0M context and images,
next to the plain and `-flex` and `-speed` variants of `deepseek-v4-flash`.
Their speed against `synthetic` is measured in "Provider speed,
  measured 2026-11-08T01:01:36-05:00".
A driven session needs `neuralwatt` credentials in its isolated Pi home before it can use one.
- A second task sent through the inbox while the agent was idle became a new prompt in the same session,
  and the model started on it by reading the files its context did not yet hold.
- No dialog request arrived in the runs so far.

## Provider speed, measured 2026-11-08T01:01:36-05:00

Method:
a fresh `pi --mode rpc --no-session --no-tools` process per run in the owner's real home,
so `thinking-default` applied and `get_state` read `xhigh` on every arm.
`--no-tools` and `--no-session` are for this measurement only:
a first attempt with every tool enabled produced multi-turn runs (thousands of output tokens and hundreds of thousands
of cache-read tokens for a one-word reply) whose tool time counted as generation time,
and it was discarded.
Arms alternated within each repeat,
three repeats each:
`neuralwatt` / `deepseek-v4.1-flash`,
`neuralwatt` / `deepseek-v4.1-flash-speed`,
and `synthetic` / `hf:deepseek-ai/DeepSeek-V4.1-Flash-1017`.
Two prompts,
each led by a unique line so a prefix cache could not serve it:
a short one asking for the numbers 1 to 200,
and a 472 KB file (about 130000 input tokens on every arm).
Generation rate is output tokens over the time from the first message update to `agent_settled`;
output tokens include thinking.
Prefill is the time from sending the prompt to the first message update.
The probe script was `pi-rpc-speed.ts` in scratch and is not kept.

Results:

- Generation,
  short prompt,
  tokens per second over three runs:
  `neuralwatt` 351,
  197,
  368 (median 351);
  `neuralwatt-speed` 284,
  567,
  826 (median 567);
  `synthetic` 161,
  132,
  163 (median 161).
  Every `neuralwatt` run beat every `synthetic` run,
  about twice as fast at the median.
  The `-speed` variant's three runs overlap the plain model's,
  so the two are not separated by this sample.
- First update after the large prompt:
  `neuralwatt` 5.2,
  5.7,
  5.6 seconds;
  `neuralwatt-speed` 11.0,
  4.0,
  5.2 seconds;
  `synthetic` 3.9,
  3.3,
  10.3 seconds.
  The medians are 5.6,
  5.2 and 3.9 seconds,
  and each of the `-speed` and `synthetic` arms had one slow run,
  so `synthetic` is not slower at prefill here.
- Time to the first update on the short prompt was 0.35 to 0.62 seconds on every arm,
  with overlapping ranges.

Limits:
three runs per cell,
one time of day,
one prompt of each size.
The tokens-per-second figures for the large prompt are meaningless (tiny outputs,
or text arriving at once) and are not
reported.
Cached-input counts differ by provider,
so prefill is compared by time and not by token rate.
Behavior of the neuralwatt model under real tool-using work is not measured here.

## The detailed-brief run

Task:
change the goal reviewer rubric and add tests.
The brief named the files,
the exact new sentences,
the function names,
and the test cases.

- From prompt to settle:
  about 250 seconds on the `synthetic` provider.
- 49 assistant messages,
  67 tool calls (29 `read`,
  25 `grep`,
  6 `edit`,
  5 `ls`,
  2 `find`),
  and 2 tool errors:
  the `find` with a bad directory,
  and a `read` with an offset past the end of the file.
- Usage as reported:
  85006 input tokens,
  26904 output tokens,
  0.136 in cost.
  The input figure is the `usage.input` field summed over messages,
  and whether it excludes cached tokens was not checked.
- No dialog request.
- The edits matched the brief exactly,
  including the verbatim rubric sentences.
  `lint:types`,
  `test:unit` and `lint` all exit 0 after a rebuild,
  with no correction round.
- One review finding:
  its tests sit in a second `describe` that reuses the existing suite name.

### The stale-build trap

Both the type check and the unit tests failed on the model's correct edit before a rebuild.
The test files import the built artifact (`../dist/final/node/index.mjs`),
so until `mise run //package/pi-plugin/goal:build` runs,
the type check reports the old signature (TS2353 on the new field)
and the tests see the old prompt (assertion failures that look like logic errors).
This was the lead's verification mistake,
not the model's.
Run `build` before `lint:types` and `test:unit` in this package,
and check that the new test names appear in the output.

## The unassisted run

The owner's calibration is that DeepSeek V4.1 Flash at `xhigh` is about a Sonnet 5.5 low-level model.
The owner asked whether it works satisfactorily with less handholding.
The brief gave the problem,
the evidence,
and the owner's intent,
and nothing about files,
names,
wording,
steps,
or tests.
The model had a shell,
the repository's own checks to find,
and the read-only git shim of that time.

What the feed showed:

- It oriented for about three minutes before the first edit,
  reading nine source files,
  the existing tests,
  and the oxlint configuration.
  It found the reviewer prompt,
  the call site,
  and the `mode` signal without pointers.
- It applied the repository's rules from `AGENTS.md`:
  every command starts with `cd -- <path> &&`,
  and its code follows the repository's TSDoc and destructured-parameter style.
- It also found and read this repository's own write-up of the problem under `doc/`.
  The unassisted run therefore had the diagnosis from the repository,
  not only from the brief.
- It chose a different design from the detailed-brief run:
  a post-filter module that replaces reviewer guidance matching phrase lists with a fixed instruction,
  a mode-keyed rubric builder,
  audit fields on the persisted denial event that keep the reviewer's original text,
  validation of those fields in `events.ts`,
  and a decision not to rewrite a human's own manual-rejection text.
- It wrote a dedicated unit test file for its new module,
  extended the built-artifact runtime verifier,
  and made the fake session context take a mode so the tests reach the production wiring.
- It hit the stale-build trap,
  read the error as an out-of-date artifact,
  and ran `build` without being told.
- It ran two mutation checks on its own:
  it broke a guard condition with `perl -pi`,
  rebuilt,
  confirmed that tests failed,
  and restored the line.
  It did the same to the prompt builder.
  The repository requires a guard test to be shown to fail.
  After each restore the lead checked the source by hand.
- It ran `lint`,
  tallied the findings by rule,
  fixed them in rounds,
  and reached 0 warnings and 0 errors.
  It captured real exit codes with `${PIPESTATUS[0]}` after noticing that `| tail` hides them.
- One lint run printed a 5924-character stack trace from inside the repository's oxlint plugin
  (`readonlySourceEvidence`,
  `openSemanticFile`) instead of findings.
  The cause of that trace was not read.

Whether the result is satisfactory is judged in `doc/handover/pi-goal-reviewer-human-input-rubric.md`
and recorded in `doc/agent/pi-goal-extension-driven-through-rpc.md` when the comparison is done.

## A small wording change on `neuralwatt`, run 2026-11-08T13:10:43-05:00

Task:
say "Claude Code native" where current instruction documents mean Claude Code's own subagents.
The brief gave the owner's words,
the lead's reading of each candidate as a suggestion,
the files not to touch,
and a limit to the words the change needs.
The run used `neuralwatt` / `deepseek-v4.1-flash` in the owner's real home,
with every tool enabled and the read-only `git` shim first on `PATH`.
`get_state` read `xhigh`.

- From prompt to settle:
  about 256 seconds,
  35 turns and 56 tool calls.
- `sessionStats` reported a cost of 0.44.
  The driver's own `tokens.costTotal` said 0.044,
  so read the cost from `sessionStats`.
  The cause of the difference was not read.
- Four one-line edits and no other file changed.
  No tool touched `/tmp`,
  and no dialog request arrived.
- One tool error:
  this repository's search shadow refused `rg -rn`,
  explaining that `-r` means `--replace`,
  and the model read that and carried on.
- It measured the changed rule against the rule-length limit by reading the file.
- It disagreed with one suggestion and cited the commit that added the sentence.
  The disagreement was right:
  "a Claude subagent is a second pass" contrasts vendors,
  so naming a runtime there would excuse a Claude model run through Pi.
- It wrote two `.mjs` measurement scripts against rule `NMJ`,
  and deleted them.
- Its report named documents the lead had missed,
  the Claude Code plugin READMEs.

The follow-up for those ran as a new process with `--session <session file>` after the first driver had exited.
The session then held both user messages and the model used its earlier line list without being told it again.
That took about 76 seconds to settle,
8 turns and 10 tool calls,
and made four more one-line edits.

`pi auth check --provider neuralwatt --model deepseek-v4.1-flash --json --no-refresh`
printed `{"status":"invalid","reason":"invalid_state"}` before these runs,
and the model worked in all of them.

## The skill's recipe with real tools, run 2026-11-08T13:34:56-05:00

The shell blocks of `.agents/skills/pi-as-subagent/SKILL.md` ran as written,
with only the placeholders filled in,
against a throwaway Git folder holding a copy of `AGENTS.md`.

The prompt had Pi run `git status --short` and `git commit --allow-empty --message shim-check`.
The status ran,
the shim refused the commit with exit status 1,
and the throwaway held no commits afterward.
The refusal is the run's only entry in `toolErrors`.
A text file written to `inbox_<tag>` after the first settle reached Pi as a new prompt,
and Pi read rule `ATH` from `AGENTS.md` and replied with it.
The unit exited 0 after 87 seconds,
of which the driver's `IDLE_MS` wait was 60.

The waiting step changed because of a check on the same day.
`timeout 4 tail --pid="$(systemctl --user show --property=MainPID --value <unit>)" --follow /dev/null`,
pointed at a unit that had already run `/usr/bin/false`,
ended with status 124,
so it waited out the whole timeout.
`systemd-run --user --wait` on the same program ended after 16 ms with status 1 and printed the unit's result.
On a unit over `RuntimeMaxSec=2` it ended after 2.2 seconds with status 1 and the result `timeout`.

Still weak:
the driver and the shim exist only as code blocks in this doc,
copied out with `awk` for each run,
so nothing type-checks,
lints or tests them.
Resolved 2026-11-08:
the driver moved to `package/pi-shared/subagent-driver`,
and the shim went,
because the owner decided
"We have to be happy with Pi committing directly on main.
We are not building another git wrapper or shim when we're already so busy with one."

## Cross-vendor review of the recipe, run 2026-11-08T13:49:08-05:00

`openai/gpt-6.1-sol` read the skill and this doc with no tools and named nine defects.
Each was checked against the code before it was fixed.
Pi's own `docs/rpc.md` confirms the `agent_end` one:
that event ends one low-level run,
and retries,
compaction,
steering or follow-up work can still come before `agent_settled`.

A stand-in `pi` that speaks just enough of the protocol took the driver down each failure path,
before and after the fixes:

- A rejected first prompt ran to the wall clock and exited 0 with `endReason` `idle`.
  It now ends in 77 ms,
  exits 1,
  and says `prompt-rejected`.
- A `pi` that crashed at start exited 0 with `idle`.
  It now exits 1.
- A message dropped between `agent_end` and `agent_settled` went out as a new prompt.
  It now goes as a steer.
- A message Pi refused was deleted.
  It now stays as `.failed-<name>`.

Two endings were only run after the fixes:
a `pi` that exits mid-run ends with `pi-exited` and status 1,
and one that never settles ends with `wall-clock` and status 1.

`git config -- example.value --get` passed the shim and wrote the value,
because the shim looked for a read-only flag anywhere in the arguments.
It now reads only the options before the first operand,
and the same command is refused.

The real model then ran the recipe as written.
A message dropped during a tool call went as a steer,
one dropped after settling went as a new prompt,
and a new unit with `PI_SESSION` resumed the session and named the commands from its first prompt.

The two paths that left unrun went against the real Pi at 2026-11-08T13:52:56-05:00,
each beside a control that had to behave differently.

A session file whose path holds spaces resumed through `PI_SESSION`:
the session held five user messages and the model named the commands from its first prompt.
The same path through `PI_EXTRA_ARGS` was split on the spaces.
Pi opened a new session in a file named for the first word,
exited 0,
and the model replied that the session held no such request,
so that route fails silently.

A prompt consumed by a one-line extension command came back with the disposition `handled` and no `agent_settled`.
The driver ended with `idle` and status 0 after 9 seconds.
With the line that counts a handled prompt removed,
the same run went to its 45-second wall clock and ended with `wall-clock` and status 1.

## Tool and feed quirks

- A multi-block `edit` is rejected whole when one `oldText` does not match.
  The model recovered by grepping the file for the current text and sending the blocks again.
- The model guessed file extensions (`.d.mts` for a `.d.ts` package) and got silent empty results,
  then switched to a recursive search.
- The `bash` tool result carries `structuredContent` with `output`,
  `truncated`,
  `exit_code`,
  and `wall_time_seconds`.
  The driver's feed keeps only the first 500 characters of a result and ignores `structuredContent`,
  so a long lint run shows its tail and no verdict.
  Record `exit_code` in the feed next time.
- Two Monitor tasks on one feed deliver every event twice.
  Stop the old one with `TaskStop`.
  `tail -n 0 -F` waits for a feed file that does not exist yet.
- To wait for a subagent without polling,
  start its unit with `systemd-run --user --wait` in a background command.
  Following its main process ID with `tail --pid` waits out the whole timeout when the unit has already finished.

## The driver package, run 2026-11-08T14:39:12-05:00

The package's unit tests run the real entry file against a stand-in `pi`.
All its suites passed,
and its lint finished with no warnings or errors.
Its guards were then removed from the source,
all but one in a single combined test run and that one alone,
and each removal made its named test fail.
That one needed a stronger test:
in the combined run,
the removed wall-clock guard also ended a handled prompt as `idle`,
which hid the missing count,
so the test now asserts the count.

The real Pi then ran the skill's command unchanged in the main checkout
and removed this doc's section on the read-only git shim.
It made commit `db7471a3e2` on `main` itself:
signed,
one file,
55 lines removed,
and on `origin/main` when it checked.
The run took 1 minute 55 seconds,
20 turns and 24 tool calls,
at a cost of 0.0141 by `sessionStats`,
and ended with `endReason` `idle` and status 0.
`sessionStats` named the session file,
and `lastAssistantText` held the whole report.

Pi ran `git push origin main` after the commit although the prompt said the commit auto-pushes,
and the push found nothing to send.
It also found a leftover the prompt had not named:
a sentence in `## The unassisted run` that pointed at the removed section.

## Many agents at once, recorded 2026-11-08T17:18:47-05:00

Run with up to sixteen Pi agents in their own worktrees through the driver.

### Prove a merge landed

A coordinator reported two branches as merged when they were not.
`git merge ... | tail` exited 0 because `tail` was the last command of the pipe,
which hid `would be overwritten by merge`.
The check that holds is the count of the branch's commits main lacks,
which must be zero after the merge.

### An agent in a worktree can still work in the main checkout

One agent given a worktree ran its builds and edits in the main checkout,
so those builds did not test its worktree's code,
and the uncommitted files it left there blocked merges of other branches.
Its feed proved it.
The prompt now names the worktree and tells the agent to stay inside it,
as `.agents/skills/pi-as-subagent/SKILL.md` says.
Reading the feed needs a regular expression,
because the feed truncates tool arguments and so a line is not always valid JSON.

### The unit count can exceed the agents

An agent may start capped helper units of its own.
One documentation agent started some inside its worktree,
and they counted towards `systemctl --user list-units 'pi-*'`,
so the count of active units is not the count of agents.

### An agent can spend its time limit on one command

One agent lost two hours in `du -sh` over a build directory.
Another ended at its wall-clock limit with its commits pushed and no report.
Resuming its session with a short deadline and the order to report recovered the report.

## Retrying a failed attempt, read 2026-11-09T08:05:00+00:00

Pi retries a failed assistant turn itself,
and this machine did not on that day:
the owner's `~/.pi/agent/settings.json` held `"retry": {"enabled": false}`,
so one transient provider failure ends a run.

The file was read again at 2026-11-11T22:33:00+00:00 and held
`retry.enabled` true,
`retry.maxRetries` 8,
`retry.baseDelayMs` 1000 and
`retry.provider.maxRetryDelayMs` 0,
while the runs of that evening reported `maxAttempts` 3 and delays of 1 s, 2 s and 4 s in each `auto_retry_start`.
So a run here now meets a 429 only after Pi's retry has finished with it,
and the driver leaves every 429 to that retry,
answering with its own attempts only the model errors that are no 429
(`package/pi-shared/subagent-driver/README.md`, "Which layer retries what").

The settings `docs/settings.md` names in Pi 1.5.0 are
`retry.enabled` (`true` by default),
`retry.maxRetries` (3),
`retry.baseDelayMs` (2000),
`retry.maxAgentDelayMs` (60000),
and the provider-level `retry.provider.*` group.

Pi also answers RPC commands about it.
`{"type": "set_auto_retry", "enabled": true}` turns the agent's retry on,
and `{"type": "abort_retry"}` cancels one that is waiting
(`docs/rpc-commands.md`,
section "Retry").
The first is not a per-run switch,
which a real invocation showed:
with `PI_CODING_AGENT_DIR` at a throwaway folder holding
`{"retry": {"enabled": false, "maxRetries": 3}}`,
`pi --mode rpc` answered `{"command": "set_auto_retry", "success": true}`,
and the folder's `settings.json` then read `"enabled": true`.
The source says the same:
`session.setAutoRetryEnabled` calls `settingsManager.setRetryEnabled`,
which writes `retry.enabled` into the agent's settings and saves it
(`packages/coding-agent/src/core/agent-session.ts`,
`packages/coding-agent/src/core/settings-manager.ts`).
So a driver that used it would change the owner's settings,
and would leave them changed if Pi were killed before it could put them back.

What Pi treats as retryable comes from `isRetryableAssistantError`
(`packages/ai/src/utils/retry.ts`):
overload and `5xx` wording,
`429` and rate-limit wording,
network and stream failures,
and the timeouts,
while `insufficient_quota`,
`billing`,
plan-limit text and `429` bodies naming a usage limit are not retried.
Context overflow is excluded before that check,
because compaction handles it (`_isRetryableError`).

An errored assistant message is not sent back to the provider:
`transform-messages.ts` drops a message whose `stopReason` is `error` or `aborted`
before building the request,
"the model should retry from the last valid state".
That is why answering a settled error with a new prompt in the same session
reads the same state as resuming the session file by hand,
which is what the coordinator did by hand for the runs of 2026-11-09.

A real Pi 1.5.0 on `neuralwatt`/`deepseek-v4.1-flash` took a second prompt in one session after it settled,
which is what that retry sends:
each prompt answered `"disposition": "started"`,
and each was followed by `agent_start`,
`turn_start`,
one assistant message,
`agent_end` and
`agent_settled`.
The probe asked two questions that each answer with one word and wrote its session to a scratch file,
so no session folder and no setting of the owner's was touched;
the first settle there was a success rather than an error,
so what it measures is that a settled session takes another prompt,
not the error case itself.
