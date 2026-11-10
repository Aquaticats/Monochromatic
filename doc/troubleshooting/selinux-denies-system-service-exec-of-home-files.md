# systemd 259.10 on an SELinux-enforcing host: a system service whose program sits under the home directory or carries a container label fails with `status=203/EXEC`, although `systemd-run` accepted the unit

The host is Bazzite (Fedora 44 base,
SELinux `Enforcing`,
targeted policy,
`systemd 259 (259.10-1.fc44)`).
A transient unit created with `systemd-run` as root is accepted (`systemd-run` exits 0),
then fails when the system manager
looks for the program,
because the system manager (`init_t`) is denied `execute` on files labelled `data_home_t`
(everything under `~/.local/share`,
such as the toolchain `node`) and on files labelled `container_file_t`
(a build directory relabelled by a `podman run --volume ...:Z`).
The sentinels of `wg-quicker` and `wg-quicker-exempt` hit this,
and so would any root service that names such a file as its program.

## Symptom

The system journal (`sudo journalctl --grep=sentinel`),
verbatim:

```text
# journalctl (system), 2026-10-31
Oct 31 22:24:40 bazzite (node)[55439]: wg-quicker-sentinel-55398-14084935.service: Unable to locate executable '/var/home/user/.local/share/monochromatic-dev/toolchain/node/26.13.0/node-v26.13.0-linux-x64/bin/node': Permission denied
Oct 31 22:24:40 bazzite (node)[55439]: wg-quicker-sentinel-55398-14084935.service: Failed at step EXEC spawning /var/home/user/.local/share/monochromatic-dev/toolchain/node/26.13.0/node-v26.13.0-linux-x64/bin/node: Permission denied
Oct 31 22:24:40 bazzite systemd[1]: wg-quicker-sentinel-55398-14084935.service: Main process exited, code=exited, status=203/EXEC
Oct 31 22:24:43 bazzite (wg-quicker-exempt)[60217]: wg-quicker-exempt-sentinel-58850-14085157.service: Unable to locate executable '/var/home/user/Monochromatic/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt': Permission denied
Oct 31 22:24:43 bazzite (wg-quicker-exempt)[60217]: wg-quicker-exempt-sentinel-58850-14085157.service: Failed at step EXEC spawning /var/home/user/Monochromatic/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt: Permission denied
Oct 31 22:24:43 bazzite systemd[1]: wg-quicker-exempt-sentinel-58850-14085157.service: Main process exited, code=exited, status=203/EXEC
```

`systemd-run` itself exits 0,
so a launcher that only checks for a refused start sees success.
`wg-quicker up` then reported both sentinels as not confirmed,
which is the loud report working as designed.
Root's discretionary permissions cannot cause this:
the files are mode 0755 and owned by the user,
and root passes any such check.

## Root cause

### The check that fails is an access check, not the exec

