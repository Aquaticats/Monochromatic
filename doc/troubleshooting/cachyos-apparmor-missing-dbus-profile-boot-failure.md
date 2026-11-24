# CachyOS dbus-broker 37 aborts an AppArmor-enabled policy-free boot and leaves KWin without a runtime directory

Status:
incomplete and unapproved.
The failed boot was diagnosed on 2026-11-23.
A policy-free
diagnostic boot then succeeded.
Broad profiles,
a live-ABI cache,
positive denial logging,
and
required workflows have since been tested.
Early-policy cold-boot behavior and final enforcement
remain unverified.

## Symptom

The first diagnostic boot enabled the AppArmor LSM while `apparmor.service` was disabled and
`/etc/apparmor/earlypolicy` was absent.
tty1 autologin did not produce a working Plasma session.
`kwin_wayland_wrapper` dumped core.
Disassembly identified the fatal message
`Could not create wayland socket`.

The failed boot's audit sequence contains failed starts for `dbus-broker`,
`avahi-daemon`,
`bluetooth`,
and `systemd-logind`.
The tty1 PAM session omitted `pam_systemd` and
`pam_systemd_home`.
The crashed KWin wrapper's environment omitted `XDG_RUNTIME_DIR`,
`WAYLAND_DISPLAY`,
and `DISPLAY`.

Removing `apparmor,` from the Limine entry's `lsm=` list restored the normal desktop.
The failed
boot contains no AVC record,
and the relevant processes were `unconfined`;
a profile denial is not
the cause.

## Root cause

The package-owned `/usr/lib/systemd/system/dbus-broker.service.d/apparmor.conf` requests an
exec-time profile transition:

```ini
[Service]
AppArmorProfile=dbus-system
```

