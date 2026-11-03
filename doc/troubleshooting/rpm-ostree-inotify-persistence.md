# Bazzite rpm-ostree 2026.3: persistent inotify configuration

## Symptom

The user requested a permanent `fs.inotify.max_user_watches` limit of `2000000`,
then asked whether placing the setting in `/etc/sysctl.d/` works on an rpm-ostree distribution.
No persistence failure was observed.
The verification date is 2026-11-02.

## Root cause and persistence mechanism

OSTree preserves local additions and modifications in `/etc` when preparing an upgrade.
Its [atomic-upgrade documentation][etc-merge] describes the merge from the booted deployment.
The override belongs in the writable administrator configuration directory.

The upstream `v2026.4` source at commit `4f4445a8912f70c604fa84a057a8f4e904e35a24`
copies added configuration files into the new deployment.
The relevant excerpt is
[`src/libostree/ostree-sysroot-deploy.c:592`][merge-source]:

```c
// src/libostree/ostree-sysroot-deploy.c
  for (guint i = 0; i < added->len; i++)
    {
      GFile *file = added->pdata[i];
      g_autofree char *path = g_file_get_relative_path (modified_etc, file);

      g_assert (path);

      if (!copy_modified_config_file (orig_etc_fd, modified_etc_fd, new_etc_fd, path, flags,
                                      cancellable, error))
        return FALSE;
    }
```

The installed `sysctl.d(5)` manual says that `systemd-sysctl.service` applies these files at boot,
with the last assignment in filename order taking precedence.
The installed `rpm-ostree(1)` manual's rollback section says that each deployment has a separate `/etc`.
Consequently, normal reboots and forward upgrades preserve this configuration,
but booting a deployment created before the change can use its older configuration.

## Verification

The host reports Bazzite `testing-45.20261102.4`,
rpm-ostree `2026.3`,
and libostree `2026.4`.
The packaged libostree reports Git identifier `21481dbbd6d1fc838270b6b3713ca3a20fbc24bc`.
Fetching that identifier from the upstream repository failed with
`fatal: remote error: upload-pack: not our ref 21481dbbd6d1fc838270b6b3713ca3a20fbc24bc`.
The source excerpt therefore documents the upstream release tag,
not a verified match to the packaged build's exact source revision.

Read-only checks produced these results:

- `sysctl fs.inotify.max_user_watches` returned `fs.inotify.max_user_watches = 2000000`.
- `sudo --non-interactive ostree admin config-diff` listed `A    sysctl.d/90-inotify-watches.conf`.
- `/usr/lib/systemd/systemd-sysctl --tldr --no-pager` listed the override after the KDE and vendor defaults,
  with no later assignment to this key.
- `systemctl --no-pager status systemd-sysctl.service` showed successful completion during the current boot.
  That boot preceded the configuration change.
- Both external citation selections were opened in `agent-browser`,
  and their referenced text was confirmed in the rendered pages.

The initial change applied the configuration through the boot service's executable,
`/usr/lib/systemd/systemd-sysctl --strict --verify --prefix=/fs/inotify/max_user_watches`,
which exited successfully.
No reboot, upgrade, rollback, or exhaustion test allocating watches was performed.
Persistence is supported by the documented mechanism and configuration checks,
not by an observed reboot or upgrade.

## Verified configuration

The administrator override is root-owned with mode `0644`:

```ini
# /etc/sysctl.d/90-inotify-watches.conf
# Keep the per-user inotify watch limit at 2,000,000 across reboots.
fs.inotify.max_user_watches = 2000000
```

This is normal configuration rather than an upstream workaround.
Its scope is a per-user kernel limit,
and a future later assignment could override it.
The retained deployment's configuration was not changed.

## What does not work

Running `ostree admin config-diff` without elevation returned
`error: You must be root to perform this command`.
Running the same read-only check through noninteractive `sudo` succeeded.
The vendor default was `524288`,
and the KDE-generated file specified `532480` before that vendor file in filename order.
Neither supplied the requested value without the administrator override.

## Upstream filing decision

There is nothing to report upstream because no tool defect was observed.

- Upstream fault: not established; the documented configuration mechanism works in the live check.
- Fix feasibility: not applicable without a defect.
- Supported use case: local `/etc` configuration is covered by the OSTree documentation.
- Contribution acceptance: not assessed because no contribution is proposed.
- Maintainer intent to fix: not assessed because no defect is proposed.
- Prototype: not applicable; the change is administrator configuration.

No issue or comment draft is needed,
and no upstream message was sent.

[etc-merge]: https://ostreedev.github.io/ostree/atomic-upgrades/#:~:text=as%20soon%20as%20you%20modify%20or%20add%20a%20file%20in%20%2Fetc
[merge-source]: https://github.com/ostreedev/ostree/blob/4f4445a8912f70c604fa84a057a8f4e904e35a24/src/libostree/ostree-sysroot-deploy.c#L592-L601
