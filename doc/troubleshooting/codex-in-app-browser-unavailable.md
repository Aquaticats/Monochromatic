# Codex Browser 26.930.21537 URL selection fails with no available browser

## Disabled by user request at 2026-11-03T09:41:51+00:00

During the Chromium upstream contribution task,
`agent.browsers.getForUrl('https://issues.chromium.org/')` returned
`No browser is available`,
and the documented discovery check returned `[]`.
This was a failure of the bundled Browser plugin's discovery path,
not evidence that browser automation was unavailable.
The source hash still matches the version recorded in this document.

`agent-browser` successfully loaded the public Chromium tracker and its Google sign-in page.
Its separate `--auto-connect` attempt returned
`No running Chrome instance found. Launch Chrome with --remote-debugging-port or use --cdp.`
That establishes only failure to find an exposed debugging endpoint.
It does not prove absence of a normal browser or of a signed-in Google account.

The user requested disabling the bundled plugin to prevent its instructions being loaded.
The following effective user configuration was changed in `${HOME}/.codex/config.toml`:

```toml
# User Codex config.toml
[plugins."browser@openai-bundled"]
enabled = false

[mcp_servers.node_repl]
enabled = false
```

Other entries in both sections were retained.
`codex mcp get node_repl --json` returned `"enabled": false`.
`codex plugin list --marketplace openai-bundled --json` returned empty installed and available arrays;
that CLI result does not verify the current desktop session's already-loaded skill catalog.
No desktop restart or fresh model session was performed during this change.
The fetched official Codex configuration reference documents both enable switches.
The user chose `agent-browser` for browser work.

No new upstream plugin defect or fix was established.
The historical filing decision in this document is not a current prohibition on authorized Codex work;
the current `.out-of-scope/codex-harness.md` describes a maintenance commitment only.

## Symptom

On 2026-10-15,
the Codex Browser runtime initialized,
but `agent.browsers.getForUrl('https://depot.dev/')` threw `No browser is available`.
The subsequent `agent.browsers.list()` returned `[]`.
A second discovery call also returned `[]`.

This prevented inspection of the signed-in Depot account,
including its plan and concurrency caps.
No browser tab opened and no authentication attempt occurred.
The observation therefore says nothing about Depot authentication or account settings.

## Root cause

The reason browser discovery returned no entries remains unknown.
The installed runtime establishes the immediate error boundary only.

In `$HOME/.codex/plugins/cache/openai-bundled/browser/26.930.21537/scripts/browser-service.mjs:4406`,
the minified `getForUrl` implementation obtains the browser list,
passes it and the URL to a selector,
and throws when that selector returns a nullish result:

```javascript
// browser/26.930.21537/scripts/browser-service.mjs:4406, excerpt
async getForUrl(e){let r=await xF(await this.getBrowsers(),e,this.browserPreference);if(r==null)throw new Error("No browser is available");return r}
```

This source excerpt does not establish whether the empty list reflects host configuration,
a disconnected browser,
or a defect.
No upstream fault was established.

## Verification

The installed plugin directory identifies version `26.930.21537`.
The inspected `scripts/browser-service.mjs` has SHA-256:

```text
# browser/26.930.21537/scripts/browser-service.mjs
770df1706f82057af518bd72269fbe7df4747e17f9b2c05205f11181921fdae3
```

Run these cells through the browser skill's Node REPL tool in a fresh session.
The bootstrap cell assumes the same plugin version is installed under the current user's home:

```javascript
// Node REPL bootstrap cell
var { homedir } = await import('node:os');
var { setupBrowserRuntime } = await import(
  `${homedir()}/.codex/plugins/cache/openai-bundled/browser/26.930.21537/scripts/browser-client.mjs`
);
await setupBrowserRuntime({ globals: globalThis });
```

```javascript
// Node REPL failing selection cell
await agent.browsers.getForUrl('https://depot.dev/');
```

```javascript
// Node REPL discovery cell, run after the selection error
console.log(await agent.browsers.list());
```

### Operations that completed

- Runtime bootstrap completed.
- Reading `agent.documentation.get('bootstrap-troubleshooting')` completed.
- Browser discovery completed twice and returned `[]` each time.
  Completion here means the discovery API returned;
  it does not mean a usable browser existed.

### Operations that failed

- URL selection for `https://depot.dev/` threw `No browser is available`.
- Account inspection remained unperformed because no browser was selected.
  This was not a Depot page failure.

## Verified workarounds

None.
No alternate browser connection was established,
and no authenticated account data was inspected.
Public documentation can support general product research,
but cannot verify this account's plan or limits.

## What does not work

Successful runtime bootstrap did not establish browser availability.
Reading bootstrap troubleshooting documentation did not itself restore a browser:
the discovery calls still returned an empty list.
No reconnect,
restart,
configuration change,
or source modification was tested.

## Upstream filing artifact

Nothing to add upstream from this investigation.
There is no established defect or candidate fix to report.
No issue or comment was drafted or sent.

### Upstream filing decision

The exemption in [Codex harness scope](../../.out-of-scope/codex-harness.md)
places Codex-specific harness bugs and integration work out of scope.
The investigation therefore ends at the installed runtime evidence.
No upstream tracker search or source clone was performed.

1.  Upstream fault:
    unknown;
    an empty discovery result alone does not establish a defect.
2.  Upstream fixability:
    unassessed because the cause is unknown.
3.  Supported use case:
    the runtime exposes URL selection,
    but availability requirements for this host were not established.
4.  Contribution policy:
    not investigated under the harness exemption.
5.  Maintainer willingness:
    not investigated under the harness exemption.
6.  Minimal fix:
    none prototyped because no defect or cause was established.
