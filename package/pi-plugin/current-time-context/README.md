# pi-current-time-context

Hidden current local time context extension for pi.

The extension adds one hidden custom message before every agent turn:

```text
<time>HH:MM</time>
```

## Pi dependency trust

PJW:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Installation

Install the extension into pi from the repository root:

```sh
pi install ./package/pi-plugin/current-time-context
```

## Quick test

Load the TypeScript entry point directly while developing:

```sh
pi -e ./package/pi-plugin/current-time-context/src/index.ts
```

A normal prompt then receives hidden context for the current local hour and minute.

## Behavior

The extension subscribes to pi's `before_agent_start` event.
For each agent turn,
 it returns a hidden custom message with `customType: "current-time-context"`.

`display: false` keeps the message out of the visible transcript UI.
Pi still includes hidden custom messages in LLM context,
 so the model can use local wall-clock time.

Local wall-clock time is used because the agent answers in the user's active working context.
The payload includes only hour and minute,
 with no seconds,
 date,
 or timezone.
That keeps the context coarse and avoids unrelated temporal detail.

## Validation

Run package validation from the repository root:

```sh
mise run //package/pi-plugin/current-time-context:build
mise run //package/pi-plugin/current-time-context:test:unit
mise run //package/pi-plugin/current-time-context:lint
mise run //package/pi-plugin/current-time-context:verify:extension
```
