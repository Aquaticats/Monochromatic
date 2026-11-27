# `PR_SET_PDEATHSIG` is cleared by an exec that enters an AppArmor profile

Recorded 2026-11-24T21:52:00-05:00 by the round `clone-rung-fixes-after-the-alpha-45-pass`.
UNAPPROVED and NON-FINAL:
this page states what one round measured and read, and only the project's owner approves it.

## What a person sees

The wrapper ends the child of its early size read with `PR_SET_PDEATHSIG`, set between `fork` and `exec`
(`package/git-policy/cli/src/native/transaction/transaction_git.rs:161`, `end_with_the_starting_thread`):

```rust
fn end_with_the_starting_thread(command: &mut Command) {
```

On this machine that backstop does not fire for a `git` child,
because every exec that enters an enforced AppArmor profile clears the signal.
The control `a_process_that_ends_with_a_running_read_leaves_no_child_behind`
(`package/git-policy/cli/src/native/worktree/worktree_clone_conversion_size_read_tests.rs:966`) failed at its last
assertion with:

```text
the read's child ended with the process that started it, not after its 30 s sleep
```

and the child that survived was the fixture's `exec sleep 30`, whose parent id 5 s later was 2528, the user manager:
its own parent had ended, and nothing ended it.
The release pass over alpha.45 recorded the same failure as "the size read's child control" in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md` without a cause;
this page is the cause.

## Root cause

Three rules combine.

First, the wrapper asks the kernel for the signal in the child, between `fork` and `exec`
(`package/git-policy/cli/src/native/transaction/transaction_git.rs:167`):

```rust
if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong) != 0 {
```

Second, the kernel zeroes that value during the exec when the exec is a "secure exec",
in `begin_new_exec()` of `fs/exec.c` at upstream commit `8d7fe9885c62fc9a95bf55d9db244d9672a8aadc`:

```c
	if (bprm->secureexec) {
		/* Make sure parent cannot signal privileged process. */
		me->pdeath_signal = 0;
```

(`fs/exec.c` lines 1240 to 1244 of that commit,
https://github.com/torvalds/linux/blob/8d7fe9885c62fc9a95bf55d9db244d9672a8aadc/fs/exec.c#L1240-L1244).
The same clearing happens in `commit_creds()` when the exec changes the effective ids or narrows the capability set
(`kernel/cred.c` lines 384 to 390 of that commit,
https://github.com/torvalds/linux/blob/8d7fe9885c62fc9a95bf55d9db244d9672a8aadc/kernel/cred.c#L384-L390):

```c
		/* mm-less tasks share init_task's exec_state */
		if (task->mm)
			task_exec_state_set_dumpable(suid_dumpable);
		task->pdeath_signal = 0;
```

Third, AppArmor marks an exec that enters a profile as a secure exec.
`profile_transition()` and `profile_onexec()` set the flag the kernel reads
(`security/apparmor/domain.c` lines 776 to 783 and 855 to 862 of that commit,
https://github.com/torvalds/linux/blob/8d7fe9885c62fc9a95bf55d9db244d9672a8aadc/security/apparmor/domain.c#L776-L783):

```c
	if (!(perms.xindex & AA_X_UNSAFE)) {
		...
		*secure_exec = true;
	}
```

and `apparmor_bprm_creds_for_exec()` sets it directly for an unsafe exec
(`security/apparmor/domain.c` line 1077,
https://github.com/torvalds/linux/blob/8d7fe9885c62fc9a95bf55d9db244d9672a8aadc/security/apparmor/domain.c#L1077).
A profile that attaches to a program's path is such a transition:
on this machine `/etc/apparmor.d/git` attaches the `git` profile to `/{,usr/}bin/git` and its siblings, and
`/etc/apparmor.d/sleep` attaches `sleep` to `/{,usr/}bin/sleep`.
So the exec of `/usr/bin/git` or of `/usr/bin/sleep` is a secure exec, and the child keeps no parent-death signal.

The host's kernel here answers `uname -r` with `7.3.5-2-cachyos`.
Its own source is not installed on this machine, so the quotations above are upstream's at the commit named;
the probe below shows the host applies the rule.

### An earlier reading of the probe log was wrong

The release pass's probe printed "gone (signal held)" for `/usr/bin/git cat-file --batch`
(`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/clone-rung/keep/pdeathsig-probe4.log`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
That process was gone because `git cat-file --batch` exits by itself outside a repository,
not because the signal reached it:
the same probe's `/usr/bin/git hash-object --stdin`, which waits on its standard input, was alive under the `git`
profile. The reading to keep is "alive under an enforced profile", so that probe has to give the child a reason to wait.

## Verification

The probe is `probe-loss2.py` in this round's `keep/`, run as
`python3 -I probe-loss2.py` in a directory under `~/temp/agent`.
It forks a middle process that forks the target, sets `PR_SET_PDEATHSIG` to `SIGKILL` in the grandchild,
has it `exec` the target, lets the middle process end, and 0.8 s later reads `/proc/<pid>` and
`/proc/<pid>/attr/current`:

- Patterns that keep the signal (the child is gone, so the signal fired):
  a copy of `/usr/bin/sleep` at an unprofiled path under `~/temp/agent` (`sleep-copy 30`),
  a copy of `/usr/bin/git` at an unprofiled path (`git-copy hash-object --stdin`),
  `tail -f /dev/null`, and `python3 -I -c 'import time; time.sleep(30)'`.
- Patterns that lose it (the child is alive, with `ALIVE sleep (enforce)` or `ALIVE git (enforce)`):
  `/usr/bin/sleep 30`, `/usr/bin/git hash-object --stdin`, and a fixture script whose `ls-tree` branch is
  `exec sleep 30` (the shape of the control's fake Git).
- The same probe with no exec at all keeps the signal,
  so the loss is tied to the exec and not to the fork or to the middle process ending.

The wrapper's own control was then shown red and green:

- Red on `main` (`92759cdc6b`) run alone in a capped scope:
  `cargo test --lib worktree::worktree_clone_conversion_size_read::tests::a_process_that_ends_with_a_running_read_leaves_no_child_behind`
  ended 101 at the assertion quoted above.
- Green with the change below, in the same shape.

## What the wrapper does about it

The kernel offers no other way to end a child when its parent dies:
`PR_SET_PDEATHSIG` is the only such mechanism, and a secure exec clears it by design.
So the read ends its own child instead of leaving it to that signal:

- `SizeReadBesideTheCheck` has a `Drop` that ends the Git child it published
  (`package/git-policy/cli/src/native/worktree/worktree_clone_conversion_size_read.rs:667`),
  and both it and an explicit `abandon` share one `end_the_child` (`:580`) whose compare-and-swap makes the ending happen
  once.
- `PR_SET_PDEATHSIG` stays as the backstop for a child whose exec is not secure,
  which is every host with no profile for the Git it runs, the container gate's image among them.
- A `Drop` runs on a return, on an error and on a panic's unwind.
  An add the kernel kills outright (`SIGKILL`, `SIGTERM` with no handler) runs no destructor,
  so that add leaves its `git ls-tree` child to run to its own end, a few hundred milliseconds on this repository
  and the step's ceiling at worst.
  Ending that window needs a mechanism the wrapper does not have:
  a survivor process or a handler, both of which the wrapper's rules against starting itself and writing shell text
  rule out.

## What does not work

- Setting the signal again after the exec from the parent: `PR_SET_PDEATHSIG` is per-task and applies to the task that
  called it, so only the child can set its own, and the child is Git.
- A shell preamble (`sh -c 'exec git …'`): the profile transition happens at the `git` exec, so the signal is cleared
  there, and the wrapper never writes shell text
  (`doc/planning/cli-git-no-self-invocation.md`, "The rule").
- Relying on the child's parent being confined already: a transition into the same profile still sets `AT_SECURE`
  through the `profile_transition` path above, and the test process that starts the read is unconfined anyway.
- `alarm()`: an alarm is not inherited across `execve`.
- `RLIMIT_CPU`: it is inherited across `execve`, but a child that sleeps burns no processor time, so the limit ends
  nothing.
- A surviving watcher process that kills the child: it needs a program of its own, and the wrapper starts neither itself
  nor a shell, as above.

## Upstream filing decision

Nothing to file, on all six constraints:

1. Upstream's fault: this is working as designed in both projects. The kernel's comment says why it clears the signal
   ("Make sure parent cannot signal privileged process"), and AppArmor's flag is what tells the kernel the exec is
   privileged.
2. Could upstream fix it: no. Clearing the signal is the point of the rule; a fix would remove the protection.
3. Support for our use case: the documented behavior is the one we meet; `man 2 PR_SET_PDEATHSIG` names the set-user-ID
   and capability cases, and an LSM's `AT_SECURE` is the same category.
4. Welcome for a contribution: no change to contribute.
5. Would they fix it: no defect exists to fix.
6. Anything to report: nothing; the finding is a design limit our own code now handles.

## Links not confirmed in a browser

The three GitHub links above were not loaded in `agent-browser`:
its launch on this machine fails before any page opens with
`Socket path too long: /home/user/temp/agent/clone-rung-fixes-after-the-alpha-45-pass-2026-11-24/run-1/temporary-files-pi-wrote/org.chromium.Chromium.<random>/SingletonSocket`,
whichever profile, temporary directory or runtime directory the run was given.
The line numbers were read from the files at that commit, fetched with `gh api` into this round's `scratch/`,
so the numbers are the commit's own; the fragment links are unverified by a browser.
