# DeepSeek Harness run wrapper 0.2.2-rc.1: fork and thread starts fail with `EAGAIN` when the session's own `run-p*.scope` task and memory budgets fill, surfacing through git-policy-cli `startRetry` and `git`'s threaded lstat

## Symptom

During the qwen-token-plan-individual build of 2026-11-11
(UTC 11:00:48 to 11:16),
`git` invocations inside the session repeatedly failed while three
teammate build pipelines ran in the same session.
Two tool surfaces reported it.

`monochromatic-git-policy-cli` 0.6.1-alpha.9
(the git shadow wrapper,
`git --version` prints it)
retried child starts and warned once per refusal:

```text
[warn] [2026-11-11T11:08:35.239Z] [monochromatic-git-policy-cli] [startRetry] starting git update-index --refresh was refused for lack of processes (operation would block); trying again in 10 ms
```

and when its 5-second retry budget ran out,
plain `git` failed inside the same wrapper:

```text
fatal: unable to create threaded lstat: Resource temporarily unavailable
```

Three further shapes appeared in the same window,
all `EAGAIN` (errno 11)
on task creation:

```text
node[550966]: pthread_create: Resource temporarily unavailable
bash: fork: retry: Resource temporarily unavailable
Error: subprocess scope exited before its bootstrap consumed the launch request; the provider probes native containment again before its next command
```

The last one is the harness itself failing to start a command scope
bootstrap,
and it clears on the next call.

## Root cause

### The mechanism

Every process and every thread is one task,
and `fork`, `clone` and `pthread_create` fail with `EAGAIN` when a
`pids.max` in the task's cgroup ancestry is full.
That mechanism is established with kernel journal evidence in
[agent-browser-eagain.md](agent-browser-eagain.md)
(section "The first shape: the per-user task ceiling was reached")
and again in
[node-26-child-stuck-when-a-full-task-ceiling-refuses-its-threads.md](node-26-child-stuck-when-a-full-task-ceiling-refuses-its-threads.md).
This entry adds nothing to that mechanism;
it records a ceiling *above* the leaf command scopes that bound first
in this session.

### An earlier reading was wrong: this was not host process exhaustion

A first diagnosis claimed the "host process table is saturated".
That is disproved by the measurements taken in the same window:

```text
ps -e --no-headers | wc -l                        -> 1278 processes
/proc/sys/kernel/threads-max                      -> 503082
user-1000.slice  pids.max=32768  pids.current     -> ~5000
ulimit -u                                        -> 32768
```

The host had headroom everywhere.
The binding budget is the session's own run wrapper control group.

### The binding limit: the run wrapper scope

The harness runs each session inside a sized wrapper scope
(`run-p<pid>-i<id>.scope` under `wg-quicker-update-app.slice`,
naming per this run),
and every command scope this session starts is a *descendant* of it.
Measured on 2026-11-11 at 11:17 UTC,
read-only,
for this session's wrapper
`run-p2476658-i316681074.scope`:

```text
pids.max=256        pids.current=181..199        pids.events: max 6466
memory.max=1073741824 (1 GiB)   memory.current=1073729536 (pinned at the cap)
memory.events: max 8170134   oom 0   oom_kill 0
cpu.max=100000 100000 (1 CPU)
```

`pids.events: max 6466` is the direct evidence:
the wrapper refused 6466 task creations,
and `memory.events: max 8170134` shows the 1 GiB memory budget pinned
at its boundary through the whole window
(no OOM kills inside the wrapper;
the `oom 3143 / oom_kill 87` counters higher up the tree belong to
other services in `user@1000.service`).

By contrast the leaf command scopes look generous and never bind:

```text
.../dsh-subprocess-1794886-<hex>.scope   pids.max=75462   pids.current=0..30
```

Because cgroup v2 `pids.max` is hierarchical,
every descendant task counts against the wrapper's 256,
so a leaf budget of 75462 is unreachable while the wrapper is full.
Reading only the leaf scope is what made the first
"no limit is near" reading misleading.

### Why the failures cluster in bursts

The wrapper's task count rises and falls with the session's concurrent
work:
this session runs three teammate build pipelines
(`pnpm install`, `tsc`, `rolldown`, node test runners,
each spawning dozens of node tasks)
alongside the lead's own commands.
During the failing window those bursts filled the wrapper;
in a quiet interval the very same `git commit` succeeded on first try
(11:16:57 UTC,
captured by the harness in the Verification section below).
`git`'s threaded lstat spawns one thread per core
(here 32),
which is exactly the shape that exhausts a tight task budget first.

### Why the message misleads

The retry message is emitted by our own wrapper,
`package/git-policy/cli/src/native/push/start_retry.rs:109`:

```rust
"starting {what} was refused for lack of processes ({error}); trying again in {} ms",
```

`operation would block` is Rust's `ErrorKind::WouldBlock` for `EAGAIN`.
The wording "lack of processes" is accurate for the errno
but does not name which budget refused,
which is how the first reading drifted to host exhaustion.
The wrapper's own module doc
(`package/git-policy/cli/src/native/push/start_retry.rs:5-8`)
already says "a process limit, a cgroup limit",
so no code change is warranted;
the gap is this record.

## Verification

Versions under test,
measured 2026-11-11:

```text
git version 2.56.0
monochromatic-git-policy-cli 0.6.1-alpha.9, shadowing git at /usr/bin/git
node v26.13.0
DeepSeek Harness run wrapper run-p2476658-i316681074.scope (naming per this run)
```

### Harness

A probe that prints the running command's own cgroup chain and wrapper
budgets,
then runs the affected command as its child while sampling task counts,
so a refusal window can be matched against the limit that bound.
Scratch file in `~/temp/agent/scope-probe.ts` for this run;
minimal form:

