# Firefox ESR disk responsiveness audit

## Scope at 2026-11-03T03:13:00-05:00

The user requested the Firefox ESR equivalent of:
"Look through the entire Chromium source and flag anything blocking on disk and could impact UI responsiveness."

The audit is read-only with respect to Firefox and browser configuration.
Mozilla's product-details endpoint reports `153.6.0esr` as `FIREFOX_ESR`.
The release tag `FIREFOX_153_6_0esr_RELEASE` resolves to
`a34c4160e3f75ab9918fbe4669ff7ae56f1977c8` in `mozilla-firefox/firefox`.
The legacy `FIREFOX_ESR115` track is separate from this selected current ESR.

Sources:

- <https://product-details.mozilla.org/1.0/firefox_versions.json>
- <https://api.github.com/repos/mozilla-firefox/firefox/git/ref/tags/FIREFOX_153_6_0esr_RELEASE>

The full shallow clone is acquiring at `${HOME}/temp/agent/firefox-esr-disk-audit-20261103`.
Its systemd scope is `firefox-esr-audit-clone.scope`,
bounded to two CPUs,
`6 GiB` memory,
`128` tasks,
and `IOWeight=10`.

## Work areas

- Acquire and inventory the complete tracked release tree and account for exclusions.
- Scan the complete tree for blocking file APIs,
  mappings,
  synchronous waits,
  and explicit main-thread I/O warnings.
- Trace desktop UI,
  downloads,
  file pickers,
  extensions,
  preferences,
  and profile/session initialization.
- Trace storage,
  SQLite,
  Places,
  cookies,
  permissions,
  certificates,
  and cache dependencies.
- Trace fonts,
  graphics,
  resource loading,
  mapped files,
  and native platform paths.
- Consolidate evidence with exact release source references,
  separate source-level risks from measured stalls,
  render-check the report,
  and commit the results.

The research skill's background-agent workflow is being used for source tracing.
Scratch evidence files are `firefox-esr-audit-ui.md`,
`firefox-esr-audit-storage.md`,
and `firefox-esr-audit-mapped.md` under `${HOME}/temp/agent`.
These are same-vendor source passes,
not independent reviews.

## Limits and next action

Source evidence can establish a blocking path and its thread,
but cannot establish stall duration or frequency on this machine.
An asynchronous disk call is not automatically a UI blocker.
Content-main-thread stalls and parent UI stalls must be identified separately.
No Firefox build or runtime disk-stress experiment is authorized or needed for the source audit.

Next action:
complete source acquisition,
validate the release identity and file inventory,
and generate the complete-tree candidate scan.
