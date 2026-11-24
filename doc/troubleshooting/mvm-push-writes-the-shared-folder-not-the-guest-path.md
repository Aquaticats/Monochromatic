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
`/home/user/temp/agent/zfs-step-12-move-refusal-and-unmounted-report-built-ahead-2026-11-23/keep/guest-logs/`.
The corrected driver's run is recorded in the same round's report;
the line above describes the cause, and that run is its test.
