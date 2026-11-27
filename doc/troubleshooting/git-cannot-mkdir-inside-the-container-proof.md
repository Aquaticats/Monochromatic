# Git cannot create a directory inside a `localhost/git-policy-native-test` container, so a container proof that makes a worktree fails

Unapproved and non-final:
it records a reproduction and what it ruled out,
not a root cause.

## Symptom

Inside a container of
`localhost/git-policy-native-test:gpc-gate-wrapper-next-alpha-on-1c8acd5bfb-20261127T075019-0500`,
which is the gate image this host builds,
every directory Git creates fails:

```text
$ podman run --rm --user=0 --network=none --security-opt=label=disable \
    --entrypoint=/bin/sh localhost/git-policy-native-test:gpc-gate-wrapper-next-alpha-on-1c8acd5bfb-20261127T075019-0500 \
    -c '/usr/bin/git init --quiet /tmp/zz'
fatal: cannot mkdir /tmp/zz: Permission denied
```

The same container creates the same directory for another program:

```text
$ podman run --rm ... -c 'mkdir /tmp/yy; echo "mkdir=$?"'
mkdir=0
```

`strace` names the call and its errno:

```text
mkdir("/work/two", 0777)                = -1 EACCES (Permission denied)
fatal: could not create leading directories of '/work/two/.git': Permission denied
```

## What it blocks

`package/git-policy/cli/bin/service-manager-install-proof.ts` cannot run.
Its `makeWorktrees` step (`bin/service-manager-worktrees.ts`) makes a second worktree with
`/usr/bin/git worktree add --quiet --detach /work/two`,
and that call fails as above.
A run on 2026-11-27 reached one container holding 257 transient units,
with both releases copied in,
and ended at this step with
`NativeVerificationError: podman exec failed (status 128, signal none). fatal: could not create leading directories of '/work/two/.git': Permission denied`.
The run's output is kept at
`~/temp/agent/issue-6581-oxlint-clean-fence-measurement-and-service-manager-scripts-scratch/keep/install-proof-after.txt`.

## What was ruled out

Each probe ran in a fresh container of that image,
with `--rm --network=none --security-opt=label=disable`:

- **Not the directory's owner or mode**:
  `mkdir /work/two` succeeds from the container's shell in the same container,
  immediately after Git's `mkdir` of the same path failed.
  `/work` is `drwxr-xr-x 1000:1000`,
  left by the image build,
  and `/tmp` behaves the same way.
- **Not the user**:
  the failure is identical at `--user=0` and `--user=1000`.
- **Not the syscall number**:
  a direct `syscall(83)`,
  the legacy `mkdir`,
  from Python in the same container returns 0 and creates the directory.
- **Not only `/work`**:
  `git init /tmp/zz`,
  `git worktree add /tmp/wt-one` and
  `git worktree add /work2/wt` all fail the same way,
  so the path is not what decides it.
- **Not confirmed as AppArmor**:
  `journalctl --since -25 min --lines 3000` holds no
  `apparmor="DENIED"` or `audit` denial,
  and `dmesg` is not readable from a session (`Operation not permitted`),
  so a kernel audit record may exist that this session cannot see.

## What was not done

- The root cause is not established.
  A denial attached to the `git` program alone,
  while another program's identical call in the same container and user succeeds,
  is the shape of a per-program policy,
  an AppArmor profile or a seccomp filter keyed on the binary,
  and neither was confirmed.
- No workaround was applied.
  `AGENTS.md` rule `AAW` sends an AppArmor denial to the coordinating session,
  which widens the profile,
  and this session does not move the work to another path or another tool to get past it.

## Upstream filing decision

Nothing is filed.
Constraint 1 of the six fails:
the failing component is not shown to be Git's,
since another program in the same container makes the same call successfully,
so the defect is in this host's confinement of that container,
not in Git.
