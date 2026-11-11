# Plasma System Monitor 6.8.2: missing temporary directory hides pages, and sensor startup can fail

## Incident observed at 2026-11-11T06:25:23-05:00

The installed packages were `plasma-systemmonitor-6.8.2-1.fc45.x86_64`,
`libksysguard-6.8.2-1.fc45.x86_64`,
and `ksystemstats-6.8.2-1.fc45.x86_64` on Bazzite `testing-45.20261105`.
Keep the following failures separate:

- The initial window had no page navigation or content.
  All four pages failed to load.
- After restarting with a clean environment,
  navigation and saved layouts returned,
  but their sensors were unavailable.
  Starting the sensor service and restarting the application restored live readings.

## Symptom

`plasma-systemmonitor` logged this diagnostic for each page:

```text
# User journal, original application PID 1931207
Failed to load data from "/usr/share/plasma-systemmonitor/history.page"
```

The same message named the personal `overview.page`,
`processes.page`,
and `applications.page` files.
Those files were present and readable.

The subsequent failure displayed:

```text
# Overview warning supplied in the user's screenshot
This page is missing some sensors and will not display correctly.
```

## Root cause of missing pages

The original application's environment contained
`TMPDIR=/home/user/temp/agent/kimi133-e2e/tmp`.
That directory did not exist.
The current user service manager no longer advertised `TMPDIR`,
but restarting the application was necessary to discard its inherited value.
The investigation did not establish which earlier operation introduced it.

In Plasma System Monitor tag `v6.8.2`,
commit `e4cd4bcdfe56eb67c08bcf42b5af708f9d96a8c2`,
[`src/page/PageController.cpp:121`][temporary-file]
creates a temporary configuration file before loading page contents:

```cpp
// src/page/PageController.cpp:121
m_temporaryConfigFile = std::make_unique<QTemporaryFile>();
if (!m_temporaryConfigFile->open()) {
    return false;
}
```

[`src/page/PageManager.cpp:189`][skip-page]
then excludes that page:

```cpp
// src/page/PageManager.cpp:189
if (!controller->load()) {
    qWarning() << "Failed to load data from" << controller->m_path.string();
    continue;
}
```

## Separate sensor activation failure

After the page-loading repair,
`plasma-ksystemstats.service` was inactive and its D-Bus name was only activatable.
Activating `org.kde.ksystemstats1` and restarting System Monitor restored the sensor subscription.

[`src/main.cpp:111`][activation-name]
in the same release requests a different service name:

```cpp
// src/main.cpp:111
busInterface->asyncCall(QStringLiteral("StartServiceByName"), QStringLiteral("org.kde.ksystemstats"), 0u);
```

The installed `/usr/share/dbus-1/services/org.kde.ksystemstats1.service` instead declares:

```ini
# /usr/share/dbus-1/services/org.kde.ksystemstats1.service
[D-BUS Service]
Name=org.kde.ksystemstats1
Exec=/usr/bin/ksystemstats
SystemdService=plasma-ksystemstats.service
```

A real `org.freedesktop.DBus.StartServiceByName` call with `org.kde.ksystemstats`
returned `org.freedesktop.DBus.Error.ServiceUnknown: The name is not activatable`.
The call using `org.kde.ksystemstats1` successfully activated the installed service.
The wrong explicit activation name is established;
the complete reason automatic sensor-client activation failed was not traced.

## Verification

A disposable copy of the personal configuration and page files was tested with the installed binary,
a separate `dbus-run-session`,
`QT_QPA_PLATFORM=offscreen`,
and `strace --follow-forks --trace=openat`.
The fixture used private directories under `${HOME}/temp/agent`.
Only the `TMPDIR` destination changed between the compared launches.

- Missing temporary directory:
  all four temporary-file opens returned `ENOENT`,
  and the journal recorded all four page-loading failures.
- Existing temporary directory:
  the corresponding temporary-file opens succeeded,
  and the application reached sensor-service activation.
- Repaired desktop session:
  the Overview screenshot contained the saved memory,
  CPU-thread,
  filesystem,
  temperature,
  and GPU charts without the missing-sensors banner.
  Separate D-Bus samples returned CPU usage of `34.568651275820173` and `28.711656441717793`,
  with changing memory values.
- Applications and Processes:
  verified screenshots showed populated lists and usage values.
- History:
  an accessibility-tree traversal reached all `277` nodes without truncation.
  It included the `History` page title,
  total CPU usage of `46.3%`,
  per-core readings,
  GPU usage of `22.0%`,
  physical memory of `23.6 GiB`,
  and download/upload values of `75.5 KiB/s` and `19.7 KiB/s`.
  Captures containing another application were rejected as evidence.

