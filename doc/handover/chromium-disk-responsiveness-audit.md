# Chromium disk responsiveness audit

## Scope at 2026-11-03T03:40:00-05:00

The user requested:
"Look through the entire Chromium source and flag anything blocking on disk and could impact UI responsiveness."

The audit is read-only with respect to Chromium and the user's browser configuration.
The source baseline is the installed release `156.0.8078.73`,
commit `2f9a11097447f226133bc1a31c787291a5eeab98`.
The GitHub tag API confirmed that association.

The full shallow clone is being downloaded to
`${HOME}/temp/agent/chromium-disk-audit-20261103`.
The clone scope is `chromium-disk-audit-clone.scope`,
bounded to two CPUs,
`6 GiB` memory,
`128` tasks,
and `IOWeight=10`.

## Work areas

- Acquire and inventory the entire tracked source tree,
  including an explicit accounting of dependency repositories and binary/generated exclusions.
- Scan for blocking file APIs,
  explicit blocking allowances,
  memory maps,
  and synchronous waits;
  retain the complete candidate inventory.
- Trace desktop UI interactions and profile initialization through disk operations.
- Trace mapped metrics,
  resources,
  snapshots,
  fonts,
  and graphics paths.
- Trace mobile,
  renderer,
  startup,
  and shutdown dependencies on disk.
- Consolidate findings with exact source references,
  distinguish demonstrated source paths from runtime measurements,
  verify the rendered documentation,
  and commit the results.

The research skill's background-agent workflow is being used for source tracing.
Agent outputs are scratch evidence documents named
`chromium-disk-audit-ui.md`,
`chromium-disk-audit-mapped.md`,
and `chromium-disk-audit-platform.md`.
They are same-vendor source passes,
not independent reviews.

## Evidence already available

[`helium-window-freezes.md`](../troubleshooting/helium-window-freezes.md)
records a correlated Helium main-thread write fault in the disk-backed `BrowserMetrics` mapping.
It also records ordinary Chromium main-thread filesystem waits without identifying their mapped file.
These are separate observations.
The new source audit must not relabel every candidate as a reproduced freeze.

## Next action

Finish source acquisition,
validate the checkout inventory,
and run the complete-tree candidate scan while tracing the highest-risk paths.
