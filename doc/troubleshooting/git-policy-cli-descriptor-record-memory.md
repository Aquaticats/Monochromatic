# Linux 7.3.3, reading another process's `/proc/<pid>/fdinfo/<n>`: the kernel builds the whole record before the reader gets a byte, so one inotify descriptor's record costs hundreds of megabytes in the reader's control group and a capped scope is killed for it

A scan that looks for the process holding a file open reads the descriptor record of every descriptor it walks.
The kernel builds each record whole,
in one buffer,
before the reading process receives its first byte,
so the record's size is decided by whoever holds the descriptor.
An inotify descriptor's record holds one line for every watch it holds,
and the limit on watches on this machine was ten million
(`/proc/sys/fs/inotify/max_user_watches`):
one read of such a record costs the record's size in kernel memory,
charged to the reading process's control group with `GFP_KERNEL_ACCOUNT`.
On 2026-11-12 at 09:40:23 the kernel killed a wrapper process in a scope capped at 512 MB for this,
while 35 GiB of the machine were free (repository issue 6287).

## Symptom

A Git command through the wrapper prints nothing and does not return.
`ps -o pid,stat,wchan:24` shows it asleep in the kernel,
in the function that prints the record:

```text
D inotify_show_fdinfo
```

which is `ps -o stat=,wchan=` for such a process (seen in another agent's session on this machine at 09:20:48 on 2026-11-12).

No deadline ends it.
The scan's own time bound is checked between processes,
and a record read cannot be interrupted once it has begun,
so `SIGTERM` stays pending for as long as the record is being built.
More than one wrapper process can be in that state at once:
three were,
at 09:20:56 on 2026-11-12,
because the readers of one descriptor's record wait on each other
(see "Readers of one record wait on each other").

Where the memory is capped,
the kernel ends the process and the desktop reports a machine low on memory.
The kernel's own report of that ending
(`journalctl --dmesg --since '2026-11-12 09:40:22' --until '2026-11-12 09:40:25'`):

```text
oom-kill:constraint=CONSTRAINT_MEMCG,nodemask=...,cpuset=...,mems_allowed=...
memory: usage 524288kB, limit 524288kB, failcnt 86629
anon 0
file 0
kernel 536809472
vmalloc 535171072
```

`anon 0` and `file 0` are the point:
nothing of the program's own address space was large.
The charge is `kernel`,
almost all of it `vmalloc`,
and the killed process's stack is inside `seq_read_iter` under `vfs_read`,
allocating with `__kvmalloc_node`.

A trace of the same command for twelve seconds,
in a scope capped high enough to survive
(`/var/home/user/temp/agent/probe-recover-hang/strace.txt`),
holds 3,844 opens of `/proc/<pid>/fdinfo/<n>` and ends in one whose read never returned:

```text
openat(AT_FDCWD, "/proc/178989/fdinfo/3", O_RDONLY|O_CLOEXEC) = 8
read(8, ...
```

`/proc/178989/fd/3` is `anon_inode:inotify`,
of the process `fsnotifier`,
the file watcher of an editor.
The limit on watches on this machine was ten million (`/proc/sys/fs/inotify/max_user_watches`).

Measured on this machine on 2026-11-12 with the installed release `0.6.1-alpha.13` and the same command
on the branch that fixes this (`fix/git-policy-cli-descriptor-scan-never-reads-a-record-whose-size-another-process-decides`),
each in its own scope with `MemoryMax=8G`,
waiting for an `index.lock` that a process of the run's own held for eight seconds:

```text
installed wrapper: 53.75 s, scope memory.peak 887,349,248 bytes
branch build:       7.72 s, scope memory.peak   5,861,376 bytes
```

The editor's watcher was running throughout,
and its descriptor's link text was `anon_inode:inotify`
(read with `readlink`,
never through its record).

## Root cause

### The scan read the record before asking whether it could use it

Three read sites opened a descriptor's record for every descriptor they walked,
before any test of whether the descriptor could be the file they look for
(in this repository,
`package/git-policy/cli`):

- `src/native/process/process_use.rs`,
  `descriptor_is_file_with`,
  for a file named by device and inode
- `src/native/process/process_use.rs`,
  `mounted_descriptor_is_file`,
  for a file named by mount and inode
- `src/native/process/open_file_of_a_descriptor.rs`,
  `mount_id_of`,
  the second read of the same record

The record's inode number was the cheap test that came first,
and it cost whatever the record's size cost.

### The kernel builds a whole record before the reader gets a byte

An inotify descriptor's record is printed by `inotify_show_fdinfo`,
which hands `show_fdinfo` a printer and a group,
and `show_fdinfo` walks the group's whole mark list inside that one call
([fs/notify/fdinfo.c:26-36][fdinfo-show],
[fs/notify/fdinfo.c:76-97][fdinfo-inotify]):

```c
	fsnotify_group_lock(group);
	list_for_each_entry(mark, &group->marks_list, g_list) {
		show(m, mark);
```

Every mark prints one line,
so the record holds one line per watch.
A read of the record reaches `seq_read` through the fdinfo file's own operations
([fs/proc/fd.c:60-70][fdinfo-show-call],
[fs/proc/fd.c:108-113][fdinfo-ops]),
and `seq_read_iter` builds the record in one buffer:
when it does not fit,
it frees the buffer,
doubles it and builds the record again from the start
([fs/seq_file.c:242-249][seq-grow]):

```c
		// need a bigger buffer
		m->op->stop(m, p);
		kvfree(m->buf);
		m->count = 0;
		m->buf = seq_buf_alloc(m->size <<= 1);
```

and `seq_buf_alloc` refuses a size above `MAX_RW_COUNT` and allocates with `kvmalloc(size, GFP_KERNEL_ACCOUNT)`
([fs/seq_file.c:34-40][seq-alloc]):

```c
	if (unlikely(size > MAX_RW_COUNT))
		return NULL;

	return kvmalloc(size, GFP_KERNEL_ACCOUNT);
```

Two consequences decide this defect.

- The reader's count bounds nothing.
  `read(fd, buf, 4096)` of a record of 8 MB still builds 8 MB in the kernel,
  because the printer fills one buffer and has no way to yield.
- The charge is the reader's.
  `GFP_KERNEL_ACCOUNT` charges the buffer to the control group of the task that read,
  which is why the killed process's cgroup showed `kernel` and no `anon`.

The largest buffer a record costs is the largest doubling of a page that is not above `MAX_RW_COUNT`,
which is 1 GiB:
the buffer doubles from a page,
and the doubling past 1 GiB is 2 GiB,
above `MAX_RW_COUNT` (`INT_MAX` rounded down to a page,
about 2 GiB).
Past it the allocation is refused and the read fails with `ENOMEM`,
but a scope capped below it is killed first:
a scope capped at 4 GiB peaked at 580,087,808 bytes within twelve seconds of the same command,
and the read had still not returned.

### Readers of one record wait on each other

`show_fdinfo` holds `fsnotify_group_lock(group)` for the whole mark list,
and that lock is a mutex on the group
([include/linux/fsnotify_backend.h:287-291][group-lock]):

```c
static inline void fsnotify_group_lock(struct fsnotify_group *group)
{
	mutex_lock(&group->mark_mutex);
```

So two wrapper processes reading the same descriptor's record do not read it twice as fast:
the second waits for the first to finish building the whole record.
Each has its own `seq_file` and builds its own buffer in turn,
so N readers take the record's build time N times over,
one after another.
The cost of one Git command therefore grows with the number of wrapper commands running on the machine,
which is why three processes were asleep in that function at 09:20:56.

### How the kind is known before the record is opened

The descriptor's kind has to be established without opening its record,
and without walking the descriptor's link,
because a walk revalidates the target's directory entry on the target's own file system and a stopped server holds it
(`doc/troubleshooting/fuse-descriptor-attributes.md`).

`readlink(2)` of `/proc/<pid>/fd/<n>` is a different operation from walking it.
The kernel's `proc_pid_readlink` fetches the descriptor the target process already holds,
asks `d_path` for a name and copies that name out
([fs/proc/base.c:1815-1852][proc-readlink]):

```c
	pathname = d_path(path, tmp, PATH_MAX);
```

`d_path` answers from the dentry cache:
for a kernel object that has no name in a mount tree it calls the
pseudo file system's `d_dname`,
and otherwise walks `d_parent` and the mount table
([fs/d_path.c:265-297][d-path]):

```c
	if (path->dentry->d_op && path->dentry->d_op->d_dname &&
	    ...
		return path->dentry->d_op->d_dname(path->dentry, buf, buflen);
```

Neither path calls a file system's `d_revalidate`,
and neither reads an inode's attributes,
so no server is asked anything and no file system can hold the read.
A name that is not a path is therefore a descriptor of no file of a file system,
and an inotify descriptor's name is `anon_inode:inotify` (no brackets,
on this kernel).