The scratch reproduction is `${HOME}/temp/agent/probe-systemmonitor-temp.ts`.
Its raw evidence is `${HOME}/temp/agent/systemmonitor-probe-ayBHK0`.
These local artifacts are not a portable regression suite.
The harness's stderr failure count is not a verdict:
Qt emitted these diagnostics to the journal,
so the syscall trace and journal establish the failure.

A read-only sensor query used during verification was:

```sh
# Query the installed sensor service while System Monitor is open
gdbus call --session \
  --dest org.kde.ksystemstats1 \
  --object-path /org/kde/ksystemstats1 \
  --method org.kde.ksystemstats1.sensorData \
  '["cpu/all/usage", "memory/physical/application", "memory/physical/used"]'
```

## Verified workarounds

Restarting System Monitor from the clean user service manager restored the pages.
No page reset or package replacement was needed.
The original configuration and page files were backed up to
`${HOME}/temp/agent/systemmonitor-backup-h1PVg4`.
A recursive comparison confirmed the saved page files were unchanged.
Normal application navigation changed `lastVisitedPage` in `systemmonitorrc`.

The local sensor-service drop-in is:

```ini
# ~/.config/systemd/user/plasma-ksystemstats.service.d/keep-sensors-available.conf
[Service]
ExecStart=
ExecStart=/usr/bin/ksystemstats --remain
```

`ksystemstats --help` documents `--remain` as
`Do not quit when last client has disconnected`.
`systemctl --user add-wants graphical-session.target plasma-ksystemstats.service`
added the service to desktop-session startup.
After a manager reload and service restart,
`systemctl --user show` confirmed the active command includes `--remain`.
The package's existing `PartOf=graphical-session.target` retains desktop-session shutdown ownership.
The same sensor-service PID remained active after closing System Monitor,
and reopening the application with its Applications page produced a populated list.

Tradeoff:
the statistics service stays resident while the graphical session exists,
including when System Monitor is closed.
Remove this local override and startup link after verifying reliable cold activation in a fixed package.
Do not modify the distribution-owned service or `.page` files.

## What does not work

- Resetting layouts does not repair a missing `TMPDIR` destination.
  Layout reset was not attempted because readable personal and system pages failed identically.
- Checking only page-load diagnostics missed the separate sensor failure.
  The user's follow-up screenshot exposed the missing-sensors banner.
- Active-window capture was not reliable while the person using the desktop changed focus.
  Application activation and KWin script completion did not guarantee the captured window's identity.
  Overview,
  Applications,
  and Processes captures were checked by title and content;
  History was verified through its rendered accessibility tree.
- The initial source-verification browser stalled.
  Its enclosing cgroup had `TasksMax=128` and recorded `54` denied task creations.
  Relaunching the owned browser in a fresh scope with `TasksMax=512` completed the source checks.
  This is evidence about the diagnostic command's limit,
  not a cause of the System Monitor failures.

All source links were opened in `agent-browser` and checked for the quoted text and expected line-selection fragment.
The committed Markdown was also checked in GitHub's rendered view.
The scoped `lint:monochromatic-lint` task passed.

## Verification rule proposal

Tighten `AGENTS.md` rule `SCF` to state:
before accepting a screenshot,
verify the target application/window and intended rendered state;
recapture mismatches or transitional frames.
This clarification was proposed to the owner;
no agent-rule edit is part of the system repair.

## Upstream filing decision

- Fault:
  the deleted temporary directory was local environment contamination.
  The explicit sensor activation name differs from the installed service name.
- Fixability:
  correcting the activation name is a concrete candidate,
  but the entire cold-start path has not been verified with that change.
- Supported use:
  the application ships an explicit sensor-service activation call.
- Contribution policy:
  not assessed for an upstream submission during this local repair.
- Maintainer disposition:
  not assessed;
  no rejection or acceptance is inferred.
- Prototype:
  the user-service workaround was exercised locally;
  no upstream source patch was built.

No upstream message was sent and no filing-ready artifact is claimed.
The upstream tracker,
contribution policies,
and other maintained release lines were not audited.
This document records a local recovery,
not a recommendation to file an issue.

[temporary-file]: https://github.com/KDE/plasma-systemmonitor/blob/e4cd4bcdfe56eb67c08bcf42b5af708f9d96a8c2/src/page/PageController.cpp#L100-L125
[skip-page]: https://github.com/KDE/plasma-systemmonitor/blob/e4cd4bcdfe56eb67c08bcf42b5af708f9d96a8c2/src/page/PageManager.cpp#L189-L194
[activation-name]: https://github.com/KDE/plasma-systemmonitor/blob/e4cd4bcdfe56eb67c08bcf42b5af708f9d96a8c2/src/main.cpp#L109-L112
