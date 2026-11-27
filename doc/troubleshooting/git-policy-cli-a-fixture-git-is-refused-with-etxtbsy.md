# A cli-git control's fixture Git was refused with ETXTBSY, because another thread's forked child held its write descriptor

`package/git-policy/cli`'s tests run as threads of one process under `cargo test --lib`
(`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
A control that copies the host's Git into its fixture and then starts it
can meet `Text file busy (os error 26)`,
because `execve` refuses a file that any process holds open for writing,
and a `fork` by another thread duplicates the whole descriptor table.
This is issue 6517,
measured on 2026-11-26;
issue 6516 reports the same defect with its own measurement.

## Symptom

One module of 13 controls,
run with default threads on 2026-11-26:

`cargo test --locked --offline --lib --features test-only-switches invocation::git_version_cache:: -- --skip guest --skip _arm_tests`,
with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`,
each run a fresh test process:

- run 3:
  `test result: FAILED. 12 passed; 1 failed`,
  `without_a_cache_or_an_identity_nothing_is_recorded`.
- run 4:
  `test result: FAILED. 12 passed; 1 failed`,
  `a_same_size_rewrite_with_the_old_mtime_is_noticed`.
- runs 1,
  2,
  5,
  6,
  7 and 8:
  `test result: ok. 13 passed; 0 failed`.

The wider suite failed the same way,
in one of three runs:

`cargo test --locked --offline --lib --features test-only-switches invocation:: -- --skip guest --skip _arm_tests`

- run 1:
  `test result: ok. 401 passed; 0 failed`.
- run 2:
  `test result: FAILED. 400 passed; 1 failed`,
  `invocation::git_version_cache::tests::an_executable_rewritten_with_other_size_is_noticed`.
- run 3:
  `test result: ok. 401 passed; 0 failed`.

Every failure carried the wrapper's own refusal,
with the fixture path shortened here:

```text
left: Err("monochromatic-git-policy-cli: real Git at <fixture>/git reports nothing (Text file busy (os error 26)), so its release cannot be read; commits, hooks and the locks that coordinate with them depend on it. Nothing was run. Point the wrapper at a working Git, then run the command again.")
 right: Ok((2, 56, 1))
```

A different control failed each time,
and each of them passed alone (`--exact`).

## Root cause

The control writes a fixture executable and then starts it,
and the write is opened in whichever process runs the control.

- `invocation/git_version_cache_tests.rs:40` (`copy_git`) calls `std::fs::copy(REAL_GIT, &target)`,
  which opens `target` for writing in this process.
- `an_executable_rewritten_with_other_size_is_noticed` calls `std::fs::copy("/usr/bin/true", &git_copy)`.
- `a_same_size_rewrite_with_the_old_mtime_is_noticed` calls `std::fs::write(&git_copy, ...)`
  and then `std::fs::OpenOptions::new().write(true).open(&git_copy)`.
- `invocation/git_version.rs:199` (`run_git_version`) starts the copy through `run_git`
  (`transaction/transaction_git.rs:247`),
  which is a `fork` and an `exec` of that path.

Every descriptor this crate opens carries close-on-exec,
and the control closes its own write descriptor before it starts the file.
What it cannot close is a copy another thread's `fork` made:
a `fork` duplicates the whole descriptor table of the process,
so any child forked while the control's write descriptor was open inherits it,
and it lives in that child until the child's own `exec` reaches the point where close-on-exec descriptors are closed.

The measurement is a trace of a failing run,
`strace -f -tt -e trace=openat,open,close,execve,clone,clone3,fork,vfork <the test binary> invocation::git_version_cache::`:

- 16:28:01.368886 the control's thread 3302960 opened its fixture Git for writing
  (`O_WRONLY|O_CREAT|O_TRUNC|O_CLOEXEC`).
- 16:28:01.369908 it closed that descriptor.
- 16:28:01.369937 it opened the same file again for writing,
  to put its modification time back,
  and 16:28:01.370006 it closed that one too,
  2.2 ms before its start.
- 16:28:01.369964 thread 3302959,
  another test of the same run,
  forked child 3302962 with `clone3(CLONE_VM|CLONE_VFORK|...)`,
  inside the 69 us the second write descriptor was open.
- 16:28:01.371950 that child entered `execve` of another test's fixture Git,
  and its `execve` did not return until 16:28:01.372762.
- 16:28:01.372063 the control forked its own `git version` child,
  pid 3302964.
- 16:28:01.372208 that child's `execve(fixture/git, ["git", "version"])` answered `-1 ETXTBSY (Text file busy)`,
  inside the window child 3302962 held the inherited descriptor.

So the holder is a child of another test,
and the descriptor lives only from that child's `fork` to its `exec`.
A scan of every process's descriptors taken at the moment the error surfaced found no holder,
which is why the trace,
not a scan,
is what names it.

Traced runs rarely fail:
one failure in 26 traced attempts before the one above,
and none in 400 traced attempts after it,
while untraced runs failed twice in eight.
The tracer stops each tracee at each traced call,
which narrows the window the failure needs;
this is an inference from those counts,
not a reading of `ptrace`.

## Verification

The code under test is `package/git-policy/cli` at commit `84e0686a70`,
with `origin/main` at `dde4ad5da9`,
cargo 1.102.0-nightly (ede6cefea 2026-11-11),
on Linux on this host.

The harness is the first command in the Symptom section,
with the four controls that failed named there,
all of them writing a fixture executable with this process's own descriptor.

After the change:

- 15 threaded runs of `invocation::git_version_cache::`,
  `test result: ok. 13 passed; 0 failed` in every one.
- three threaded runs of `invocation::`,
  `test result: ok. 401 passed; 0 failed` in every one,
  and one more after `origin/main` was merged at `eb2e29dd9f`,
  `test result: ok. 406 passed; 0 failed; 2 ignored`.
- each control whose body moved,
  run alone with `--exact`,
  `test result: ok. 1 passed; 0 failed` for all 11.
- `source_guard::`,
  `test result: ok. 165 passed; 0 failed`,
  before and after the merge.

The fix is visible in which process writes the fixture:
run one control alone onto a kept fixture root,
`CLI_GIT_KEEP_FIXTURES=1 CLI_GIT_TEST_ROOT=<root> <the test binary> --exact invocation::git_version_cache::tests::a_miss_is_recorded_and_the_next_ask_hits --test-threads=1`,
and the fixture folder is named after the child,
`process-3483866` for a test process of 3483864,
while `the_cached_answer_equals_the_live_answer`,
which writes no executable,
makes its folder as `process-3484063`,
the test process itself.

The wall clock:
each moved control,
run alone including its child's start,
took 27 to 48 ms on this host,
and the two controls that stayed in the test process took 15 ms and 6 ms.
The bound the moved bodies now run under is `CHILD_TEST_DEADLINE`,
120 s (`testing/child_test_process.rs`),
2500 times the slowest measured control.

## Verified workarounds

Run the body in a child of the test binary:

```rust
run_the_body_in_a_child_of_its_own(
    module_path!(),
    "a_miss_is_recorded_and_the_next_ask_hits",
    || {
        let root: PathBuf = fixture("version-cache-hit");
        // ... the body ...
    },
);
```

`testing/process_wide_state_child_test_support.rs:141` starts the test binary on that one test,
with standard input on the null device,
so the body is the only test of its process and no other thread forks between its write and its start.
Its tradeoff:
each moved control pays one more start of the test binary (5 to 25 ms measured above),
its fixtures are made in the child under the child's own name,
and a panic in the body reaches the parent as the child's failed test rather than as its own.

Write the executable through a child process instead,
which is what `testing/fixture_executable.rs:42` (`executable`) already does with `tee`,
and what `binary/binary_support_fixture_directories.rs:260` (`copy_executable`) does with `cp --reflink=never`.
A file only ever opened for writing by another process cannot be inherited by any child of this one
(`testing/fixture_executable.rs:27-36`).
Its tradeoff:
the payload is written by a program,
so a mode or a modification time that the fixture must keep is a second step,
and the copy control that puts the old modification time back
(`a_same_size_rewrite_with_the_old_mtime_is_noticed`)
still needs an open-for-writing handle of its own for `set_modified`.

## What does not work

Scanning `/proc` for the holder when the start fails.
The descriptor lives from a child's `fork` to its `exec`,
so a scan of every process's descriptors taken at the moment the error surfaced found none:
measured,
one failure reported `ISSUE6517 HOLDERS of <fixture>/git ...: []`,
and the traced failure's holder released its descriptor 554 us after the refused start.

Widening the window with a tracer and reading the rate from traced runs.
Traced runs failed once in 486 attempts while untraced runs failed twice in eight,
so a traced run cannot measure how often this happens.

Retrying the start in the wrapper.
Nothing in this crate retries a start that `execve` refused with `ETXTBSY`:
`transaction/transaction_git.rs:285` (`refused_by_the_system`) reads Git's own
`Resource temporarily unavailable` output for a refused process or thread,
which is a different refusal,
and a `fork` that fails here returns the system's error to the caller
(`invocation/git_version.rs:199`).
This is a reading of that code,
not a run,
and no release change was made:
a real Git open for writing while a package manager replaces it is a separate question
for the owner.

## The same refusal in issue 6540's named-Git reader control, whose failure said nothing

`repo_read::repo_read_hook::tests::a_caller_that_names_only_its_git_still_gets_in_process_answers`
failed once in a threaded run of `repo_read::` on 2026-11-26 (issue 6540),
after a merge of `main`,
in `package/git-policy/cli`:

```text
test result: FAILED. 237 passed; 1 failed; 0 ignored; 0 measured; 10530 filtered out; finished in 140.23s
thread 'repo_read::repo_read_hook::tests::a_caller_that_names_only_its_git_still_gets_in_process_answers' panicked at src/native/repo_read/repo_read_hook_tests.rs:266:10:
the readers answer without the forward-target variable
```

That control writes the Git it starts,
as `invocation/git_version_cache_tests.rs`'s controls do:
`std::fs::copy(REAL_GIT, &named_git)` into `bin/git` of its fixture,
and `try_request` then starts it to learn the system configuration file.

### The cause, made to happen on purpose

The refusal needs a window of microseconds,
so the control was made to fail for it deliberately,
in an uncommitted change that was reverted before the fix:

- the copy was written slowly,
  one mebibyte per `write_all` with 300 ms between them,
  so the destination stayed open for writing for about 1.8 s;
- a sibling control of the same file forked a child every 500 ms whose `pre_exec` slept 500 ms,
  so each child held what it inherited across its own `exec`,
  as the trace in this file found a child doing for 0.8 ms.

Run this, with `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`,
five tests, default threads,
the test process started afresh:

```sh
# in package/git-policy/cli
cargo test --locked --offline --lib --features test-only-switches repo_read::repo_read_hook::tests -- --skip guest --skip _arm_tests
```

```text
---- repo_read::repo_read_hook::tests::a_caller_that_names_only_its_git_still_gets_in_process_answers stdout ----
PLANT copy window opens at 12.426126ms
PLANT copy window closes at 1.969954493s

thread 'repo_read::repo_read_hook::tests::a_caller_that_names_only_its_git_still_gets_in_process_answers' (3901801) panicked at src/native/repo_read/repo_read_hook_tests.rs:271:10:
the readers answer without the forward-target variable

test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 10767 filtered out; finished in 20.23s
```

The failure named what the control wanted and not why it got nothing.

### Why the answer was silent

The question itself was already answered with a reason by `ask_git_var_explaining`
(issue 6541's repair, `repo_read/git_system_config_asking.rs`),
and the caller that turns it into an answer or nothing threw that reason away:

```rust
fn ask_git_var(real_git: &Path, variable: &str) -> Option<Vec<u8>> {
    match ask_git_var_explaining(real_git, variable) {
        Ok(stdout) => return Some(stdout),
        Err(_) => return None,
    }
}
```

`None` makes `ScopeEnvironment::from_variables` set `unresolvable` (`repo_read/repo_read_config_scan.rs`),
which makes every in-process reader decline
(`outer_scopes_avoid_keys` answers `None`),
so `try_request` answered nothing and the control could say only what it wanted.

### The repair

- `ask_git_var` says the reason in the wrapper's diagnostics before it answers `None`:
  the executable, the question, and the start's refusal or the ceiling's sentence,
  which names the program and its length.
- the control reads the lines the wrapper wrote for its request (`diagnostic_log::take_lines_written`)
  and puts them into its own failure,
  so a later failure names the start error or the ceiling.
- the control's body runs in a child of its own (`run_the_body_in_a_child_of_its_own`,
  `testing/process_wide_state_child_test_support.rs`),
  which is the workaround this file records in `Verified workarounds`.
- the asking moved into `repo_read/git_system_config_asking.rs`,
  because `repo_read/git_system_config.rs` sat at the linter's 300-line limit.

The same planted change, after the repair, in the same run shape:

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10769 filtered out; finished in 20.23s
```

A control of the line,
`repo_read::git_system_config::tests`,
reads it from a question whose program cannot be started
(the fixture path stands for a file that is not a program):

```text
[warn] [2026-11-27T10:30:03.403Z] [monochromatic-git-policy-cli] [ask_git_var] starting <fixture>/git for `git var GIT_CONFIG_SYSTEM` gave no answer: Permission denied (os error 13)
```

### What the repair does not cover

- The ceiling's sentence travels in the same `Err` the diagnostics print,
  and no control waits two minutes to reach it:
  the sentence itself is `ceiling_reached_text`'s, asserted by the controls of
  `process/child_waited_for_within_a_ceiling_tests.rs`.
- `repo_read/git_system_config_tests.rs`'s controls that copy or rewrite a fixture Git
  were moved into children of their own by issue 6541's repair, before this round merged;
  the module run of this round passed after that merge.
- The ban in `source_guard/no_control_writes_a_fixture_git_in_the_run_tests.rs` still reads only
  `invocation/git_version_cache_tests.rs`,
  so neither the control in `repo_read/repo_read_hook_tests.rs` nor those two files is held to it.

This record is unapproved and non-final.

## Upstream filing decision

Nothing is filed,
and there is nothing to file.
The cache controls write an executable and start it in a process that shares its descriptor table with other tests;
the kernel's refusal is the correct answer to that,
so no upstream behaviour is a defect.

- Constraint 1,
  is it upstream's fault:
  no,
  the kernel's `execve` refuses a file open for writing by design,
  and this crate's test code created that state.
- Constraint 2,
  can upstream fix it:
  it is not a defect,
  so there is nothing to fix.
- Constraints 3 to 5,
  supporting the use case,
  welcoming a contribution,
  likely to fix:
  not reached,
  because constraint 1 fails.
- Constraint 6,
  a prototyped minimal fix:
  the fix belongs to this repository and is in it.

`.out-of-scope/` was not searched,
because nothing is filed;
this record is for this repository's own investigators.
