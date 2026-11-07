# pi-agent-settled-notification

Pi extension that sends a Freedesktop desktop notification when an agent becomes idle.

## Pi dependency trust

PJP:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Behavior

The extension subscribes only to Pi's `agent_settled` event.
This event fires after automatic retries,
compaction recovery,
and queued follow-up work are complete.
It also fires after provider errors and user aborts.

The extension calls `notify-send` with a static title and body.
Its notification subprocess is limited to one second,
so Pi can wait briefly for a delivery result without an unbounded stall.
A missing desktop session,
D-Bus server,
or executable is logged once per Pi runtime and never fails the agent run.

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
then emits its registered `agent_settled` handler.
Use a D-Bus monitor alongside this task to verify desktop delivery without requiring a model request:

```sh
# Terminal 1
dbus-monitor --session "interface='org.freedesktop.Notifications',member='Notify'"

# Terminal 2
mise run //package/pi-plugin/agent-settled-notification:verify:pi-runtime
```
