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

The full shallow clone was acquired at `${HOME}/temp/agent/firefox-esr-disk-audit-20261103`.
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

## Completed source audit at 2026-11-03T03:41:02-05:00

The [source audit report](../audit/firefox-esr-disk-responsiveness.md)
records direct UI filesystem operations,
waits and locks carrying background disk latency to the UI,
render-thread shader reads,
content-thread localStorage waits,
mapped resources,
and startup/shutdown dependencies.
It includes feature guards and rejected asynchronous-path overclaims.

The complete checkout inventory and ripgrep discovery agree on `468,910` regular files.
No tracked regular file was missing from discovery.
The unignored search reported `82,115` matching lines in `22,108` files.
Tests,
vendored dependencies,
hidden files,
and mobile sources were retained in the scan.
The report states that this is complete-tree lexical coverage plus detailed candidate-family tracing,
not exhaustive manual adjudication of every match.

Retained evidence sits beside the report:

- `firefox-esr-disk-responsiveness.scan.json`
- `firefox-esr-disk-responsiveness.inventory.json.gz`
- `firefox-esr-disk-responsiveness.candidates.jsonl.gz`
- `firefox-esr-disk-responsiveness.sources.json.gz`
- `firefox-esr-disk-responsiveness.browser.json.gz`
- `firefox-esr-disk-responsiveness.external.json`
- `firefox-esr-disk-responsiveness.render.json`
  `firefox-esr-disk-responsiveness.browser.json.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `firefox-esr-disk-responsiveness.candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `firefox-esr-disk-responsiveness.inventory.json.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `firefox-esr-disk-responsiveness.sources.json.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

The browser check matched `205` Firefox source selections to the pinned local source.
GitHub virtualizes long files;
the completed check navigates affected anchors individually and reads their rendered line elements.
Whitespace-only lines were normalized because empty GitHub line elements contain their own newline.
Missing rendered lines were rechecked after navigation,
not accepted as matching.

The source-reference second pass corrected callback locations,
a macOS function name,
download-directory platform guards,
and several narrow line ranges.
It did not retract a blocking-path finding.
The report's upstream GTK and fontconfig sources are pinned,
with installed package versions recorded separately from actual running-library provenance.

Commits already recorded:

- `4fcc005374`:
  scope and work areas.
- `21f33ad9a3`:
  full-tree inventory and candidate scan.
- `4615353fcb`:
  detailed findings.
- `5f393a21be`:
  evidence links and automation recovery record.
- `c640aa2508`,
  `8f3b371f9f`,
  and `1803939da8`:
  precise source references and download guards.
- `3e4440eaa0`:
  native-library citation mirrors and completed-audit record.

The incidental browser-session recovery is documented in
[`agent-browser-eagain.md`](../troubleshooting/agent-browser-eagain.md).
No browser-exit root cause was inferred from the failed automation session.

## Limits and completion checks

Source evidence can establish a blocking path and its thread,
but cannot establish stall duration or frequency on this machine.
An asynchronous disk call is not automatically a UI blocker.
Content-main-thread stalls and parent UI stalls must be identified separately.
No Firefox build or runtime disk-stress experiment was performed.

Completion at `2026-11-03T03:42:34-05:00`:
the native-library source selections matched in the browser,
and `agent-browser --session firefox-esr-source-audit close` returned `Browser closed`.
The browser-visible fontconfig GitLab page presented an `Oh noes!` page;
the citations use the project's GitHub mirror at the identical verified Git commit.
GNOME's GitHub mirror also contains the exact pinned GTK commit.

The scoped command
`mise run format:monochromatic-lint -- doc/audit/firefox-esr-disk-responsiveness.md doc/handover/firefox-esr-disk-responsiveness-audit.md doc/troubleshooting/agent-browser-eagain.md`
passed.
The final rendering receipt records Pandoc's version,
document hashes,
heading counts,
and link counts.
All source-tracing agents completed their work and reported their search scopes inactive.
The source checkout is retained as evidence;
there is no source build,
test fixture,
poll loop,
or browser session left running for this audit.

The requested static source audit is complete.
Runtime measurements are explicitly unperformed follow-up work,
not a queued continuation.
