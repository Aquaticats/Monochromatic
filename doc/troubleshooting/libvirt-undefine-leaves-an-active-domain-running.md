# `virsh undefine` in libvirt 6 through 11 leaves an active domain running and transient

## Symptom

A pipeline that rebuilds one VM reuses the domain name `monochromatic-dev`.
It probes the existing definition with `virsh dominfo`,
reads the state with `virsh domstate`,
then removes the domain and defines the fresh one.

When the existing domain is paused,
idle (`VIR_DOMAIN_BLOCKED`),
in shutdown,
or pmsuspended,
the removal step runs `virsh undefine` without a preceding `virsh destroy`.
No command fails.
The undefined domain keeps running,
and the `virsh define` that follows stores the new XML as that running domain's configuration for its next boot,
while the pipeline has already replaced the qcow2 path under it with `sudo cp`.

The user-visible result is a VM that appears imported and starts with
`virsh --connect qemu:///session start monochromatic-dev` reporting that it is already running,
while the QEMU it is talking to still has the replaced image's inode open.

This was not reproduced against a live hypervisor in the session that wrote this doc:
`virsh` is not installed on the host,
and the call chain below is read from libvirt's source.
The consumer-side behaviour is covered by a unit suite (see Verification).

## Root cause

`virsh domstate` prints eight strings,
and five of them describe a domain whose QEMU process libvirt still holds.
Only one of the five is spelled `running`.

`tools/virsh-domain-monitor.c:129-140` renders the `virDomainState` values:

```c
// tools/virsh-domain-monitor.c (libvirt master, commit d6c09bf)
VIR_ENUM_DECL(virshDomainState);
VIR_ENUM_IMPL(virshDomainState,
              VIR_DOMAIN_LAST,
              N_("no state"),
              N_("running"),
              N_("idle"),
              N_("paused"),
              N_("in shutdown"),
              N_("shut off"),
              N_("crashed"),
              N_("pmsuspended"),
);
```

`VIR_DOMAIN_BLOCKED` prints as `idle`,
so a guard comparing the state to the literal `running` misses four active states.

Removal does not fail for those states,
because `virsh undefine` accepts an active domain.
`src/qemu/qemu_driver.c:7059-7064` turns it transient and leaves it running:

```c
// src/qemu/qemu_driver.c (libvirt master, commit d6c09bf)
    /* If the domain is active, keep it running but set it as transient.
     * domainDestroy and domainShutdown will take care of removing the
     * domain obj from the hash table.
     */
    vm->persistent = 0;
    if (!virDomainObjIsActive(vm))
```

The domain is active exactly when libvirt still holds its QEMU process,
`src/conf/domain_conf.h:3789-3793`:

```c
// src/conf/domain_conf.h (libvirt master, commit d6c09bf)
static inline bool
virDomainObjIsActive(virDomainObj *dom)
{
    return dom->def->id != -1;
}
```

`virsh destroy` is the call that stops it,
and it refuses a domain with no process,
`src/qemu/qemu_driver.c:2134-2140`:

```c
// src/qemu/qemu_driver.c (libvirt master, commit d6c09bf)
    if (!virDomainObjIsActive(vm)) {
        if (starting) {
            VIR_DEBUG("Domain %s is not running anymore", vm->def->name);
            ret = 0;
        } else {
            virReportError(VIR_ERR_OPERATION_INVALID,
                           "%s", _("domain is not running"));
        }
```

So the two directions of the mistake are not symmetric:
skipping `destroy` for an active domain is silent and leaves two owners of one domain name,
while running `destroy` for an inactive one fails loudly with `domain is not running`.

### Earlier reading that was wrong

Issue 1306 of this repository states that "`virsh undefine` refuses any active domain".
The source above disproves it for the QEMU driver:
undefine succeeds and the domain keeps running.
The defect the issue names is real,
and its blast radius is larger than assumed,
because nothing fails at the removal step.

## Verification

Sources read:

