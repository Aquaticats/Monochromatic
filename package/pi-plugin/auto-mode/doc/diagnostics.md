# Diagnostic evidence and storage

## Purpose and callers

`native/contract/integration/action-policy/assessment-diagnostics.mjs` records selected model requests and replies.
`native/contract/integration/action-policy/decision-diagnostics.ts` correlates source clauses,
scores,
routing,
and final outcomes by review and operation identity.
Both use the existing tagged logger.

This evidence is necessary to distinguish a model's uncertain answer from a request that received no answer.
The reported MiMo incidents demonstrate the difference:
one review records an approval distribution of `0.48`,
`0.34`,
and `0.18`;
another reaches an HTTP transport exception before any final answer is admitted.

## Storage ownership

The shared logger owns the file sink and destination selection.
Auto-mode's `mise.toml` sets `MONOCHROMATIC_LOG_DIRECTORY` to
`${HOME}/temp/agent/pi-auto-mode-diagnostics` for package tasks.
This preserves diagnostic records outside package source and release artifacts.
Other consumers can select their own directory.
Published logger `0.4.4` keeps nearest-ancestor `node_modules/.monochromatic` as its default,
then falls back to per-user state when the project destination cannot be used.
The fallback notice identifies the actual destination and escapes terminal control characters.
Explicit directory arguments and nonempty `MONOCHROMATIC_LOG_DIRECTORY` stay strict.
The logger's [file destination contract](../../../logger/core/doc/file-destinations.md)
defines platform layouts and initialization probes.
The active auto-mode artifact still needs its final coherent rebuild with this logger repair.
Auto-mode does not own a parallel logging backend.
The initial package directory contains 12,728 diagnostic files totaling 7,089,712,671 apparent bytes.
The largest sampled files are historical logs from 2026-10-14.
Their dominant records are repeated resource-input snapshot messages.

Current `native/contract/collector/resource-input-custody/diagnostics.ts` batches those snapshot counts
until the next microtask.
It retains context,
system,
append,
and unknown attempt counts without writing a record for every synchronous snapshot.
The historical storage total is therefore not evidence that current code still emits the old per-copy messages.

## Invariants and alternatives

Decision reconstruction retains selected source material,
question identities,
model scores,
fallback reasons,
and the original operation.
Transport failures exclude credentials and arbitrary upstream exception text.
The current HTTP invocation category permits bounded recovery without retaining a borrowed error cause.
The original reported transport failure's discarded cause cannot be recovered retroactively.

Silencing all diagnostics would remove the evidence needed to diagnose these failures.
Adding package-specific file retention would duplicate the shared logger's responsibility.
Storage maintenance must preserve investigation evidence and account for active writers.
Compression or archival is preferable to discarding captured decisions without an evidence policy.

## Completed historical archive

The 2026-10-17 maintenance archives only unchanged files predating the current boot
(`2026-10-17T03:34:00.816Z`).
The shared logger chooses fresh timestamped paths for new sinks;
current-boot files remain in place.
No whole-directory cleanup is used.

The archive contains 12,626 files totaling 7,064,174,193 original bytes
and 194,896,137 gzip bytes.
Its private location is
`${HOME}/temp/agent/pi-mimo-lint-approval-20261017/diagnostic-archive`.
The adjacent `archive-manifest.jsonl` records original paths,
archive paths,
SHA-256,
byte lengths,
and modification times.
Each original is removed only after decompression verifies its hash and length,
the archive and manifest are published,
and original inode,
size,
and modification time remain unchanged.

An independent full reread verifies every manifest entry.
Its positive control rejects a valid gzip containing changed content of the same length.
A reference scan checks all 1,670 tracked Markdown files and finds no archived filename references.
The archive preserves evidence outside the package;
it does not claim automatic retention for future logger output.

The final batch contains 622 additional files unchanged since inventory,
with no writes after 08:03 UTC.
The same decompressed-hash and length checks preserve their 265,115,894 original bytes
as 10,014,676 compressed bytes in the existing private archive.
The combined manifest retains both batches.
Routine package tasks now route diagnostics outside the package through the shared sink setting.
The package diagnostic directory is empty at the final footprint measurement.

## Verification

The native fallback tests assert correlated model requests,
question evidence,
and outcomes.
The transport tests include hostile credential-bearing failures.
Snapshot diagnostics can be inspected through their batched count records.
The footprint inventory and sampled log-shape report are retained privately under
`${HOME}/temp/agent/pi-mimo-lint-approval-20261017`.

The shared logger's real-filesystem directory test writes through an explicit sink and the default logger.
Independent child processes prove that an unset destination writes to disposable ancestor `node_modules`,
while a configured destination receives its record without appending that record to the ancestor directory.
This is a destination change,
not automatic retention or a reduction in diagnostic content.
