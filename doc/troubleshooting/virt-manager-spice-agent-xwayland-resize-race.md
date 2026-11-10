# virt-manager auto resize fails when spice-vdagent races xwayland-satellite

## Metadata

- Status:
   Resolved locally and verified after a cold guest reboot.
- Diagnosed:
   2026-08-01.
- Affected environment:
   virt-manager 5.1.0,
   QEMU 11.0.0,
  spice-gtk 0.42,
   spice-vdagent 0.23.0,
   xwayland-satellite 0.8.1,
  labwc 0.9.6,
   and Fedora 44 guest userspace.
- Scope:
   `bazzite-labwc-test`,
   a labwc Wayland guest whose X11 bridge is
  xwayland-satellite on `DISPLAY=:12`.
- Disposition:
   Local systemd composition fix.
   No upstream issue filed.

## Symptom

virt-manager had **Auto resize VM with window** enabled,
 but resizing its
console left the guest at 1280x800.
 The host console allocation changed,
while both the first DRM preferred mode and labwc's current mode remained
1280x800.

The decisive correlated symptoms were:

```text
<target type='virtio' name='com.redhat.spice.0' state='disconnected'/>
spice-vdagent[1800]: could not connect to X-server
```

`~/.local/bin/follow-preferred-mode` was alive and correctly applied the
first mode in `/sys/class/drm/card*-Virtual-1/modes`.
 Its input never changed,
so changing that helper would only have hidden the earlier break in the
resize path.

## Expected resize path

The source trace establishes this sequence:

1.  virt-manager maps its per-VM setting directly to spice-gtk's
    `resize-guest` display property
    ([`virtManager/details/viewers.py:730-741`][virt-manager-viewer]).
2.  When enabled,
     spice-gtk calculates the guest dimensions from its display
    allocation and calls `spice_main_channel_update_display`
    ([`src/spice-widget.c:1394-1402`][spice-widget]).
3.  spice-gtk's monitor-config timer returns without sending if
    `agent_connected` is false
    ([`src/channel-main.c:1540-1560`][spice-channel]).
4.  QEMU receives the SPICE monitor configuration,
     copies its dimensions into
    `QemuUIInfo`,
     and calls `dpy_set_ui_info`
    ([`ui/spice-display.c:664-694`][qemu-spice]).
5.  virtio-gpu stores those dimensions and notifies the guest of a display
    event
    ([`hw/display/virtio-gpu-base.c:100-123`][qemu-virtio-gpu]).
6.  The kernel exposes the new preferred mode.
     labwc does not apply that
    hotplug preference itself in this setup,
     so `follow-preferred-mode`
    applies it with `wlr-randr`.

This explains an initially surprising result:
 QEMU updates virtio-gpu before
normal guest handling of the monitor message,
 but spice-gtk still requires a
connected SPICE guest agent before it sends the message at all.

The agent logged `xrandr output ID NOT FOUND` under satellite's XRandR
emulation.
 Direct XRandR reconfiguration was not the successful output path:
the QEMU display event changed the kernel preference,
 then
`follow-preferred-mode` changed the real labwc output with `wlr-randr`.

## Root cause

The stock user unit is wanted by `graphical-session.target` through
`/usr/lib/systemd/user/graphical-session.target.wants/spice-vdagent.service`.
It contains:

```ini
[Unit]
PartOf=graphical-session.target
After=graphical-session.target

[Service]
ExecStart=/usr/bin/spice-vdagent -x
```

The xwayland-satellite user unit also starts from the graphical-session
transaction.
 Its local drop-in pins the server to `:12` and imports that
value only in `ExecStartPost`:

```ini
[Service]
ExecStart=
ExecStart=/usr/bin/xwayland-satellite :12
ExecStartPost=/usr/bin/systemctl --user set-environment DISPLAY=:12
ExecStartPost=-/usr/bin/dbus-update-activation-environment DISPLAY=:12
```

No dependency edge existed between the units,
 so systemd started them in
parallel.
 The failing boot journal captured the race:

```text
[19.734543] Started spice-vdagent.service
[19.736784] Starting xwayland-satellite.service
[19.770726] spice-vdagent: could not connect to X-server
[20.036120] Started xwayland-satellite.service
```

