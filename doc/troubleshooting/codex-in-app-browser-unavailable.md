# Codex Browser 26.930.21537 URL selection fails with no available browser

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