The reference line above replaced an earlier reading of this defect's neighborhood:
that a short read of a record bounds it.
It does not,
for the reason in "The kernel builds a whole record".

## Verification

Kernel under test:
`uname -r` = `7.3.3-ogc1.1.fc45.x86_64`,
sources at tag `v7.3`
(commit `c03c1a14507dfaf8b856c2608f9ac216021fc222` of `torvalds/linux`).
`fs/seq_file.c` at that tag is byte-identical to the copy this investigation read at
`/var/home/user/temp/agent/kernel-seq-file/seq_file.v7.3.c` (`diff` reported no difference).

Wrapper under test:
`0.6.1-alpha.13` installed,
and the branch build of the fix at
`package/git-policy/cli/src/native/process/descriptor_record.rs`.

### The fixture: a program that holds many watches, and a reader that measures one record

A few lines of Rust,
no dependencies beyond `libc`,
in a scope of its own:

```rust
let descriptor = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
for number in 0..count {
    let path = root.join(format!("w{number:08}"));
    std::fs::File::create(&path)?;
    unsafe { libc::inotify_add_watch(descriptor, name.as_ptr(), IN_ALL_EVENTS) };
}
```

the reader then reads that record once with `std::fs::read_to_string`,
the way the wrapper did,
and samples its own cgroup's `memory.stat` `kernel` line while the read is in flight
(the buffer is released the moment the read returns,
so a look afterwards sees nothing).
Each process runs in a `systemd-run --user --scope` of its own with `MemoryAccounting=yes` and `MemoryMax=8G`,
so no measurement can end in a kill.

