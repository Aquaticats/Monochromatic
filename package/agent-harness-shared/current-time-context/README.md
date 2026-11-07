# current-time-context shared formatter

Formatter for hidden current local wall-clock time context payloads.

This package lives under `package/agent-harness-shared/` because it is shared
by multiple agent harness integrations.
It is currently published as `@monochromatic-dev/agent-harness-shared-current-time-context`;
the path move is separate from any package-name migration.

The package exports `formatTimeContext()`,
 which turns a `Date` into:

```text
<time>HH:MM</time>
```

## Pi dependency trust

PJB:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Why this exists

Claude Code hooks and pi extensions both inject the same coarse local time context.
This package keeps the pure formatting rule in one shared agent-harness package,
 separate from either host runtime.

## Behavior

`formatTimeContext()` uses local 24-hour clock fields from the supplied `Date`.
It zero-pads hour and minute and omits seconds,
 date,
 and timezone.

## Usage

```ts
import {
  formatTimeContext,
} from '@monochromatic-dev/agent-harness-shared-current-time-context';

formatTimeContext(new Date(2_026, 4, 1, 20, 48,),);
// '<time>20:48</time>'
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/agent-harness-shared/current-time-context:test:unit
mise run //package/agent-harness-shared/current-time-context:lint
```
