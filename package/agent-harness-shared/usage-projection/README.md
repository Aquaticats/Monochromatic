# agent-harness-shared-usage-projection

Shared rate-limit usage projection and statusline segment formatting for agent harness integrations.

This package owns the host-neutral policy for usage warnings:

- render when remaining capacity is at or below 50 percent,
  or when projected end-of-window usage exceeds 100 percent
- use green above 25 percent remaining,
  yellow from 10 to 25 percent remaining,
  and red at 10 percent remaining or below
- force red for any projected overrun
- compute burn-rate projection from each snapshot's `sampledAtMs`
  while formatting reset text from the caller's `renderedAtMs`

Host packages keep their protocol readers and color systems.
They convert host data into `RateLimitSnapshot` values and pass host style callbacks to the shared formatter.

## Pi dependency trust

PJI:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Usage

```ts
import {
  PLAIN_RATE_LIMIT_STYLE,
  formatRateLimitStatus,
} from '@monochromatic-dev/agent-harness-shared-usage-projection/ts';

const status = formatRateLimitStatus({
  snapshots: [
    {
      key: 'demo',
      label: 'demo',
      resetAtMs: Date.now() + 60_000,
      windowSeconds: 300,
      paceScale: 1,
      sampledAtMs: Date.now(),
      usedPercent: 60,
    },
  ],
  renderedAtMs: Date.now(),
  style: PLAIN_RATE_LIMIT_STYLE,
});
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/agent-harness-shared/usage-projection:lint
mise run //package/agent-harness-shared/usage-projection:test:unit
mise run //package/agent-harness-shared/usage-projection:build
```