Measured on 2026-11-12,
one read per row:

```text
watches    record bytes  lines   read time     kernel during read   vmalloc during read
   1000         142,774   1004     4.1 ms                 0                    0
   4000         571,774   4004    16.7 ms           139,264                    0
  16000       2,299,679  16004   190.8 ms         4,358,144            4,194,304
  64000       9,211,679  64004   264.3 ms        16,957,440           16,777,216
 256000      37,050,144 256004  1281.3 ms        67,416,064           67,108,864
```

The record grows by about 144 bytes per watch and the kernel's charge by about 256 bytes per watch,
which is the next power of two above the record,
held while the read runs.
The 4,000-watch row is the one the kernel answered from `kmalloc`,
so its charge lands in no named
`memory.stat` line;
from 16,000 watches the buffer is a `vmalloc` chunk and shows in `vmalloc` as well.
Both are `kernel` memory in the control group's accounting,
and both are the reader's.

### The package's own controls, before and after

The package's holder controls that scan this host's process table read every process's records under the old code,
so they are a ready measure of the defect on a machine where an editor's watcher runs:

```text
process::process_use::tests::the_scanning_process_is_not_reported   163.4 s  ->  0.14 s
process::process_use::tests::two_subvolumes_that_share_an_inode_number_are_two_files  171.4 s -> 0.27 s
process::process_use::tests::the_scan_costs_nothing_for_a_record_whose_size_another_process_decides
                                                                    fails    ->  0.16 ms scan
```

The two long rows are one run of those controls against the old code in this worktree on 2026-11-12;
the same run read the editor watcher's record through the defect,
which is what took the minutes.
The new control's own line,
printed when it passes:

```text
CONTROL the descriptor scan pays nothing for a record another process sized: 2000 watches hold 285774 bytes
of record and the scan took 199.669µs, charged 0 bytes of kernel memory; 64000 watches hold 9211679 bytes and
the scan took 164.69µs, charged 0 bytes; one direct read of the large record was charged 25776128 bytes
```