The message is printed by the child that the system manager forks for the unit,
before it calls `execve`.
It looks the program up with `find_executable_full`,
and any error becomes `EXIT_EXEC` (status 203).
Source:
systemd v259.10,
commit `cc6062f23cb595c172d9a007ed99d925852bfd60`.
[`src/core/exec-invoke.c` lines 6014 to 6020](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/core/exec-invoke.c#L6014-L6020):

```c
// src/core/exec-invoke.c:6014
        r = find_executable_full(path, /* root= */ NULL, context->exec_search_path, false, &executable, &executable_fd);
        if (r < 0) {
                *exit_status = EXIT_EXEC;
                log_struct_errno(LOG_NOTICE, r,
                                 LOG_MESSAGE_ID(SD_MESSAGE_SPAWN_FAILED_STR),
                                 LOG_EXEC_MESSAGE(params, "Unable to locate executable '%s': %m", path),
                                 LOG_ITEM("EXECUTABLE=%s", path));
```

For an absolute path,
`find_executable_full` calls `open_and_check_executable`,
which opens the file with `O_PATH`
and then asks the kernel whether the caller may execute it.
[`src/basic/path-util.c` lines 688 to 693](https://github.com/systemd/systemd/blob/cc6062f23cb595c172d9a007ed99d925852bfd60/src/basic/path-util.c#L688-L693):

```c
// src/basic/path-util.c:688
        r = access_fd(fd, X_OK);
        if (r == -ENOSYS)
                /* /proc/ is not mounted. Fall back to access(). */
                r = RET_NERRNO(access(name, X_OK));
        if (r < 0)
                return r;
```

An `X_OK` check goes through the security module like an exec does.
The audit record of the failed check shows it is the `faccessat2` system call (`ausyscall x86_64 439` prints `faccessat2`)
issued by `/usr/lib/systemd/systemd-executor`,
and that SELinux denied `execute`.
`sudo ausearch --message AVC --start 10/31/26 22:24:30 --end 10/31/26 22:24:50` prints,
for the two units:

```text
# ausearch output, 2026-10-31
type=SYSCALL msg=audit(1793499880.475:61683): arch=c000003e syscall=439 success=no exit=-13 ... comm="(node)" exe="/usr/lib/systemd/systemd-executor" subj=system_u:system_r:init_t:s0 key=(null)
type=AVC msg=audit(1793499880.475:61683): avc:  denied  { execute } for  pid=55439 comm="(node)" name="node" dev="dm-0" ino=786463931 scontext=system_u:system_r:init_t:s0 tcontext=unconfined_u:object_r:data_home_t:s0 tclass=file permissive=0
type=SYSCALL msg=audit(1793499883.630:61759): arch=c000003e syscall=439 success=no exit=-13 ... comm="(wg-quicker-ex)" exe="/usr/lib/systemd/systemd-executor" subj=system_u:system_r:init_t:s0 key=(null)
type=AVC msg=audit(1793499883.630:61759): avc:  denied  { execute } for  pid=60217 comm="(wg-quicker-ex)" name="wg-quicker-exempt" dev="dm-0" ino=946602672 scontext=system_u:system_r:init_t:s0 tcontext=unconfined_u:object_r:container_file_t:s0:c311,c738 tclass=file permissive=0
```

### What the loaded policy says

The loaded policy answers access questions through `/sys/fs/selinux/access` (mode 0666;
a write computes a decision and changes nothing).
The harness in the Verification section asks it,
for the source domain `init_t` (PID 1,
which starts system services):

- `execute` on `data_home_t` files:
  denied,
  and the denial is audited (the `auditdeny` bit is set).
- `execute` on `container_file_t` files:
  denied,
  and audited.
- `execute` on `bin_t` files (such as `/usr/bin/env`):
  allowed.
- `search` and `getattr` on the home directory,
  `~/.local` and `~/.local/share`:
  allowed.
  Only `execute` is the problem,
  so the path is reachable and a service can read these files.

The type transition explains why `/usr/bin/env` helps.
Asked for the domain a new process gets when `init_t` runs a file (`/sys/fs/selinux/create`,
class `process`):

- running a `bin_t` file such as `/usr/bin/env`:
  the process becomes `system_u:system_r:unconfined_service_t:s0`.
- running a `shell_exec_t` file (`/usr/bin/bash`):
  the process becomes `initrc_t`.
- running a `data_home_t` file:
  the process would stay `init_t` (there is no transition),
  so the exec is not permitted.

`unconfined_service_t` is allowed to execute both `data_home_t` and `container_file_t` files (same harness).

### The rules in the policy file (setools)

`sesearch` is not installed on the host,
so the rules were read from a copy of the loaded policy
(`cp /sys/fs/selinux/policy` into scratch,
`policy.35`,
3,946,512 bytes) in a capped Fedora 44 container with setools 4.6.0,
mounted read-only and with the host policy untouched:

```sh
# sesearch.sh, run as: podman run --rm --memory=2g --cpus=2 --security-opt label=disable \
#   --volume <scratch>:/in:ro registry.fedoraproject.org/fedora:44 sh /in/sesearch.sh
dnf --assumeyes --quiet install setools-console
sesearch --allow --source init_t --target data_home_t --class file --perm execute /in/loaded-policy.bin
sesearch --dontaudit --source init_t --target data_home_t --class file /in/loaded-policy.bin
```

The package install took longer than ten minutes on the Fedora mirrors.
Results (the policy file is the last,
positional argument;
`--policy` is not an option of setools 4.6.0):

- `allow init_t data_home_t:file execute`,
  `allow init_t container_file_t:file execute` and `allow init_t user_home_t:file execute`:
  no rule matches (the search expands attributes,
  which the `bin_t` result in this list shows).
- `dontaudit init_t <type>:file` for `data_home_t`,
  `container_file_t`,
  `user_home_t` and `bin_t`:
  no rule matches.
  So no `dontaudit` rule exists for these accesses and nothing hides a denial.
- `allow init_t bin_t:file execute`:
  one rule,
  `allow init_t base_ro_file_type:file { execute execute_no_trans map };`.
- `type_transition init_t bin_t:process unconfined_service_t;`,
  which is why a service started behind `/usr/bin/env` runs in `unconfined_service_t`.
- `unconfined_service_t` may execute both labels,
  through `allow files_unconfined_type file_type:file { ... execute ... };`
  (and,
  for `container_file_t`,
  also through the device and filesystem unconfined attributes).

This matches the selinuxfs decisions of the harness in the Verification section.

### The dontaudit reading, disproved

The first search of the audit log,
on a window around 22:24 on 10/31/2026,
found no AVC records,
which suggested a `dontaudit` rule hiding the denial.
That reading is wrong on both counts.
The records exist (quoted in the section on the access check),
and the policy's `auditdeny` bit for `execute` from `init_t` on both labels is set,
so the denial is audited and nothing is hidden.
`ausearch` rejects a four digit year:
`sudo ausearch --message AVC --start 10/31/2026 22:24:30` fails with
`Error parsing start date (10/31/2026)`,
and `10/31/26` finds the records.
That is a verified reproduction of a command that prints an error instead of records;
which exact command produced the empty result is not known.

### Why `target/` carries a container label

`ls -Z` shows `container_file_t:s0:c311,c738` on `package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt` in the main checkout,
and other category pairs on other parts of the tree.
Measured with `getxattr` over the repository (excluding `node_modules`,
`.git` and `target`),
by label:

- 892,281 entries carry `container_file_t:s0:c190,c881` (the worktrees and most of the tree).
- 58,960 carry `container_file_t:s0:c311,c738`.
- 73 carry `container_file_t:s0:c6,c62`,
  19 carry `container_file_t:s0:c478,c695`,
  and 2 are `user_home_t`.

A bind mount with the `:Z` option relabels the mounted tree with private per-container categories,
and the label stays on disk after the container is gone.
The repository's own notes describe the effect:
`mise.toml` explains that its Playwright runner avoids `:Z` because the flag "relabels every file under the mount with per-container MCS categories",
and several troubleshooting documents run `podman run --volume "$PWD:/work:Z"`.
Which run produced `c311,c738` is not known:
`podman inspect` of all fourteen existing containers shows none that mounts the checkout with those categories
(three created,
unstarted containers mount worktrees with `c1022,c1023`),
and `--rm` runs leave no record.

Does it matter for anything else:

- For this failure it does not:
  the toolchain `node` is plain `data_home_t`,
  with no container label,
  and fails the same way.
  Removing the container label would not make the sentinels start.
- A container with other categories is normally kept from reading those files by MCS (standard behavior,
  not measured here);
  that affects other containers,
  not services.
- A service that starts behind `/usr/bin/env` ran the `c311,c738` binary without trouble (measurement `B2` under Patterns that work).

## Verification

Version under test:
systemd `259.10-1.fc44`,
kernel `7.2.11-ogc4.1.fc44.x86_64`,
targeted policy loaded from `/sys/fs/selinux/policy` (3,946,512 bytes).
All units were throwaway (`wgtest-*`),
collected on exit,
and none was left behind (`systemctl list-units --all 'wgtest-*'` was empty after each run,
for the system and the user manager).

The policy harness reads decisions from selinuxfs without executing anything:

```python
# Asks the loaded policy for a decision (class numbers and permission bits come from selinuxfs).
import pathlib
SELINUX = pathlib.Path('/sys/fs/selinux')

def decide(source, target, cls, perms):
    index = int((SELINUX / 'class' / cls / 'index').read_text())
    bits = {perm: 1 << (int((SELINUX / 'class' / cls / 'perms' / perm).read_text()) - 1) for perm in perms}
    with open(SELINUX / 'access', 'r+') as handle:
        handle.write(f'{source} {target} {index} {sum(bits.values()):x}')
        handle.seek(0)
        allowed, decided, auditallow, auditdeny = (int(word, 16) for word in handle.read().split()[:4])
    return {perm: {'allowed': bool(allowed & bit), 'audited_when_denied': bool(auditdeny & bit)} for perm, bit in bits.items()}

print(decide('system_u:system_r:init_t:s0', 'unconfined_u:object_r:data_home_t:s0', 'file', ['execute']))
```

### Patterns that fail

```text
# throwaway root units, 2026-10-31
sudo systemd-run --wait --collect --pipe --quiet --no-ask-password --unit=wgtest-a1 -- /var/home/user/.local/share/monochromatic-dev/toolchain/node/26.13.0/node-v26.13.0-linux-x64/bin/node --version
  -> exit 203 (the unit was accepted, then: status=203/EXEC)
sudo systemd-run ... --unit=wgtest-a2 -- /var/home/user/Monochromatic/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt
  -> exit 203
```

A unit started without `--wait` is accepted with exit 0 and fails a moment later,
which is the case that the start-time fallback of the launchers did not cover.

### Patterns that work

```text
# throwaway root units, behind /usr/bin/env, 2026-10-31
sudo systemd-run ... --unit=wgtest-b1 -- /usr/bin/env <node> --version                    -> exit 0, v26.13.0
sudo systemd-run ... --unit=wgtest-b2 -- /usr/bin/env <wg-quicker-exempt>                 -> exit 2, prints its usage (it ran)
sudo systemd-run ... --unit=wgtest-b3 -- /usr/bin/env cat /proc/self/attr/current         -> system_u:system_r:unconfined_service_t:s0
sudo systemd-run ... --unit=wgtest-b4 -- /usr/bin/env cat /proc/self/cgroup               -> 0::/system.slice/wgtest-b4.service
sudo systemd-run ... --unit=wgtest-d1 -- /usr/bin/env ls /run/wg-quicker                  -> lists the root-only state (it can read it)
```

End to end,
with the final launchers' own argument lists and a stand-in `sleep` as the watcher:

- the `wg-quicker-exempt` sentinel behind `/usr/bin/env` wrote its marker (`<watcher pid> <pid> <start time>`),
  reported `ManagedOOMPreference=omit`,
  ran in `/system.slice/wgtest-e2e-exempt.service` as `unconfined_service_t`,
  and its unit was gone after the stand-in watcher was killed.
- the `wg-quicker` sentinel (`node` behind `/usr/bin/env`,
  with `--setenv=SUDO_UID` and `SUDO_GID`) did the same and wrote
  `{"watcherPid":...,"pid":...,"startTime":"..."}` next to the state path.

## Verified workarounds

### Start the program behind `/usr/bin/env` (chosen)

Both launchers pass `/usr/bin/env <program> <arguments>` to `systemd-run`.
The system manager executes `env` (`bin_t`,
allowed),
the service runs in `unconfined_service_t`,
and `env` then executes the program,
which that domain may do.
Tradeoffs:

- It relies on the default policy that lets `init_t` execute `bin_t` and transition to `unconfined_service_t`.
  A site that confines `unconfined_service_t` further would break it again;
  the launchers then see a unit that never confirms,
  stop it,
  start the sentinel beside the watcher,
  and say so loudly.
- The program runs in an unconfined domain,
  the same trust as the user's own toolchain.
- `/usr/bin/env` is a fixed absolute path;
  the program's own path is still the one the launcher resolved from a root-controlled location.
- `env` reads a first argument that contains `=` as a variable assignment and has no option that prevents it:
  `/usr/bin/env /tmp/a=b __sentinel` prints `env: '__sentinel': No such file or directory` and exits 127 (reproduced).
  A program path with `=` in it is therefore refused as a service,
  with a loud line,
  and the sentinel starts beside the watcher,
  which does not go through `env`.
- After a failed confirmation the launcher stops the unit and reads its state whatever `systemctl stop` said:
  a unit that ended and was collected is no longer loaded,
  `stop` fails for it,
  and `show --property=ActiveState --value` prints `inactive`.
  Only `inactive` or `failed` counts as gone.
  Checked on the host with a nonexistent unit (counts as gone) and with a running root unit that an unprivileged caller cannot stop
  (`Failed to stop ...: Access denied`,
  state `active`,
  reported as not gone).
  A unit that cannot be shown to be gone is reported:
  two sentinels may run and one death may be reported twice.

Applying it needs a rebuilt `wg-quicker-exempt` and `wg-quicker` (the launchers changed).
For the Rust sentinel,
`wg-quicker-exempt watch-stop <key>` followed by `watch-start <key> <mark> <uid>` starts a new watcher and a new sentinel,
with the process watch and the socket marking unenforced for the moment between the two commands,
and no tunnel gap.
For the TypeScript sentinel there is no stop-and-start of the sentinel alone:
it starts with `up`.
Without a rebuild,
the same `systemd-run` argument list can be run by hand for the live watcher
(`sudo systemd-run --quiet --collect --no-ask-password --unit=<name> --property=ManagedOOMPreference=omit -- /usr/bin/env <program> <sentinel arguments>`),
which only adds a unit and touches neither the watcher nor the tunnel.

## What does not work

- **Naming the program directly.**
  Measured `A1` and `A2`:
  `status=203/EXEC`.
- **A service of the user manager** (`systemd-run --user`).
  Measured `C1` and `C2`:
  the user manager runs `node` (`unconfined_t`,
  cgroup
  `.../user@1000.service/app.slice/wgtest-c2.service`),
  but listing the state directories from a user-manager unit (`wgtest-d2`,
  `wgtest-d4`) gives
  `ls: cannot open directory '/run/wg-quicker': Permission denied` and the same for the per-watcher directory of `wg-quicker-exempt`,
  where the same listing from a root unit (`wgtest-d1`,
  `wgtest-d3`) succeeds.
  The sentinel must read root-only state,
  so it cannot run as the user.
- **A root-owned copy in a directory services may execute.**
  Measured:
  `sudo install` of the binary into `/usr/local/libexec/wgtest-e2e/`
  gives `system_u:object_r:usr_t:s0`,
  and a unit naming it directly ran (exit 2,
  usage).
  It works,
  but it needs an installer that copies on every update and a place the toolchain does not own,
  and it leaves a stale copy
  when the build changes;
  the trampoline needs none of that.
  The copy and its directory were removed after the test.
- **The start-time fallback alone.**
  It covers a refused start;
  here the manager accepted the unit and the failure came later.
  The launchers now also stop an accepted unit that never confirms and fall back to a sentinel beside the watcher.

## Not attempted

Relabelling the files (`chcon`,
`semanage fcontext`) was not tried:
it changes persistent labels or host policy,
and the toolchain recreates files under its default label on every update,
so it would need the owner's decision first.

## Upstream filing decision

Nothing to file.
`.out-of-scope/` was read on 2026-10-31 and has no entry for systemd or SELinux.

1.  Is it really upstream's fault?
    No.
    systemd reports a denial that the SELinux policy issues on purpose;
    `init_t` is not meant to execute user-home files.
2.  Can upstream fix it?
    Not applicable;
    the behavior is the policy's intent.
3.  Do they support this use case?
    Running a service from a home directory on a confined host is what the targeted policy restricts.
4.  Would the repository welcome a contribution?
    Not evaluated;
    there is nothing to contribute.
5.  Will they likely fix it?
    Not applicable.
6.  Prototype?
    The workaround (the `/usr/bin/env` start) is on our side of the boundary,
    in the launchers.

No issue or comment is drafted.
