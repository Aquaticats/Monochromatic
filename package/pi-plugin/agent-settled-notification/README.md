# pi-agent-settled-notification

Pi extension that sends a Freedesktop desktop notification when an agent becomes idle.

## Pi dependency trust

PJP:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

This package runs `notify-send` through `node:child_process`,
and depends on no third-party process library.

## Behavior

The extension subscribes only to Pi's `agent_settled` event.
This event fires after automatic retries,
compaction recovery,
and queued follow-up work are complete.
It also fires after provider errors and user aborts.

Each settlement sends one notification:

- Application name `Pi`,
  title `Pi agent finished`.
- Body `Idle in <working folder>`,
  naming which agent finished,
  because many agents run at once.
  Pi reports that folder as `ExtensionContext.cwd`.
  A host that reports no folder gets the plain body
  `Agent is idle and ready for input.`

### Why the body names the agent

KDE Plasma's notification server refuses a notification whose application name,
summary,
body,
desktop entry,
event id,
action names,
icon and URLs all match the previous one sent less than 1000 ms earlier:

```text
Failed to show notification: GDBus.Error:org.freedesktop.Notifications.Error.ExcessNotificationGeneration: Created too many similar notifications in quick succession
```

That refusal is what made every `notify-send` call of an earlier version exit 1:
its payload was identical every time,
and settlements arrive in bursts.
The rule is `libnotificationmanager/server_p.cpp:220-231` in plasma-workspace 6.8.2.
Naming the agent keeps consecutive settlements distinct,
and the retry below covers two settlements from the same folder.

### Bounds and retries

Nothing here runs on Pi's settlement path:
the `agent_settled` hook starts a delivery and returns `undefined`,
so an agent never waits for a desktop.

A delivery is bounded in three ways,
each sized from measurement on this host at load average 81 to 96,
where twenty distinct real notifications took 20 ms to 3 060 ms:

- One attempt is killed after `NOTIFICATION_ATTEMPT_TIMEOUT_MS` (6 000 ms).
  That is about twice the worst measured attempt,
  and far below the 25 s GLib method-call default that `notify-send` itself
  would wait for.
- A failed attempt is retried after `NOTIFICATION_DELAYS_BEFORE_RETRY_MS`
  (first 1 500 ms,
  then 3 000 ms).
  The first wait is longer than the 1 183 ms the server was measured to keep
  refusing one repeated payload.
- The whole delivery,
  waits included,
  stays inside `NOTIFICATION_TOTAL_BOUND_MS` (15 000 ms).
  A retry gets only what that bound leaves,
  and no delivery waits indefinitely.

Retries cover a process that ran and was refused.
A process that could not be started at all,
such as a missing `notify-send`,
is not retried,
because waiting cannot produce an executable.

### Failures are logged, never raised

A failed delivery is logged with the executable's own standard error,
bounded to 4 KiB:

```text
settled-agent desktop notification unavailable: desktop notification through notify-send failed after 3 attempts: exited with code 1; stderr: Failed to show notification: GDBus.Error:...ExcessNotificationGeneration: ...
```

A desktop refusal is warned about once per Pi runtime,
and then logged at debug level,
so an unavailable desktop does not warn on every settlement.
A failure this package cannot describe is logged at error level,
rather than escaping as an unhandled rejection.

## Installation

Add the package to global Pi settings:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/agent-settled-notification"
  ]
}
```

Restart Pi or run `/reload` after updating the setting.

## Development

Run package validation from the repository root:

```sh
mise run //package/pi-plugin/agent-settled-notification:build
mise run //package/pi-plugin/agent-settled-notification:lint
mise run //package/pi-plugin/agent-settled-notification:test:unit
mise run //package/pi-plugin/agent-settled-notification:verify:extension
mise run //package/pi-plugin/agent-settled-notification:verify:pi-runtime
```

`verify:pi-runtime` loads the built package through Pi's extension discovery API,
emits its registered `agent_settled` handler,
and waits a bounded grace for the background delivery,
so a D-Bus monitor sees it without a model request:

```sh
# Terminal 1
dbus-monitor --session "interface='org.freedesktop.Notifications',member='Notify'"

# Terminal 2
mise run //package/pi-plugin/agent-settled-notification:verify:pi-runtime
```

## Files

- `src/index.ts`:
  package entry,
  which only re-exports.
- `src/settled-notification-registration.ts`:
  the `agent_settled` hook,
  the log lines it writes,
  and the tracker a verifier waits on.
- `src/notification-payload.ts`:
  the command,
  title and body of one notification.
- `src/notification-process.ts`:
  the bounded,
  stderr-capturing subprocess boundary.
- `src/settled-notification-delivery.ts`:
  the measured bounds and the retry policy.
