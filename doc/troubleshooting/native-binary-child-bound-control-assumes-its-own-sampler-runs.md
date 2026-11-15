# A child-bound control's delay ended inside its own sampler's slip, so a commit landed with a step stopped and nothing was reported (monochromatic-git-policy-cli 0.6.1-alpha.42)

## Symptom

`child_bound::a_git_step_past_its_bound_fails_the_commit_and_names_the_class` fails inside the
whole `native_binary` suite on a slow machine and passes when it runs alone:

```text
FAIL [  27.008s] ( 122/1215) monochromatic-git-policy-cli::native_binary child_bound::a_git_step_past_its_bound_fails_the_commit_and_names_the_class
thread 'child_bound::a_git_step_past_its_bound_fails_the_commit_and_names_the_class' (772711) panicked at src/native/binary/binary_child_bound_tests.rs:73:5:
assertion `left != right` failed: a stopped step must fail the commit: monochromatic-git-policy-cli: still waiting for git version, a quick query step (process 773110) after 0 seconds; it is ended only after 1 seconds with no sign of work (CLI_GIT_CHILD_WAIT_SECONDS changes this). Hint: it is still running; if it looks stuck, check it with `ps` and report it.
  left: 0
 right: 0
```

The control read `0` where it wanted a nonzero exit status: the commit landed although the step the
control had held quiet was never stopped, and nothing in the wrapper's output named the bound.

## Root cause

The control injects a delay on the return of every `execve` under `strace` and sets the quiet window
below that delay, so the first step the wrapper watches is quiet past its bound
(`package/git-policy/cli/src/native/binary/binary_child_bound_tests.rs`):

```rust
const BOUND_SECONDS: u64 = 1;
const MARGIN_SECONDS: u64 = 1;   // the delay was bound + margin = 2 s
```

The watch ends a step only when a sample of the step's signs of work finds none across the window.
The sample is taken by the watcher task and compared with the time of the last sample that saw work
(`package/git-policy/cli/src/native/push/push_progress.rs`):

```rust
if now >= self.next_sample {
    ... meter.sample(self.root) ...
    if progressed {
        self.longest_quiet = self.longest_quiet.max(now.saturating_duration_since(self.last_progress));
        self.last_progress = now;
        ...
    }
    self.next_sample = now + SAMPLE_INTERVAL;
}
if now.duration_since(self.last_progress) >= stall {
    return Some(StopCause::Stalled(...));
}
```

`SAMPLE_INTERVAL` is 500 ms, the loop asks for a wake at most 500 ms ahead
(`package/git-policy/cli/src/native/push/child_watch_task_loop_and_notice.rs`), and the quiet window
here is twice that. The watch therefore has exactly two samples per window, and a wake that arrives
late moves the sample that would judge the step.

On a scope at a quarter of one CPU the run went:

```text
DIAGCLOCK root=914286 looked=true elapsed=147.129µs  worked=true  moved=1692 progressed=true  quiet_for=147.129µs stall=1s
DIAGCLOCK root=914286 looked=true elapsed=501.880819ms worked=false moved=1692 progressed=false quiet_for=501.73369ms stall=1s
DIAGCLOCK root=914286 looked=true elapsed=1.501127662s worked=true  moved=4847 progressed=true  quiet_for=1.500980533s stall=1s
DIAGWATCH waited=1.500344294s verdict=false quiet=false stoppable=true measured=true root_there=false
```

The wake meant for 1.0 s was delivered at 1.5 s. By then `git version` had finished: the sample read
the step's final counters (+3155 bytes), called them work, and set `last_progress` to now, so the
1.5 s of silence it had just measured was discarded and no stall was ever judged. The step then
ended by itself and its status stood, as the watch is designed to allow, so the commit landed.

Every sample at that cap is late; what decided this run was whether the step was still quiet when the
late sample ran. With a margin equal to the window, the step's own end and the judgment fell in the
same sample.

## Verification

Version: this worktree's build of `0.6.1-alpha.42` (`git rev-parse HEAD` at the run).
The harness is the package's own task, run in a scope whose CPU share is cut:

```sh
systemd-run --user --scope --property CPUQuota=25% -- bash -c \
  "mise run //package/git-policy/cli:native:test:binary:host -- --no-fail-fast \
   'child_bound::a_git_step_past_its_bound_fails_the_commit_and_names_the_class'"
```

Patterns that pass cleanly:

- the same control at `CPUQuota=400%`: `PASS [5.440s]`, and the wrapper's own output names the stop
  (`git version, a quick query step showed no sign of work and was ended; running it again`, then the
  failure sentence naming the class).
- the same control at `CPUQuota=100%`: `PASS [4.704s]`.

Patterns that fail:

- the same control at `CPUQuota=25%`, twice: `FAIL [23.772s]` and `FAIL [23.508s]`, `code=0`, with the
  two notices of a wait that was never stopped and ten Git steps each held for the delay.

## Verified workarounds

None on the wrapper: the measurement is sound. A sample can only prove silence between two samples,
so a step that moves bytes after its last sample resets the window and there is no sound way to
charge the earlier silence to it. The control is what changed, in this branch:

```rust
const MARGIN_SECONDS: u64 = 5;   // delay = 6 s, ten sample intervals past the window
```

With the margin at five seconds the step stays quiet far past the window and any late sample still
lands inside its silence. Verified after the change: `PASS [14.505s]`, `PASS [14.133s]`,
`PASS [14.332s]` at `CPUQuota=25%`, and `PASS [12.544s]` at `CPUQuota=400%`.
Tradeoff: the run costs the injected delay twice (the step that may be run twice is redone), so the
control costs about nine seconds more than it did.

## What does not work

- Raising `CLI_GIT_WAIT_NOTICE_SECONDS`: the notice cadence is not what samples the step.
- Reading the elapsed time alone as a bound: the design ends a child for being quiet and never for
  being slow, so a wall clock would stop working steps.
- Lowering the window below a second: one second is the smallest `CLI_GIT_CHILD_WAIT_SECONDS`
  accepts.

## Upstream filing artifact

Not applicable: this is this repository's own control, not an external tool's defect.
