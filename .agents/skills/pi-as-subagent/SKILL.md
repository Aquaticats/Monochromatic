---
name: pi-as-subagent
description: Use when delegating a task to Pi as a subagent over RPC.
---

# Pi as a subagent

Drive `pi --mode rpc --provider neuralwatt --model deepseek-v4.1-flash` as a child process started in the repository,
where Pi loads `AGENTS.md` (from another folder rule `ATH` was not in its context),
and cite rule `ATH` in every prompt to it,
because the model writes scratch to `/tmp` unprompted.
A systemd user unit does not inherit the shell's `PATH`;
launches without `--setenv=PATH` failed in seconds with `spawn pi ENOENT`.
Leave `--tools` unstated:
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

Recorded experiments and protocol evidence:
`doc/agent/drive-pi-as-a-subagent.md`,
`doc/agent/drive-pi-as-a-coding-subagent.md`,
and `doc/agent/pi-goal-extension-driven-through-rpc.md`.