### The command a person runs

`git merge` of a branch that fast-forwards is the wrapper path that scans for the holder of `index.lock`
(`wait_for_index_release_here` through `scan_platform_holders_bounded`),
and it needs no commit service,
so it runs with a disposable `HOME` and `XDG_RUNTIME_DIR`.
The lock was held by a process of the run's own for eight seconds and then removed,
and each arm ran in its own scope with `MemoryMax=8G`:

```text
installed wrapper: 53.75 s, scope memory.peak 887,349,248 bytes, merge exit 0
branch build:       7.72 s, scope memory.peak   5,861,376 bytes, merge exit 0
```

A plain `git status --porcelain` does not scan in this release:
traced with
`strace --trace=openat`,
it opened no `/proc/<pid>/fdinfo/<n>` at all,
with a held `index.lock`
and with a stale reference lock present.
The `status --porcelain` of the incident report ran a build of that review worktree,
not this release.

## Verified workarounds

### The gate before the read, and the bound on what is taken

The fix is
`package/git-policy/cli/src/native/process/descriptor_record.rs`,
one function every scan of another process's descriptors goes through:

- the descriptor's link text is read first,
  with `readlink(2)`;
- a descriptor whose text is not a path is not a file of a file system,
  and its record is not opened;
- a descriptor whose text begins `anon_inode:` is a kernel anonymous inode (inotify,
  fanotify,
  epoll and their family),
  which no scan here looks for,
  and its record is not opened;
- a scan for a mounted file (a pipe or a socket) reads those two kinds and still skips `anon_inode:`;
- a record that is opened is taken up to 4 KiB,
  and one longer than that is not used at all.

Tradeoffs,
which are the reason this is a gate rather than a smaller read:

