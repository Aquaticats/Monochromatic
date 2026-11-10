# Linux `/proc/<pid>/cmdline`: a process that ends mid-read answers `ESRCH`, and its open answers `ENOENT`

## Symptom

Code that lists `/proc` and then reads each listed process's `cmdline` failed,
only in some runs,
while a test suite watched short-lived children:

```text
Error: ESRCH: no such process, read  @depends.unit.test.ts:135 return readFileSync(`/proc/${entry}/cmdline`, 'utf8',) === `sleep\0${seconds}\0`;
```

The Node error carried `errno: -3`,
`code: 'ESRCH'`,
and `syscall: 'read'`.
Two later runs of the same tree were green,
because the failure needs a listed process to end inside the window between the listing and the read.

Two error variants come out of that window:

- Read of a descriptor opened while the process lived,
  after the process was released:
  `ESRCH` with `syscall: 'read'`.
- Open of `/proc/<pid>/cmdline` after the process was released:
  `ENOENT` with `syscall: 'open'`.

A guard that matches only `ENOENT` treats the second as an ordinary exit,
and treats the first as a real failure,
so a normal process exit reddens the suite.

## Root cause

The read half is decidable from the kernel source.
`proc_pid_cmdline_read` looks the task up from the open file's inode,
and returns `-ESRCH` when that lookup finds nothing
([`fs/proc/base.c:394-410`][kernel-cmdline-read],
v6.12 at commit `adc218676eef`):

```c
static ssize_t proc_pid_cmdline_read(struct file *file, char __user *buf,
				     size_t count, loff_t *pos)
{
	struct task_struct *tsk;
	ssize_t ret;

	BUG_ON(*pos < 0);

	tsk = get_proc_task(file_inode(file));
	if (!tsk)
		return -ESRCH;
	ret = get_task_cmdline(tsk, buf, count, pos);
	put_task_struct(tsk);
	...
```

A file descriptor keeps the proc inode alive after `release_task` detaches the pid,
so a read issued afterwards reaches that `-ESRCH` instead of failing at the open.
The open itself is an ordinary path lookup,
and the `/proc/<pid>` dentry is gone once the task is released,
so it fails earlier with `ENOENT`.

A process that is a zombie,
or a kernel thread,
is a third case and not an error:
the task still exists while its `mm` does not,
so `get_task_cmdline` returns zero bytes
([`fs/proc/base.c:379-392`][kernel-cmdline-stat]):

```c
	mm = get_task_mm(tsk);
	if (!mm)
		return 0;
```

An empty read therefore means "no argument vector to compare",
never "the process ended".

An earlier reading was wrong.
The helper in
`package/monorepo-task/core-exec/src/depends.unit.test.ts`
documented "an unreadable or vanished entry is a process that ended while being read",
while its guard matched `ENOENT` alone,
and the code assumed the read could report a vanished process only that way.
The field failure above disproves it:
the read reports `ESRCH`,
and the open reports `ENOENT`.

## Verification

Tested on kernel `7.2.15-ogc1.1.fc45.x86_64` with Node `v26.13.0`.
The pinned kernel source read for the root cause is v6.12,
commit `adc218676eef25575469234709c2d87185ca223a`.

The recipe is deterministic,
so it needs no process churn and no load:
open the descriptor while the child lives,
end the child,
reap it,
then read the held descriptor.

```ts
// probe.ts, run with `node probe.ts`
import { once, } from 'node:events';
import { closeSync, openSync, readFileSync, readdirSync, readSync, } from 'node:fs';
import { spawn, } from 'node:child_process';

const child = spawn('sleep', ['30'], { stdio: 'ignore', },);
await once(child, 'spawn',);
const pid = child.pid;

readdirSync('/proc').includes(String(pid)); // true, the entry is listed
readFileSync(`/proc/${pid}/cmdline`, 'utf8'); // 'sleep\030\0'
const held = openSync(`/proc/${pid}/cmdline`, 'r');

child.kill('SIGKILL');
await once(child, 'exit'); // libuv reaps the child here

readSync(held, Buffer.alloc(4096), 0, 4096, 0); // throws code=ESRCH syscall=read
readFileSync(`/proc/${pid}/cmdline`, 'utf8'); // throws code=ENOENT syscall=open
```

