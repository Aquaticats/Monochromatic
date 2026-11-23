# AppArmor 4.1.8 blocks pciutils 3.15.0 full config reads and breaks the Intel NVMe Gen3 boot workaround

## Status

The immediate failure is repaired on boot `5b86b6ae-d743-4c00-82eb-7011241cfbbc`. The Intel D7-P5520
is stable at 8.0 GT/s x4, `zfs-nvme-gen3.service` reports `Result=success`, and no system units are
failed. The repaired `setpci` profile is included in the selected early-policy cache. This document
is not an owner approval of the broader rollout.

## Symptom

After enabling default enforcement, the system became very slow and `zfs-nvme-gen3.service` failed.
The Intel NVMe link returned to 16 GT/s instead of the required 8 GT/s.

The script printed:

```text
setpci: 0000:06:00.0: Instance #0 of Capability 0010 not found - there are no capabilities with that id.
setpci: 0000:07:00.0: Instance #0 of Capability 0010 not found - there are no capabilities with that id.
FAILED: 0000:07:00.0 link is 16.0 GT/s PCIe after reset (rc=0)
```

Direct numeric reads at `0x80` returned `ffff`. The AVCs named the actual missing capability:

```text
apparmor="DENIED" operation="capable" class="cap" profile="setpci"
capability=21 capname="sys_admin"
```

## Root cause

The packaged `setpci` profile has no `capability sys_admin` rule. The kernel deliberately limits PCI
configuration reads to the first 64 bytes without that capability, regardless of the process's
ordinary UID. This is a capability check at file-open time, not an AppArmor filename denial.

