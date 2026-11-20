# A read of `/proc/<pid>/comm` answers `ESRCH` on Linux 7.3.5 when the task is released between the scan's listing and its read, so a process scan sees a failed read where the process is simply gone

## Symptom

A driver that asks what a run left running lists the process folder and then
reads each listed process's records.
When one of those processes ends between the two steps, the read throws, and
the run ends with the failure in its summary instead of waiting for the work:

```json
"workProblem": "Error: ESRCH: no such process, read '/proc/3160413/comm'"
```

The run was a writer run on 2026-11-19 that had ended its turn while a long
test run it had started was still going.
Because the ask threw, the driver ended the run `idle`, the unit stopped, and
the run's own watcher was killed with it, leaving the test container it had
started running on unwatched.

The pattern is any scan that lists a process folder and then reads a record of
each entry: `ENOENT` for a folder already gone is handled, `ESRCH` from a
record whose task went away was not.

## Root cause

`fs.readFile` on a path opens the path and then reads the open file.
A path listed while the process was alive still opens, and the read that
follows runs against the kernel's record handler:

```c
static int comm_show(struct seq_file *m, void *v)
{
	struct inode *inode = m->private;
	struct task_struct *p;

	p = get_proc_task(inode);
	if (!p)
		return -ESRCH;
```

([`fs/proc/base.c` lines 1746 to 1758][comm], v6.12 at commit
`adc218676eef25575469234709c2d87185ca223a`).
`get_proc_task` returns no task once the pid is released, and the handler
answers `-ESRCH`. So the same record answers two different codes depending on
which step of the read races the process's exit:

- path lookup after release answers `ENOENT`, which is what a scan already
  counts as a process that is gone;
- a read of an already-open record after release answers `ESRCH`.

Every record the pid registers through the `ONE` macro is read through the
same handler, which asks `get_pid_task` and answers `-ESRCH`:

```c
static int proc_single_show(struct seq_file *m, void *v)
{
	...
	task = get_pid_task(pid, PIDTYPE_PID);
	if (!task)
		return -ESRCH;
```

([`fs/proc/base.c` lines 801 to 817][single], same commit).
`cgroup` is registered there ([line 3372][cgroup]) and `stat` too
([line 3334][stat]), so both answer `ESRCH` the same way.
`cmdline` has its own handler, which answers `-ESRCH` from its first read.

The `cwd` link is the one record here that is not such a read:

```c
static int proc_cwd_link(struct dentry *dentry, struct path *path)
{
	struct task_struct *task = get_proc_task(d_inode(dentry));
	int result = -ENOENT;
```

([`fs/proc/base.c` lines 222 to 236][cwd], same commit).
A task released before the link is read gives `ENOENT`, which the scan already
counts as gone, so the link's handling needed no change. An earlier reading of
this defect counted the `cwd` link among the reads that answer `ESRCH`; the
handler above is the evidence that it does not.

## Verification

- Kernel under test: `7.3.5-1-cachyos` (`uname -r`), read on 2026-11-19.
- Node under test: `26.13.1`.
- Upstream source: `torvalds/linux`, tag `v6.12`, commit
  `adc218676eef25575469234709c2d87185ca223a`, whose `fs/proc/base.c` is the one
  the links above point at.

The harness opens a real process's `comm` while the process runs, ends the
process, waits for the operating system to reap it, and then reads the same
record twice: through the file it opened, and through the path.

```ts
// ${HOME}/temp/agent/proc-record-after-exit.ts
import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { open, } from 'node:fs/promises';

const child = spawn('sleep', ['60',], { stdio: 'ignore', },);
const pid = child.pid;
if (pid === undefined) throw new Error('the operating system started no process',);
const handle = await open(`/proc/${String(pid,)}/comm`, 'r',);
console.log(`opened /proc/${String(pid,)}/comm while pid ${String(pid,)} ran`,);
child.kill('SIGKILL',);
await once(child, 'exit',);

async function outcomeOf(read: () => Promise<string>,): Promise<string> {
  try {
    return `text ${JSON.stringify(await read(),)}`;
  }
  catch (error) {
    const carried = ((error !== null) && ((typeof error) === 'object') && ('code' in error))
      ? String(error.code,)
      : 'no code';
    return `threw ${carried}: ${error instanceof Error ? error.message : String(error,)}`;
  }
}

console.log(`read through the open record: ${await outcomeOf(async function readOpen() {
  return await handle.readFile({ encoding: 'utf8', },);
},)}`,);
console.log(`read through the record's path: ${await outcomeOf(async function readPath() {
  const reopened = await open(`/proc/${String(pid,)}/comm`, 'r',);
  return await reopened.readFile({ encoding: 'utf8', },);
},)}`,);
await handle.close();
```