Its output:

```json
{
  "pid": 2941921,
  "listed": true,
  "alive": "\"sleep\\u000030\\u0000\"",
  "readOfHeldDescriptor": "threw code=ESRCH syscall=read",
  "openAfterReap": "threw code=ENOENT syscall=open"
}
```

Catalogues from the same run:

- Works cleanly:
  the entry is listed while the process lives,
  its `cmdline` reads as `sleep\030\0`,
  and a read of a zombie or kernel thread returns zero bytes.
- Fails with `ESRCH` on `read`:
  reading a descriptor opened before the process was released.
- Fails with `ENOENT` on `open`:
  opening the path after the process was released.

## Verified workarounds

Classify both codes,
and only those,
on errors that are real error values:

```ts
const VANISHED_PROCESS_ERROR_CODES: ReadonlySet<string> = new Set(['ENOENT', 'ESRCH',],);

function isVanishedProcessError(error: unknown,): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && ((typeof error.code) === 'string')
    && VANISHED_PROCESS_ERROR_CODES.has(error.code,);
}
```

This is what `package/monorepo-task/core-exec/src/process-ids-by-cmdline.ts` does,
and the tree already used the same pair in `package/ide/app/bin/inspect-watch-limit.ts`
before this was written up.
Tradeoffs:
a thrown non-error carrying `code: 'ESRCH'` still propagates,
because a value this code cannot read as an operating-system error names a bug elsewhere.
`EACCES` from a `hidepid` mount keeps failing loudly,
which is right,
since the scan then cannot see other users' processes at all.

Keep the defect reproducible by injecting the read instead of racing a real process.
The unit test in `package/monorepo-task/core-exec/src/process-ids-by-cmdline.unit.test.ts`
passes a reader that throws each error,
so the classification is proven without starting a child:

```ts
processIdsWithCmdline({
  entries: ['43'],
  expectedCmdline: `sleep\00.5\0`,
  readCmdline: function readVanishedTask() {
    throw systemCallError({ code: 'ESRCH', syscall: 'read', },);
  },
}); // []
```

## What does not work

- Matching the message text,
  such as `/no such process/`:
  the wording comes from libc and the Node version,
  not from the syscall's contract,
  and a translated message stops matching.
- Treating an empty read as a process that ended:
  a zombie or kernel thread legitimately reads zero bytes,
  and a real `sleep` never does,
  so this would both miss and misfire.
- Retrying `readFileSync` on failure:
  the directory is gone once it answers `ENOENT`,
  and the held-descriptor `ESRCH` is permanent for that inode,
  so a retry loop only spins.
- Treating every read error as a vanished process:
  that swallows `EACCES`,
  `ENOMEM` from the kernel's page allocation in `get_mm_cmdline`,
  and every programming error with it.
- Signalling a listed process without tolerating that it already exited:
  `process.kill` answers `ESRCH` for the same window,
  with `syscall: 'kill'`,
  so a scan-then-signal cleanup loop needs the same guard as the scan.

## Upstream filing

Nothing to file.
The kernel states the contract in the code itself:
a missing task is `-ESRCH`,
which is the intended answer for a process that has been released,
and the `/proc` manual page does not list the errnos for this file at all.
The consumer-side fix is to classify the pair,
which is what the workaround above does.

[kernel-cmdline-read]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L394-L410
[kernel-cmdline-stat]: https://github.com/torvalds/linux/blob/adc218676eef25575469234709c2d87185ca223a/fs/proc/base.c#L379-L392