The agent inherited stale `DISPLAY=:0`,
 exited before satellite was ready on
`:12`,
 and had no restart policy.
 The SPICE virtio channel therefore stayed
disconnected.
 spice-gtk's `agent_connected` guard suppressed every monitor
configuration,
 QEMU retained 1280x800 as the preferred mode,
 and the labwc
follower correctly observed no change.

`~/.config/autostart/spice-vdagent.desktop` with `Hidden=true` only disables
the XDG desktop-autostart copy.
 It does not remove the native systemd user
unit from `graphical-session.target`.

## Correction

The narrow fix is a user-unit drop-in:

```ini
# ~/.config/systemd/user/spice-vdagent.service.d/override.conf
[Unit]
Requires=xwayland-satellite.service
After=xwayland-satellite.service

[Service]
Environment=DISPLAY=:12
```

Apply it with:

```bash
systemctl --user daemon-reload
systemctl --user restart spice-vdagent.service
```

`Requires` makes satellite part of the agent's transaction and propagates an
explicit satellite stop or restart to the agent,
 as specified by
[`systemd.unit(5)`][systemd-unit].
 `After` prevents the agent
from starting until satellite's start job,
 including its `ExecStartPost`,
completes.
 The explicit environment assignment prevents a later stale
user-manager value from selecting `:0`.

No `Restart=on-failure` was added.
 A restart would make this specific race
less visible but would leave the missing dependency in place.
 Ordering the
actual prerequisite is both narrower and deterministic.

The deployed file and its canonical copy had identical SHA-256 digest
`1bc38998f9ba563496877c5a1300dd7872a45fa1c1d88ad0b87aa7b6062973c6`.

## Verification

### Before the correction

- Per-VM `resize-guest=1` was present.
- Toggling the setting and reconnecting the console did not change the guest
  mode.
- The SPICE virtio channel was disconnected.
- `spice-vdagent.service` reported `could not connect to X-server` 36 ms
  after systemd logged the service as started.
- xwayland-satellite became available after the failed agent had exited.
- Disabling both SPICE agents and resizing left the preferred mode at
  1280x800.

### Causal recovery test

Restarting the native user `spice-vdagent.service` after satellite was active
made the channel connected.
 A real host console resize then changed both the
DRM preferred mode and labwc current mode from 1400x2810 to 1600x2210.
 This
isolated the failure to agent startup rather than QEMU,
 virtio-gpu,
 labwc,
 or
the preferred-mode follower.

### Cold-reboot test

After installing the drop-in,
 the guest rebooted to boot ID
`3289175d-02a4-41e6-a1c4-d8a6331d197a`.
 Its journal showed:

```text
[14.156793] Starting xwayland-satellite.service
[14.416500] Started xwayland-satellite.service
[14.419235] Started spice-vdagent.service
```

Post-boot checks confirmed:

- `xwayland-satellite.service` and `spice-vdagent.service` were active.
- The agent process had `DISPLAY=:12`.
- The SPICE virtio channel reported `state='connected'`.
- `follow-preferred-mode` was running.
- The agent journal contained no X-server connection failure.
- A 900x1000 host console frame produced preferred and current guest mode
  1800x1810.
- A 700x1500 host console frame produced preferred and current guest mode
  1400x2810.
- labwc retained output scale 2 for both sizes.

An explicit `systemctl --user restart xwayland-satellite.service` also
exercised dependency propagation.
 The satellite PID changed from 1950 to
6020 and the agent PID from 2049 to 6032.
 Both units returned active,
 the
SPICE channel reconnected,
 and a subsequent resize again produced matching
1800x1810 preferred and current modes at scale 2.
 No extra `PartOf` edge is
needed for explicit restarts because this systemd version propagates them
through `Requires`.

The differing height offsets are expected because spice-gtk sizes from the
console display allocation,
 not the outer KWin frame.
 The important invariant
is that each host allocation changed the kernel preferred mode and labwc
current mode together.

## Rejected hypotheses and ineffective workarounds

### The virt-manager setting was disabled

False.
 The per-VM value was already `resize-guest=1`,
 and virt-manager's
