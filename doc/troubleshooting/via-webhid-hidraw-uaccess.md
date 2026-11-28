# VIA sees the keyboard through WebHID but cannot initialize it without a hidraw user ACL

## Status

A device-specific `uaccess` rule gives the active local user read/write access to the three
1upsuper16v3 hidraw interfaces. Read-only open/close probes pass as the user and under enforcing
Helium, and the owner confirmed that VIA now initializes. No AppArmor policy or cache change was
required. This is an evidence record, not approval of unrelated keyboard operations.

## Symptom

VIA displayed:

```text
VIA can see 1upsuper16v3 through WebHID.
VID: 0x6F75 | PID: 0x5610

1upsuper16v3 does not seem to respond like a VIA-enabled keyboard.
```

The message suggests checking VIA-compatible firmware and reconnecting the device. Enumeration did
work, but initialization did not.

The keyboard exposed three interfaces:

```text
/dev/hidraw0
/dev/hidraw1
/dev/hidraw2
```

They were `root:root` mode `0600` with no user ACL. Opening one as the user or under the enforcing
Helium profile returned `EACCES`.

## Root cause

Udev identified all three interfaces as:

```text
ID_VENDOR_ID=6f75
ID_MODEL_ID=5610
ID_SERIAL=1upkeyboards_1upsuper16v3
```

No existing rule granted the local user access to them. The user could discover USB and HID
metadata, but could not open the raw device needed for the VIA protocol exchange.

This was not the measured AppArmor failure. Helium's Chromium abstraction already includes
`devices-u2f`, which grants `/dev/hidraw@{int} rwk`. A full audit search found no matching
`apparmor="DENIED"` record for `/dev/hidraw...`. Ordinary filesystem ownership and ACLs were the
boundary.

systemd 262.2's `builtin_uaccess()` checks for the `uaccess` tag and determines the active user on
the device seat in `src/udev/udev-builtin-uaccess.c:44-64`:

```c
r = sd_device_has_tag(dev, "uaccess");
// ...
r = device_get_seat(dev, &seat);
// ...
r = sd_seat_get_active(seat, /* ret_session= */ NULL, &uid);
```

It then applies the selected UID set to the device node at lines 123-126:

```c
r = devnode_acl(fd, uids);
if (r < 0) {
        log_device_full_errno(dev, r == -ENOENT ? LOG_DEBUG : LOG_ERR, r, "Failed to apply ACL: %m");
        goto reset;
}
```

Source: [systemd 262.2 `udev-builtin-uaccess.c`, tag and ACL application](https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/udev/udev-builtin-uaccess.c#L44-L64) and [the `devnode_acl` call](https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/udev/udev-builtin-uaccess.c#L123-L126).

The required repair was therefore a device-specific udev tag, not another AppArmor rule and not a
firmware change.

## Verification

Installed systemd is `262.2-1`; source commit `63511d7283d7d0e2a2fc28075481afaf1a7be8f5` was
inspected directly. `udevadm info --attribute-walk` supplied the exact parent `idVendor` and
`idProduct` values used by the rule.

### Patterns that fail

Before the rule, the interfaces had no `user:user` ACL entry and user opens failed with `EACCES`.
The first candidate rule also used `06f5` instead of the advertised `6f75`. Verbose udev evaluation
made that mistake explicit:

```text
ATTRS{idVendor}=="06f5": String "6f75" does not match "06f5": FAIL
No parent device passed parent conditions.
```

After correcting the ID, evaluation showed:

```text
ATTRS{idVendor}=="6f75": String "6f75" matches "6f75": PASS
ATTRS{idProduct}=="5610": String "5610" matches "5610": PASS
TAG+="uaccess": Added tag "uaccess".
```

A successful rule match and a queued `RUN{builtin}: uaccess` were not accepted as the final result.
The actual ACL appeared asynchronously afterward. Acceptance required the ACL entry and successful
open calls, not merely `udevadm trigger` returning.

### Patterns that work

The final ACL contained:

```text
user::rw-
user:user:rw-
group::---
mask::rw-
other::---
```

Each interface then passed read-only open and close as the user:

```text
opened=/dev/hidraw0
opened=/dev/hidraw1
opened=/dev/hidraw2
```

The same three opens passed under `aa-exec --profile=helium`. No HID report, firmware write, or
keyboard reconfiguration was sent by these probes. The owner then retried VIA and reported,
"Worked."

## Verified workarounds

### Grant uaccess only to this keyboard model

Install `/etc/udev/rules.d/50-qmk-1upsuper16v3.rules`:

```udev
# Allow the active local session to use VIA on the 1upsuper16v3 keyboard.
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="6f75", ATTRS{idProduct}=="5610", TAG+="uaccess"
```

Reload and re-trigger only the measured interfaces:

```sh
sudo udevadm control --reload-rules
sudo udevadm trigger --action=add \
  /sys/class/hidraw/hidraw0 /sys/class/hidraw/hidraw1 /sys/class/hidraw/hidraw2
```

Verify with `getfacl /dev/hidraw0` and an actual read-only open. If the ACL is not yet present, the
rule match and queued builtin are only intermediate evidence; the udev worker applies the ACL
asynchronously.

The tradeoff is that the active local user can read and write all three hidraw interfaces for this
vendor and product ID. A website still needs the user's WebHID authorization, but can then send
protocol commands to the keyboard. Other HID models receive no new access. The rule does not set
world-writable modes, add a group, change AppArmor, or disable the browser sandbox.

## What does not work

- Adding another Helium AppArmor rule: the existing profile already allowed hidraw access, while DAC
  denied the user first.
- Assuming the VIA warning proves incompatible firmware: in this incident the raw-device ACL, not
  the protocol firmware, prevented initialization.
- Using `06f5` for the vendor ID: the advertised value is `6f75`, and udev's verbose evaluation
  proves the mismatch.
- Granting `0666` or all-user access to every hidraw device: unnecessary and much broader than the
  measured keyboard-specific `uaccess` grant.
- Treating the tag or trigger as proof: the actual `user:user:rw-` ACL and successful open/close
  probes are required, followed by the real VIA initialization.
- Sending a test HID write: unnecessary for this permission repair and avoidable by using read-only
  open/close probes.

## Upstream filing decision

1.  **Is it really upstream's fault?** No. VIA discovered the device, and udev correctly enforced
    the existing device-node permissions. The local configuration lacked a keyboard-specific grant.
2.  **Can upstream fix it?** No VIA or systemd code change was required. The local udev rule now
    supplies the intended access.
3.  **Are they supporting this use case?** systemd's `uaccess` builtin explicitly supports granting
    an active local session access to tagged device nodes.
4.  **Would the repo welcome our contribution?** Not evaluated because no upstream defect was found
    and no public report is recommended.
5.  **Will they likely fix it?** Not predicted and not needed for this local rule.
6.  **Have we prototyped a compatible fix?** The device-specific rule passed udev evaluation,
    produced the user ACL, allowed both user and enforcing-Helium opens, and restored real VIA
    initialization.

No upstream issue or additive comment is recommended. This record does not authorize other keyboard
models, firmware changes, or arbitrary HID writes.
