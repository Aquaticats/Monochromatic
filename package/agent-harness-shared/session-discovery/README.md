# agent-harness-shared-session-discovery

Shared process-tree and PID-directory session discovery for agent harness
spawners.

This package lives under `package/agent-harness-shared/` because Pi spawn and
Claude Code spawn both resolve the calling parent session with the same deep
mechanism:
walk parent PIDs,
look up `.by-pid/<pid>` mapping files,
then fall back to the most recently written mapping when sandboxed process trees
hide the host PID namespace.

## Pi dependency trust

PJD:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Interface

Hosts provide the shallow adapter details:

- `byPidDir`,
   resolved from host-specific paths or environment
- `parseMapping`,
   owned by the host because mapping payloads differ
- `startPid`,
   usually `process.ppid`
- optional `io`,
   used by tests to fake procfs and mapping files

The package owns the process-tree walk,
procfs parent-PID parsing,
PID mapping reads,
newest-file fallback,
and tree-then-fallback composition.

## Usage

```ts
import {
  findCallingSession,
  SESSION_NOT_FOUND,
} from '@monochromatic-dev/agent-harness-shared-session-discovery/ts';

const mapping = await findCallingSession({
  byPidDir: '/tmp/spawn-results/.by-pid',
  parseMapping: (raw) => JSON.parse(raw),
  startPid: process.ppid,
});

if (mapping === SESSION_NOT_FOUND)
  throw new Error('No calling session found');
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/agent-harness-shared/session-discovery:test:unit
mise run //package/agent-harness-shared/session-discovery:lint
```