source showed that value reaches spice-gtk's `resize-guest` property.
Toggling it could not repair a disconnected agent channel.

### The preferred-mode follower was broken

False.
 The process was alive and followed every preferred-mode change once
QEMU emitted one.
 Before agent recovery,
 the kernel preference itself was
stuck at 1280x800.

### A connected spice-vdagent swallowed QEMU's kernel update

False.
 This was an earlier misdiagnosis.
 With the agent disabled,
 spice-gtk
sent no monitor configuration.
 With the agent connected,
 QEMU's SPICE display
callback updated virtio-gpu and the follower applied the resulting preferred
mode.
 The source trace and the causal recovery test agree.

### Mutter D-Bus errors were fatal on labwc

False.
 The agent logged that `org.gnome.Mutter.DisplayConfig` had no owner,
which is expected outside Mutter.
 It continued through its XRandR path via
xwayland-satellite.

### Launching a debug agent from SSH proved the graphical path

Inconclusive.
 `spice-vdagentd` identified graphical session 1,
 while the
SSH-launched agent belonged to session 23.
 Restarting the existing user unit
inside the graphical user manager avoided that session mismatch and produced
the connected-channel result.

### Adding only a restart policy was sufficient

Rejected.
 It could recover after a timing-dependent delay,
 but it would not
encode the real prerequisite or guarantee the correct display value.

## Contributing conditions

- Both native units were attached to the same target without an ordering
  edge.
- xwayland-satellite used a non-default display number.
- The user manager held stale `DISPLAY=:0` until satellite's
  `ExecStartPost` import.
- The stock agent unit does not restart after this startup failure.
- The stock satellite unit also has `Restart=no`.
  An unexpected satellite crash is a broader X11-bridge outage and is not
  repaired by this boot-ordering fix.
- labwc needs the existing preferred-mode follower to apply virtio-gpu's new
  preference after the transport path is restored.

## Upstream assessment

This is a local integration fault between a stock SPICE agent unit and a
custom xwayland-satellite session layout.
 The agent unit cannot name every
possible external Xwayland provider,
 while the satellite unit does not own
SPICE startup.
 The correct dependency belongs in this environment's user-unit
override.

No upstream issue is warranted:
 the failure is resolved,
 reproducible only
under this local composition,
 and no deferred upstream fix remains to track.

## Source and environment record

Source was inspected at these exact revisions:

- virt-manager `v5.1.0`,
   commit
  `eb4898b19e550af19daea49ae5ed15d2d70a2fc4`.
- spice-gtk `v0.42`,
   commit
  `f04479c16f0969fb394ebe74b6eff74e560a42f0`.
- QEMU `v11.0.0`,
   commit
  `98b060da3a4f92b2a994ead5b16a87e783baf77c`.

The runtime versions came from `flatpak info`,
 `virsh version`,
 guest
`rpm -q`,
 and `uname -r`.
 The virt-manager Flatpak's manifest names the
spice-gtk 0.42 tarball,
 and its library identifies itself as compiled from
spice-gtk 0.42.
 The official spice-gtk `v0.42` tag resolves to the same
`f04479c16f...` commit as the inspected checkout.
 The installed satellite
RPM is 0.8.1-1.fc44,
 although its startup banner reports version 0.8.0.
Unit structure and ordering came from
`systemctl --user cat`,
 `systemctl --user list-dependencies`,
 and
`journalctl --user -b -o short-monotonic`.

[virt-manager-viewer]: https://github.com/virt-manager/virt-manager/blob/v5.1.0/virtManager/details/viewers.py#L730-L741
[spice-widget]: https://cgit.freedesktop.org/spice/spice-gtk/tree/src/spice-widget.c?h=v0.42#n1394
[spice-channel]: https://cgit.freedesktop.org/spice/spice-gtk/tree/src/channel-main.c?h=v0.42#n1540
[qemu-spice]: https://github.com/qemu/qemu/blob/v11.0.0/ui/spice-display.c#L664-L694
[qemu-virtio-gpu]: https://github.com/qemu/qemu/blob/v11.0.0/hw/display/virtio-gpu-base.c#L100-L123
[systemd-unit]: https://www.freedesktop.org/software/systemd/man/latest/systemd.unit.html#Requires=
