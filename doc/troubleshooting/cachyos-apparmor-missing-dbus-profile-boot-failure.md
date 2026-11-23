# CachyOS dbus-broker 37 aborts an AppArmor-enabled policy-free boot and leaves KWin without a runtime directory

Status: incomplete and unapproved. The failed boot was diagnosed on 2026-11-23. A second policy-free boot with local diagnostic overrides has not run yet, and no broad AppArmor profile has been loaded.

## Symptom

The first diagnostic boot enabled the AppArmor LSM while `apparmor.service` was disabled and `/etc/apparmor/earlypolicy` was absent. tty1 autologin did not produce a working Plasma session. `kwin_wayland_wrapper` dumped core. Disassembly identified the fatal message `Could not create wayland socket`.

The failed boot's audit sequence contains failed starts for `dbus-broker`, `avahi-daemon`, `bluetooth`, and `systemd-logind`. The tty1 PAM session omitted `pam_systemd` and `pam_systemd_home`. The crashed KWin wrapper's environment omitted `XDG_RUNTIME_DIR`, `WAYLAND_DISPLAY`, and `DISPLAY`.

Removing `apparmor,` from the Limine entry's `lsm=` list restored the normal desktop. The failed boot contains no AVC record, and the relevant processes were `unconfined`; a profile denial is not the cause.

## Root cause

The package-owned `/usr/lib/systemd/system/dbus-broker.service.d/apparmor.conf` requests an exec-time profile transition:

```ini
[Service]
AppArmorProfile=dbus-system
```

No such profile was loaded. [systemd.exec(5)][systemd-apparmor] states the decisive behavior:

> Profiles must already be loaded in the kernel, or the unit will fail.

The system bus therefore could not start. logind could not initialize the normal login session and `XDG_RUNTIME_DIR`. The missing `pam_systemd` grantor and missing environment are downstream evidence of that startup failure.

KWin 6.8.3 then failed as its source predicts. `src/helpers/wayland_wrapper/kwin_wrapper.cpp:79-81` calls `wl_socket_create()` and aborts when it returns null ([source][kwin-wrapper]):

```cpp
m_socket = wl_socket_create();
if (!m_socket) {
    qFatal("Could not create wayland socket");
}
```

`src/helpers/wayland_wrapper/wl-socket.c:132-136` returns null when the runtime directory is absent ([source][kwin-socket]):

```c
const char *runtime_dir = getenv("XDG_RUNTIME_DIR");
if (!runtime_dir) {
    printf("XDG_RUNTIME_DIR not set");
    return NULL;
}
```

The user broker carries a `dbus-session` assignment in `/usr/lib/systemd/user/dbus-broker.service.d/apparmor.conf`. systemd documents `AppArmorProfile=` as unsupported in per-user managers, so this assignment is not treated as the established system-bus cause. The diagnostic override clears it explicitly.

There is also a separate broker-mediation limitation. dbus-broker 37 commit `e5b413f573b2c158bff88370944d571379977b7e`, `src/launch/launcher.c:1374-1394`, probes AppArmor support, then sets the optional mode to `CONFIG_APPARMOR_DISABLED` ([source][broker-mode]):

```c
if (enabled && !supported)
        fprintf(stderr, "Kernel is missing AppArmor DBus support. Ignoring.\n");
else if (enabled)
        fprintf(stderr, "AppArmor enabled, but not supported. Ignoring.\n");

policy.apparmor_mode = CONFIG_APPARMOR_DISABLED;
```

Local `system-local.conf` and `session-local.conf` files set `<apparmor mode="disabled"/>` to make the broker's lack of message mediation explicit. This is not a separately verified startup fix. It does not disable the exec-time `dbus-system` or `dbus-session` process profiles once real policy loads.

An initial interpretation blamed `After=apparmor.service` for delaying startup. That was incorrect: `After=` is ordering, not a requirement. An initial attempt to remove the dependency with an empty drop-in also failed. The installed `systemd.unit(5)` manual says dependencies cannot be reset to empty in a drop-in. A no-op profile loader was briefly considered and then removed as unnecessary. The decisive configuration is clearing the unavailable exec-time profiles during the policy-free diagnostic.

## Verification

Versions measured on the host:

- `apparmor 4.1.8-2.1` and `apparmor.d-git v0.4916.0.r2.g5c34009-1`.
- `dbus-broker 37-3.1` and `systemd 262.2-1`.
- `kwin 6.8.3-1.1`, matching source commit `3e528155d822d41285a509d09751a88fa956267f`.
- dbus-broker v37 source commit `e5b413f573b2c158bff88370944d571379977b7e`.

The failed boot was `510ee21efc874ba19deac2cb25dd4714`. Its retained audit output is saved under the rollout scratch directory as `failed-boot-audit.txt`. The system journal retained only the two KWin coredump records, so it is not treated as a complete service log.

The audit confirms service failures, not an AppArmor denial:

```sh
sudo ausearch --start 11/23/26 06:20:57 --end 11/23/26 06:24:52 -m AVC,USER_AVC
```

Expected result: no AVC records in the retained failed-boot audit. That does not establish that logging or policy works, because no profiles had loaded.

The configured policy-free broker properties pass these static checks:

```sh
systemctl show dbus-broker.service --property=After --property=AppArmorProfile
systemctl --user show dbus-broker.service --property=AppArmorProfile
systemctl show apparmor.service --property=ActiveState --property=ExecStart
sudo xmllint --noout /etc/dbus-1/system-local.conf /etc/dbus-1/session-local.conf
```

Expected current results:

