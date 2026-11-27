# Git cannot create a directory inside a `localhost/git-policy-native-test` container, so a container proof that makes a worktree fails

Unapproved and non-final:
it records a reproduction,
what it ruled out,
the root cause read afterwards from the audit log,
and what the two container proofs print now that both refusals behind the message are closed.

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

## What fixed it: the folder is given to the user Git runs as

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

## The second refusal behind the same message: the commit service's marks file

With the folder handed over,
`makeWorktrees` passes on both proofs,
and the next refusal is the landing's own.
`/var/log/audit/audit.log` held 18 records for the `git` profile in the hour before the round that
first met it
(`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
"Seventeenth batch"):

```text
apparmor="DENIED" operation="mknod" class="file" profile="git" name="/root/.local/state/monochromatic-git-policy-cli/service/<hash>/marks-<n>.lock" comm="git" requested_mask="c" denied_mask="c"
```

The service hands `git fast-import` an unnamed scratch file where it can.
Where it cannot,
as in this container,
`src/native/landing/service_land_objects_of_batch.rs` passes
`--export-marks=<journal>/marks-<seq>`,
and Git makes `marks-<seq>.lock` beside the journal.
No rule covered that name,
so Git was refused,
the landing failed,
and the service answered `outcome=failed:repository-permission-denied`.

`/etc/apparmor.d/local/git` gained a rule for that name,
and the rule matched nothing until it was corrected:
it read `marks-@{int}`,
and `@{int}` stops at ten digits while the service numbers its marks with sixteen.
Two more refusals of the same name followed from this record's own runs,
at 09:30:03 and 09:32:08,
so every run that started before 2026-11-27T09:32:43 met the refusal the first round met and says nothing new.
The rule as corrected,
the control that proves it in a container of the gate image,
and the rest of that batch are in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
"Seventeenth batch".
The audit log is the coordinating session's to read,
not a round's;
what a round can show is what the proofs print.

## What the proofs print now, from runs that started after the correction

- `mise run //package/git-policy/cli:test:service-manager:install -- --image <this host's gate image> --older-wrapper <the 0.6.1-alpha.47 executable> --candidate-wrapper <the 0.6.1-alpha.48 executable>`
  runs to its end and passes.
  Its own lines,
  in order:

  ```text
  machine gpc-reload-proof-4006674-1795791384: holding 257 units, so a reload of it costs what a loaded host's costs
  the program pointer the older release's texts name: /opt/release-old/git
  the installing command: Updating the commit service... | [main e711326] install | status=0
  every instance started again by the manager: 4 active
  the second worktree's next commit: [detached HEAD 77b0555] second-worktree |  1 file changed, 1 insertion(+), 1 deletion(-) | status=0
  starts of a wrapper service this run made: 3
  starts that failed at the EXEC step: 0
  units that reached the start limit: 0
  units the manager left failed: none
  the pair of commits at once: 0 and 0; commits 3 to 5 (2 added)
  THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
  ```

  That is the run on the tree that merged `main` at `653cb33d01`;
  the runs before those merges printed the same lines and passed,
  one of them with the service's own ten-second notice before the landing.

  So both worktrees were seeded through the older release,
  both landed their commits,
  the installing commit landed,
  the units the older release's text named started as `0.6.1-alpha.48` after that release was removed,
  no unit was left failed,
  and the pair of commits made at once added exactly two commits.
  Nothing about this refusal is in it:
  every Git step of both worktrees exits 0.
- `mise run //package/git-policy/cli:test:service-manager -- --image <this host's gate image> --main-wrapper <the 0.6.1-alpha.48 executable>`
  also runs every step of both arms,
  each machine holding 1557 units,
  with `commits that did not land: none` on both.
  It fails its own judgement,
  for one reason that is not this refusal:
  the arm from `main` asks for no reload and ends no stand-in,
  because every release this host holds was built after the remedy of 2026-11-12 (`ef1bbc27b6`).
  Its closing lines:

  ```text
  the arm from main asked for 0 manager-wide reload(s); the arm from this worktree asked for 0
  NativeVerificationError: THE PROOF FAILED: the release from main asked for no reload, so this fixture would prove nothing about it; the release from main did not end the unrelated desktop, so this fixture does not reproduce the harm
  ```

  The control arm's choice is proposed in
  `doc/planning/git-wrapper-reload-proofs-control-arm-without-a-pre-remedy-release.md`,
  and nothing is changed for it.

## `dac_override` stays refused, and no path stands behind a `capable` record

The same runs left `capable` records for the `git` profile asking for `dac_override`
(7 at 09:30 and 5 at 09:32,
in the audit log the coordinating session read).
That capability stays refused on purpose,
for the reason recorded under
"The `git` `dac_override` records are the any-name fixture's Git (issue 6459)":
a grant would let a root Git pass every file permission check.
A `capable` record names no path,
so what asked for it is read from the steps instead:
the folder every Git call of these proofs writes is owned by the user that call runs as,
and after the correction no Git step of either proof failed with `Permission denied`.
Nothing the proofs write was refused for a capability.

## What a failing proof leaves behind

Every machine is removed however the run ends.
`startMachine` and `stopMachine` are in `bin/service-manager-machine.ts`,
and `removeKeptMachines` (`bin/service-manager-machine.ts:156`) removes every container this run started.
`runArm` (`bin/service-manager-proof.ts`) holds its machine in an `await using`,
whose value's `Symbol.asyncDispose` (`bin/service-manager-machine.ts:549`) is what `await using` calls when a step throws,
and `main_` and `prove` hold `removeKeptMachines` the same way,
so a throw removes the containers rather than leaving them to nobody.
A machine is a container whose init is a system manager,
so a container left behind keeps a manager and its units running on this host,
and the `/etc/machine-id` it was given is a file in this package's
`node_modules/.monochromatic/git-policy/`,
named `machine-id-<container>`,
which `stopMachine` removes after `podman rm --force`.

Measured on 2026-11-27 with a planted failure:
`mise run //package/git-policy/cli:test:service-manager -- --main-wrapper /home/user/temp/no-such-wrapper-for-the-planted-failure --units 5`
starts the `main` arm's machine,
then throws in `prepareArm` at
`NativeVerificationError: podman cp failed (status 125, signal none). Error: ".../no-such-wrapper-for-the-planted-failure" could not be found on the host: no such file or directory`,
and afterwards
`podman ps --all --filter name=gpc-reload-proof-` names no container
and the identity folder holds no file.
The same run with `GPC_KEEP_CONTAINER=1` keeps the container it started and its
`machine-id-gpc-reload-proof-...` file,
which were then removed by hand,
so the check can fail and the empty answer means what it says.

## What is still owed

Nothing of the two refusals in this record:
the folder hand-over above and the corrected marks rule are what it owed,
and the two proofs now show that in their own lines.
The reload proof's control arm is still open,
for a reason of its own that is not this refusal,
and its choices are in the proposal named above.

## Upstream filing decision

Nothing is filed.
The failing component is this host's confinement of a program by its path,
working as its profile is written,
and the defect is the proof's choice of user.