[Linux 7.3.5's
`pci_read_config`](https://github.com/gregkh/linux/blob/988117cc1bb17338c586a4e21ade88b5397eac33/drivers/pci/pci-sysfs.c#L707-L724)
shows the exact boundary:

```c
// drivers/pci/pci-sysfs.c:707-724
static ssize_t pci_read_config(struct file *filp, struct kobject *kobj,
                               const struct bin_attribute *bin_attr, char *buf,
                               loff_t off, size_t count)
{
        struct pci_dev *dev = to_pci_dev(kobj_to_dev(kobj));
        unsigned int size = 64;
        loff_t init_off = off;
        u8 *data = (u8 *) buf;

        /* Several chips lock up trying to read undefined config space */
        if (file_ns_capable(filp, &init_user_ns, CAP_SYS_ADMIN))
                size = dev->cfg_size;
        else if (dev->hdr_type == PCI_HEADER_TYPE_CARDBUS)
                size = 128;

        if (off >= size)
                return 0;
```

The Intel device is `0000:07:00.0`, behind chipset switch downstream port `0000:06:00.0`. Its PCI
Express capability is at `0x80`, and Target Link Speed is at capability offset `0x30`. Without
`CAP_SYS_ADMIN`, pciutils cannot read either location, so it cannot discover the capability or
perform the masked speed-cap write correctly.

The service script also ignored `setpci` failures and reset the device anyway. That made the
diagnostic worse: `rc=0` described the device reset, not the failed speed-cap writes.

## Verification

### Failing catalog

The installed `setpci` reported version 3.15.0. Before the capability grant:

- `setpci -s 0000:06:00.0 0x00.W` returned vendor ID `1022`.
- `setpci -s 0000:06:00.0 0x34.B` returned `50`.
- Reads at `0x50` and above returned `ff` or `ffff`.
- Direct sequential sysfs reads returned valid bytes, including `10 c0 62 01` at `0x80`.
- `pread` returned only the first 64 bytes, then EOF. This is consistent with the kernel source
  above.
- The AVCs named `capability=21`, `sys_admin`, and profile `setpci`.
- Both speed-cap commands failed, but the old script still reset the device and returned failure
  only after seeing 16 GT/s again.

An early claim that `setpci` passed a garbage offset was retracted after checking the wrapper's
`r10` register: the offset was the correct `0x80`. The empty `pread` result was the relevant
evidence.

### Passing catalog

After adding only the capability rule and reloading `setpci`:

- `setpci -s 0000:06:00.0 0x80.W` returned `c010`, exposing the real capability bytes.
- `setpci -s 0000:06:00.0 CAP_EXP+0x30.W` returned `0004`, the old 16 GT/s target.
- `setpci -s 0000:07:00.0 CAP_EXP+0x30.W` returned `0004` before the service repair.
- The same reads also succeeded through explicit `aa-exec -p setpci`.
- Restarting the existing service produced
  `OK: 0000:07:00.0 link now 8.0 GT/s PCIe x4 (reset rc=0)`.
- The real unit is `active (exited)`, `Result=success`, and current link speed is `8.0 GT/s PCIe`
  with width `4`.
- Installing the failure guard and restarting again produced
  `0000:07:00.0 already stable at 8 GT/s; no reset needed`.
- A fixture with `setpci` deliberately returning failure printed
  `failed to cap 0000:06:00.0 at 8 GT/s; refusing to reset` and did not reset the device.

## Verified workarounds

### Grant the exact full-config capability

```text
# /etc/apparmor.d/local/setpci
# The kernel exposes the full PCI config space only to CAP_SYS_ADMIN openers.
# pci_read_config otherwise truncates reads at 64 bytes, hiding CAP_EXP and LNKCTL2.
capability sys_admin,
```

Reload the profile and rebuild the early-policy cache after all local changes:

```bash
sudo apparmor_parser --replace /etc/apparmor.d/setpci
```

This grants the confined `setpci` process the same full-config privilege the kernel requires. It
does not unconfine the service script, add `/dev/mem` access, or allow alternate ECAM access. The
ECAM probe was rejected because `/dev/mem` is outside the measured requirement.

### Abort on a failed speed-cap write

The live `/usr/local/sbin/zfs-nvme-gen3.sh` now checks each write before any reset:

```bash
# /usr/local/sbin/zfs-nvme-gen3.sh
if ! setpci -s "$port" CAP_EXP+0x30.W=0003:000f; then
    log "failed to cap $port at 8 GT/s; refusing to reset"
    exit 1
fi
if ! setpci -s "$bdf" CAP_EXP+0x30.W=0003:000f; then
    log "failed to cap $bdf at 8 GT/s; refusing to reset"
    exit 1
fi
```

Tradeoff: a future failed speed-cap write leaves the current link speed unchanged and marks the unit
failed instead of performing a reset that cannot produce the intended state. This preserves evidence
and avoids masking the failed prerequisite. The pre-guard script is preserved in the rollout
artifact directory as `zfs-nvme-gen3.before-setpci-failure-guard.sh`.

## What does not work

- Treating `ffff` as valid config data hides the 64-byte capability boundary.
- Resetting after failed speed-cap writes returns to 16 GT/s and does not repair the workaround.
- Adding generic filesystem rules does not grant `CAP_SYS_ADMIN`; the missing rule is a capability
  grant, not another path.
- Using `aa-exec` to run shell utilities under `setpci` fails because that profile intentionally has
  no general `cat`, `grep`, `bash`, or second `setpci` exec grant. The successful explicit probe
  invoked `/usr/bin/setpci` directly.
- Switching to ECAM is unnecessary for this failure and would introduce a separate `/dev/mem` and
  physical-memory privilege boundary.
- Granting unconfined execution, using `pux`, or removing the profile would bypass the workload
  instead of repairing the measured missing capability.

## Upstream filing artifact

### Upstream filing decision

Nothing to add upstream. The installed kernel and pciutils behavior matches their source, and the
local profile omitted the capability that the kernel requires.

- Upstream fault: not established. The missing grant is in this deployment's profile.
- Upstream fix capability: no application or kernel change is required for the verified local
  repair.
- Supported use case: the service's documented Intel Gen3 workaround was exercised directly.
- Contribution policy: no filing is proposed. This is a local configuration record, not an upstream
  issue report.
- Likelihood of an upstream fix: not assessed because no upstream defect is claimed.
- Minimal compatible prototype: the local capability rule and the failure guard are the
  consumer-side changes. The failing and passing operations above are the verification.

Do not file a pciutils or kernel issue from this record. Reassess only if the same 64-byte
truncation occurs after the process has `CAP_SYS_ADMIN` in the initial user namespace and a valid
enforcing profile.
