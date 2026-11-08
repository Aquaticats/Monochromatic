---
name: pi-as-subagent
description: Use when delegating a task to Pi (DeepSeek or another model) as a subagent over RPC.
---

# Pi as a subagent

Drive `pi --mode rpc --provider neuralwatt --model deepseek-v4.1-flash` as a child process.
Leave `--tools` off:
it is an allowlist for every tool,
extension tools included,
so naming some removes the rest.
Pi saves the session by default and the owner wants that kept,
so a run can be read back or resumed with `--session <path|id>`.

## Protocol

- One JSON command per line on stdin,
  one JSON record per line on stdout,
  logs on stderr.
  Split stdout on LF only,
  because Unicode line separators are valid inside JSON strings.
  Closing stdin ends the process.
- Install the stdout listener before the first command.
  Every command takes an optional `id` that its `response` repeats.
- `{"id":"c1","type":"prompt","message":"..."}` answers `disposition: "started"`:
  accepted,
  not finished.
  A prompt sent mid-run needs `"streamingBehavior":"steer"` and answers `queued`;
  without it Pi rejects the prompt (documented,
  not run).
  `{"type":"steer","message":"..."}` and `{"type":"follow_up","message":"..."}` also queue during a run.
  `{"type":"get_last_assistant_text"}` returns `data.text`.
- Wait for `agent_settled` before reading the final text;
  `agent_end` ends only one low-level run.
- A dialog request (`extension_ui_request` with method `select`,
  `confirm`,
  `input`,
  or `editor`) waits for an answer,
  and the source leaves it pending forever unless a timeout field or an abort resolves it.
  Answer each with `{"type":"extension_ui_response","id":<the request id>,"cancelled":true}`.
  The other UI methods (`notify`,
  `setStatus`,
  `setWidget`,
  `setTitle`,
  `set_editor_text`) need no reply.
- A prompt Pi rejects comes back as a response,
  not as an event.
- A process holds one active session at a time;
  `fork`,
  `clone`,
  `switch_session`,
  and `new_session` replace it.
  Parallel agents need separate processes.
- A mistyped `:suffix` on `--model` does not fail:
  Pi warns and sends it to the provider as a custom model ID.
  Leave suffixes off and check `get_state` `data.model.id` after startup.

## Pi quirks

- A systemd user unit does not inherit the shell's `PATH`;
  the recorded launches without `--setenv=PATH` failed in seconds with `spawn pi ENOENT`.
- The owner's real home has a `thinking-default` plugin that sets the level to `xhigh` on every start and model change,
  whatever the command line says.
  Leave the thinking level alone:
  pass no `--thinking` and send no `set_thinking_level`.
- `PI_CODING_AGENT_DIR=<dir>` points Pi at another home when a run needs different providers or packages:
  a directory holding `settings.json` (its `packages` list) and a copy of `auth.json` (mode 600 in a mode 700 directory).
  A provider that comes from a package needs that package in whichever home the child uses,
  and the `thinking-default` plugin needs to be in it too,
  or the model runs below `xhigh`.
  Delete the `auth.json` copy afterwards.
- In a throwaway folder,
  project `.pi/settings.json` was ignored with no message unless the child got `--approve`;
  other project resources were not tested.
- A multi-block `edit` is rejected whole when any one `oldText` does not match,
  and a `find` given a path as its pattern returns "No files found" with `isError` false,
  so a soft miss and a success look alike until the result text is read.
- Timings recorded in the repository docs come from the slower `synthetic` provider;
  `neuralwatt` is unmeasured.

## DeepSeek V4.1 Flash quirks

The owner rates it at about a low-level Sonnet 5.5 when run at `xhigh`,
and finds it works poorly at any other thinking level.
It finished a multi-file change unassisted from a problem statement alone,
so a brief needs no step-by-step handholding.

- It stops and explains when an objective is hard or open.
  Given "prove the Collatz conjecture" it answered that the problem is open and ended its turn,
  and every later nudge got the same explanation plus a menu of alternative tasks.
  Told to attempt a proof,
  it did the work:
  a written proof attempt and compiled verifiers.
  Phrase a hard objective as attempts to make,
  and steer with one concrete next attempt when it stalls.
- Told to use a tool it does not have (`ask_user_question` under a `--tools` allowlist),
  it says so and asks in plain text with a numbered list.
- It slips on tool arguments and recovers without help:
  a path as a `find` pattern,
  guessed file extensions,
  a stale `oldText`.
  It lists directories,
  searches recursively,
  or greps and resends.
- It verifies thoroughly:
  confirms exports and types,
  re-reads after editing,
  greps other uses,
  and issues independent calls together.
  A one-line import change took 34 seconds.
- At `xhigh` the feed went quiet for 20 seconds or more while it thought.
  Check the process state and the time of the last event before calling a run stuck.
- It writes outside the task:
  scripts and binaries in `/tmp`,
  notes and sources in its working directory,
  and long compiled jobs (one build used OpenMP).
  Delete those leftovers by exact name afterwards.
- It reads this repository's docs,
  including ones that describe the problem it was given.
  Record that as a confound when judging an unassisted attempt.
- It builds more than the problem needs.
  Unassisted,
  it wrote a prompt change plus a phrase-list post-filter and tests,
  a much larger change than the prompt-only change that measured the same.
  Read the diff's scope against the problem.

Evidence for each line:
`doc/agent/drive-pi-as-a-subagent.md`,
`doc/agent/drive-pi-as-a-coding-subagent.md`,
and `doc/agent/pi-goal-extension-driven-through-rpc.md`.
