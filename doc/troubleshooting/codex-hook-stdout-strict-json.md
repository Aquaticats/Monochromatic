# Codex CLI 0.166.0 hooks: a trailing log line on hook stdout fails every event as `hook returned invalid post-tool-use JSON output`

## Symptom

Codex prints one card per tool call while a hook runs:

```text
• Hook failed
  └ hook returned invalid post-tool-use JSON output
```

Trigger pattern:
any registered hook whose stdout is a JSON object followed by non-JSON text
(a log line after the payload),
on any event whose handler validates output.
The message names the event,
so the same defect prints sibling wordings:
`hook returned invalid pre-tool-use JSON output`,
`hook returned invalid stop hook JSON output`,
`hook returned invalid session start JSON output`,
and `hook returned invalid user prompt submit JSON output`
(openai/codex issue [#27052](https://github.com/openai/codex/issues/27052) collects the family).
A hook that exits 0 with empty stdout,
or with plain non-JSON text,
produces no card at all.

Reported from this repository's Claude Code cost-dismissal hook
(`node package/claude-code-plugin/cost-dismissal/bundle/node/index.mjs`
registered in `~/.codex/hooks.json` at `PreToolUse`,
`PostToolUse`,
and `Stop`),
whose Claude Code behavior looked healthy:
the same bundle produced no error cards in Claude Code sessions,
so the failure surfaced only under Codex.

## Root cause

Two halves meet:
the hook writes diagnostics onto the protocol channel,
and Codex parses that channel strictly.

1.
   The hook runtime writes the writer's payload to stdout
   (`package/claude-code-plugin/source/src/runtime/handler-runtime.ts:108`),
   while logger-core's console sink sent `info` records to `console.info`,
   which is stdout
   (`package/logger/core/src/sink/console.ts:188`,
   `info: 'info'` at `package/logger/core/src/sink/console.ts:193`).
   Measured pre-fix stdout of one `PostToolUse` probe
   (159 bytes,
   payload plus one record):

   ```text
   {}[info] [2026-11-01T00:12:19.406Z] [cost-dismissal] [createCostDismissalHandler] [costDismissalHandler] cost-dismissal shadow recorded 0 hits of 0 text units
   ```

2.
   Codex's `PostToolUse` handler treats exit 0 with non-empty stdout as an
   output payload,
   and only when the payload parses does it proceed;
   when parsing fails but the text starts with `{` or `[`,
   it records the error
   ([`codex-rs/hooks/src/events/post_tool_use.rs` at openai/codex@67f0994b](https://github.com/openai/codex/blob/67f0994b532b8740f14b5b738af7b7b2a915b1d6/codex-rs/hooks/src/events/post_tool_use.rs#L256-L261)):

   ```rust
   // codex-rs/hooks/src/events/post_tool_use.rs:256-261 (openai/codex@67f0994b)
   } else if output_parser::looks_like_json(&run_result.stdout) {
       status = HookRunStatus::Failed;
       entries.push(HookOutputEntry {
           kind: HookOutputEntryKind::Error,
           text: "hook returned invalid post-tool-use JSON output".to_string(),
       });
   ```

3.
   The parser trims,
   requires one JSON object,
   and then deserializes strictly,
   so unknown fields fail the same way trailing text does
   ([`codex-rs/hooks/src/engine/output_parser.rs` at openai/codex@67f0994b](https://github.com/openai/codex/blob/67f0994b532b8740f14b5b738af7b7b2a915b1d6/codex-rs/hooks/src/engine/output_parser.rs#L345-L364)):

   ```rust
   // codex-rs/hooks/src/engine/output_parser.rs:345-364 (openai/codex@67f0994b)
   fn parse_json<T>(stdout: &str) -> Option<T>
   where
       T: for<'de> serde::Deserialize<'de>,
   {
       let trimmed = stdout.trim();
       if trimmed.is_empty() {
           return None;
       }
       let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
       if !value.is_object() {
           return None;
       }
       serde_json::from_value(value).ok()
   }

   pub(crate) fn looks_like_json(stdout: &str) -> bool {
       let trimmed = stdout.trim_start();
       trimmed.starts_with('{') || trimmed.starts_with('[')
   }
   ```

   The same file's test suite pins both edges:
   `{"unexpectedField":true}` is an invalid shape
   ([`output_parser.rs:535-545` at openai/codex@67f0994b](https://github.com/openai/codex/blob/67f0994b532b8740f14b5b738af7b7b2a915b1d6/codex-rs/hooks/src/engine/output_parser.rs#L535-L545)),
   and plain text that does not look like JSON is ignored with no error
   (`plain_stdout_is_ignored_for_post_tool_use`,
   [`post_tool_use.rs:548`](https://github.com/openai/codex/blob/67f0994b532b8740f14b5b738af7b7b2a915b1d6/codex-rs/hooks/src/events/post_tool_use.rs#L548-L560)).

So the failing class is narrow:
JSON-shaped-but-invalid stdout.
The log line after `{}` is exactly that shape.

An earlier reading was wrong and is recorded so it is not re-derived:
GitNexus PR [#1878](https://github.com/abhigyanpatwari/GitNexus/pull/1878) concluded Codex wants a
top-level `additionalContext` and rejects Claude Code's `hookSpecificOutput`.
The pinned source rejects neither:
its own test feeds `{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"..."}}`
and records the context
(`additional_context_is_recorded`,
[`post_tool_use.rs:395`](https://github.com/openai/codex/blob/67f0994b532b8740f14b5b738af7b7b2a915b1d6/codex-rs/hooks/src/events/post_tool_use.rs#L395-L430)).
Their case was an older or differently shaped payload,
not a `hookSpecificOutput` ban.

## Verification

Versions under test:
codex-cli 0.166.0
(`codex --version`,
binary at `/var/home/user/.local/share/monochromatic-dev/toolchain-global/exposure/codex`),
against upstream source pinned at
[`openai/codex@67f0994b`](https://github.com/openai/codex/tree/67f0994b532b8740f14b5b738af7b7b2a915b1d6);
the reproduction reports on the tracker also cover 0.142.5 and 0.149.1.

Harness one (consumer side):
pipe one Codex-shaped event into the registered hook command
and capture the streams byte for byte.

```bash
# package/claude-code-plugin/cost-dismissal/bundle/node/index.mjs, one PostToolUse event
printf '%s' '{"session_id":"probe","transcript_path":null,"cwd":"/var/home/user/Monochromatic","hook_event_name":"PostToolUse","model":"gpt-5","tool_name":"Shell","tool_use_id":"probe1","tool_input":{"command":"ls"}}' \
  | node package/claude-code-plugin/cost-dismissal/bundle/node/index.mjs \
  > stdout.bin 2> stderr.txt
```

Pre-fix output:
`stdout.bin` held the 159 bytes quoted in `Root cause`
and `stderr.txt` was empty.
Post-fix output:
`stdout.bin` is exactly `{}`
and `stderr.txt` carries the `[info]` record.

Harness two (host side):
one bounded codex session in a scratch directory,
prompted to run exactly one shell tool call,
which fires `PreToolUse`,
`PostToolUse`,
and `Stop`.

```bash
codex exec --skip-git-repo-check --sandbox workspace-write \
  "Run exactly one shell command: echo hook-probe-ok. Then stop and report the output. Do not run any other tool."
```

Observed on 2026-10-31 with the fixed bundle:
`hook: SessionStart Completed`,
`hook: PreToolUse Completed` twice
(the bash-limit hook and the cost-dismissal hook),
`hook-probe-ok` from the tool,
`hook: PostToolUse Completed`,
`hook: Stop Completed`,
and no `Hook failed` card anywhere in the transcript.
The same run also shows the rebuild does not disturb hook trust,
whose hash covers the hook declaration rather than the script file.

Catalog of output shapes:

- Clean:
  empty stdout with exit 0;
  plain non-JSON text
  (ignored by `plain_stdout_is_ignored_for_post_tool_use`);
  a bare `{}`;
  `{"decision":"block","reason":"..."}`;
  `{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"..."}}`.
- Failing with `hook returned invalid post-tool-use JSON output`:
  `{}[info] ...` (this bug);
  any JSON syntax error that starts with `{` or `[`;
  `{"unexpectedField":true}`
  (strict wire types,
  `output_parser.rs:535-545`).
- Failing with a distinct `unsupported` wording:
  `{"hookSpecificOutput":{"hookEventName":"PostToolUse","updatedToolOutput":...}}`
  (openai/codex issue [#31015](https://github.com/openai/codex/issues/31015)),
  and `decision:block` without a non-empty `reason`.

## Verified workarounds

1.
   Keep diagnostics off the protocol channel:
   `runHookPlugin` now installs `MONOCHROMATIC_LOG_CONSOLE=stderr`
   (`package/claude-code-plugin/source/src/runtime/handler-runtime.ts:89`),
   and logger-core gained the matching knob
   (`off` drops console-sink records,
   `stderr` writes every run to process stderr,
   in `package/logger/core/src/sink/console.ts`).
   Landed in this repository as commit `21015b9adf`.
   Tradeoffs:
   hook diagnostics move to stderr,
   which hosts surface only in hook failure details,
   and a handler that writes to stdout with raw `console.log`
   (forbidden by `TLG` for production code)
   would still pollute the channel,
   because the knob routes tagged-logger records only.
2.
   Emit nothing:
   exit 0 with empty stdout is a successful no-op on every Codex hook event.
   Tradeoffs:
   no `additionalContext`,
   no `systemMessage`,
   no decisions;
   acceptable for observe-only hooks.
3.
   `MONOCHROMATIC_LOG_CONSOLE=off`
   drops console-sink records entirely
   (other sinks,
   such as the file sink,
   still receive them).
   Tradeoff:
   no console-visible hook diagnostics at all,
   so failures become harder to trace.

## What does not work

- A JSON payload followed by log lines.
  The same bundle produced no error cards in Claude Code sessions,
  but Codex rejects it,
  which is why the defect hid in the other host.
- Putting Claude Code hook fields Codex's wire type rejects
  (for example `updatedToolOutput`,
  see openai/codex issue [#31015](https://github.com/openai/codex/issues/31015)).
- The GitNexus diagnosis
  ("Codex wants top-level `additionalContext` only").
  The pinned source accepts `hookSpecificOutput.additionalContext`,
  so the fix direction does not transfer;
  see `Root cause`.
- `--dangerously-bypass-hook-trust`.
  The hooks were trusted
  (their declaration hash is unchanged by rebuilding the script),
  and the failure is output validation,
  not trust.

## Upstream filing decision

Decision:
nothing to add;
do not file.
The defect was ours
(diagnostics on a protocol channel),
and the two upstream threads that touch this behavior already ask for
everything our trace would add.

Duplicate and prior-art searches on 2026-10-31
(`gh search issues --repo openai/codex` with
`"invalid post-tool-use JSON output"`,
`"hook stdout log JSON"`,
and `"hook returned invalid stop hook JSON"`):

- [#31015](https://github.com/openai/codex/issues/31015),
  same error string for an unsupported output field,
  open.
- [#27052](https://github.com/openai/codex/issues/27052),
  hook failure messages should identify the failing hook and classify
  "stdout was invalid JSON" against "valid JSON plus extra non-JSON logs",
  open,
  with a candidate fix by a commenter.
- [#18887](https://github.com/openai/codex/issues/18887),
  stop-hook invalid JSON errors are opaque for unsupported fields,
  open.

Work-in-progress check on 2026-10-31
(`gh api repos/openai/codex/commits?path=codex-rs/hooks/src/engine/output_parser.rs&per_page=3`):
the newest commits on the path are `cbfd999db78c`
(interrupted-turn hooks,
2026-08-25),
`6f647caa9bd6`
(async command hooks,
2026-08-08),
and `a576be2b7337`
(tighten hook output event schemas,
2026-05-28).
No maintainer churn in the path this quarter,
and no comment claims a fix for the diagnostics gap.

Out-of-scope check:
`.out-of-scope/codex-harness.md` records a maintenance commitment for
Codex harness plugins in this repository;
it is not an upstream-filing exemption and matched nothing here.

Six-constraint walk:

1.
   Is it really upstream's fault?
   No.
   The hook emitted JSON plus text;
   strict parsing and the terse message are defensible,
   and the message-gap request is already tracked in #27052.
2.
   Can upstream fix it?
   The diagnostics could name the failing hook and the parse failure class
   (#27052's request),
   but there is nothing to fix in the parse itself.
3.
   Are they supporting this use case?
   Yes for JSON hook output;
   the wire types and tests enumerate the accepted shapes
   (`output_parser.rs`).
4.
   Would the repo welcome our contribution?
   The root listing carries no `CONTRIBUTING.md`,
   `VOUCH`,
   or similar policy file
   (checked via `gh api repos/openai/codex/contents/` on 2026-10-31),
   and `.github/` holds issue templates;
   no ban was found.
   Moot here because the decision is not to file.
5.
   Will they likely fix it?
   #27052 carries a candidate fix from a commenter;
   no maintainer decline is recorded.
   Moot here for the same reason.
6.
   Have we prototyped a minimal fix compatible with their architecture?
   Not applicable:
   the fix belongs at our boundary
   (workaround one in `Verified workarounds`),
   and no upstream patch is proposed.

Risk to the poster:
filing puts a real person's account in front of strangers,
and repositories run trust or blocklist systems that can follow a person
across projects.
Nothing here recommends filing,
and any future filing stays the poster's decision.