- both broker profile properties are empty;
- the normal `After=apparmor.service` dependency remains;
- the real profile loader is present in `ExecStart`, and the service is disabled and inactive on the recovered AppArmor-disabled boot;
- both local bus XML files parse.

These checks distinguish AppArmor activation from the missing-profile startup defect. A recovered-boot PAM session includes both `pam_systemd_home` and `pam_systemd`, confirming that the broken PAM environment is not a permanent host configuration. The static checks alone do not establish that the new overrides boot correctly.

The Safari browser bridge checked the three commit-pinned source links and the systemd text-fragment link used here. It displayed the quoted `wl_socket_create()` fatal branch, the `XDG_RUNTIME_DIR` null return, the broker's disabled mode assignment, and the systemd missing-profile rule.

## Recovery and verified workarounds

The demonstrated recovery is to press `e` on the enabled Limine entry and remove `apparmor,` from its `lsm=` list before booting. This produced a working desktop on 2026-11-23. The persistent `linux-cachyos-lts` recovery entry also sets `apparmor=0`, but that exact LTS build has not been runtime-tested.

`/etc/apparmor/recovery.md` records the recovery procedure. `bootctl set-oneshot CachyOS/linux-cachyos` selects the main kernel for a subsequent diagnostic without changing the recovery entry.

## Configured diagnostic awaiting live verification

For a policy-free diagnostic, clear the exec-time assignments without copying the vendor units:

```ini
# /etc/systemd/system/dbus-broker.service.d/zz-no-policy-diagnostic.conf
[Service]
AppArmorProfile=
```

```ini
# /etc/systemd/user/dbus-broker.service.d/zz-no-policy-diagnostic.conf
[Service]
AppArmorProfile=
```

Keep `apparmor.service` disabled and `/etc/apparmor/earlypolicy` absent. The two local broker files contain:

```xml
<busconfig>
  <apparmor mode="disabled"/>
</busconfig>
```

This removes only temporary exec-time profile requests and broker-level message-mediation probes. The tradeoff is that this boot is intentionally unprofiled, so it cannot test protection. Once verified live-derived policy is loaded, delete the two unit overrides, reload both managers, and enable the real `apparmor.service`. Do not delete the local mediation overrides unless a compatible kernel and broker are measured.

Static configuration is not boot verification. The corrected boot still needs checks for the system and user buses, logind, PAM session environment, and a running KWin compositor.

## What does not work

- **An empty `After=` in a drop-in.** systemd retains dependencies. `systemctl show` still listed `apparmor.service`; an entire unit replacement plus masking the vendor drop-in would have been required. This approach was abandoned.
- **A no-op enabled profile loader.** It preserves `After=` only by introducing a new service state that the diagnostic did not need. The override was removed and the real loader restored.
- **Treating the missing runtime directory as the first cause.** It is a downstream KWin failure caused by broken system startup. Merely exporting a fallback `XDG_RUNTIME_DIR` would not repair D-Bus or logind.
- **Calling the KWin abort a profile denial.** No AVC exists in the failed boot's retained audit, and the process was `unconfined`.
- **Restarting the working desktop's system or user bus as a test.** That would unnecessarily disrupt the recovered session. Static checks and a controlled reboot are the verification path.
- **Claiming that disabling AppArmor in the bus XML removes all process confinement.** It disables optional bus-message mediation only; the exec-time profiles are distinct controls.

## Upstream filing decision

The available `.out-of-scope/` records were listed; none match AppArmor, dbus-broker, or systemd. No new-issue draft is kept because there is nothing additive to file.

1. **Upstream fault:** the failed boot came from a local policy-free configuration that retained a mandatory exec-time profile name. KWin's null check and systemd's missing-profile failure are explicit designed behavior.
2. **Can upstream fix it:** no upstream patch is needed for the known missing-profile case. The local policy-free overrides address the diagnostic. The broker's limited mediation is separately documented by its own source and local configuration.
3. **Supported use case:** the package's normal workflow expects the profile loader and profiles to exist before the broker transition. A deliberately policy-free diagnostic must clear the profile assignments locally.
4. **Contribution fit:** no contribution is proposed. This is not an upstream patch candidate.
5. **Likely fix:** broker AppArmor support was already discussed in `bus1/dbus-broker` issue 169, and AppArmor enablement pull request 410 is merged. The local incident does not add a version-confirmed defect beyond those records.
6. **Prototype:** the consumer-side diagnostic overrides were installed and their computed properties measured, but the corrected boot has not run. This is not a fileable upstream reproduction.

The prior-art searches used the web queries `dbus-broker fails at boot apparmor enabled no policy CachyOS` and `site:github.com/bus1/dbus-broker "Kernel is missing AppArmor DBus support" exit boot issue`. They found the existing AppArmor support issue and the merged enablement pull request. No issue or comment will be filed for this local diagnostic incident.

[systemd-apparmor]: https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html#:~:text=Profiles%20must%20already%20be%20loaded%20in%20the%20kernel%2C%20or%20the%20unit%20will%20fail
[kwin-wrapper]: https://github.com/KDE/kwin/blob/3e528155d822d41285a509d09751a88fa956267f/src/helpers/wayland_wrapper/kwin_wrapper.cpp#L74-L82
[kwin-socket]: https://github.com/KDE/kwin/blob/3e528155d822d41285a509d09751a88fa956267f/src/helpers/wayland_wrapper/wl-socket.c#L132-L136
[broker-mode]: https://github.com/bus1/dbus-broker/blob/e5b413f573b2c158bff88370944d571379977b7e/src/launch/launcher.c#L1374-L1394
