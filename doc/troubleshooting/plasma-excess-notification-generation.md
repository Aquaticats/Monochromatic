# notify-send exits 1 on KDE Plasma 6.8.2 when two identical notifications arrive within a second

`notify-send` 0.8.9 against Plasma 6.8.2 exits 1 and prints
`Failed to show notification: GDBus.Error:org.freedesktop.Notifications.Error.ExcessNotificationGeneration: Created too many similar notifications in quick succession`
whenever a caller sends the same notification twice less than one second apart,
so any program that repeats one fixed payload fails on every settlement after the first.

## Symptom

A program that calls the same `notify-send` command repeatedly reports:

```text
Failed to show notification: GDBus.Error:org.freedesktop.Notifications.Error.ExcessNotificationGeneration: Created too many similar notifications in quick succession
```

and exits 1,
with no notification shown.
The first call in a burst succeeds.
`notify-send --version` reports 0.8.9,
and the server answers `GetServerInformation` with `Plasma`,
`KDE`,
`6.8.2`,
`1.2`.

This is what agent runs under `${HOME}/temp/agent/pi-runs` logged:
287 of the 1 513 `stderr.log` files there carry the message,
counted on 2026-11-12.
It is also what the settled-agent notification extension in
`package/pi-plugin/agent-settled-notification` hit on every settlement after the
first inside a burst,
because it passed `--app-name=Pi 'Pi agent finished' 'Agent is idle and ready for input.'`
and discarded the subprocess standard error.

Variants:

- Two identical notifications less than 1000 ms apart:
  refused,
  exit 1.
- The same two more than 1000 ms apart:
  both accepted,
  exit 0.
- Two notifications differing only in the body:
  both accepted,
  exit 0.
- No session bus at all:
  exit 1 with
  `Failed to show notification: Could not connect: No such file or directory`,
  a different variant that this document does not cover.

## Root cause

Walked from the caller to the server reply.

1. `notify-send` builds one `Notify` call and waits for its reply synchronously.
   `tools/notify-send.c:533` in libnotify 0.8.9 calls
   `retval = notify_notification_show (notify, &error);`.
2. That call is a blocking D-Bus call with no client timeout of its own:
   `libnotify/notification.c:1248` sends `Notify` through `g_dbus_proxy_call_sync`,
   and its timeout argument is `-1 /* FIXME ? */` at
   `libnotify/notification.c:1258`,
   which GLib reads as `G_DBUS_DEFAULT_TIMEOUT_MSEC` (25 000 ms).
   A desktop that never answers therefore holds `notify-send` for 25 s,
   which is the reason a caller needs its own bound rather than relying on this one.
