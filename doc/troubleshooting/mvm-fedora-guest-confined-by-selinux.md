# mvm's Fedora 43 guests refuse push, exec and pull because SELinux confines qemu-guest-agent to `virt_qemu_ga_t`

`mvm` creates a Fedora 43 guest whose QEMU guest agent runs in Fedora's
`virt_qemu_ga_t` domain.
That domain may not read the virtiofs share,
write the guest's own files,
execute a file it wrote,
or open a TCP connection,
so every file route into the guest is refused while the guest's SELinux is
enforcing.
The report below is from `mvm`'s libvirt backend on 2026-11-24,
in a guest carrying `selinux-policy-42.12-1.fc43` and
`qemu-guest-agent-10.1.0-7.fc43`.

## Status

Diagnosed and fixed in `mvm` on 2026-11-24 by an agent round.
The work is unapproved and not final.
Done and checked:
the cause is traced to Fedora's policy module and measured in
the guest,
the fix is in `package/mvm/cli` at commit `46222892f7`,
its unit tests and `lint:types` pass,
and the routes are shown refused in a guest whose template an earlier `mvm`
baked and working in a guest whose template that commit baked,
with the guest's SELinux enforcing in both.
Open:
a template baked before that commit has to be baked again by hand,
and a future change to Fedora's unit arguments has to be mirrored in the
drop-in.
Section [Fix](#fix) describes the change,
[Verification](#verification) the runs before and after it,
and [Verified workarounds](#verified-workarounds) the drop-in to add by hand to
a guest whose template an earlier `mvm` baked.

## Symptom

Everything `mvm` asks the guest agent to do still works:
`mvm exec guest -- 'uname -a'` prints the kernel,
and the agent answers pings,
so `mvm create` reports the VM ready.
Every route that carries a file in,
or runs one,
is refused.
Each failure is a bare `Permission denied`,
with no audit line the caller can see,
and `mvm exec` itself runs in the confined domain:

```text
# mvm exec selinux-round-1 -- 'id; cat /proc/self/attr/current'
uid=0(root) gid=0(root) groups=0(root) context=system_u:system_r:virt_qemu_ga_t:s0
system_u:system_r:virt_qemu_ga_t:s0
```

### The virtiofs share

`mvm push` itself succeeds:
it only copies the file into the VM's host-side `shared/` directory.
The guest then cannot open it.
Its directory listing shows the file without metadata:

```text
# mvm exec selinux-round-1 -- 'ls -la /mnt/shared'
total 9
drwxr-xr-x. 2 root root  3 Nov 24 06:33 .
drwxr-xr-x. 1 root root 12 Nov 24 06:33 ..
-?????????? ? ?    ?     ?            ? handler
ls: cannot access '/mnt/shared/handler': Permission denied
```

```text
# mvm exec selinux-round-1 -- 'cp /mnt/shared/handler /root/handler'
cp: cannot stat '/mnt/shared/handler': Permission denied
```

### The guest agent's own file commands

With the VM's `meta.json` set to `"fileTransfer": "guest-agent"`
(the route a host without `virtiofsd` picks),
`mvm push` into a directory the agent may not write fails at
`guest-file-open`:

```text
error: guest agent command failed: unable to execute QEMU agent command
'guest-file-open': failed to open file '/root/bool-pushed' (mode: 'wb'):
Permission denied
```

`mvm pull` from such a path fails the same way.

### Running a pushed file

A file written through `mvm exec` lands in the guest with the agent's own tmp
label,
and running it is refused:

```text
# mvm exec selinux-round-1 -- 'printf "#!/bin/sh\necho hi\n" > /var/tmp/h; chmod 755 /var/tmp/h; /var/tmp/h'
/bin/bash: line 2: /var/tmp/h: Permission denied
```

A file `mvm push` wrote through the agent landed world-writable but without an
execute bit,
which is a second,
unrelated refusal:
`/root/agent-pushed: Permission denied` until `chmod +x`.

### The host as a file server

The workaround this repository records for a host without `virtiofsd`,
serving files to the guest over loopback HTTP,
is refused too.
`10.0.2.2` is SLIRP's host gateway:

```text
# mvm exec selinux-round-1 -- 'bash -c "exec 3<>/dev/tcp/10.0.2.2/22"'
bash: connect: Permission denied
bash: line 1: /dev/tcp/10.0.2.2/22: Permission denied
```

`curl http://10.0.2.2:18231/...` fails the same way,
reported in the earlier round as `Immediate connect fail ... Permission denied`.

### Reading the guest's own records

The agent cannot read the guest's audit log or ask about its policy:

```text
# mvm exec selinux-round-1 -- 'cat /var/log/audit/audit.log'
cat: /var/log/audit/audit.log: Permission denied
# mvm exec selinux-round-1 -- 'getenforce'
getenforce:  security_getenforce() failed:  Permission denied
```

`ls -la /usr/bin/sudo`,
`/usr/bin/journalctl` and `/usr/bin/rpm` are refused for
the same reason,
which makes a confined guest look as if those programs were missing:
`bash` prints `command not found` for a program it may not even `getattr`.

## Root cause

### The routes all run in one confined domain

`mvm` reaches the guest only through the QEMU guest agent.
`package/mvm/cli/src/guest-exec.ts` starts commands with `guest-exec` and the
guest's shell,
`package/mvm/cli/src/guest-file-transfer.ts` moves files with
`guest-file-open`/`guest-file-read`/`guest-file-write`,
and `package/mvm/cli/src/file-transfer.ts` copies into the virtiofs share only
on the host side,
leaving the guest to read `/mnt/shared` itself.
The domain is a property of the process the distro started,
so every one of those children inherits it:
`system_u:system_r:virt_qemu_ga_t:s0`.

### What the guest's own policy answers

The loaded policy in the guest answers access questions through
`/sys/fs/selinux/access`.
That call changes nothing;
it returns the decision,
whether it is audited,
and which domain a new process would get.
The probe below was pushed with `mvm push`,
run as root with `mvm exec`,
and reads selinuxfs only.
Its output,
verbatim:

```text
== the domain Fedora starts qemu-guest-agent in
  file         getattr=DENY                             system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:virtiofs_t:s0
  file         read=DENY                                system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:virtiofs_t:s0
  dir          add_name=DENY                            system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:admin_home_t:s0
  file         write=DENY                               system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:admin_home_t:s0
  file         read=allow write=allow execute=DENY      system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:virt_qemu_ga_tmp_t:s0
  file         execute=allow                            system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:bin_t:s0
  tcp_socket   name_connect=DENY                        system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:unreserved_port_t:s0
  file         read=DENY                                system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:auditd_log_t:s0
  file         entrypoint=allow                         system_u:system_r:virt_qemu_ga_t:s0 -> system_u:object_r:virt_qemu_ga_exec_t:s0

== the same questions for the domain /usr/bin/env starts
  file         getattr=allow read=allow                 system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:virtiofs_t:s0
  dir          add_name=allow                           system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:admin_home_t:s0
  file         write=allow execute=allow                system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:admin_home_t:s0
  file         execute=allow                            system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:virt_qemu_ga_tmp_t:s0
  tcp_socket   name_connect=allow                       system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:unreserved_port_t:s0
  file         read=allow                               system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:auditd_log_t:s0
  file         execute=allow                            system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:virt_qemu_ga_exec_t:s0
  file         entrypoint=DENY                          system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:virt_qemu_ga_exec_t:s0
  file         entrypoint=allow                         system_u:system_r:unconfined_service_t:s0 -> system_u:object_r:bin_t:s0

== the transitions the system manager needs
  process      transition=allow                         system_u:system_r:init_t:s0 -> system_u:system_r:unconfined_service_t:s0
  process      transition=allow                         system_u:system_r:init_t:s0 -> system_u:system_r:virt_qemu_ga_t:s0
  process      transition=DENY                          system_u:system_r:unconfined_service_t:s0 -> system_u:system_r:virt_qemu_ga_t:s0
```

The probe itself,
runnable in any SELinux guest:

```python
#!/usr/bin/env python3
# doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md
# Asks the loaded SELinux policy for decisions through selinuxfs; changes nothing.
import pathlib

SELINUX = pathlib.Path('/sys/fs/selinux')


def decide(source, target, cls, perms):
    """Asks the loaded policy whether `source` may use `perms` on `target`."""
    index = int((SELINUX / 'class' / cls / 'index').read_text().strip())
    bits = {perm: 1 << (int((SELINUX / 'class' / cls / 'perms' / perm).read_text().strip()) - 1)
            for perm in perms}
    mask = 0
    for bit in bits.values():
        mask |= bit
    with open(SELINUX / 'access', 'r+') as handle:
        handle.write(f'{source} {target} {index} {mask:x}')
        handle.seek(0)
        words = handle.read().split()
    allowed, decided = (int(word, 16) for word in words[:2])
    verdict = []
    for perm, bit in bits.items():
        if not decided & bit:
            verdict.append(f'{perm}=not-decided')
        elif allowed & bit:
            verdict.append(f'{perm}=allow')
        else:
            verdict.append(f'{perm}=DENY')
    print(f'  {cls:12} {" ".join(verdict):40} {source} -> {target}')


AGENT = 'system_u:system_r:virt_qemu_ga_t:s0'
ENV = 'system_u:system_r:unconfined_service_t:s0'
print('== the domain Fedora starts qemu-guest-agent in')
decide(AGENT, 'system_u:object_r:virtiofs_t:s0', 'file', ['getattr'])
decide(AGENT, 'system_u:object_r:virtiofs_t:s0', 'file', ['read'])
decide(AGENT, 'system_u:object_r:admin_home_t:s0', 'dir', ['add_name'])
decide(AGENT, 'system_u:object_r:admin_home_t:s0', 'file', ['write'])
decide(AGENT, 'system_u:object_r:virt_qemu_ga_tmp_t:s0', 'file', ['read', 'write', 'execute'])
decide(AGENT, 'system_u:object_r:bin_t:s0', 'file', ['execute'])
decide(AGENT, 'system_u:object_r:unreserved_port_t:s0', 'tcp_socket', ['name_connect'])
decide(AGENT, 'system_u:object_r:auditd_log_t:s0', 'file', ['read'])
decide(AGENT, 'system_u:object_r:virt_qemu_ga_exec_t:s0', 'file', ['entrypoint'])
print('== the same questions for the domain /usr/bin/env starts')
decide(ENV, 'system_u:object_r:virtiofs_t:s0', 'file', ['getattr', 'read'])
decide(ENV, 'system_u:object_r:admin_home_t:s0', 'dir', ['add_name'])
decide(ENV, 'system_u:object_r:admin_home_t:s0', 'file', ['write', 'execute'])
decide(ENV, 'system_u:object_r:virt_qemu_ga_tmp_t:s0', 'file', ['execute'])
decide(ENV, 'system_u:object_r:unreserved_port_t:s0', 'tcp_socket', ['name_connect'])
decide(ENV, 'system_u:object_r:auditd_log_t:s0', 'file', ['read'])
decide(ENV, 'system_u:object_r:virt_qemu_ga_exec_t:s0', 'file', ['execute', 'entrypoint'])
decide(ENV, 'system_u:object_r:bin_t:s0', 'file', ['entrypoint'])
print('== the transitions the system manager needs')
decide('system_u:system_r:init_t:s0', ENV, 'process', ['transition'])
decide('system_u:system_r:init_t:s0', AGENT, 'process', ['transition'])
decide(ENV, AGENT, 'process', ['transition'])
```

The guest's kernel audit log carries the same decisions as AVC records,
read as root on the serial console,
copied out through `/mnt/shared`.
Lines are quoted whole;
they are longer than this document's prose limit:

```text
type=AVC msg=audit(1795502400.812:214): avc:  denied  { getattr } for  pid=1179 comm="ls" path="/mnt/shared/handler" dev="virtiofs" ino=12902997 scontext=system_u:system_r:virt_qemu_ga_t:s0 tcontext=system_u:object_r:virtiofs_t:s0 tclass=file permissive=0
type=AVC msg=audit(1795502400.884:218): avc:  denied  { read } for  pid=1183 comm="cat" name="audit.log" dev="vda4" ino=2103 scontext=system_u:system_r:virt_qemu_ga_t:s0 tcontext=system_u:object_r:auditd_log_t:s0 tclass=file permissive=0
type=AVC msg=audit(1795502402.635:229): avc:  denied  { execute } for  pid=1263 comm="bash" name="h" dev="vda4" ino=2230 scontext=system_u:system_r:virt_qemu_ga_t:s0 tcontext=system_u:object_r:virt_qemu_ga_tmp_t:s0 tclass=file permissive=0
type=AVC msg=audit(1795502402.900:232): avc:  denied  { add_name } for  pid=1266 comm="touch" name="x" scontext=system_u:system_r:virt_qemu_ga_t:s0 tcontext=system_u:object_r:admin_home_t:s0 tclass=dir permissive=0
type=AVC msg=audit(1795502403.011:234): avc:  denied  { name_connect } for  pid=1274 comm="bash" dest=22 scontext=system_u:system_r:virt_qemu_ga_t:s0 tcontext=system_u:object_r:ssh_port_t:s0 tclass=tcp_socket permissive=0
```

The full logs are `keep/audit-before-fix-1.log`,
`keep/audit-before-fix-2.log` and
`keep/audit-with-read-nonsecurity-boolean.log` in the round folder
`/home/user/temp/agent/mvm-fedora-guest-refuses-every-file-route-2026-11-24/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

### The rules that decide it

Fedora's policy module for the agent is
`policy/modules/contrib/virt_supplementary.te`,
read here at tag `v42.12`
(commit `415b33792f9ea17d816a9e2602cddf21c16e7255`,
which the guest's `selinux-policy-42.12-1.fc43` carries;
the file is unchanged on `main` when read on 2026-11-24).
The domain and its entry point,
[lines 59 to 61](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.te#L59-L61):

```te
# policy for qemu_ga
type virt_qemu_ga_t;
type virt_qemu_ga_exec_t;
init_daemon_domain(virt_qemu_ga_t, virt_qemu_ga_exec_t)
```

[`init_daemon_domain`](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/system/init.if#L249-L265)
turns the unit's `ExecStart` into that domain and grants the
`entrypoint` permission to that domain alone:

```te
interface(`init_daemon_domain',`
	...
	domain_type($1)
	domain_entry_file($1, $2)

	type_transition initrc_domain $2:process $1;
```

This is why an explicit `SELinuxContext=` naming another domain cannot start
the unit;
see [What does not work](#what-does-not-work).

Its own tmp directory is the only place the agent may write,
and the rules grant reading and writing there,
never executing,
[lines 171 to 173](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.te#L171-L173):

```te
manage_dirs_pattern(virt_qemu_ga_t, virt_qemu_ga_tmp_t, virt_qemu_ga_tmp_t)
manage_files_pattern(virt_qemu_ga_t, virt_qemu_ga_tmp_t, virt_qemu_ga_tmp_t)
files_tmp_filetrans(virt_qemu_ga_t, virt_qemu_ga_tmp_t, { file dir })
```

`manage_files_pattern` expands to `create`,
`open`,
`read`,
`write`,
`getattr`,
`setattr`,
`unlink`,
`rename` and the rest of the file permissions,
and it does not include `execute`.
Everything the agent creates in `/tmp` or `/var/tmp` therefore carries
`virt_qemu_ga_tmp_t` and cannot be run,
whatever mode bits it has.

The domain may execute programs,
but only from the ordinary command directories,
[lines 188 and 189](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.te#L188-L189):

```te
corecmd_exec_shell(virt_qemu_ga_t)
corecmd_exec_bin(virt_qemu_ga_t)
```

That is why `mvm exec guest -- 'uname -a'` works while running a file written in
the guest does not.

The module has no `corenet_tcp_connect_*` rule for `virt_qemu_ga_t` at all;
its only network calls are `sysnet_dns_name_resolve` and the virtual switch
helpers of the other domains,
[line 218](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.te#L218).
A TCP connection is therefore refused for every port.

The two booleans the module offers do not reach these decisions.
[Lines 239 to 241](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.te#L239-L241)
grant reading,
and only reading:

```te
tunable_policy(`virt_qemu_ga_read_nonsecurity_files',`
	files_read_non_security_files(virt_qemu_ga_t)
')
```

And `virt_qemu_ga_run_unconfined`
([lines 243 to 247](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.te#L243-L247))
is about the fsfreeze hook scripts,
not about the agent:

```te
tunable_policy(`virt_qemu_ga_run_unconfined',`
    domtrans_pattern(virt_qemu_ga_t, virt_qemu_ga_unconfined_exec_t, virt_qemu_ga_unconfined_t)
',`
    can_exec(virt_qemu_ga_t, virt_qemu_ga_unconfined_exec_t)
')
```

`virt_qemu_ga_unconfined_exec_t` is the label of
`/etc/qemu-ga/fsfreeze-hook.d/**` and `/usr/libexec/qemu-ga/fsfreeze-hook.d/**`
([lines 68 to 72 of the file contexts](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/contrib/virt_supplementary.fc#L68-L72)),
so the boolean lets a snapshot hook script run unconfined;
the agent itself stays in `virt_qemu_ga_t`.

### Why `/usr/bin/env` reaches a domain that may

`/usr/bin/env` is labeled `bin_t`.
Fedora's `unconfined.te` gives `unconfined_service_t`
blanket access (`unconfined_domain(unconfined_service_t)`,
line 14)
and the `entrypoint` permission on `bin_t`
(`corecmd_bin_entry_type(unconfined_service_t)`,
line 20),
[lines 9 to 20](https://github.com/fedora-selinux/selinux-policy/blob/415b33792f9ea17d816a9e2602cddf21c16e7255/policy/modules/system/unconfined.te#L9-L20):

```te
type unconfined_service_t;
domain_type(unconfined_service_t)
role system_r types unconfined_service_t;
init_nnp_daemon_domain(unconfined_service_t)

unconfined_domain(unconfined_service_t)
...
corecmd_bin_entry_type(unconfined_service_t)
```

A unit whose `ExecStart` is `/usr/bin/env` therefore starts in
`unconfined_service_t`,
and `env` then executes `/usr/bin/qemu-ga` there.
No type transition leads back to `virt_qemu_ga_t` from there,
which the probes show as `transition=DENY`,
so the agent and every child it starts keep the domain that may read the share,
write the guest's files,
execute them and open sockets.

Which rule gives the system manager the `init_t` + `bin_t` to
`unconfined_service_t` transition was not traced to a source line;
what the guest does is measured:
the process label,
and `transition=allow` for that pair.

### The unit whose arguments the drop-in mirrors

Fedora's `qemu-guest-agent.service`,
read in the guest
(`qemu-guest-agent-10.1.0-7.fc43`,
identical to
[the file on the package's `f43` branch](https://src.fedoraproject.org/rpms/qemu/raw/f43/f/qemu-guest-agent.service),
a branch URL because that server exposes no commit-pinned one):

```ini
[Service]
UMask=0077
EnvironmentFile=/etc/sysconfig/qemu-ga
ExecStartPre=/bin/sh -c "if grep '^BLACKLIST_RPC' /etc/sysconfig/qemu-ga >/dev/null ; then sed -i 's/^BLACKLIST_RPC=/QEMU_GA_ARGS=--block-rpcs=/' /etc/sysconfig/qemu-ga ; fi"
ExecStart=/usr/bin/qemu-ga \
  --method=virtio-serial \
  --path=/dev/virtio-ports/org.qemu.guest_agent.0 \
  -F${FSFREEZE_HOOK_PATHNAME} \
  $QEMU_GA_ARGS
Restart=always
RestartSec=0
```

QEMU's own unit is only `ExecStart=-/usr/bin/qemu-ga`
([`contrib/systemd/qemu-guest-agent.service` at `v10.1.0`](https://github.com/qemu/qemu/blob/v10.1.0/contrib/systemd/qemu-guest-agent.service));
the arguments and the environment file come from Fedora's packaging,
so a drop-in that replaces `ExecStart` has to repeat them.

### Two readings that were wrong

The round that read the refusals on 2026-11-24
(its `keep/state-so-far.md`,
lines 30 to 55)
read them as the guest's SELinux confining the guest agent and stopped there,
after finding that `getenforce` and `setenforce 0` are refused as well.
That denial is the agent's own domain being refused the policy file,
not a policy that cannot be relaxed:
the console's unconfined root reads `Enforcing`,
and so does the agent once it
runs in a domain that may.

The first candidate fix was to set the module's boolean,
`virt_qemu_ga_read_nonsecurity_files`.
Measured in the same guest with it on
(`setsebool -P virt_qemu_ga_read_nonsecurity_files on`,
stock unit restarted),
reading the share and the audit log works,
while writing `/root/x`,
executing a file written in `/var/tmp`,
and connecting to `10.0.2.2` are all still refused;
the AVCs are in `keep/audit-with-read-nonsecurity-boolean.log`.
Push would still not place a file where its directory may not be written,
and exec would still not run one.

The second was `[Service] SELinuxContext=system_u:system_r:unconfined_service_t:s0`
in a drop-in,
which fails at start:

```text
qemu-guest-agent.service: Failed at step EXEC spawning /usr/bin/qemu-ga: Permission denied
qemu-guest-agent.service: Main process exited, code=exited, status=203/EXEC
type=AVC msg=audit(1795502402.220:262): avc:  denied  { entrypoint } for  pid=1445 comm="(qemu-ga)" path="/usr/bin/qemu-ga" dev="vda4" ino=3828 scontext=system_u:system_r:unconfined_service_t:s0 tcontext=system_u:object_r:virt_qemu_ga_exec_t:s0 tclass=file permissive=0
```

The `entrypoint` permission belongs to the target domain,
and `virt_qemu_ga_exec_t` carries it only for `virt_qemu_ga_t`.

## Fix

In `package/mvm/cli`,
committed as `46222892f7`:

- `src/registry.ts` gives `LinuxGuestConfig` an optional
  `guestAgentIsSelinuxConfined`,
  set on the `fedora` image entry,
  with the reason in its TSDoc.
- `src/cloud-init-init-systems.ts` gains
  `GUEST_AGENT_DROP_IN_PATH` and the `write_files` entry that starts the agent
  through `/usr/bin/env`,
  and `templateRuncmd` writes both the drop-in and `systemctl daemon-reload`
  before `systemctl enable --now qemu-guest-agent` for such a guest.
- `src/cloud-init.ts` passes the flag into `templateRuncmd` and exports
  `templateUserData` for the built artifact's tests.
- `src/cloud-init.unit.test.ts` asserts the Fedora template user-data carries
  the drop-in,
  that the wrapped command mirrors the distro unit's arguments and leaves
  `${FSFREEZE_HOOK_PATHNAME}` and `$QEMU_GA_ARGS` for systemd to expand,
  that `daemon-reload` precedes the enable,
  and that Ubuntu's user-data carries none of it.

The drop-in the template writes to
`/etc/systemd/system/qemu-guest-agent.service.d/mvm-unconfined.conf`:

```ini
[Service]
ExecStart=
ExecStart=/usr/bin/env /usr/bin/qemu-ga --method=virtio-serial --path=/dev/virtio-ports/org.qemu.guest_agent.0 -F${FSFREEZE_HOOK_PATHNAME} $QEMU_GA_ARGS
```

It is in the template's cloud-init,
before the unit is enabled,
so the first agent process of every guest from that template already runs
unconfined:
no restart is needed at first boot,
and `mvm create` cannot hand the caller a confined agent.

The tradeoffs:

- The drop-in repeats the argument list of Fedora's unit.
  A Fedora change to those arguments has to be mirrored here,
  or the agent starts with the older list.
  Which flag a future qemu-guest-agent or Fedora unit drops was not measured.
  The failure is loud when it happens:
  the unit exits,
  `mvm create` waits five minutes for an agent that never answers and then
  fails,
  and the guest's journal names the rejected argument.
- The agent runs with the same access as an unconfined root service,
  which is the access an Ubuntu guest's agent already has:
  the same `mvm` push,
  exec and pull routes work there on this host,
  as the round that hit this defect reported.
  `unconfined_service_t` can enter the other domains a service normally
  enters,
  so this is not a general escape from the policy,
  only the removal of the agent's own confinement.
- A template baked before this change keeps the confined agent.
  Bake it again when no guest's disk is backed by it;
  see [Verified workarounds](#verified-workarounds).
- A custom template of another SELinux distro is used as it is,
  with `CUSTOM_GUEST_DEFAULTS`,
  so `mvm` cannot add the drop-in there.
  The same drop-in by hand is the answer,
  as below.

## Verification

Version under test:
`mvm` at commit `46222892f7`,
guest `Fedora 43 (Cloud Edition)`,
kernel `6.17.1-300.fc43.x86_64`,
`glibc-2.42-4.fc43`,
`qemu-guest-agent-10.1.0-7.fc43`,
`selinux-policy-42.12-1.fc43` enforcing.
Host:
this repository's host,
`mvm`'s libvirt backend,
`virtiofsd` available so the guests had the virtiofs share.

The guests were created and destroyed through `mvm` alone,
by the names these runs printed.
The second arm ran with `HOME` set to a directory of its own,
so its template bake neither read nor replaced the shared
`~/.local/share/mvm/images/template-fedora.qcow2`,
which a running guest of another session uses as its backing file.

### Harness

```sh
# doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md
# Arm 1: the template an earlier mvm baked.
mvm create --image fedora selinux-fixed-1
mvm exec selinux-fixed-1 -- 'cat /proc/self/attr/current'
mvm exec selinux-fixed-1 -- 'getenforce'
mvm push selinux-fixed-1 ./handler handler
mvm exec selinux-fixed-1 -- 'cp /mnt/shared/handler /root/handler'
mvm exec selinux-fixed-1 -- 'printf "#!/bin/sh\necho hi\n" > /var/tmp/h4; chmod 755 /var/tmp/h4; /var/tmp/h4'

# Arm 2: a template the fixed mvm baked, in a HOME of its own.
export HOME=<scratch>/mvm-home
mvm create --image fedora selinux-fixed-2
mvm exec selinux-fixed-2 -- 'cat /proc/self/attr/current; getenforce'
mvm exec selinux-fixed-2 -- 'ps -eo label,args | grep qemu-ga'
mvm exec selinux-fixed-2 -- 'cat /etc/systemd/system/qemu-guest-agent.service.d/mvm-unconfined.conf'
mvm push selinux-fixed-2 ./handler handler
mvm exec selinux-fixed-2 -- 'cp /mnt/shared/handler /root/handler; chmod +x /root/handler; /root/handler'
mvm exec selinux-fixed-2 -- 'echo pulled-through-share > /mnt/shared/out.txt'
mvm pull selinux-fixed-2 out.txt ./out-after-fix.txt
```

The audit log of a confined guest is readable only from the serial console,
which `mvm shell` opens on the autologin user with passwordless `sudo`:

```sh
# doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md
# virsh console needs a terminal, so `script` supplies one and the commands are piped in.
( sleep 6; printf 'sudo -n cp /var/log/audit/audit.log /mnt/shared/audit.log\n'; sleep 6; printf '\035' ) \
  | script -qec 'mvm shell selinux-round-1' /dev/null
```

### Patterns that fail, before the fix

```text
# guest from the template an earlier mvm baked, 2026-11-24
cat /proc/self/attr/current                     -> system_u:system_r:virt_qemu_ga_t:s0
getenforce                                      -> security_getenforce() failed:  Permission denied
cp /mnt/shared/handler /root/handler            -> cp: cannot stat '/mnt/shared/handler': Permission denied
/var/tmp/h4 (written and chmod 755)             -> Permission denied, exit 126
echo >/dev/tcp/10.0.2.2/22                      -> connect: Permission denied
cat /var/log/audit/audit.log                    -> cat: /var/log/audit/audit.log: Permission denied
mvm push ... /root/agent-pushed (guest agent)   -> guest-file-open ... Permission denied
```

The same guest with `virt_qemu_ga_read_nonsecurity_files` on:

```text
cat /mnt/shared/handler                         -> works
getenforce                                      -> Enforcing
touch /root/x                                   -> Permission denied
/var/tmp/h3 (chmod 755)                         -> Permission denied, exit 126
exec 3<>/dev/tcp/10.0.2.2/18231                 -> connect: Permission denied
mvm push ... /root/bool-pushed (guest agent)    -> guest-file-open ... Permission denied
```

### Patterns that work, after the fix

```text
# guest from the template the fixed mvm baked, 2026-11-24
cat /proc/self/attr/current                     -> system_u:system_r:unconfined_service_t:s0
getenforce                                      -> Enforcing
ps -eo label,args | grep qemu-ga                -> system_u:system_r:unconfined_service_t:s0 /usr/bin/qemu-ga ...
cat .../qemu-guest-agent.service.d/mvm-unconfined.conf -> the drop-in above
mvm push selinux-fixed-2 ./handler handler      -> /mnt/shared/handler
cp /mnt/shared/handler /root/handler            -> works, /root/handler is admin_home_t
chmod +x /root/handler && /root/handler         -> hi
echo pulled-through-share > /mnt/shared/out.txt -> works
mvm pull selinux-fixed-2 out.txt                -> pulled-through-share
mvm push ... /root/agent-pushed (guest agent)   -> works, mode 0666
chmod +x /root/agent-pushed && /root/agent-pushed -> hi
mvm pull ... /root/pulled.txt (guest agent)     -> agent-pull-after-fix
curl -sS http://10.0.2.2:18231/handler          -> the file, exit 0
grep -c 'avc:' /var/log/audit/audit.log         -> 0
```

`mvm destroy` succeeded for both guests,
and `mvm list` showed neither afterwards.

## Verified workarounds

### The chosen fix: the drop-in baked into the template

Described in [Fix](#fix).
A guest created from a template baked by `mvm` at `46222892f7` needs nothing
else.

### The same drop-in by hand, for a guest whose template predates the fix

No file has to be re-baked for a guest that already exists.
As root in the guest,
over the serial console when the agent is confined:

```sh
# doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md
mkdir -p /etc/systemd/system/qemu-guest-agent.service.d
cat > /etc/systemd/system/qemu-guest-agent.service.d/mvm-unconfined.conf <<'EOF'
[Service]
ExecStart=
ExecStart=/usr/bin/env /usr/bin/qemu-ga --method=virtio-serial --path=/dev/virtio-ports/org.qemu.guest_agent.0 -F${FSFREEZE_HOOK_PATHNAME} $QEMU_GA_ARGS
EOF
systemctl daemon-reload
systemctl restart qemu-guest-agent
```

Tradeoffs:
it is per guest and lost when the guest is destroyed or recreated;
it has to be repeated after a template re-bake only if the new template lost it.
The restart interrupts whatever the agent was doing,
so run it before the first `mvm exec` rather than during a transfer.

### Bake the template again

Deleting `~/.local/share/mvm/images/template-fedora.qcow2` makes the next
`mvm create --image fedora` bake a template that carries the drop-in.
Tradeoffs:
it re-bakes one image (not the whole registry,
as `mvm update`
does),
and it must not happen while a guest created from that template runs,
because those disks use the file as their qcow2 backing file.
That is why this round verified the fix in a `HOME` of its own instead.

### The module's read-only boolean

`setsebool -P virt_qemu_ga_read_nonsecurity_files on` makes reads work:
the share,
the audit log and the policy.
Tradeoffs:
it grants no write and no execute,
so push into a directory the agent may not write and executing a pushed file
stay refused,
and it opens every non-security file in the guest to the agent's reads.

## What does not work

- **`SELinuxContext=` in a drop-in.**
  Measured:
  the unit fails with `status=203/EXEC` and
  `avc: denied { entrypoint }` for `virt_qemu_ga_exec_t`.
  The permission belongs to the target domain,
  and only `virt_qemu_ga_t` has it.
- **`virt_qemu_ga_run_unconfined`.**
  It grants a transition to `virt_qemu_ga_unconfined_t` for scripts under
  `fsfreeze-hook.d` only;
  the agent does not move.
- **`virt_qemu_ga_read_nonsecurity_files` alone.**
  Read-only;
  measured above.
  It cannot make push place a file,
  or exec run one.
- **Turning SELinux off,
  or permissive.**
  Never used and not a fix:
  the guest's own enforcement is the distro's decision,
  and the reported problem is that `mvm`'s agent was in the wrong domain.
- **Relabelling `/usr/bin/qemu-ga` or the guest's files.**
  Not attempted:
  it edits the distro's files,
  `restorecon` and the next policy update revert
  it,
  and it is a persistent guest-wide change where a unit drop-in is not.
- **Expecting `mvm push` to place a file through the agent in a directory the
  agent may not write.**
  The refusal names the mode and the path;
  `mvm` cannot create the parent directory,
  and the fix here is the drop-in,
  not a relocation of the file.
- **Reading the guest's audit log through the agent while the agent is
  confined.**
  `auditd_log_t` reads are denied;
  the serial console is the way in.

## Upstream filing decision

`.out-of-scope/` was read on 2026-11-24;
no entry covers SELinux,
qemu-guest-agent,
Fedora's policy or `mvm`.

1.  Is it really upstream's fault?
    No.
    `virt_qemu_ga_t` is confined on purpose:
    the agent is a root RPC surface that the hypervisor drives,
    and the policy decides what such a channel may touch.
    The defect is `mvm`'s,
    which provisioned the agent without noticing the domain the distro gives it.
2.  Can upstream fix it?
    Not applicable;
    there is nothing to fix upstream that is not a deliberate policy choice.
3.  Do they support this use case?
    Fedora documents the agent's booleans for reading non-security files and
    for unconfined fsfreeze hooks,
    which is the use case the policy supports.
    An agent with full root access from the guest side is not among them.
4.  Would the repository welcome a contribution?
    Not evaluated;
    there is nothing to contribute.
5.  Will they likely fix it?
    Not applicable.
    No upstream issue asks for an unconfined agent by default,
    and the module's booleans show the maintainers' position.
6.  Has a minimal fix been prototyped?
    The fix is on this repository's side of the boundary,
    in `package/mvm/cli`,
    and it is verified in [Verification](#verification).

Decision:
nothing to file,
no duplicate search was needed,
and no draft is kept.
