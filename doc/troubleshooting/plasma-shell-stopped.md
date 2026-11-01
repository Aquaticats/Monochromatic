# Plasma 6.7.5 desktop and taskbars disappear while the shell service is stopped

## Symptom

On 2026-10-17,
the user reported that KDE and all taskbars had disappeared.
The installed package was `plasma-workspace-6.7.5-1.fc44.x86_64`,
verified with `rpm --query plasma-workspace`.

## Root cause boundary

`systemctl --user status plasma-plasmashell.service --no-pager --lines=25`
reported `inactive (dead)` since 00:31:42 EDT.
The previous process exited with `status=0/SUCCESS`.
The reason for that exit was not established.
This is a separate incident from the input stalls documented in
[the Bazzite stall investigation](bazzite-desktop-input-stalls.md).

The installed `/usr/lib/systemd/user/plasma-plasmashell.service:10`
contains this recovery policy:

```ini
# /usr/lib/systemd/user/plasma-plasmashell.service:8
[Service]
ExecStart=/usr/bin/plasmashell --no-respawn
Restart=on-failure
Type=dbus
BusName=org.kde.plasmashell
```

No crash cause or upstream defect was diagnosed.
The installed unit and live session were sufficient for the requested recovery;
no upstream source investigation or patch was performed.

## Verification

Before recovery,
the service was inactive.
After recovery,
`systemctl --user show plasma-plasmashell.service`
reported `ActiveState=active`,
`SubState=running`,
and `MainPID=71892`.
The active timestamp was 00:35:16 EDT.

The live DBus interface was inspected before calling its scripting method.
This read-only query succeeded:

```sh
# Live desktop layout query, run in the user's graphical session.
gdbus call --session --dest org.kde.plasmashell --object-path /PlasmaShell \
  --method org.kde.PlasmaShell.evaluateScript \
  'print(JSON.stringify({desktops: desktops().length, panels: panels().map(function(p) { return {id:p.id, screen:p.screen, location:p.location, widgets:p.widgets().length}; })}));'
```

It returned four desktops and bottom panels with IDs 5,
32,
34,
and 36 on screens 0 through 3.
The first panel had six widgets;
the remaining panels each had one.
This verifies the loaded layout and responsive scripting host,
but is not a screenshot or independent visual confirmation.

## Verified workaround

Start the existing service:

```sh
# Restore the stopped desktop shell in the existing graphical session.
systemctl --user start plasma-plasmashell.service
```

The invocation returned successfully and the live layout query passed.
Starting the service restored its loaded layout without resetting configuration.
This recovery does not identify or prevent the initiating exit.

## What does not work

- The restricted agent command environment could not connect to the user bus:
  `systemctl` emitted `Failed to connect to user scope bus via local transport: Operation not permitted`.
  Running the recovery through approved host execution succeeded.
- An initial layout query had an extra closing parenthesis.
  Plasma returned a syntax error identifying an expected semicolon.
  The corrected query in Verification succeeded.

## Upstream filing decision

1.  Upstream fault is unproven because only a clean shell exit was established.
2.  Fixability cannot be assessed without an initiating cause.
3.  The installed desktop unit supports running Plasma Shell.
4.  Contribution policies were not investigated because no upstream defect was established.
5.  Maintainer intent and tracker duplicates were not investigated because no attributable defect was established.
6.  No source fix was prototyped; the requested action was session recovery.

No upstream filing or draft is appropriate from this evidence.
No upstream message was sent.
