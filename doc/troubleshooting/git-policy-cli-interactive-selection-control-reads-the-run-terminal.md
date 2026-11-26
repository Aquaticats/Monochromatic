# The commit index control's interactive selection read the terminal the test run was started from

`package/git-policy/cli/src/native/commit/commit_index_selection.rs` runs
`git add --interactive` or `git add --patch` with the standard input of the process that calls it,
because the person's terminal answers the selection.
`cargo test --lib` runs the crate's tests as threads of one process,
so that process's standard input is the terminal the run was started from,
and one control's Git asked it a question nobody answered.

## Symptom

`commit::commit_index::tests::interactive_selection_succeeds_without_changes` never ended.
The run printed the Git subcommand's own menu,
which the test binary's capture does not hold (the child writes the process's own standard output),
and then nothing:

```console
$ cargo test --locked --offline --lib --features test-only-switches index:: -- --skip guest --skip _arm_tests
  1: s)tatus  2: u)pdate  3: r)evert  4: a)dd untracked
  5: p)atch  6: d)iff  7: q)uit  8: h)elp
What now> test commit::commit_index::tests::interactive_selection_succeeds_without_changes has been running for over 60 seconds
the unit-test run showed no sign of work for 120.1 s (no task able to run or waiting for the disk, no processor time, no bytes moved, and no process or task started): a test is stuck
error: test failed, to rerun pass `--lib`

Caused by:
  process didn't exit successfully: `.../git_policy_cli-...` (signal: 6, SIGABRT: process abort signal)
```

