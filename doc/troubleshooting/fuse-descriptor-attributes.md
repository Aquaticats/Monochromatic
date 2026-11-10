# Linux 7.2.15, `statx(2)` of a `/proc/<pid>/fd/<n>` link: a stalled FUSE or NFS server holds the read for ever, and no signal ends it

A read of a descriptor's file attributes through the path of its `/proc/<pid>/fd` link waits on the file system's own
dentry revalidation,
so a FUSE server that has stopped,
or an NFS server that is unreachable,
holds that read in
uninterruptible sleep with no deadline,
and `SIGTERM` does not end it.
A read of the same open file through a copy of the descriptor,
with `statx(2)` and `AT_EMPTY_PATH` plus
`AT_STATX_DONT_SYNC`,
answers in milliseconds from what the kernel already holds.

## Symptom

A command that inspects another process's descriptors stops with no output and no error.
`ps -o stat=` shows `D`:

```text
$ ps -o pid,stat,wchan:24,comm -p 1733568
    PID STAT WCHAN                    COMMAND
1733568 D    fuse_wait_request        python3
```

Nothing that runs a deadline helps.
`timeout 5 stat /proc/<pid>/fd/3` and `timeout 5 python3 statx_probe.py /proc/<pid>/fd/3 0x4000` both outlive their
deadline:
the process is asleep in the kernel,
so the signal stays pending until the request is answered.
Only aborting the FUSE connection ends it.
Measured here on 2026-11-09,
one such read stayed asleep for a whole 25-minute window with the server stopped,
and a second run of the same probe for 71 minutes;
the connection was aborted from outside in both cases.

The same hazard reaches a program that never touches the stalled mount:
a scan that identifies a **lock** by the device
and inode it has,
and resolves every descriptor whose record names that inode number,
asks each descriptor's file
system for attributes.
Any unrelated mount can then hold the scan.

## Root cause