No such profile was loaded.
[systemd.exec(5)](https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html#:~:text=Profiles%20must%20already%20be%20loaded%20in%20the%20kernel%2C%20or%20the%20unit%20will%20fail)
states the decisive behavior:

> Profiles must already be loaded in the kernel,
> or the unit will fail.

The system bus therefore could not start.
logind could not initialize the normal login session and
`XDG_RUNTIME_DIR`.
The missing `pam_systemd` grantor and missing environment are downstream evidence
of that startup failure.

KWin 6.8.3 then failed as its source predicts.
`src/helpers/wayland_wrapper/kwin_wrapper.cpp:79-81`
calls `wl_socket_create()` and aborts when it returns null
([source](https://github.com/KDE/kwin/blob/3e528155d822d41285a509d09751a88fa956267f/src/helpers/wayland_wrapper/kwin_wrapper.cpp#L74-L82)):

```cpp
m_socket = wl_socket_create();
if (!m_socket) {
    qFatal("Could not create wayland socket");
}
```

`src/helpers/wayland_wrapper/wl-socket.c:132-136` returns null when the runtime directory is absent
([source](https://github.com/KDE/kwin/blob/3e528155d822d41285a509d09751a88fa956267f/src/helpers/wayland_wrapper/wl-socket.c#L132-L136)):

```c
const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
if (!runtime_dir) {
    printf("XDG_RUNTIME_DIR not set");
    return NULL;
}
```

The user broker carries a `dbus-session` assignment in
`/usr/lib/systemd/user/dbus-broker.service.d/apparmor.conf`.
systemd documents `AppArmorProfile=` as
unsupported in per-user managers,
so this assignment is not treated as the established system-bus
cause.
The diagnostic override clears it explicitly.

There is also a separate broker-mediation limitation.
dbus-broker 37 commit
`e5b413f573b2c158bff88370944d571379977b7e`,
`src/launch/launcher.c:1374-1394`,
probes AppArmor
support,
then sets the optional mode to `CONFIG_APPARMOR_DISABLED`
([source](https://github.com/bus1/dbus-broker/blob/e5b413f573b2c158bff88370944d571379977b7e/src/launch/launcher.c#L1374-L1394)):

```c
if (enabled && !supported)
        fprintf(stderr, "Kernel is missing AppArmor DBus support. Ignoring.\n");
else if (enabled)
        fprintf(stderr, "AppArmor enabled, but not supported. Ignoring.\n");

policy.apparmor_mode = CONFIG_APPARMOR_DISABLED;
```

Local `system-local.conf` and `session-local.conf` files set `<apparmor mode="disabled"/>` to make
the broker's lack of message mediation explicit.
This is not a separately verified startup fix.
It
does not disable the exec-time `dbus-system` or `dbus-session` process profiles once real policy
loads.

An initial interpretation blamed `After=apparmor.service` for delaying startup.
That was incorrect:
`After=` is ordering,
not a requirement.
An initial attempt to remove the dependency with an empty
drop-in also failed.
The installed `systemd.unit(5)` manual says dependencies cannot be reset to
empty in a drop-in.
A no-op profile loader was briefly considered and then removed as unnecessary.
The decisive configuration is clearing the unavailable exec-time profiles during the policy-free
diagnostic.

## Verification

Versions measured on the host:

- `apparmor 4.1.8-2.1` and `apparmor.d-git v0.4916.0.r2.g5c34009-1`.
- `dbus-broker 37-3.1` and `systemd 262.2-1`.
- `kwin 6.8.3-1.1`,
  matching source commit `3e528155d822d41285a509d09751a88fa956267f`.
- dbus-broker v37 source commit `e5b413f573b2c158bff88370944d571379977b7e`.

The failed boot was `510ee21efc874ba19deac2cb25dd4714`.
Its retained audit output is saved under the
rollout scratch directory as `failed-boot-audit.txt`.
The system journal retained only the two KWin
coredump records,
so it is not treated as a complete service log.

The audit confirms service failures,
not an AppArmor denial:

```sh
sudo ausearch --start 11/23/26 06:20:57 --end 11/23/26 06:24:52 -m AVC,USER_AVC
```

Expected result:
no AVC records in the retained failed-boot audit.
That does not establish that
logging or policy works,
because no profiles had loaded.

The configured policy-free broker properties pass these static checks:

```sh
systemctl show dbus-broker.service --property=After --property=AppArmorProfile
systemctl --user show dbus-broker.service --property=AppArmorProfile
systemctl show apparmor.service --property=ActiveState --property=ExecStart
sudo xmllint --noout /etc/dbus-1/system-local.conf /etc/dbus-1/session-local.conf
```

The policy-free configuration produced these results before the corrected boot:

- both broker profile properties are empty;
- the normal `After=apparmor.service` dependency remains;
- the real profile loader is present in `ExecStart`,
  and the service is disabled and inactive on the
  recovered AppArmor-disabled boot;
- both local bus XML files parse.

These checks distinguish AppArmor activation from the missing-profile startup defect.
A
recovered-boot PAM session includes both `pam_systemd_home` and `pam_systemd`,
confirming that the
broken PAM environment is not a permanent host configuration.
The corrected boot established the
behavior that static checks could not prove.
Kernel `7.3.5-2-cachyos` had AppArmor enabled,
the
system and user brokers and logind were active,
and no failed units remained.
tty1 PAM included
`pam_systemd`.
`XDG_RUNTIME_DIR=/run/user/1000`,
`DISPLAY=:0`,
and `WAYLAND_DISPLAY=wayland-1` were
present,
and the KWin wrapper and compositor were running.
At that point no profiles had loaded;
this was a startup-chain check,
not a protection check.

The Safari browser bridge checked the three commit-pinned source links and the systemd text-fragment
link used here.
It displayed the quoted `wl_socket_create()` fatal branch,
the `XDG_RUNTIME_DIR`
null return,
the broker's disabled mode assignment,
and the systemd missing-profile rule.

The live feature file has SHA-256
`620f1543c890b7302d4f077c6d8d6e543931fa53b6dcd7ca9de4d4f90b24703a`.
The live D-Bus mask is
`acquire send receive`.
The early-cache `.features` file has the same hash.
Building this cache with
forced `--Complain` wrote no artifacts;
compiling from stored profile modes wrote the cache,
and
manual runtime loading independently specified complain mode.

A separate enforcing profile allowed one fixture read,
then denied another.
The denied `open`
returned `Permission denied` and produced a raw `apparmor="DENIED"` AVC with profile
`apparmor-rollout-positive-control`.
An explicit `audit deny` rule was needed for that targeted
audit record.
The control profile and cache artifact were removed after the proof.

The installed `aa-log` reads those AVCs.
The installed `ausearch` emits
`Malformed event skipped, rc=9` for the same valid kernel records before `<no matches>`.
Raw
audit-file inspection and `aa-log` are therefore the verification path here.
The `ausearch` parser
rejection is an unresolved tool issue,
not evidence that kernel records are absent.

## Recovery and verified workarounds

The demonstrated recovery is to press `e` on the enabled Limine entry and remove `apparmor,` from
its `lsm=` list before booting.
This produced a working desktop on 2026-11-23.
The persistent
`linux-cachyos-lts` recovery entry also sets `apparmor=0`,
but that exact LTS build has not been
runtime-tested.

`/etc/apparmor/recovery.md` records the recovery procedure.
`bootctl set-oneshot CachyOS/linux-cachyos` selects the main kernel for a subsequent diagnostic
without changing the recovery entry.

## Completed diagnostic and remaining boot check

The policy-free diagnostic used an empty `AppArmorProfile=` in both broker unit scopes,
without
replacing the vendor units.
The tradeoff was an intentionally unprofiled startup.
Both local broker
files retained:

```xml
<busconfig>
  <apparmor mode="disabled"/>
</busconfig>
```

After the successful policy-free boot,
both overrides matched their intended saved copies and were
removed.
Both managers were reloaded and the real `apparmor.service` was enabled.
Its active service
start and reload both returned success.
A privileged trace confirmed
`/sbin/apparmor_parser --replace -- /etc/apparmor.d` with exit status 0;
this is not a no-op loader.

The computed system broker property is again `dbus-system`.
The computed user broker property is
`dbus-session`.
Kernel status contains both profiles in complain mode,
but current processes started
before policy loading retain their old labels.
Only a cold boot can verify early attachment to these
services.

The live cache exists under `/etc/apparmor/earlypolicy/48da56f3.0`.
Its loader use remains the
outstanding test.
No cache-path AVC appeared during the successful live reload,
and the unit journal
did not capture the loader traces,
so neither is cited as proof of cache selection.
Do not use
`systemctl reload` on this enabled but inactive service:
systemd rejects an inactive reload before
invoking the script.
Start the service once,
then the reload path is available.

The local bus XML remains because it controls optional message mediation,
not process attachment.
Early policy,
`pam_systemd`,
`XDG_RUNTIME_DIR`,
and KWin must be rechecked together after the
real-policy cold boot.

## What does not work

- **An empty `After=` in a drop-in.**
  systemd retains dependencies.
  `systemctl show` still listed
  `apparmor.service`;
  an entire unit replacement plus masking the vendor drop-in would have been
  required.
  This approach was abandoned.
- **A no-op enabled profile loader.**
  It preserves `After=` only by introducing a new service state
  that the diagnostic did not need.
  The override was removed and the real loader restored.
- **Treating the missing runtime directory as the first cause.**
  It is a downstream KWin failure
  caused by broken system startup.
  Merely exporting a fallback `XDG_RUNTIME_DIR` would not repair
  D-Bus or logind.
- **Calling the KWin abort a profile denial.**
  No AVC exists in the failed boot's retained audit,
  and the process was `unconfined`.
- **Restarting the working desktop's system or user bus as a test.**
  That would unnecessarily
  disrupt the recovered session.
  Static checks and a controlled reboot are the verification path.
- **Claiming that disabling AppArmor in the bus XML removes all process confinement.**
  It disables
  optional bus-message mediation only;
  the exec-time profiles are distinct controls.

## Upstream filing decision

The available `.out-of-scope/` records were listed;
none match AppArmor,
dbus-broker,
or systemd.
No
new-issue draft is kept because there is nothing additive to file.

1.  **Upstream fault:**
    the failed boot came from a local policy-free configuration that retained a
    mandatory exec-time profile name.
    KWin's null check and systemd's missing-profile failure are
    explicit designed behavior.
2.  **Can upstream fix it:**
    no upstream patch is needed for the known missing-profile case.
    The
    local policy-free overrides address the diagnostic.
    The broker's limited mediation is separately
    documented by its own source and local configuration.
3.  **Supported use case:**
    the package's normal workflow expects the profile loader and profiles to
    exist before the broker transition.
    A deliberately policy-free diagnostic must clear the profile
    assignments locally.
4.  **Contribution fit:**
    no contribution is proposed.
    This is not an upstream patch candidate.
5.  **Likely fix:**
    broker AppArmor support was already discussed in `bus1/dbus-broker` issue 169,
    and AppArmor enablement pull request 410 is merged.
    The local incident does not add a
    version-confirmed defect beyond those records.
6.  **Prototype:**
    the consumer-side diagnostic overrides were installed,
    measured,
    and exercised in
    the successful policy-free boot.
    They were then removed after a live-ABI cache was prepared.
    The
    remaining early-cache cold boot and final broad-profile promotion are local rollout steps,
    not a
    fileable upstream reproduction.

The prior-art searches used the web queries
`dbus-broker fails at boot apparmor enabled no policy CachyOS` and
`site:github.com/bus1/dbus-broker "Kernel is missing AppArmor DBus support" exit boot issue`.
They
found the existing AppArmor support issue and the merged enablement pull request.
No issue or
comment will be filed for this local diagnostic incident.