Its output on the kernel above, with the process numbers replaced:

```text
opened /proc/<pid>/comm while pid <pid> ran
read through the open record: threw ESRCH: ESRCH: no such process, read
read through the record's path: threw ENOENT: ENOENT: no such file or directory, open '/proc/<pid>/comm'
```

The catalogue:

- Works cleanly: reading a record while its process runs returns the record's
  text; reading by path after the process is reaped returns `ENOENT`.
- Fails: reading an already-open record after the process is reaped returns
  `ESRCH` from the read, with no path in the message.

## Verified workarounds

At the driver's boundary, `ENOENT` and `ESRCH` are the same answer: the process
is gone. `package/pi-shared/subagent-driver/src/process-ended-codes.ts` holds
that pair once, and each read keeps its own answer for a record it cannot read:

- `comm`: the program is unknown, which is what a missing record already gave,
  so the entry stays in the scan with `?` as its program;
- `cgroup`: no control group, so the process is not counted as the run's work;
- `stat`: no parent record, so a chain of parents ends without reaching the run;
- `cmdline`: no words, so the entry names nothing.

Tradeoff: a scan that reads `comm` as `?` keeps a process that has ended,
because the folder it worked in was read while it lived. The run then waits one
poll interval (30 s at `WORK_POLL_MILLISECONDS`) longer than it needs to for a
process that is already gone; the next look at the folder no longer lists it.
Dropping the entry instead would give `ESRCH` and `ENOENT` different answers,
which is the inconsistency this change removes.

For a test, the read is the seam: `package/pi-shared/subagent-driver/src/reads-of-a-process-folder.ts`
hands over the text and link reads, and a test replaces one with a function
that throws an error carrying `code: 'ESRCH'`. A plain file in a fixture
folder cannot produce `ESRCH`, and the harness above produces it only through
an open file, which is no seam for a reader that opens by path.

## What does not work

- A plain file in a fixture process folder. A missing file answers `ENOENT`;
  a file that exists is read successfully, whatever the kernel would answer for
  a record of a released task.
- Racing a real process's exit between the driver's listing and its read. The
  window is two system calls wide, and no amount of scheduling control makes
  the outcome deterministic in a test; a test that tries is a flake.
- Reading through an already-open descriptor as the driver's own read. It does
  answer `ESRCH`, but the driver opens the record and reads it inside one
  `readFile`, so an injected descriptor cannot stand in for that call.

## Upstream filing decision

Nothing to file upstream, and no draft is kept.

1. Is it really upstream's fault? No. Answering `ESRCH` for a record whose task
   is released is the documented purpose of the handler, and the interface's
   contract is that a record of a process that is gone cannot be read.
2. Can upstream fix it? There is nothing to fix: `ESRCH` is the correct answer.
3. Are they supporting this use case? Not applicable: this is not a use case
   upstream declined, it is the kernel telling a reader that the process ended.
4. Would the repo welcome our contribution? Not applicable: no contribution is
   proposed.
5. Will they likely fix it? No: the behaviour is the fix's answer, not a bug.
6. Have we prototyped a minimal fix? Not applicable: the fix belongs at the
   consumer, which this package now carries.

`.out-of-scope/` was read on 2026-11-19 and holds no exemption for kernel
behaviour, so the decision rests on the six constraints above.
Any future filing would be the poster's decision and would carry that person's
risk; this doc recommends none.

[comm]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L1746-L1758
[single]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L801-L817
[cgroup]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L3372
[stat]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L3334
[cwd]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L222-L236