No `test result` line was printed, so the run proved nothing about the other 229 tests it had finished,
and the `git add --interactive` the aborted run left behind held the run's output pipe open:
the capped scope around that run stayed alive 21 minutes after the abort and was stopped by hand
(`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`, item 3's round note for issue 6507).
With standard input from the null device the same control passes in 0.04 s,
which is why no run of a build machine or a container gate had shown it.

A terminal on the run's standard input is what the symptom needs,
and the first reproduction attempt here did not have one:
`script --quiet --return --command ... /dev/null` with the caller's own standard input at the end of file
closes the pseudo-terminal at once,
so the Git read an end of file, answered `Bye.` and the run was green.
The runs below keep the pseudo-terminal's master open with a writer that never writes
(`sleep | script ...`), which is the shape a terminal at a keyboard has.

## Root cause

`prepare_interactive_selection` builds one `git_command` and starts it without a `stdin` of its own
(`commit/commit_index_selection.rs`):

```rust
let mut command = git_command(
    context.real_git.as_path(),
    arguments.as_slice(),
    context.overlay.as_slice(),
    StartOwner::Wrapper,
);
command
    .current_dir(cwd)
    .env("GIT_INDEX_FILE", commit_index)
    .env("GIT_OBJECT_DIRECTORY", object_directory);
let status: std::process::ExitStatus = match command.status_retrying() {
```

A child of a `std::process::Command` with no `stdin` inherits its parent's file descriptors,
so the Git read the descriptor the test binary inherited from the run.
`git add --interactive` prints its menu and waits for an answer whether or not the index holds a change
(`git add --patch` prints `No changes.` and ends, which is why only the `--interactive` control hung),
so the control waited for a person at a terminal that was the caller's shell.

The control asked the process for an answer, and the process is the whole run:
`cargo test` holds its tests as threads of one process,
which is the shape debt ledger item 3 collects.

## The fix

Both controls that start the selection run their body in a child of the test binary
(`testing/process_wide_state_child_test_support.rs`),
whose standard input is the null device,
so the Git reads an end of file and ends:

- `commit::commit_index::tests::interactive_selection_succeeds_without_changes`,
- `commit::commit_index::tests::interactive_selection_reports_a_failed_git_add`.
  This one failed on a missing pathspec file before it could read anything and passed either way,
  so it moved with its sibling: the rule is that no control of this crate hands a Git the run's standard input,
  not that one control was seen to hang.

`source_guard/no_control_starts_an_interactive_git_on_the_run_s_input_tests.rs` keeps the cause from coming
back:
a control of a test-only source that names `prepare_interactive_selection(` in code
and does not name `run_the_body_in_a_child_of_its_own(` is a finding.
It reads a test source through `source_guard/process_wide_fact_readers_tests.rs`,
the scan the bans over a preparation's ledger and a commit's ledgers use,
so a control's code places and controls are read once and not twice.

Nothing of the release changed.
The selection's Git reads the person's terminal in the wrapper,
which is what a `commit -p` or `commit --interactive` asks for,
and `package/git-policy/cli/SPEC.md` says the wrapper's own commits run with inherited stdio
("When real Git builds the commit,
it runs in the scratch repository after policy evaluation settles,
with inherited stdio").

## What the measurements say

With a terminal on the run's standard input, on this host:

- before, the issue's `index::` filter printed `What now>`,
  ran 228 test lines with one failure of another module
  (`worktree_clone_add_route::add_index::tests::a_second_private_write_and_a_publication_over_a_name_refuse`,
  which passes in the runs where this control does not hang),
  and was aborted at 120.1 s with no `test result` line;
- before, the control alone, with its body in the run's own process, printed the menu
  and was aborted at 120.1 s;
- after, the same filter answered `test result: ok. 230 passed; 0 failed` in 13.47 s,
  and the control alone `ok. 1 passed; 0 failed` in 0.04 s;
- after, both selection controls together answered `ok. 2 passed; 0 failed`.

After `main` renamed the child helper (`run_the_body_in_a_child_of_its_own`, which takes the body as a
closure) and this branch merged it again, the same runs answered
`test result: ok. 230 passed; 0 failed` in 17.94 s with a terminal on the run's standard input,
`ok. 52 passed; 0 failed` for `commit::commit_index::`,
`ok. 148 passed; 0 failed` for `source_guard::`
and `ok. 2 passed; 0 failed` for `testing::process_wide_state_child_test_support`;
the run left no Git process behind, and its capped scope ended with it.

With the control's child starter taken out, the guard answered
`test result: FAILED. 2 passed; 1 failed` and named
`commit/commit_index_tests.rs:977 (interactive_selection_succeeds_without_changes)`.
The same removal was measured again after the merge, against the closure form of the helper,
and the guard answered `FAILED. 2 passed; 1 failed` naming
`commit/commit_index_tests.rs:981 (interactive_selection_succeeds_without_changes)`.

The reproduction itself was shown to work for that second measurement:
with the child marker `CLI_GIT_PROCESS_WIDE_STATE_CHILD` armed by hand,
so the control's body ran in the run's own process, its Git printed `What now>` and the watchdog aborted the
run at 120.1 s; the same control under the child the fix starts answered `ok. 1 passed; 0 failed` in 0.04 s.

## The other controls that could read the same terminal

`prepare_interactive_selection` is this crate's only shipped start of a Git that reads the process's standard
input, and after the fix no control reaches it in the run's own process.
Every other Git start a control makes gives its Git an input that is not the run's:

- every step of `run_git` puts `Stdio::null()` in place of the standard input unless the request carries input
  (`transaction/transaction_git_step_about_to_start.rs`),
  so the property control that asks real Git about every generated `--interactive` and `--patch` line of
  `git commit` (`commit/commit_form_refusal_properties_tests.rs`) reads no terminal:
  measured with a terminal on the run's standard input it answered `ok. 1 passed; 0 failed` in 1.63 s,
  against `ok. 1 passed; 0 failed` in 1.72 s with the null device;
- the fixture helpers run real Git through `.output()`, which gives it the null device
  (`testing/fixture_git_commands.rs`, `command/command_test_support.rs`),
  or through `real_git_within`, which gives it the null device when the caller passes no input;
- the binary controls pipe the case's own bytes into the wrapper and close the pipe
  (`binary/binary_support_observation.rs`),
  so a `commit -p` or `commit --patch` there reads the answers its case names, and an end of file otherwise,
  never the terminal;
- the `git add --patch` stand-in of `lock/index_writer_forward_tests.rs` ends itself at the `add` word
  before it runs the real Git.

The wrapper's own plain commit also runs with the terminal attached
(`commit/commit_preparation.rs`'s `run_native_commit_within`),
because an editor or a signing program may ask the person something:
`package/git-policy/cli/SPEC.md` decides it, "When real Git builds the commit,
it runs in the scratch repository after policy evaluation settles, with inherited stdio".
Every control that calls it passes a message (`--message=` or `-m`), so no editor opens,
and the binary harness pins `GIT_EDITOR=true` (`binary/binary_service_landing_support.rs`, `pinned`).
An editor-opening commit line would need the child as well, and no guard reads a commit's arguments for it:
`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md` states the rule instead.

## What this page does not cover

- Debt ledger item 3's other mechanisms are unchanged
  (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`).
- The container gate was not run for this change,
  as `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` and the round's rules say.
