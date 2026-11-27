# Claude Code and Pi refuse deletions whose paths are not written literally

Status: accepted by the owner on 2026-11-24T06:40:00-05:00, through the question tool.
This record holds what the owner decided;
how the guard reads commands is in the packages' own READMEs
(`package/agent-harness-shared/shell-command-analyzer/README.md`,
`package/claude-code-plugin/guardrail/README.md`,
`package/pi-plugin/guardrail/README.md`).

## What the owner asked for

The owner caught a subagent issuing an `rm` whose path held a shell variable,
refused that call,
and watched the subagent write the same deletions into a script and run it.
The owner then asked for Claude Code and Pi to refuse violations of the literal-path rule for `rm` automatically,
on four surfaces:
edits,
file writes,
Bash,
and code mode,
and added that both Pi and Claude Code have a code mode.

The rule,
as `AGENTS.md` states it:
"`rm` commands as issued never contain environment variables (`$HOME`, `${DIR}`):
write every path literally,
because an unset or empty variable widens what gets deleted."

## What was measured before the choices were put

- Claude Code 2.1.331's code mode is a REPL tool.
  A REPL script's nested `Bash` and `Write` calls each reach PreToolUse hooks with their ordinary names and inputs,
  and a hook's deny holds inside the script;
  the REPL call itself reaches no hook,
  so no hook sees the script's text,
  and `require` and `import()` give the script no file system.
  The probe records are in
  `~/temp/agent/build-literal-rm-guard-in-both-harnesses-2026-11-24/keep/`
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here) (files starting `claude-code-mode-`).
- Pi 1.6.0's `codemode` call reaches the guardrail's `tool_call` handler with its script as `code`,
  and each nested call reaches it too,
  with a block honoured at either level
  (`keep/measurement-pi-codemode-tool-call.md` in the same folder
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here)).

## The owner's answers

### Globs

Asked whether `rm -f /home/user/temp/agent/x/*.log` should be refused too.
Answer:
"Allow globs".

### The home-folder tilde

Asked whether `~` in an `rm` path should be refused,
since an empty `HOME` turns `~/x` into `/x`.
Answer:
"Allow ~".

### Deleting commands besides `rm`

Asked which other deleting commands the literal-path rule holds.
Answer:
"rm, rmdir, unlink, shred, find -delete (Recommended)".

### Targets an empty value collapses to

Asked whether every surface should also refuse an `rm` target that looks like an empty-variable collapse,
because a command a Claude Code REPL script builds in JavaScript reaches the hook as plain text.
The option read:
"Also refuse collapsed targets (Recommended)",
described as refusing `/`,
the home folder itself,
a top-level folder such as `/home`,
or a path holding `//`,
`undefined` or `null`.
Answer:
"Also refuse collapsed targets (Recommended)".

## What this settles

- Globs and `~` stay allowed in deletion paths.
- `rmdir`,
  `unlink`,
  `shred` and `find -delete` are held to the same literal-path rule as `rm`,
  on every surface the guard reads.
- Every surface also refuses a deletion target that is `/`,
  the account's home folder itself,
  a top-level folder,
  or a path holding `//`,
  `undefined` or `null`.

## State

The first build,
covering `rm` on all four surfaces in both harnesses,
merged as `65dc4c3400`.
The deletion commands and the collapsed-target check this record adds are built in a follow-up round
on the same branch,
`feat/guardrail-literal-rm-paths`.
