# Codex 0.167.0 lets the bypass flag override `--sandbox read-only`

## Symptom

The reported command combined a read-only sandbox request with the sandbox bypass:

```sh
# Reported invocation, run from this repository.
codex exec --sandbox read-only --dangerously-bypass-approvals-and-sandbox \
  "Write a dummy file to ~/temp/agent/dummy.txt and delete it"
```

Codex `0.167.0` accepted the combination and printed:

```text
# Relevant startup fields from the supplied transcript.
approval: never
sandbox: danger-full-access
```

The shell successfully created and deleted the file.
The surprising behavior is that the command accepts contradictory-looking switches and gives the bypass precedence.
The run did not select a read-only sandbox and then escape it.
The startup banner already identified the selected policy as `danger-full-access`.

## Root cause

Source inspected at release tag `rust-v0.167.0`,
commit `086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a`.

In [`codex-rs/utils/cli/src/shared_options.rs:38`][arguments],
the sandbox option and bypass boolean have no mutual conflict declaration:

```rust
// openai/codex: codex-rs/utils/cli/src/shared_options.rs:40
#[arg(long = "sandbox", short = 's')]
pub sandbox_mode: Option<SandboxModeCliArg>,
```

```rust
// openai/codex: codex-rs/utils/cli/src/shared_options.rs:54
#[arg(
    long = "dangerously-bypass-approvals-and-sandbox",
    alias = "yolo",
    default_value_t = false
)]
pub dangerously_bypass_approvals_and_sandbox: bool,
```

[`codex-rs/exec/src/lib.rs:352`][precedence] then chooses the bypass policy explicitly:

```rust
// openai/codex: codex-rs/exec/src/lib.rs:352
let sandbox_mode = if dangerously_bypass_approvals_and_sandbox {
    Some(SandboxMode::DangerFullAccess)
} else {
    sandbox_mode_cli_arg.map(Into::<SandboxMode>::into)
};
```

This branch checks whether the bypass is present,
not which switch appeared last.
The selected policy is passed into configuration alongside the headless approval default in
[`codex-rs/exec/src/lib.rs:591`][headless]:

```rust
// openai/codex: codex-rs/exec/src/lib.rs:593
review_model: None,
// Default to never ask for approvals in headless mode. Rebuild below if
// the fully resolved reviewer is AutoReview.
approval_policy: Some(AskForApproval::Never),
approvals_reviewer: None,
sandbox_mode,
```

Skipping approval prompts and disabling filesystem restrictions are separate choices.
The bypass flag requests both.
Its installed help describes it as
"Skip all confirmation prompts and execute commands without sandboxing."

The transcript's under-development-feature warning is a separate startup diagnostic.
The explicit policy-selection branch explains this result without attributing it to those features or to the model.

## Verification

The supplied transcript identifies Codex `0.167.0` and model `gpt-6-astra`.
The installed binary independently reported `codex-cli 0.167.0`.
Source was cloned from `openai/codex` at `rust-v0.167.0`.

On 2026-11-09,
a disposable local Responses provider drove the installed binary's actual `exec_command` tool.
It used fresh working directories and configuration homes,
model identifier `gpt-6-astra`,
and reasoning effort `xhigh`.
The provider supplied a fixed tool call rather than asking a model to decide whether to obey a write request.
No account credentials or production files were used.

Each case attempted this same shell command in its own fixture directory:

```sh
# Disposable working directory created under ~/temp/agent.
printf 'dummy\n' > dummy.txt && cat dummy.txt && rm dummy.txt && test ! -e dummy.txt
```

The captured tool outputs establish these results:

- `--sandbox read-only --dangerously-bypass-approvals-and-sandbox`:
  banner `sandbox: danger-full-access`,
  shell exit `0`,
  output `dummy`,
  deletion check passed.
- `--dangerously-bypass-approvals-and-sandbox --sandbox read-only`:
  the same banner and successful shell result.
