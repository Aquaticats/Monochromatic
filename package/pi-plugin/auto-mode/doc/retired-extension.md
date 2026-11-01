# Retired extension pipeline

## Why it was removed

The installed command uses the native complete-group host.
The old extension entry separately registered a signal flagger,
judge selection,
trust directives,
reusable approvals,
and tool-event evaluation.
Retaining that pipeline created another execution-permission implementation with different inputs and outcomes.

The 2026-10-17 source graph finds 33 modules reachable only from that old entry
and absent from the native policy entry's dependency graph.
The refactor removes the exclusive implementation,
its public exports,
its exclusive characterization tests,
and package metadata advertising it as an extension.
`src/index.ts` remains a named-export entry for maintained code and built-artifact tests.
The installed native command remains the host entry.

## Retained responsibilities

Native confirmation,
visible audit history,
explicit bypass,
and the fixed virtual-input guard remain active.
Shared shell analysis is imported directly from the existing analyzer package.
The package no longer needs a pass-through command-analysis wrapper.

The old heuristic command and secret-pattern catalogs are removed.
Their absence does not remove native permission review:
the native host still reviews complete prepared groups against selected instructions and authorization.
It also retains the explicitly fixed virtual-input protection.

## Alternatives and evidence

Keeping both pipelines would require maintaining separate authorization semantics and duplicate tests.
Moving the old implementation to another runtime package would preserve that maintenance burden.
Deleting the native implementation would remove the installed user interaction being repaired.
The retained direction is one installed native execution path with common confirmation and projection helpers.

The private source graph records every scanned literal import and re-export.
The removal manifest also includes tests that import obsolete names from built package output.
Dynamic source reads are checked separately;
the source-text assertion for the removed guard command is removed with its owner.
Build,
type checks,
native permission and prohibition controls,
and configured installed-consumer replays verify the remaining path.

Historical migration evidence remains in
`doc/handover/pi-auto-mode-axiom-evaluation.md`
and `doc/troubleshooting/pi-sdk-staging.md` at repository scope.
Legacy source paths in those records identify removed implementations as of 2026-10-17;
their current responsibilities are described by this package's architecture documents.