- A descriptor whose link text cannot be produced (a path longer than the kernel's own `PATH_MAX` buffer)
  is a descriptor that may hold the file,
  so a lock behind such a path is unproven and stays.
  The person is told which file to remove,
  as every unproven lock's notice says.
- A record past the bound is a file that may be held,
  for the same reason.
- A record that cannot be read at all stays the older case,
  a process the scan could not look at,
  which the service tolerates as it always has.
- A pipe's record is still read,
  because a step's own output pipe is found by its mount number and inode.

The scan of the process table also carries a two-second ceiling now
(`process_use::PROCESS_SCAN_CEILING`,
the same bound and the same measurement as the commit service's
`SERVICE_SCAN_BUDGET`),
checked before every process and every descriptor,
and a scan that reaches it reports that it did not finish,
which every caller already reads as a table it may not act on.
None of this bounds a single record read,
which cannot be interrupted:
what bounds a record read is the gate.

Commits:
`cea6b00c3f` (the gate,
the bound and the scan's ceiling),
`d4dc13e2bb` (its controls),
`2e4b00911a` (the rest of the family).

### One record read that another process still decides the cost of

A descriptor whose link text is a path still has its record read,
because the inode and mount number it holds are how a file is matched.
The record of a path-named descriptor is `pos:`,
`flags:`,
`mnt_id:`,
`ino:`,
the file's own locks and whatever that file type's `show_fdinfo` prints,
which for every type reachable through a path here is a handful of lines.
A file type that printed a list would still make the kernel build its record whole;
the 4 KiB take keeps the wrapper from holding it,
and a record past it is a file that may be held,
but the kernel's own buffer for such a record stays the one read whose cost another program can decide.
No file type was found here that does this through a path.

## What does not work

- A short read:
  `read(fd, buf, 4096)` of an 8 MB record still builds 8 MB in the kernel
  (`fs/seq_file.c`,
  `seq_read_iter`),
  because the printer cannot yield inside one record.
- `O_NONBLOCK`:
  the blocking happens inside the printer that builds the record,
  not in the read's wait for data.
- The scan's time bound alone:
  it is checked between processes,
  and one record read outlives it
  (measured:
  126 s in one wrapper before a reviewer ended it,
  and 53.75 s for one 8 MB record here).
- `statx(2)` on the descriptor's link to learn its kind:
  that walks the link and asks the target's file system to
  revalidate its directory entry,
  which a stopped FUSE or NFS server holds for ever
  (`doc/troubleshooting/fuse-descriptor-attributes.md`).
- `pidfd_getfd(2)` to copy the descriptor:
  closing the copy runs the file system's flush,
  which a stopped server holds,
  and the copy is refused for a process this one may not attach to.
- Reading the record of a process this investigation did not start,
  to look or to measure:
  that read is the defect.
  A record other than one of the run's own fixture processes was not read here;
  where a measurement needed one,
  the fixture held the descriptor.
- `/proc/<pid>/environ`,
  `/proc/<pid>/cmdline`,
  `/proc/<pid>/maps`,
  `/proc/locks` and `cgroup.procs`:
  looked for with `rg` over the release sources of `package/git-policy/cli` on 2026-11-12,
  none of them is read.

## The rest of the family: whole reads whose length the system's state decides

Each of these was read whole,
with nothing bounding it,
until this branch.
All now go through `foundation::plain_file::read_plain_file` or `read_plain_text_at`,
which refuse a file past the caller's limit rather than reading it whole
(and refuse a handle that is not a plain file at all).

- `src/native/process/open_file_of_a_descriptor.rs`,
  a process's `/proc/<pid>/mountinfo`:
  4 MiB
  (`plain_file::MOUNT_TABLE_LIMIT`;
  this host's own table was 5,250 bytes on 2026-11-12).
  A table past it is a mount the scan cannot classify,
  so the file is one that may be held.
- `src/native/subvolume/fs_mount_options.rs`,
  this process's `/proc/self/mountinfo`:
  the same 4 MiB.
- `src/native/landing/unreachable_git_dir_proven_gone.rs`,
  `/proc/self/mountinfo`:
  the same 4 MiB.
- `src/native/repository_link_check/repository_link_probe.rs`,
  `/proc/self/mountinfo`:
  the same 4 MiB.
- `src/native/process/process_tree.rs`,
  one thread's `/proc/<pid>/task/<tid>/children`:
  1 MiB,
  which is about forty thousand children;
  a list past it is not used,
  as a missing one already was not.
- `src/native/lock/stale_ref_lock_judge_environment.rs`,
  `/proc/stat`:
  4 MiB,
  one line per processor (4,639 bytes here on eight).

These are read whole and are already bounded by the kernel's own format,
one small fixed record each,
so they stay as they are:

- `/proc/<pid>/stat` (the kernel's fields,
  with a command name of 16 bytes),
  `/proc/<pid>/io`,
  `/proc/<pid>/status`,
  `/proc/<pid>/comm`,
  `/proc/<pid>/wchan`,
  `/proc/self/task/<tid>/io` and `/proc/self/task/<tid>/stat`
- `/proc/uptime`,
  `/proc/loadavg`,
  `/proc/stat`'s boot line,
  `/proc/sys/kernel/pid_max`,
  `/proc/sys/kernel/random/boot_id`,
  `/proc/self/cgroup` (one line),
  `/sys/fs/cgroup/.../pids.max` and `pids.current` (one number each),
  `/proc/cmdline`
- a descriptor's own record,
  now read through the gate above

## Upstream filing decision

Nothing to file.
The record has to list every watch,
and `seq_file` has to build one record before it can answer a read;
both are the interface,
not a defect in it.
A streaming `show_fdinfo` would change `seq_file`'s contract for every file under `/proc`,
and the cost here is a consumer's:
a scan that opens a record it cannot use.
The consumer's rule is therefore:
know the descriptor's kind before opening its record,
and bound what is taken from one that is opened.

[fdinfo-show]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/notify/fdinfo.c?plain=1#L26-L36
[fdinfo-inotify]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/notify/fdinfo.c?plain=1#L76-L97
[fdinfo-show-call]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/proc/fd.c?plain=1#L60-L70
[fdinfo-ops]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/proc/fd.c?plain=1#L108-L113
[seq-grow]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/seq_file.c?plain=1#L242-L249
[seq-alloc]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/seq_file.c?plain=1#L34-L40
[group-lock]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/include/linux/fsnotify_backend.h?plain=1#L287-L291
[proc-readlink]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/proc/base.c?plain=1#L1815-L1852
[d-path]: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/d_path.c?plain=1#L265-L297