3. Plasma's notification server compares each new request against the previous one:

   ```cpp
   // If multiple identical notifications are sent in quick succession, refuse the request
   if (m_lastNotification.applicationName() == notification.applicationName() && m_lastNotification.summary() == notification.summary()
       && m_lastNotification.body() == notification.body() && m_lastNotification.desktopEntry() == notification.desktopEntry()
       && m_lastNotification.eventId() == notification.eventId() && m_lastNotification.actionNames() == notification.actionNames()
       && m_lastNotification.icon() == notification.icon() && m_lastNotification.urls() == notification.urls()
       && m_lastNotification.created().msecsTo(notification.created()) < 1000) {
       qCDebug(NOTIFICATIONMANAGER) << "Discarding excess notification creation request";

       sendErrorReply(QStringLiteral("org.freedesktop.Notifications.Error.ExcessNotificationGeneration"),
                      QStringLiteral("Created too many similar notifications in quick succession"));
       return 0;
   }
   ```

   The block is `libnotificationmanager/server_p.cpp:220-231` in
   plasma-workspace 6.8.2,
   [commit `30beb0095873`](https://github.com/KDE/plasma-workspace/blob/30beb0095873364c6b34ebcfd292ecafd4db7f61/libnotificationmanager/server_p.cpp#L220-L231).
   Eight fields must match and the gap must be under 1000 ms.
   `m_lastNotification` is the last notification the server accepted or refused,
   not the last one from the same application,
   so an unrelated notification in between breaks the comparison and the next
   identical request is accepted.
4. The D-Bus error reply makes `notify_notification_show` return `FALSE`,
   so `tools/notify-send.c:538` builds
   `msg = g_strdup_printf ("Failed to show notification: %s", error->message)`,
   prints it to standard error,
   and `tools/notify-send.c:573` runs `if (hint_error || show_error) exit (1);`.

Earlier readings that were wrong,
recorded so they are not re-derived:

- "The one-second caller timeout kills `notify-send`."
  Wrong:
  the refusal arrived in 8 to 133 ms,
  and a killed child is reported by its calling harness as a signal,
  not as exit 1.
- "The subagent driver strips a variable `notify-send` needs."
  Wrong:
  the same refusal appeared with `DBUS_SESSION_BUS_ADDRESS` removed,
  and the accepted calls ran with the full environment.

## Verification

Versions under test:

- libnotify 0.8.9 (`/usr/bin/notify-send`,
  Arch `libnotify` package),
  commit `9b53a46b0307` ("Releasing version 0.8.9"),
  [source](https://github.com/GNOME/libnotify/blob/9b53a46b0307fe3a7ddd2545de1e395ee2f94d18/tools/notify-send.c#L533-L545).
- plasma-workspace 6.8.2 notification server,
  commit `30beb0095873`,
  [source](https://github.com/KDE/plasma-workspace/blob/30beb0095873364c6b34ebcfd292ecafd4db7f61/libnotificationmanager/server_p.cpp#L220-L231).

Runnable harness,
in one Node script that spawns the real binary and records exit
code,
duration and standard error:

```ts
const child = spawn('notify-send', ['--app-name=Pi', 'Probe title', 'Probe body',], {
  stdio: ['ignore', 'ignore', 'pipe',],
},);
```

Catalog:
patterns that work,
measured at load average 81 to 96 on this host.

- Payloads differing in the body,
  20 sends:
  exit 0,
  durations 20,
  24,
  52,
  81,
  89,
  96,
  107,
  107,
  120,
  137,
  141,
  178,
  198,
  296,
  305,
  347,
  515,
  632,
  792 and 3 060 ms.
- The same payload sent again after more than 1000 ms:
  exit 0.

Catalog:
patterns that fail.

- The same payload sent twice within 1000 ms:
  the second exits 1 in 8 to 133 ms
  with `ExcessNotificationGeneration`,
  quoted above.
- Retrying the identical payload every 100 ms after a refusal:
  nine refusals,
  accepted on the tenth attempt 1 183 ms after the first accepted send,
  which is the 1000 ms window plus the attempts' own durations.

## Verified workarounds

1. Make the payload identify the occurrence.
   Sending the settled agent's working folder as the body makes two agents'
   notifications differ,
   so the comparison fails and the server accepts both.
   Verified:
   two concurrent sends differing only in the body were both accepted in
   328 ms and 373 ms.
   Tradeoff:
   the body reveals a local path to the desktop,
   and two settlements of
   the same agent in the same folder still collide.
2. Retry a refusal after more than the 1000 ms window.
   Verified:
   a second identical send was refused,
   then accepted 1 916 ms after it
   started with a 1 500 ms wait between attempts.
   Tradeoff:
   the caller must bound the whole retry sequence,
   or a permanently refusing desktop keeps it alive.
   Retrying sooner than the window just earns another refusal.
3. Do nothing about the payload and accept that repeated settlements are silent.
   Tradeoff:
   the first notification of a burst is shown and the rest are lost,
   which is what the extension did before this work.

## What does not work

- Ignoring the exit code or the standard error.
  The notification is not shown either way,
  and a caller that discards standard error cannot tell this refusal from a
  missing session bus.
- Raising the caller's timeout.
  The refusal is a fast error reply (8 to 133 ms),
  never a slow answer,
  so a longer bound changes nothing about it.
- Sending the same payload from a differently named application.
  The application name is one of the eight compared fields,
  so this does evade the refusal,
  but it makes every Pi notification claim to come from another program.
- Using `--wait`.
  `--wait` keeps `notify-send` alive until the notification closes
  (`tools/notify-send.c:528` arms `on_wait_timeout` at the expiry time),
  it does not change the server's comparison,
  and it holds the caller for the whole popup instead of one reply.

## Upstream filing decision

Nothing is filed,
and nothing should be.

`.out-of-scope/` was checked:
no exemption covers KDE,
Plasma,
notifications or D-Bus,
so filing was decided on the six constraints alone.

1. Is it really upstream's fault?
   No.
   The server's comment says the comparison exists to discard excess creations,
   so the refusal is the feature working,
   and the fault is a caller that repeats one payload.
2. Can upstream fix it?
   Not without dropping the de-duplication they wrote deliberately.
3. Are they supporting this use case?
   Partly:
   repeated identical notifications are what the guard targets.
4. Would the repo welcome our contribution?
   Not applicable:
   Plasma bugs live on bugs.kde.org rather than GitHub issues,
   and the change we would want is a policy change,
   not a defect fix.
   A GitHub search for the message found no KDE report to comment on
   (`gh search issues "ExcessNotificationGeneration"`,
   15 results,
   none in
   KDE/plasma-workspace or KDE/plasma-desktop).
5. Will they likely fix it?
   No signal either way,
   and bugs.kde.org was not queried because this session has
   no account there.
6. Minimal prototype compatible with their architecture?
   Not attempted,
   because constraints 1 and 5 already say the change belongs at the
   caller:
   the guard is intentional,
   and a consumer-side distinct payload solves it.

Filing is the poster's decision,
and the risk is theirs:
a report arguing against a deliberate guard reads as a low-effort filing and can
follow the poster's account across projects.

## Citation check

Every source link above was fetched and matched against the quoted lines.
`agent-browser` could not confirm them because it cannot reach the network from
this host:
both `https://invent.kde.org/...` and `https://github.com/...` failed
with `client error (Connect): tcp connect error: Software caused connection abort
(os error 103)`.
The same URLs were fetched with the repository's `web_fetch` tool instead,
and the cited line ranges were confirmed against the file contents at those
commits.