- `--sandbox read-only`:
  banner `sandbox: read-only`,
  shell exit `1`,
  `/bin/bash: line 1: dummy.txt: Read-only file system`.

Each run reported `approval: never`.
The successful write cases are the positive control for the rejected write.
The outer `codex exec` process exited `0` even in the rejected-write case because the fixture supplied a final reply.
The shell's returned exit status and diagnostic are the enforcement evidence.

A manual reproduction can use a fresh directory and the original operation:

```sh
# Run from a disposable fixture, not against an existing ~/temp/agent/dummy.txt.
probe_dir=$(mktemp --directory "${HOME}/temp/agent/codex-sandbox.XXXXXXXX")
codex exec --cd "$probe_dir" --sandbox read-only \
  --dangerously-bypass-approvals-and-sandbox \
  --config 'model_reasoning_effort="xhigh"' \
  'Create dummy.txt in the working directory, read it, delete it, and verify deletion.'
```

This manual form depends on model behavior.
The deterministic verification explicitly attempted the write in every case.
Windows,
macOS,
managed permission requirements,
remote servers,
and other Codex releases were not exercised.

## Verified workaround

For noninteractive execution that retains filesystem restrictions,
remove the bypass flag:

```sh
# Read-only exec configuration, with no sandbox bypass.
codex exec --sandbox read-only --config 'model_reasoning_effort="xhigh"' \
  'Inspect the project and report your findings.'
```

The local provider control verified both the `read-only` banner and an actual denied write with this flag selection.
The tradeoff is intentional:
commands needing filesystem writes fail,
and headless `exec` does not turn those failures into interactive approval prompts in the tested configuration.
This is a shell-tool restriction,
not a claim that Codex itself writes no session or configuration data.

## What does not work

- Moving `--sandbox read-only` after the bypass flag:
  the reversed-order runtime control still allowed the write.
- Treating the bypass as an approvals-only switch:
  both its help and the policy-selection branch explicitly include sandbox bypass.
- Using the absence of `dummy.txt` after the run as proof that writes were blocked:
  the successful command deleted its own output.
- Treating the outer process's successful exit as proof that its shell write succeeded:
  the rejected-write control also ended with outer exit `0`.

## Upstream filing decision

The [Codex harness maintenance policy](../../.out-of-scope/codex-harness.md) was checked.
It does not prohibit this investigation or issue tracking.

- Upstream fault:
  the requested bypass is implemented as described;
  no sandbox-enforcement defect was established.
  Rejecting the combination or warning about precedence would be a separate interface change.
- Fixability:
  argument validation or a diagnostic could change this interface,
  but no such change was requested or prototyped.
- Supported use case:
  read-only execution works in the control.
  Keeping read-only restrictions while requesting the sandbox bypass is not the behavior implemented by this release.
- Contribution policy:
  [`docs/contributing.md:3`][contributing] welcomes reports and analyses;
  line `5` says "We do not accept external code contributions or pull requests."
- Maintainer willingness:
  no maintainer decision about rejecting this exact combination was established.
- Minimal fix:
  none prepared because the verified remedy is to select the intended existing mode.

Tracker searches used `gh search issues --repo openai/codex` with
`"dangerously-bypass-approvals-and-sandbox" "read-only"`
and `gh search prs --repo openai/codex` with `sandbox bypass`.
The issue query returned no results;
the returned pull requests did not establish a fix for this flag-precedence behavior.
This is a bounded search result,
not a claim that no related discussion exists.

Nothing to add as a demonstrated upstream defect.
No issue or comment was drafted or sent.
Latest-release comparisons,
fork-branch searches,
trust-system checks,
and a clean-runner publication were not performed;
no filing readiness is claimed.

[arguments]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/utils/cli/src/shared_options.rs#L38-L59
[precedence]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/exec/src/lib.rs#L352-L356
[headless]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/codex-rs/exec/src/lib.rs#L591-L600
[contributing]: https://github.com/openai/codex/blob/086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a/docs/contributing.md#L3-L5
