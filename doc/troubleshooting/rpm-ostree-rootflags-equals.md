# rpm-ostree 2026.3 rejects a rootflags replacement containing equals signs

## Symptom

The user requested permanent access-time disabling.
The host runs Bazzite with `rpm-ostree 2026.3-2.fc45`
and `ostree-libs 2026.4-1.fc45`.
Replacing the root mount options with this argument failed:

```text
# Observed rpm-ostree kargs argument and diagnostic
--replace=rootflags=subvol=root,user_subvol_rm_allowed,compress=zstd:3,noatime
error: No karg 'rootflags=subvol' found
```

The failed transaction did not save the requested boot argument.
The live mounts and `/etc/fstab` had already been updated separately.

## Root cause

The inspected upstream implementation of
[`ostree_kernel_args_new_replace`, `src/libostree/ostree-kernel-args.c:325`][replacement-source]
first separates the key from its value.
A further equals sign selects the `key=old_value=new_value` interpretation:

```c
// Upstream src/libostree/ostree-kernel-args.c:341
if (val && strchr (val, '='))
  {
    g_autofree char *old_val = g_strdup (val);
    const char *new_val = split_keyeq (old_val);
```

For the rejected argument,
this makes `rootflags` the key and `subvol` the old value to find.
The existing value starts with `subvol=root`,
so that lookup fails.
The same function emits the observed diagnostic:

```c
// Upstream src/libostree/ostree-kernel-args.c:349
if (!ot_ptr_array_find_with_equal_func (entries, old_val, kernel_args_entry_value_equal,
                                        &i))
  return glnx_throw (error, "No karg '%s=%s' found", key, old_val);
```

The source snapshot is commit `896d5225cd670484d57351b26cf6e561178f9938`.
It explains the observed parsing result;
it is not a verified reconstruction of every Fedora packaging patch.
The assumption that a single existing `rootflags` value made this `--replace` form unambiguous was wrong.
The nested equals sign still selects the old-value form.

## Verification

The rejected command was an attempted configuration change,
not a test deliberately repeated against the host.
Its result is recorded in the `rpm-ostreed` journal as:

```text
# rpm-ostreed journal, 2026-11-12T17:52:56-05:00
Txn KernelArgs on /org/projectatomic/rpmostree1/default failed: No karg 'rootflags=subvol' found
```

The installed `rpm-ostree kargs --help` lists key deletion and argument append separately.
The replacement attempt failed;
the combined deletion and append transaction is in progress.
Read-only inspection with `rpm-ostree kargs` succeeded before the change.

The source selection was opened with `agent-browser`.
The rendered page contained the parser branch and diagnostic,
and its URL retained `#L325-L353`.

## Verified workarounds

The selected workaround removes the existing `rootflags` key and appends its full replacement
in the same `rpm-ostree kargs` invocation:

```text
# Submitted configuration command, pending completion
rpm-ostree kargs --delete=rootflags --append=rootflags=subvol=root,user_subvol_rm_allowed,compress=zstd:3,noatime
```

This preserves the existing subvolume,
compression,
and user-subvolume-removal options.
It moves the argument to the end of the argument list.
Deleting the key would remove every value associated with it,
so this host was first checked to have a single `rootflags` value.
The other kernel arguments must be compared after completion.

## Installed access-time policy at 2026-11-12T22:53:11Z

`/etc/fstab` now requests `noatime` for `/`,
`/boot`,
`/boot/efi`,
`/home`,
and `/var`.
Existing compression,
read-only,
and filesystem-check settings were preserved.
The encrypted SATA mount already requested `noatime`
in `/etc/systemd/system/mount-860.service`.

Live bind remounts applied `noatime` to the disk mountpoints
without changing their filesystem settings.
`findmnt --types btrfs,ext4,vfat --output TARGET,FSTYPE,VFS-OPTIONS`
confirmed `noatime` for `/etc`,
`/sysroot`,
`/sysroot/ostree/deploy/default/var`,
`/var`,
`/var/home`,
`/var/mnt/encrypted`,
`/boot`,
and `/boot/efi`.
`/sysroot` remains read-only.

Disposable file probes ran in `/etc`,
`/var`,
`/var/home`,
`/boot`,
and `/boot/efi`.
Each set an old access time and read the file.
Before the remounts,
all five reads changed access time,
proving the positive control.
After the remounts,
all five access times stayed unchanged.
The fixture directories were removed by their creating harness.

Backups,
the configuration harness,
and before/after probe results are retained in
`~/temp/agent/disable-atime-dVPlXi/`.
`findmnt --verify --verbose --tab-file` reported zero parse errors for both fstab versions.
Both versions produced the same existing `/home` ordering error,
noncanonical-path warning,
and unprivileged swapfile-inspection warnings.

The policy covers the configured local disk mounts.
Kernel virtual filesystems,
RAM filesystems,
application-managed cloud mounts,
and future manual mount commands are outside this verified coverage.
No reboot was performed.
The saved boot argument still requires final verification.

## What does not work

Shell quoting does not remove the equals signs within the argument passed to the replacement parser.
The rejected command was already passed as a single argument.
The rejected `--replace` form is not the selected way to update this nested value.

## Upstream filing decision

- Upstream fault: not established.
  The attempted argument selects the existing old-value replacement grammar.
- Fix feasibility: extending the interface is possible in principle,
  but no upstream change is needed for the host configuration.
- Supported use case: installed help explicitly offers deletion and append;
  unrestricted nested-value replacement was not established.
- Contribution policy: not assessed because no defect report is proposed.
- Maintainer intent: not assessed;
  no willingness or refusal is inferred.
- Prototype: none,
  because the upstream-fault gate did not pass.

`gh search issues '"No karg" rootflags' --repo coreos/rpm-ostree --json number,title,url`
returned an empty result on 2026-11-12.
This narrow query does not establish the absence of related reports.
There is no upstream issue or comment to add from this invocation error.
No filing,
draft,
or readiness recommendation was made.

[replacement-source]: https://github.com/ostreedev/ostree/blob/896d5225cd670484d57351b26cf6e561178f9938/src/libostree/ostree-kernel-args.c#L325-L353