`/proc/<pid>/fd/<n>` is a magic link to the open file
([fs/proc/fd.c:174](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/proc/fd.c#L174-L191),
`proc_fd_link`):

```c
static int proc_fd_link(struct dentry *dentry, struct path *path,
			struct file **file)
```

and [fs/proc/fd.c:214](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/proc/fd.c#L214) puts it on the descriptor's inode (`ei->op.proc_get_link = proc_fd_link;`).
A walk that follows it does not look the file up by name,
but the VFS still revalidates the dentry it lands on,
in the
walk itself ([fs/namei.c:1036, `d_revalidate`](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/namei.c#L1036-L1043)):

```c
		return dentry->d_op->d_revalidate(dir, name, dentry, flags);
```

For FUSE that revalidation is a request to the server ([fs/fuse/dir.c:391](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/fuse/dir.c#L391-L392),
`fuse_dentry_revalidate`,
which sends `FUSE_LOOKUP` at [fs/fuse/dir.c:432](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/fuse/dir.c#L432)):

```c
		fuse_lookup_init(&args, get_node_id(dir), name, &outarg);
```

A stopped or dead server never answers,
and the walk does not come back.
No `statx` flag reaches this step:
the flags govern the attribute read that follows
([fs/stat.c:181](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/stat.c#L181-L219),
`vfs_getattr_nosec`,
which narrows them to `AT_STATX_SYNC_TYPE` and hands them to the file system's `getattr`).

`AT_STATX_DONT_SYNC` does govern the attribute read,
and both file systems that can stall honour it.
FUSE ([fs/fuse/dir.c:1558](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/fuse/dir.c#L1558-L1567)):

```c
	if (!request_mask)
		sync = false;
	else if (flags & AT_STATX_FORCE_SYNC)
		sync = true;
	else if (flags & AT_STATX_DONT_SYNC)
		sync = false;
```

and NFS ([fs/nfs/inode.c:986](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/nfs/inode.c#L986-L990)):

```c
	if ((query_flags & AT_STATX_DONT_SYNC) && !force_sync) {
		if (readdirplus_enabled)
			nfs_readdirplus_parent_cache_hit(path->dentry);
		goto out_no_revalidate;
	}
```

with `generic_fillattr` at [fs/nfs/inode.c:1052](https://github.com/torvalds/linux/blob/7475ea8b58616b33c04afb976cf0fbaad9dd63fb/fs/nfs/inode.c#L1052-L1058) and FUSE's equivalent on the same `sync = false` path.
That is why a **descriptor** read answers and a **path** read does not:
the path reaches the same `getattr` only after the
walk has already asked the server to revalidate the entry.

### An earlier reading that was wrong

The first reading of this was that `AT_STATX_DONT_SYNC` bounds the read of the path too,
because the flag means "do not
synchronise,
take what is cached".
The measurement disproved it:
with the server stopped,
the path read blocked with the flag set (5006 ms,
ended only by
the probe's own `timeout`),
while the descriptor read of the same file answered in 21 ms.
The flag is honoured by `getattr`;
the walk's `d_revalidate` is a separate ask that no `statx` flag covers.

## Verification

Kernel under test:

```text
$ uname -r
7.2.15-ogc1.1.fc45.x86_64
$ fuse-overlayfs --version
fuse-overlayfs: version 1.17
$ fusermount3 --version
fusermount3 version: 3.18.3
```

Harness:
a FUSE overlay served by `fuse-overlayfs` inside a **private** mount namespace
(`unshare --mount --user --map-root-user --propagation private`),
a file in it opened by a process of the same account,
a `statx(2)` probe written with `ctypes`
(`AT_FDCWD` for a path,
`AT_EMPTY_PATH` for a descriptor),
and the server stopped with `SIGSTOP` after the mount's
attribute timeout had passed.

A private namespace is required,
not optional:
a FUSE mount created in the shared mount table propagates to every
process on the host,
and a stopped server there hung another agent's `find` under the same tree while this was being
investigated.

Patterns that answer,
with the same stalled mount:

```text
$ python3 statx_probe.py /proc/self/fd/7 0x4000     # AT_STATX_DONT_SYNC
dev=4194403 ino=423806924 mnt_id=3323                #  21 ms
$ python3 statx_fd.py 7 0x4000                       # AT_EMPTY_PATH | AT_STATX_DONT_SYNC
dev=4194403 ino=423806924 mnt_id=3323                #  21 ms
$ python3 -c 'import os,sys; os.lstat(sys.argv[1])' /proc/self/fd/7
                                                      #  10 ms, the link's own procfs inode
```

Patterns that do not answer,
with the same stalled mount:

```text
$ timeout 5 python3 statx_probe.py /proc/self/fd/7 0x0000   # following the link, sync
exit 124 after 5006 ms
$ timeout 5 python3 statx_probe.py /proc/self/fd/7 0x4000   # following the link, DONT_SYNC
exit 124 after 5005 ms
$ timeout 5 python3 statx_fd.py 7 0x0000                    # the descriptor, forced sync
exit 124 after 5009 ms
$ timeout 5 stat /proc/self/fd/7
exit 124 after 5005 ms                                       # coreutils stat, which statx's the path
```

Every kernel citation above is a commit-pinned permalink to the text it quotes.
Each one was checked by loading it in `agent-browser`
(the FUSE page for `fuse_dentry_revalidate` and for the `AT_STATX_DONT_SYNC` branch was read in the browser,
and the rest were checked line by line against the raw file at the same commit
`7475ea8b58616b33c04afb976cf0fbaad9dd63fb`,
because GitHub's viewer renders only a window of a very large file such as `fs/nfs/inode.c`).

NFS was read from the source only:
no NFS server was available on this host,
so the `nfs_getattr` branch above is not
measured here.
Its `goto out_no_revalidate` answers before any request is sent,
which is what `statx(2)` documents of the flag.

## Verified workarounds

### Copy the descriptor, then read it in memory

`pidfd_getfd(2)` copies another process's descriptor into this one without walking a path,
and `statx(2)` on the copy
with `AT_EMPTY_PATH` and `AT_STATX_DONT_SYNC` asks only for what the kernel holds.
This is what `src/native/process/open_file_of_a_descriptor.rs` of this repository does.

```rust
let pidfd: OwnedFd = pidfd_of(pid)?;                                   // pidfd_open(2)
let copied: OwnedFd = copied_descriptor_of(pidfd.as_raw_fd(), number)?; // pidfd_getfd(2)
let mut found: libc::statx = unsafe { std::mem::zeroed() };
unsafe { libc::statx(copied.as_raw_fd(), b"\0".as_ptr().cast(),
                     libc::AT_EMPTY_PATH | libc::AT_STATX_DONT_SYNC,
                     libc::STATX_BASIC_STATS, &mut found) };
```

Tradeoffs:

- `pidfd_getfd(2)` needs the permission a scan of `/proc/<pid>/fd` already needs (the same account,
  `ptrace_may_access`
  with `PTRACE_MODE_ATTACH_REALCREDS`,
  and a target that has not hidden itself with `PR_SET_DUMPABLE`).
  A refusal must be read as "this process cannot be looked into",
  never as "this process does not hold the file":
  the first leaves a lock unproven,
  the second would delete a live lock.
- It needs Linux 5.6 or later,
  and a sandbox whose seccomp filter allows `pidfd_getfd`.
- The copy is a reference to the same open file,
  so nothing on the file system is touched on the way in or out.
- `AT_STATX_DONT_SYNC` can return attributes up to the file system's cache timeout out of date.
  For a device number and an inode that is what the scan wants:
  they do not change while the file is open.

### Do not resolve a descriptor at all

A scan that treats every descriptor it cannot answer for as an unreadable process is bounded by construction,
since it
never touches the file system.
The tradeoff is the live-holder veto:
a program that holds the file open would no longer be found,
and a lock it holds
would be reported unproven rather than held.

## What does not work

- `timeout` and `SIGTERM`:
  the thread is asleep in the kernel,
  so the signal waits for the answer (measured:
  25 minutes,
  then 71 minutes on a second run,
  both ended only by aborting the FUSE connection).
- A worker thread with a deadline:
  the caller is unblocked,
  but the thread never returns,
  the mount cannot be unmounted
  while the request is pending,
  and a program that then needs the same path waits behind it.
- `statx(2)` with `AT_STATX_DONT_SYNC` on the path:
  blocks in the walk (measured 5006 ms).
- `AT_SYMLINK_NOFOLLOW` on the link:
  answers at once,
  with the procfs link's own inode,
  which says nothing about the
  file.
- `readlink(2)` and comparing the text:
  a bind mount,
  another mount namespace,
  a rename and a deleted file all make the
  text an ambiguous name for the file,
  so a holder is missed or another file is taken for it.

### A copy of the descriptor does not avoid the server either

`pidfd_getfd(2)` copies another process's descriptor without walking a path,
and `statx(2)` on the copy with
`AT_EMPTY_PATH` and `AT_STATX_DONT_SYNC` answered in 21 ms where the path read blocked,
as the measurements above show.
Two things make it unusable here.

Closing the copy runs the file system's `flush`,
not only the last reference's release:
`close(2)` reaches
`filp_close` → `f_op->flush`,
and FUSE's `fuse_flush` sends `FUSE_FLUSH` and waits for the answer
(`fs/fuse/file.c`,
`fuse_flush`).
With the server stopped,
the descriptor copy's close waits exactly as the path read
did,
inside a scan that checks its deadline only between processes.

The copy is also refused where the scan can still read:
`pidfd_getfd(2)` needs `PTRACE_MODE_ATTACH_REALCREDS`,
while
reading `/proc/<pid>/fd` needs only `PTRACE_MODE_READ`.
With `kernel.yama.ptrace_scope=1` (this host's
`/proc/sys/kernel/yama/ptrace_scope` is `0`) every unrelated process of the same account is readable and not
attachable,
so the copy fails with `EPERM` for the very holder the scan is looking for.

### The gate the wrapper uses instead

The descriptor's record (`/proc/<pid>/fdinfo/<n>`) names its mount (`mnt_id:`),
and that process's own mount table
(`/proc/<pid>/mountinfo`,
readable across accounts) says what the mount is.
The file is read only when the mount is on a
fixed list of local file systems,
where `statx` answers from the kernel's memory and the walk reaches no server;
every
other mount,
and a mount the table does not name,
is refused and its caller treats the file as one that may be held.
Measured on this host:
`/proc/1/mountinfo` and `/proc/<pid>/mountinfo` are readable as an unprivileged user,
and a file
in a btrfs subvolume reports an anonymous device (`0:108`),
while a file in a **nested** subvolume reports another
(`5243089`),
so the device and inode pair still separates two subvolumes' files whose mount numbers and inode numbers
agree.

### A container's `overlay` names layers it cannot reach

A kernel `overlay` mount is not a file system whose attributes are its own:
the overlay's own read asks a layer only when that layer sets a revalidation of its own,
and one overlay layer over
a stopped server can therefore hold the walk exactly as a direct read of that file would.

The layers are named in the mount's own options,
and a rule that reads those options and then asks the reading process
where the layers are cannot answer for a container,
because the layers are the host's directories.
Measured on 2026-11-09 inside this package's gate container (rootless `podman`):

```text
$ cat /proc/self/mountinfo | awk '$5=="/"'
3911 3649 0:2174 / / rw,relatime - overlay overlay rw,lowerdir=/var/home/user/.local/share/containers/storage/overlay/8bdcf7e8.../diff:...,upperdir=/var/home/user/.local/share/containers/storage/overlay/812b6dea.../diff,workdir=/var/home/user/.local/share/containers/storage/overlay/812b6dea.../work,...
$ ls -d /var/home/user/.local/share/containers/storage
ls: cannot access '/var/home/user/.local/share/containers/storage': No such file or directory
$ stat -c '%d %i' /work/probe-file
8388756 1752073190
```

The host's own view of the same tree is `/dev/dm-1` on btrfs,
so the layers are local,
and nothing inside the container
can say so.
Refusing that mount is not free either:
every descriptor of every process is on it,
so no scan can prove an empty
`index.lock` unheld,
an abandoned lock is kept for ever,
and every commit behind it waits its ceiling and then fails.
The release gate's log for `742076c71d` shows seventeen such failures in one run.

The device a mount table names is also not the device a file has,
which rules out the cheaper design of taking the
device from the mount's `major:minor` line:

```text
# on the host, a file on /var/mnt/encrypted (btrfs)
st_dev=4194388   mountinfo major:minor=0:79
# in the container, a file on / (kernel overlay)
st_dev=8388756   mountinfo major:minor=0:2196
```

btrfs and overlayfs both report an anonymous device for their inodes,
so those two numbers are different by design.

### What the wrapper does with an `overlay` descriptor

The walk is made by a child of the reading process,
and the answer is waited for only until a ceiling:

```rust
// src/native/process/open_file_of_a_descriptor_in_a_child.rs
let child = libc::fork();            // the child reads the link, writes its answer to a pipe, and exits
let answer = wait_for_answer(read_end, CHILD_READ_CEILING); // poll(2), 300 ms
```

The child allocates nothing after the fork:
the descriptor's path and the answer buffer are made before it.
A child that has not answered by the ceiling is sent `SIGKILL`,
and one the kernel does not let go of is handed to a
thread that collects it whenever it does,
the same bargain a command's stuck child gets.
The ceiling is `Unread::File` to the caller,
which a removal reads as a holder that may be alive,
never as one that is
not,
so a layer over a stopped server costs a leak of one kernel-stuck child and never a blocked commit.

Measured on this host on 2026-11-09:
one `fork`,
one descriptor read and one pipe answer took 0.5 ms on average over
two hundred runs,
and the slowest of them took 2.1 ms,
which is why the ceiling is 300 ms.

`fuse-overlayfs` is a different file system type,
served by a process of this machine,
and stays refused with the
other types a stopped server can back.

A descriptor of the reading process itself needs none of this:
`statx(2)` with `AT_EMPTY_PATH` on its own descriptor number walks no path and asks no file system to revalidate
anything,
so the count of its own descriptors of a lock is the same on every mount type.

## Upstream filing decision

Nothing is filed.
Walk of the six constraints:

1.  **Is it really upstream's fault?**
    No. The `d_revalidate` in the walk is by design,
    `statx(2)` documents `AT_STATX_DONT_SYNC` as "on a network filesystem,
    it may not involve a round trip",
    and the kernel offers the descriptor route that answers without a walk.
    This is behaviour of the interface,
    not a defect in it.
2.  **Can upstream fix it?**
    It could add a flag that forbids dentry revalidation,
    but nothing is broken that a fix would repair,
    so this constraint is not reached.
3.  **Are they supporting this use case?**
    Yes:
    `AT_STATX_DONT_SYNC` exists for callers that must not wait,
    and `pidfd_getfd(2)` exists for callers that need another process's open file.
4.  **Would the repo welcome our contribution?**
    Not reached,
    since there is nothing to contribute.
5.  **Do we have a reproduction they can run?**
    The harness above is ours,
    it needs a private mount namespace,
    a `fuse-overlayfs` build and a stopped server,
    and it reproduces a documented behaviour rather than a bug.
6.  **Is a filing in the public interest?**
    No.
    It would report a design as a bug.

The durable lesson is ours:
never read a descriptor's file through the path of its `/proc` link,
and never create a FUSE mount outside a private mount namespace.