- libvirt master at commit `d6c09bf592632a242e3475e832187610aba71763` (committer date 2026-11-06),
  [the state spellings](https://github.com/libvirt/libvirt/blob/d6c09bf592632a242e3475e832187610aba71763/tools/virsh-domain-monitor.c#L129-L140),
  [undefine leaving an active domain running](https://github.com/libvirt/libvirt/blob/d6c09bf592632a242e3475e832187610aba71763/src/qemu/qemu_driver.c#L7059-L7064),
  [destroy refusing an inactive domain](https://github.com/libvirt/libvirt/blob/d6c09bf592632a242e3475e832187610aba71763/src/qemu/qemu_driver.c#L2134-L2140),
  [the active test](https://github.com/libvirt/libvirt/blob/d6c09bf592632a242e3475e832187610aba71763/src/conf/domain_conf.h#L3789-L3793).
- Tags `v6.0.0`,
  `v8.0.0` and `v10.0.0` of the same file all carry the
  "keep it running but set it as transient" branch,
  so the behaviour is not new in any recent line.
- Each permalink above was loaded in `agent-browser` and its page text checked for the quoted C.

No live hypervisor was available in the session:
`rpm -q libvirt-client` reports the package is not installed,
and installing packages on the host is outside an agent's authorization here.
The removed step of the harness is therefore the `virsh` calls themselves.

Runnable harness for the consumer side,
which is where the decision lives:

```sh
# package/vm-builder/cli
mise run //package/vm-builder/cli:build
mise run //package/vm-builder/cli:test:unit
```

The suite `removing an existing libvirt domain` supplies a runner that records the argument vectors,
and asserts one verdict per `domstate` spelling:

- Works,
  `destroy` runs before `undefine`:
  `running`,
  `idle`,
  `paused`,
  `in shutdown`,
  `pmsuspended`.
- Works,
  `destroy` is skipped:
  `no state`,
  `shut off`,
  `crashed`.
- Works,
  no call at all:
  `virsh dominfo` rejects,
  so the domain is not defined.
- Fails on purpose,
  `UnknownDomainStateError`:
  any other string,
  so a libvirt change stops the pipeline instead of guessing.

## Verified workarounds

Classify the state before removing the domain,
and destroy for every active spelling.
`package/vm-builder/cli/src/domain-removal.ts` carries the two lists and the verdict:

```ts
// package/vm-builder/cli/src/domain-removal.ts
const ACTIVE_DOMAIN_STATES: readonly string[] = [
  'running',
  'idle',
  'paused',
  'in shutdown',
  'pmsuspended',
];
```

Tradeoffs:

- A state string that neither list carries stops the pipeline with `UnknownDomainStateError` after the image build,
  which is the expensive end of the run.
  The alternative,
  guessing,
  is the failure this document describes.
- `crashed` skips `destroy`,
  because libvirt has no process left to stop there and the call would fail.
- The lists are a copy of libvirt's enum,
  not a query:
  `virsh domstate` prints a name,
  not the integer the API returns.
  A future libvirt release that adds or renames a spelling fails loudly rather than misbehaving.

## What does not work

- Comparing to `running` only.
  Four active spellings slip through and nothing errors;
  this is the defect being documented.
- Running `virsh destroy` unconditionally.
  A domain in `shut off`,
  `crashed` or `no state` has no process,
  and the call fails with `domain is not running`.
- Trusting `virsh undefine` to stop the domain.
  It removes the definition and leaves the QEMU running as a transient domain.
- Reading the state from `virsh dominfo`.
  `dominfo` prints a state line with its own formatting inside a table,
  while `domstate` prints the bare name a comparison needs.

## Upstream filing decision

No issue,
comment or patch is filed,
and none should be.
The six constraints,
walked:

1. Is it upstream's fault?
   No.
   Undefining an active domain while leaving it running is documented in the source comment,
   and it is the behaviour a transient domain needs.
   The defect was in this repository's guard.
2. Can upstream fix it?
   Not applicable without a defect.
3. Are they supporting this use case?
   Not applicable;
   a state name rather than an integer is `virsh`'s presentation choice.
4. Would the repository welcome our contribution?
   Not applicable without a change to propose.
5. Will they likely fix it?
   Not applicable;
   nothing in libvirt's tracker or release notes was consulted,
   because no report is needed.
6. Have we prototyped a minimal fix?
   Not applicable upstream.
   The consumer-side fix is landed and tested in this repository.

`.out-of-scope/` holds no libvirt entry,
so the exemption check found nothing to apply;
the decision above rests on constraint 1 alone.
