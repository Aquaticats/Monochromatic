# KDE libkscreen 6.8.2 help omits the per-monitor dimming command

## Symptom

A request for a 95% darkness overlay on one KDE Wayland monitor appears to need another application.
The installed `kscreen-doctor --help` documents `brightness` but does not list `dimming`.
The read-only `kscreen-doctor --outputs` command nevertheless reports a separate dimming percentage.

## Root cause

The capability exists in upstream source independently of its help entry.
[libkscreen `src/doctor/doctor.cpp:472`][dimming-parser] parses the `dimming` subcommand,
rejects values greater than `100`,
and converts the supplied percentage to a multiplier:

```cpp
// Upstream libkscreen: src/doctor/doctor.cpp:479
output->setDimming(dimming / 100.0);
```

Consequently,
`5` requests a `0.05` multiplier,
while `100` removes that multiplier.
This is a luminance control,
not a measurement of perceived darkness or an exact reproduction of a translucent black window.

[KWin `src/backends/drm/drm_output.cpp:713`][software-dimming] distinguishes DDC/CI monitors
from other brightness devices when updating hardware brightness:

```cpp
// Upstream KWin: src/backends/drm/drm_output.cpp:713
// With DDC/CI, dimming should be applied in software only,
// to avoid unnecessary EEPROM writes on the display side
if (!m_state.brightnessDevice->usesDdcCi()) {
    brightnessFactor *= newDimming;
}
```

This supports using the existing compositor dimmer for an external monitor.
The inspected source is the pinned upstream revision,
not an extraction of Fedora's installed source package.

## Verification

### Desktop and source inspection at 2026-11-12T22:19:15-05:00

The session reports `XDG_CURRENT_DESKTOP=KDE` and `XDG_SESSION_TYPE=wayland`.
`rpm --query --file /usr/bin/kscreen-doctor` reports `libkscreen-6.8.2-1.fc45.x86_64`.

Successful read-only command catalog:

- `kscreen-doctor --help`: brightness syntax is present; dimming syntax is absent.
- `kscreen-doctor --outputs`: the center portrait monitor is `DP-2`,
  with logical geometry `1080,0 1080x1920`,
  brightness `50%`,
  and dimming `100%`.
- `kscreen-doctor --outputs`: the lower landscape monitor is `HDMI-A-2`,
  with logical geometry `976,1920 1280x720`.

Source revisions inspected in fresh upstream clones:

- libkscreen: `e2665b4677b9855709ea8ce7d00fc6a3b47f64cc`.
- KWin: `75175b104504748aacd3d6a955a79d19751f8310`.

Both source-selection links were opened in `agent-browser`.
Browser evaluation confirmed their URL fragments and quoted source lines.

Unsuccessful diagnostic catalog:

- An out-of-range `kscreen-doctor output.DP-2.dimming.101` probe returned without a captured diagnostic.
  This does not verify the expected rejection path.
- An initial browser launch emitted Chrome's `pthread_create: Resource temporarily unavailable (11)`.
  Closing the owned session and reopening it with `TasksMax=512` completed the source checks.
  The cap changed together with session recreation;
  this does not establish the crash's cause.

A subsequent read-only output query still reported `DP-2` brightness `50%` and dimming `100%`.
No valid dimming change was applied to the user's desktop.
The proposed setting and its visible result remain untested at the display boundary.
There is no claim about persistence across login,
idle dimming,
or monitor reconnection.

## Available command and verification limits

For the center portrait monitor,
the source-supported command is:

```sh
# Terminal command for the center portrait monitor
kscreen-doctor output.DP-2.dimming.5
```

To remove that dimming multiplier:

```sh
# Terminal command restoring the dimming multiplier
kscreen-doctor output.DP-2.dimming.100
```

If the intended screen is the lower landscape monitor,
substitute `HDMI-A-2` for `DP-2`.
The tradeoff is a compositor luminance multiplier rather than an independently managed overlay window.
The commands are supported by the source trace;
they are not presented as a completed visual verification.

## What does not work as evidence

- A missing help entry does not establish that a feature is unavailable.
- The separate `brightness` setting does not establish an overlay-only operation.
- A successful read-only query does not prove the visible effect of a dimming change.
- The out-of-range probe does not count as a successful valid-value test.

## Upstream filing decision

Nothing to file for the requested dimming behavior:
the capability exists,
and no failure of a valid dimming command was reproduced.

- Upstream fault: no malfunction of the requested operation established.
- Fix feasibility: no behavioral fix required by the evidence.
- Supported use case: the dimming parser and DDC/CI source branch cover per-output dimming.
- Contribution policy: not evaluated because no filing is proposed.
- Maintainer willingness: not evaluated because no filing is proposed.
- Fix prototype: not applicable without a reproduced behavioral defect.

No duplicate search,
filing-readiness audit,
or upstream message was performed.
The help omission is recorded as a discovery limitation,
not an adopted upstream repair task.

[dimming-parser]: https://github.com/KDE/libkscreen/blob/e2665b4677b9855709ea8ce7d00fc6a3b47f64cc/src/doctor/doctor.cpp#L472-L480
[software-dimming]: https://github.com/KDE/kwin/blob/75175b104504748aacd3d6a955a79d19751f8310/src/backends/drm/drm_output.cpp#L709-L717