```ts
// ~/temp/agent/scope-probe.ts: reads /proc/self/cgroup, walks the chain's
// pids.max / pids.current / memory.max, samples every 50 ms, runs a command.
import { readFileSync } from 'node:fs';
import { execFile } from 'node:child_process';

const rel = readFileSync('/proc/self/cgroup', 'utf8').trim().replace(/^0::/, '');
const rd = (dir: string, f: string) => { try { return readFileSync(`/sys/fs/cgroup/${dir}/${f}`, 'utf8').trim(); } catch { return '?'; } };
let acc = '';
for (const part of rel.split('/').filter(Boolean)) {
  acc += '/' + part;
  console.log(part, 'pids.max=' + rd(acc, 'pids.max'), 'pids.cur=' + rd(acc, 'pids.current'),
    'mem.max=' + rd(acc, 'memory.max'), 'mem.cur=' + rd(acc, 'memory.current'));
}
execFile('git', ['-C', '/var/mnt/encrypted/repository/Monochromatic', 'status', '--short'], console.log);
```

### Catalog: patterns that fail

In the 11:00:48 to 11:15:56 UTC window of 2026-11-11,
with teammate builds running:

- `git commit --message <text> -- <paths>`:
  `startRetry` warnings,
  then `fatal: unable to create threaded lstat: Resource temporarily unavailable`
  (exit 128),
  five separate attempts,
  two of them background jobs.
- `git status --short`:
  same fatal,
  and once it hung past 60 s and had to be killed.
- Plain `bash` steps inside those calls:
  `bash: fork: retry: Resource temporarily unavailable`.
- `node` invocations:
  `node[<pid>]: pthread_create: Resource temporarily unavailable`.
- One harness call:
  `subprocess scope exited before its bootstrap consumed the launch request`.

### Catalog: patterns that work cleanly

In the same session at 11:16:57 to 11:17:09 UTC,
a quiet interval after a teammate build step ended:

- `git commit` as a child of the probe: first try,
  `[main 68a52155a6] ... 1 file changed`,
  wrapper readings taken in the same call showed headroom
  (probe leaf scope peaked at 30 tasks).
- `git log`, `git status --short`, `cat` of cgroup files:
  clean at every attempt outside the burst windows.
- Every commit by the teammate pipelines themselves
  (`7540a24850`, `b0cae19e36`, `8ee0610f2d`, `b4c874f9af`, `1f982cb128`)
  landed during their own quieter moments.

## Verified workarounds

1. **Re-run after the burst.**
   `monochromatic-git-policy-cli` already retries for 5 s
   (`START_RETRY_BUDGET`,
   `package/git-policy/cli/src/native/push/start_retry.rs:39`);
   a command that outlasts it lands on the next quiet window
   (this is how `68a52155a6` landed).
   Tradeoff: no bound on the wait,
   and under sustained parallel builds the window may not come.

2. **Thin the session's concurrent task load.**
   Run heavy gates of one pipeline at a time,
   or let a teammate finish its build before starting another's.
   Tradeoff: wall-clock time in exchange for determinism;
   the wrapper budget itself is the real fix point and is
   harness-side,
   outside this repository.

3. **For the memory side of the same budget**
   (`memory.max=1 GiB` pinned,
   which fails `pthread_create` at stack allocation with the same
   `EAGAIN`),
   the same relief applies:
   fewer concurrent node processes inside the wrapper.

## What does not work

- **Blaming host process exhaustion.**
  Disproved above:
  1278 processes against `ulimit -u` 32768,
  kernel `threads-max` 503082,
  user slice at 5000 of 32768.
  The host was never close.

- **Reading only the leaf `dsh-subprocess-*` scope.**
  Its `pids.max=75462` never binds;
  the wrapper's 256 does.
  The leaf reading is what produced the first "no limit is near"
  conclusion.

- **Raising `ulimit -u` or the user-slice `pids.max`.**
  Neither was the refusing budget;
  changing them changes nothing.

- **Treating `git`'s "unable to create threaded lstat" as a git bug.**
  It is `pthread_create` returning `EAGAIN` from the same cgroup
  budget; git reports it accurately.

## Upstream filing decision

Nothing to file.
The six constraints,
walked on 2026-11-11:

1. **Is it really upstream's fault?**
   No.
   The sized run wrapper is the harness working as designed
   (AGENTS.md RXH/RXS describe per-command cgroups
   "sized to it");
   the task count grew because this session ran three concurrent build
   pipelines.
   The nearest "upstream" surfaces are the harness
   (budget sizing is intentional),
   git (message accurate),
   and our own `git-policy-cli`
   (message accurate for the errno;
   its module doc already names cgroup limits).

2. **Can upstream fix it?**
   Not applicable;
   there is no defect on the constraint-1 candidates.

3. **Are they supporting this use case?**
   The harness supports running work in sized cgroups by design;
   nothing claims unlimited task counts inside one wrapper.

4. **Would the repo welcome our contribution?**
   Not applicable;
   nothing to contribute.

5. **Will they likely fix it?**
   Not applicable.

6. **Have we prototyped a minimal fix?**
   Not applicable;
   the consumer-side relief is workaround 1 and 2 above.

`.out-of-scope/` holds no entry for the harness run wrapper
(checked 2026-11-11;
`codex-harness.md` covers the Codex CLI and does not match),
so no exemption applies and the decision rests on the constraints.
Duplicate search and work-in-progress search are moot
with no tracker to file to.
No filing artifact exists;
the "nothing to file" note above is the record.
