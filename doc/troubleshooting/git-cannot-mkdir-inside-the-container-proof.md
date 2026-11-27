# Git cannot create a directory inside a `localhost/git-policy-native-test` container, so a container proof that makes a worktree fails

Unapproved and non-final:
it records a reproduction,
what it ruled out,
and the root cause read afterwards from the audit log.

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

## What it blocked

`package/git-policy/cli/bin/service-manager-install-proof.ts` could not run,
and `package/git-policy/cli/bin/service-manager-proof.ts` stopped at the same step;
the fix is in the section `What fixed it, and what the proofs print now` below.
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
- **Not visible to the round as AppArmor**:
  `journalctl --since -25 min --lines 3000` holds no
  `apparmor="DENIED"` or `audit` denial,
  and `dmesg` is not readable from a session (`Operation not permitted`).
  The records are in `/var/log/audit/audit.log`,
  under "Root cause".

## Root cause

Read by the coordinating session on 2026-11-27T08:38:28-05:00 from `/var/log/audit/audit.log`,
which a round's session cannot read.

This host's AppArmor profile `git` attaches to `/usr/bin/git` by its path,
inside a container as well as outside it,
so the container's Git runs confined by the host's profile while the container's shell and Python do not.
That is why only Git's calls fail.
Two different refusals stand behind the one message:

- In `/work`,
  which the profile grants to its owner (`owner /work{,/**} rwlk`),
  the proof runs Git as the container's root,
  and `/work` is owned by the container's user.
  Root needs the capability `dac_override` to write there,
  and the profile refuses it:
  17 records `apparmor="DENIED" operation="capable" profile="git" comm="git" capname="dac_override"`
  at 08:26 and 08:27.
  A shell's `mkdir` of the same path succeeds because the shell holds the capability and no profile takes it away.
- In `/tmp` and `/work2`,
  the profile holds no rule at all,
  so Git's `mkdir` is refused for any user:
  six records `operation="mkdir" profile="git" name="/tmp/zz/"` and the like,
  `requested_mask="c"`.

The capability is refused on purpose
(`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
"The `git` `dac_override` records are the any-name fixture's Git (issue 6459)"),
so the remedy is in the proof:
it runs Git as the owner of the directory it writes,
as the wrapper's own fixtures do since issue 6459.
The probes in `/tmp` are not work anything needs,
and no rule was added for them.

## What fixed it, and what the proofs print now

Issue 6587's fix is the proof's own.
`makeWorktrees` (`bin/service-manager-worktrees.ts`)
hands `/work` to the user every command inside the machine runs as,
which is the container's root (`containerUser` in `bin/service-manager-machine.ts`),
before its first Git call,
so every Git call writes files whose owner is the user running it,
and no call asks for a capability.
The folder was the image build's all along
(`COPY --chown=1000:1000 ${imageRootDirectory} /work`,
`bin/native-test-containerfile.ts`),
and the proofs made the folder themselves before the image began to carry it,
which is why this step worked until 2026-11-13 and stopped.

The hand-over was measured on 2026-11-27 in one container of that image,
as the container's root.
After `chown 0:0 /work`,
`git init`,
`git add`,
`git commit` and
`git worktree add` below `/work` each exit 0,
where before it each ended with
`fatal: could not create leading directories ...: Permission denied`.
The two proofs were then run once each on that image,
as their tasks name them:

- `mise run //package/git-policy/cli:test:service-manager` now passes `makeWorktrees` on both arms:
  each machine holds 1557 units,
  the desktop stand-in comes up,
  and every step of both arms runs.
  It then fails its own judgement,
  for reasons that are not this refusal:
  every commit through a release inside the machine answers `status=1`,
  and the arm from `main` asks for no reload,
  because every release this host holds was built after the remedy of 2026-11-12 (`ef1bbc27b6`),
  so no argument list makes that control reproduce the 2026-11-12 harm.
- `mise run //package/git-policy/cli:test:service-manager:install` also passes `makeWorktrees`,
  and stops at its seeding commit:
  the release before this one answers
  `This account has no permission to write /work/one, so this commit was not made. ... status=1`,
  and the machine's own journal answers
  `outcome=failed:repository-permission-denied`,
  with `git update-ref` as its last Git step.
  Every file below `/work/one` is owned by the container's root,
  and that machine's service and its Git run as that root,
  so this is not a folder another user owns:
  the refusal stands in the landing's branch write,
  and what refused it is what `/var/log/audit/audit.log` records,
  which a round cannot read.

## What is still owed

The install proof has not run to its end yet.
Its seeding commit is refused with `repository-permission-denied`,
which is a later defect than the one this record is about,
and its Git calls are no longer refused for a capability:
the hand-over above is what this record owed.

## Upstream filing decision

Nothing is filed.
The failing component is this host's confinement of a program by its path,
working as its profile is written,
and the defect is the proof's choice of user.
