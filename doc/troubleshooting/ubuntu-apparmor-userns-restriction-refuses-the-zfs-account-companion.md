# Ubuntu 24.04's AppArmor user-namespace restriction refuses the ZFS account companion at `setgroups`

Ubuntu 24.04's default AppArmor restriction on unprivileged user namespaces makes the account companion's
`/proc/self/setgroups` open answer `EACCES`,
and the companion has no skip for that step,
so every managed ZFS add on that distribution refuses the managed attach,
and the slot takes the reflink or ordinary Git route instead of a clone.
Measured 2026-11-26 in a throwaway `mvm` guest on kernel `6.8.0-147-generic` with OpenZFS 2.4.4-1.

## Symptom

An add run as the delegated account,
where the companion runs its shipped check,
prints this on standard error,
and then takes the ladder instead of making a clone:

```text
managed add refused: the account companion's setgroups failed: open /proc/self/setgroups (EACCES, errno 13); this slot takes the reflink or ordinary Git route.
```

The sentence is the `Refusal::ArmFailed` arm of
`package/git-policy/cli/src/native/account_companion/companion_lines.rs`:
it names the step,
the call that answered and its errno,
and the route the slot takes instead.
`the_managed_attach_may_go_ahead` answers false,
`run_a_zfs_add` returns `None`,
and the ordinary route serves the add,
so the slot becomes a plain Git worktree:
there is no clone dataset,
no origin snapshot and no binding record,
and nothing says a clone was ever wanted.

The step order that reaches it is the companion child's,
in `package/git-policy/cli/src/native/account_companion/companion_child.rs`:73 to `:74`:

```rust
    check(status, buffers, CompanionStep::UserNamespace, unshare_user_namespace);
    write_one(status, buffers, CompanionStep::Setgroups, buffers.setgroups.as_c_str(), b"deny", CompanionCall::OpenSetgroups, CompanionCall::WriteSetgroups);
```

## Root cause

Three facts together.

First,
an unprivileged user namespace on Ubuntu 24.04 transitions the creating process into the AppArmor profile
`unprivileged_userns`,
which denies every capability inside the namespace.
Read in the guest at `/etc/apparmor.d/unprivileged_userns`:

```text
profile unprivileged_userns {
     audit deny capability,
     audit deny change_profile,
     ...
     allow file rwlkm /**,
     allow userns,
```

The audit log shows the transition and a denied capability together,
for the process that asked:

```text
apparmor="AUDIT" operation="userns_create" class="namespace" info="Userns create - transitioning profile" profile="unconfined" requested="userns_create" target="unprivileged_userns"
apparmor="DENIED" operation="capable" class="cap" profile="unprivileged_userns" capability=21  capname="sys_admin"
```

