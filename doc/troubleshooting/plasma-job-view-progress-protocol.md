# Plasma 6.7.5 ignores the freedesktop `value` hint, so progress must be a job view (`org.kde.kuiserver`)

The music player reports true-peak analysis progress outside its own window (D107 in
`package/music-player/design/decisions.md`).
The first plan was one freedesktop notification with the `value` hint, which GNOME Shell, dunst and similar servers draw
as a bar.
On KDE the same call shows a notification with no bar.
This doc records why, the job view protocol that does draw one (the widget Dolphin's "Compress to .tar" uses), and the
wire details that fail silently when wrong.

## Symptom

A `Notify` call carrying `value` (int32 percent) shows the title and body in Plasma and no bar, however often it is
replaced.
Wire mistakes on the job view path fail without an error message:

- `requestView` with an unsigned capabilities argument: the signature becomes `sua{sv}` and Qt finds no such method.
- `update` or `terminate` sent as ordinary calls: Qt never replies to them, so a blocking caller stalls until its reply
  timeout.
- Pressing Pause does nothing visible: the button stays "Pause" until the app reports `suspended`.
- A finished job stays in the notification history.
- A development run without an installed `.desktop` file shows a bare process name in the job header.

## Root cause

Sources: `KDE/plasma-workspace` tag `v6.7.5` (commit `cc1ab7963f94a9b76836412855caed6b94066188`), the version installed
on the host (`rpm -q plasma-workspace` gives `plasma-workspace-6.7.5-1.fc44`).
The D-Bus XML files, `jobsmodel_p.cpp` and `notification.cpp` are identical on `master` at commit
`424374ebdc132a1e4a01920e634d153665712799` (2026-10-11);
`job_p.cpp` differs there only by speed-chart sampling.

The notification server never reads `value`.
`libnotificationmanager/notification.cpp:393` to `:438` (`processFdoHints`) reads exactly these freedesktop hints:

```cpp
// libnotificationmanager/notification.cpp:393
setDesktopEntry(hints.value(QStringLiteral("desktop-entry")).toString());
// :407
const int urgency = hints.value(QStringLiteral("urgency")).toInt(&ok); // DBus type is actually "byte"
// :423
resident = hints.value(QStringLiteral("resident")).toBool();
// :424
transient = hints.value(QStringLiteral("transient")).toBool();
// :438
category = hints.value(QStringLiteral("category")).toString();
```

A search for the `value` hint in `libnotificationmanager` and `applets/notifications` finds nothing.
The only `ProgressBar` in the applet is in the job widget (`applets/notifications/components/SpeedChart.qml:241`).
The specification does not list `value` as a hint either, and its categories are `transfer`, `transfer.complete` and
`transfer.error`, so there is no `transfer.progress`.
A 2019 post by a Plasma notification developer says progress goes through `org.kde.kuiserver` and
`org.kde.JobViewServer`: <https://blog.broulik.de/2019/05/next-generation-plasma-notifications/>.

The job view path, and where each silent failure comes from:

```xml
<!-- libnotificationmanager/dbus/org.kde.JobViewServerV2.xml:13 -->
<arg name="capabilities" type="i" direction="in"/>
<!-- libnotificationmanager/dbus/org.kde.JobViewV3.xml:18 and :27, on terminate and update -->
<annotation name="org.freedesktop.DBus.Method.NoReply" value="true"/>
```

```cpp
// libnotificationmanager/jobsmodel_p.cpp:290
job->setSuspendable(capabilities & KJob::Suspendable);
job->setKillable(capabilities & KJob::Killable);
// :315 and :324
if (hints.value(QStringLiteral("immediate")).toBool()) { ... }  // shown after 50 ms, not the default 500 ms
if (hints.value(QStringLiteral("transient")).toBool()) { job->setTransient(true); }
// :253 and :254, the header name and icon before the KService lookup at :258
QString applicationName = hints.value(QStringLiteral("application-display-name")).toString();
QString applicationIconName = hints.value(QStringLiteral("application-icon-name")).toString();
```

```cpp
// libnotificationmanager/job_p.cpp:304, in JobPrivate::finish()
if (m_error == KIO::ERR_USER_CANCELED || (!m_error && m_transient)) { Q_EMIT closed(); return; }
// :329, the only place the paused state changes; reached from the "suspended" property at :508
void JobPrivate::setSuspended(bool suspended) { ... job->setState(Notifications::JobStateSuspended); ... }
```

```qml
// applets/notifications/components/JobItem.qml:69 and :71
readonly property bool paused: jobItem.modelInterface.jobState === NotificationManager.Notifications.JobStateSuspended
visible: jobItem.modelInterface.suspendable
```

So: the capabilities bit field is a signed integer (Killable 1, Suspendable 2), `update` and `terminate` are `NoReply`,
Plasma flips Pause to Resume only after the app sends `update({suspended: b})`, a `transient` job ended with code 0 is
removed without notice, and the header falls back to the process name when neither the desktop file nor the two
`application-*` hints name the app.
The property names `update` reads are `title`, `infoMessage`, `percent` (unsigned), `suspended`, `destUrl`, `speed`,
`processedBytes` and the `description*` and `total*` fields;
the `descriptionField` key that the human's `compress-to-sqfs` script sends is not among them.

A previous reading in this investigation expected the `value` hint to work on Plasma because several other servers draw
it; the grep above disproved it.

## Verification

Versions: `plasma-workspace` v6.7.5 sources as above; `zbus` 5.19.0; private `dbus-daemon` from the Fedora 44 host.

Harness: a scratch crate compiles the new player modules in place and runs them against mock
`org.kde.JobViewServerV2`, `org.kde.JobViewV3` and `org.freedesktop.Notifications` services on a private bus.
The mock records the `NoReplyExpected` flag of every `update` and `terminate`, and a probe test proves it can tell a
normal call from a no-reply one.
The bus uses `--config-file` with a listen address, EXTERNAL authentication, an allow-all policy and no service
directories, because `dbus-daemon --session` would load the standard service directories and a call to
`org.freedesktop.Notifications` could start the host's real daemon.
The player probes `org.kde.kuiserver` with `NameHasOwner` so the probe cannot activate anything.

Patterns that work (mock bus, all pass):

- `requestView("monochromatic.music-player", 2, {application-display-name, application-icon-name, transient})`, then
  `update({title, infoMessage, percent, suspended})`, later updates without `title`, `suspendRequested` answered by
  `update({suspended: true})`, and `terminate(0, "", {})`.
- One `Notify` replaced by the returned id, with `value` (int32), `urgency` (byte 0), `suppress-sound`, `category`
  `transfer`, `desktop-entry`, `resident`, and `expire_timeout` 0.

Patterns that fail on the real server, from the source above and not yet seen on a live Plasma session:

- `value` hint on Plasma: no bar.
- Capabilities sent as `u`: no method match.
- `update` or `terminate` as ordinary calls: the caller waits for a reply that Qt never sends.

## Closing the connection logs a zbus warning

Closing the blocking `zbus` connection (`Connection::close`) while a signal listener still holds a message stream makes
`zbus::message_stream` log `WARN Failed to remove match rule: I/O error: Broken pipe` as the stream drops.
The player therefore never closes the connection on shutdown; the process exit ends the listener threads.
This was seen in a real shutdown of the release app hosted in the nested compositor.

## Verified workarounds

The player (`package/music-player/desktop-app/src/analysis_notification*.rs`) picks the Plasma job when
`org.kde.kuiserver` has an owner and falls back to the freedesktop notification otherwise.
Tradeoffs: the job needs the Plasma shell, so GNOME gets the notification and its bar depends on the server;
the job is suspendable only when the app can actually pause the analysis, because a Pause button that does nothing is
worse than none.
The `transient` hint means a failed job is also not kept in the history, but the failure message is still shown by
`terminate(2, message, {})` while the job exists.

## What does not work

- `value` on Plasma's own notification server (above).
- The `descriptionField` key in the V3 `update` (not read).
- Relying on Plasma to flip the Pause button (it reports nothing until the app sends `suspended`).
- Using the host session bus for tests (it has the real services).
- Blocking waits on `update` and `terminate`: both are `NoReply`.

## Upstream filing decision

Nothing is filed.
Constraint 1: the behavior is a design choice (progress is a job), not a defect.
Constraint 3: Plasma supports progress through the job view and documents it only in source and a developer blog,
not through the freedesktop hint.
Constraints 2, 4, 5 and 6 are moot without a defect.
`.out-of-scope/` was not consulted because no filing is proposed.
A search for a Plasma bug about `value` on `bugs.kde.org` could not be run from this environment (the web search tool
cannot query that site), so no duplicate check was done;
nothing is drafted that could duplicate one.
