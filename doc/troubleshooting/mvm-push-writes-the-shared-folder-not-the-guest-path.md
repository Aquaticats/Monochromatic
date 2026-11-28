# `mvm push` writes the shared folder, not the guest path it is given

## Symptom

A guest script extracted a tarball from a path an earlier run had left behind.
The driver pushed the new tarball to `/root/subject.tar.gz`
and the guest then read exactly that path,
so the run built and ran the earlier run's code.
The only sign was a panic that named a line the current worktree no longer holds,
which reads as a stale build rather than as a stale input.

Measured on 2026-11-23,
in the throwaway guest `zfs-step-12-slot-state`
(`mvm --backend libvirt`):

```text
host:   md5sum <log dir>/subject.tar.gz
        41cb614a461981dbe2c62ee012ffeec0
guest:  md5sum /mnt/shared/subject.tar.gz
        41cb614a461981dbe2c62ee012ffeec0
guest:  md5sum /root/subject.tar.gz
        d09230987eb8d66b9cf3030d8536d3bd
        (written 40 minutes earlier by a run that pushed to that same path)
```

## Root cause

`mvm push` routes a file through the VM's shared directory whenever the VM has one
(`package/mvm/cli/src/file-transfer.ts`, module doc and `pushFile`).
The guest sees the file at `<shared mount>/<basename of the host path>`,
which is `/mnt/shared/<basename>` for a Linux guest
(`GUEST_MOUNT_POINT` in `package/mvm/cli/src/config.ts:63` is `/mnt/shared`;
the guest's cloud-init mounts the virtiofs share there,
`package/mvm/cli/src/cloud-init.ts`).
The `guestPath` argument selects only the file name, and `pushFile` returns the
path the guest can read:

```ts
// package/mvm/cli/src/file-transfer.ts
// guestPath => '/mnt/shared/setup.sh'
```

A VM created without `virtiofsd` has no shared directory;
there `push` copies through the QEMU guest agent and honours the absolute guest path.
Which of the two a VM has is recorded in its metadata,
so the same command means two different places from one VM to the next.

## The rule

A guest script that reads a pushed file reads it from the path `push` printed,
or from `/mnt/shared/<basename>`,
and never from the guest path the push command was given.
A push whose remote path matters is a push to a guest without a shared directory.
`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`
copies the tarball from `/mnt/shared/subject.tar.gz` for this reason,
and `package/git-policy/cli/e2e/slot-state-and-move-refusal-guest.ts` does the
same.

## Verification

The source citations above were read at the host's `package/mvm` checkout on
2026-11-23;
the two hashes are from one run's logs in
`/home/user/temp/agent/zfs-step-12-move-refusal-and-unmounted-report-built-ahead-2026-11-23/keep/guest-logs/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
The corrected driver's run is recorded in the same round's report;
the line above describes the cause, and that run is its test.

## The guest's read of a pushed file can be one run behind

### Symptom

A build inside a guest compiled the previous run's sources.
The driver pushed the archive and the guest copied `/mnt/shared/subject.tar.gz` at once,
as the two drivers above do,
and the copy was a different file:

```text
host:   stat -c %s <log dir>/subject.tar.gz          14581741   (written by this run)
host:   stat -c %s ~/.local/share/mvm/vms/<name>/shared/subject.tar.gz
                                                     14581741
guest:  stat -c %s /root/subject.tar.gz              14587156   (the copy the build unpacked)
```

The guest built that copy,
so its test binary still held the line the worktree no longer had,
and the run reported a panic naming a function that no longer exists.

### Root cause

The shared directory is a virtiofs mount,
and the guest's view of a file the host replaces at the same path is not refreshed by the
host's write.
Every run of a guest driver pushes its archive to the same name,
so a copy taken at once can be the previous run's bytes.
`mvm push` returning says the host wrote its own file, not that the guest reads it.

### The rule

A guest script that reads a pushed file waits for something pushed **after** it and then checks
the bytes it copied:

- the driver computes the archive's SHA-256 and pushes it as a stamp,
- the guest waits, with a bound, until the stamp's text holds that digest,
- the guest copies the archive and runs `sha256sum --check --strict` against the digest before it
  unpacks anything.

`package/git-policy/cli/e2e/step-21-arms/crate-build.ts` does exactly this,
and `e2e/step-21-arms/step-21-fixture.ts` holds the stamp's name.
A driver that pushes one file only has nothing to order it after,
so its guest script must at least wait for the file's own digest to change,
and a driver whose input never changes needs none of this.

The other direction is unaffected:
a log the guest writes into the shared directory is read by the host from the host's own file
system, so `mvm pull` and a direct read of the host path see it.

### Verification

Measured on 2026-11-28 in the throwaway guest `zfs-r13-step21-arms`
(`mvm --backend libvirt`),
in the round `.claude/worktrees/pi-zfs-step-21-guest-arms-for-dependency-repair-and-watches`.
The two sizes above are that run's own files;
the guest's built binary named a function the worktree no longer held,
which is what sent the round to the archive it had unpacked.
The remedy's run is recorded in that round's report.