Second,
on that guest the open of `/proc/self/setgroups` for write answers `EACCES`,
before any write the design reasoned about.
That is the kernel's own refusal:
`proc_setgroups_open` answers `-EACCES` for a write open whose caller lacks
`CAP_SYS_ADMIN` in the file's user namespace
(`kernel/user_namespace.c:3140` to `:3144` of the guest's 6.8 series),
so the capability the profile denies is asked for at the open and not at the
write.
A probe that makes the companion's three calls in the companion's order,
as the account,
answers:

```text
unshare 0 ok
setgroups deny: failed [Errno 13] Permission denied: '/proc/self/setgroups'
uid_map failed [Errno 1] Operation not permitted
gid_map failed [Errno 1] Operation not permitted
```

The design expected the other order of failures.
Its skip paragraph says the first mapping write is where Ubuntu's profile answers,
and that "the `setgroups` write succeeds because `proc_setgroups_write` asks for no capability to write `deny`",
in `doc/planning/cli-git-zfs-start-probe-design.md`,
the Ubuntu restriction paragraph.
That is a reading of mainline source,
and `:1254` to `:1256` of the same file states what the design makes of a failure at that step:

```text
an `EPERM` from the
`setgroups` write, which `proc_setgroups_write` gives only to a `deny` written after a `gid_map` (`:1264` to
`:1278`, a defect of the companion's order) and not to this design's own write, which asks for no capability
```

So the design puts a `setgroups` failure on the refused side on purpose,
and the guest measures a failure there that is not the companion's order:
the write is never reached,
because the open is refused.

Third,
the companion's skip list has no arm for that step at all,
in `package/git-policy/cli/src/native/account_companion/companion_steps.rs`,
`is_skip`:

```rust
    return match (step, call) {
        (CompanionStep::UserNamespace, CompanionCall::UnshareUserNamespace) => {
            matches!(errno, libc::ENOSPC | libc::EPERM | libc::EACCES) || (errno == libc::EINVAL && !user_namespace_entry)
        }
        (CompanionStep::UidMap, _) => errno == libc::EPERM,
        (CompanionStep::MountNamespace, CompanionCall::UnshareMountNamespace) => errno == libc::EPERM,
        (CompanionStep::PrivatePropagation, _) => errno == libc::EPERM || errno == libc::EACCES,
        (CompanionStep::ScratchTmpfs, CompanionCall::MountTmpfs) => matches!(errno, libc::EPERM | libc::EACCES),
        _other => false,
    };
```

One control holds that shape deliberately,
in `package/git-policy/cli/src/native/account_companion/companion_steps_tests.rs`,
`the_mount_namespace_and_the_scratch_mount_skip_their_named_answers`:
`!is_skip(CompanionStep::Setgroups, CompanionCall::WriteSetgroups, libc::EPERM, ENTRY_PRESENT)`,
with the reason "the setgroups write asks for no capability".

So the refusal is the code doing what its accepted records say,
on a host whose answer those records did not foresee.
The earlier reading,
that Ubuntu's profile lets the `setgroups` write through and answers at the first mapping write,
is what the guest disproves:
the write is never reached.

## Verification

Version under test:
an Ubuntu 24.04 guest made by `create` and `prepare` in
`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`,
kernel `6.8.0-147-generic` (the recipe's `EXPECTED_KERNEL_RELEASE`),
OpenZFS `2.4.4-1` (the guest's `zfs version`),
and `/proc/sys/kernel/apparmor_restrict_unprivileged_userns` reading `1`.

The harness,
run inside the guest as root:

```sh
runuser -u ubuntu -- python3 -c '
import ctypes, os
libc = ctypes.CDLL("libc.so.6", use_errno=True)
print("unshare", libc.unshare(0x10000000))
try:
    fd = os.open("/proc/self/setgroups", os.O_WRONLY); os.write(fd, b"deny"); print("setgroups ok")
except OSError as error:
    print("setgroups failed", error)
'
```

Patterns that fail there:

```text
unshare 0 ok
setgroups deny: failed [Errno 13] Permission denied: '/proc/self/setgroups'
```

- `unshare --user -- sh -c 'echo deny > /proc/self/setgroups'`:
  `Permission denied` at the redirect.
- `unshare --user --map-root-user -- true`:
  `unshare: write failed /proc/self/uid_map: Operation not permitted`.
- the wrapper's own add as the account:
  the `managed add refused` line in "Symptom",
  and a plain Git worktree at the slot with no clone dataset.

Patterns that work cleanly,
as the control on this host (kernel `7.3.5-2-cachyos`,
and no `apparmor_restrict_unprivileged_userns` entry):

```text
unshare 0 ok
setgroups deny: ok
uid_map failed [Errno 1] Operation not permitted
gid_map failed [Errno 1] Operation not permitted
```

The same probe therefore succeeds at `setgroups` where the restriction is off,
and fails where it is on,
and the mapping write answers `EPERM` in both.
The measured lines are kept in the round folder
`/home/user/temp/agent/zfs-add-clone-inherits-legacy-issue-6462-2026-11-25/keep/guest-logs/companion-measurements.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

## Verified workarounds

### The companion's test switch, for an arm

`CLI_GIT_TEST_ONLY_ACCOUNT_COMPANION=disabled` makes `run_under` answer `CompanionVerdict::NotRun`,
in `package/git-policy/cli/src/native/account_companion/companion_parent.rs` and
`companion_test_switch.rs`,
so the add goes ahead and the rest of the route can be measured as the account.
Tradeoff:
the companion's check does not run at all,
so a run using it proves nothing about the companion,
and its log must say so.
It is a test-only switch and no shipped run sets it.

### The distribution's setting, for a person's machine

```sh
sysctl --write kernel.apparmor_restrict_unprivileged_userns=0
```

Tradeoff:
this removes a distribution security default for every unprivileged process on the machine,
and this route may not ask for it,
because a route that needs it is a route that does not work on a stock Ubuntu 24.04.

### What shipped code needs

The route needs the owner's decision rather than a workaround:
either the skip list gains the `setgroups` step for the answers this host gives,
which amends the skip reach in `doc/decision/cli-git-zfs-start-probe.md` and the design's skip paragraph and
their control,
or the managed route stays off on Ubuntu 24.04,
and the ladder serves every add there.

The owner took the first on 2026-11-26T00:05-05:00,
answering "Add the skip and make the doctor tell user what to do."
(`doc/decision/cli-git-zfs-start-probe.md`,
"The account companion's group drop on Ubuntu 24.04"):
the open's `EACCES` is a skip that prints its line and lets the add go ahead,
and `doctor` reports the skip,
its reason and the remedies above without running an add.

## What does not work

- Expecting the first mapping write to be the first refusal.
  It is the second:
  the `setgroups` open answers first on this guest,
  so the skip arm for `UidMap` never runs.
- Running the add as root to avoid the companion.
  Root bypasses the ZFS delegation too,
  so such an arm proves nothing about the account's requests,
  which is why the arm for this issue hands the checkout to the account.
- `unshare --map-root-user` as a stand-in for the companion.
  It fails at the same restriction,
  with `write failed /proc/self/uid_map: Operation not permitted`.
- Reading a skip line as this host's outcome.
  A skip prints and the add goes ahead;
  here the companion refuses,
  the add answers no route,
  and the slot becomes a plain worktree.

## Upstream filing decision

Nothing to file with Ubuntu,
AppArmor or the kernel.
Walked through the six constraints:

1. It is not upstream's fault.
   `audit deny capability,` inside `unprivileged_userns` is the distribution's stated restriction,
   and this route's reading of which call answers it was wrong.
2. Upstream cannot fix it,
   because the behaviour is the intended policy.
3. Ubuntu supports unprivileged user namespaces only through that profile,
   so the combination is supported by design and not by accident.
4. There is no contribution to make,
   and no file in the distribution's tree this repository would patch.
5. No signal that the distribution will reverse the restriction.
6. There is nothing to prototype upstream:
   the change belongs to this repository's skip list,
   and it needs the owner's decision on the skip's reach.

`.out-of-scope/` was read and holds no exemption for this tool or bug class,
and `gh issue list --state all --search "setgroups"` found no issue in this repository about it.
