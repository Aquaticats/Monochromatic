# Claude Code 2.1.233 opens stdio MCP connections with `initialize`, so a revision 2026-07-28 server reports as failed

## Symptom

`claude mcp list` and `claude mcp get <name>` report a healthy modern-only stdio server as
unreachable:

```text
mvm: node /home/user/Monochromatic/package/mcp/mvm/dist/final/node/index.mjs
  - ✘ Failed to connect — -32601: MCP error -32601: Method not found: initialize.
    This server implements MCP revision 2026-07-28, which removed the initialize handshake:
    call server/discover and declare the revision in each request's params._meta instead
```

The server is fine.
Driven directly over stdio it answers `server/discover`,
 `tools/list`,
 and `tools/call` correctly,
 and returns its first frame in about 40 ms.

The failure is not stable over time,
 which is the confusing part.
The same binary and the same client version reported `✔ Connected` earlier the same day and
`✘ Failed to connect` later,
 with no change to either side.

## Root cause

Claude Code's MCP client defaults to the legacy handshake and only probes `server/discover`
when a remote feature gate says to.
For stdio that gate defaults to off,
 so the client opens with `initialize`,
 which a modern-only server must refuse.

Extracted with `tweakcc unpack` from the native binary at
`~/.local/share/claude/versions/2.1.233`.
The bundle is minified,
 so identifiers below are its mangled names rather than source names.

The default negotiation mode is legacy:

```js
ssb=-32022,asb,gsb="legacy",_sb=class e{_pending;_probeCounter=0;...
```

`gsb` is the fallback in the mode resolver,
 which also reads an environment variable and then switches on transport kind:

```js
function $qa(e){
  let t=V.MCP_PROTOCOL_NEGOTIATION,
      r=t==="legacy"||t==="auto"?t:void 0;
  if(t!==void 0&&r===void 0)
    w(`MCP_PROTOCOL_NEGOTIATION=${t} is invalid; expected 'legacy' or 'auto' — ignoring`,{level:"warn"});
  if(r==="legacy")return{mode:"legacy"};
  ...
  if(r==="auto"){ if(!xpS.has(e))return{mode:"legacy"};
    return e==="stdio"?{mode:"auto",probe:o}:{mode:"auto",probe:n}; }
  switch(e){
    case"http":return rt("tengu_mcp_protocol_negotiation_http",!1)===!0?{mode:"auto",probe:n}:{mode:"legacy"};
    case"claudeai-proxy":return rt("tengu_mcp_protocol_negotiation_claudeai",!1)===!0?{mode:"auto",probe:n}:{mode:"legacy"};
    case"stdio":return rt("tengu_mcp_protocol_negotiation_stdio",!1)===!0?{mode:"auto",probe:o}:{mode:"legacy"};
    case"ccr-proxy":case"sse":case"ws":case"ide":case"in-process":case"sdk-control":return{mode:"legacy"}
  }
}
```

`rt(name, !1)` reads a feature gate whose default argument is `false`.
So for `stdio`,
 discovery probing happens only while `tengu_mcp_protocol_negotiation_stdio` is enabled for
the account.
That gate is remote configuration:
 it can change between sessions with no local change,
 which is what makes the same setup connect in the morning and fail in the afternoon.

When the mode is legacy the client calls `_legacyHandshake`,
 which sends `initialize` directly:

```js
async _legacyHandshake(e,t){
  let r=tXs(this._supportedProtocolVersions);
  ...
  let o=await this.request({method:"initialize",params:{protocolVersion:n,...}},t);
```

The probe machinery exists and is complete,
 including the request id this repo's earlier tap recorded (`server-discover-probe-${++this._probeCounter}`),
 so the client is genuinely dual-era.
It simply does not reach that code path while the gate is off.

An earlier reading in `doc/planning/mcp-stdio-2026-07-28-upgrade.md` concluded that
"Claude Code 2.1.233 probes `server/discover` first".
That reading was correct when taken and is wrong as a general statement:
 it observed the gate while it happened to be on.

## Verification

Client version 2.1.233,
 build `f8d57569aaf350fe25dc4dfa10cad59db8ea4d45`,
 extracted with `tweakcc@4.3.3`.

Tap a disposable registration so the exchange is visible rather than inferred:

```js
// tap.mjs
import { spawn } from 'node:child_process';
import { appendFileSync } from 'node:fs';
const LOG = process.env.TAP_LOG;
const child = spawn('node', [process.env.TAP_BIN], { stdio: ['pipe', 'pipe', 'inherit'] });
process.stdin.on('data', d => { appendFileSync(LOG, 'IN  ' + d); child.stdin.write(d); });
child.stdout.on('data', d => { appendFileSync(LOG, 'OUT ' + d); process.stdout.write(d); });
process.stdin.on('end', () => child.stdin.end());
child.on('exit', c => process.exit(c ?? 0));
```

```sh
claude mcp add --scope local mvmtap \
  --env TAP_LOG=/tmp/tap.log --env TAP_BIN=/abs/path/to/server.mjs -- node /abs/path/to/tap.mjs
claude mcp get mvmtap
claude mcp remove mvmtap -s local
```

Observed,
 in full:

```text
IN  {"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{...}}}
OUT {"jsonrpc":"2.0","id":0,"error":{"code":-32601,"message":"Method not found: initialize..."}}
```

One message in,
 one error out.
`server/discover` never appears on an `IN` line;
 grepping the log for it matches only the server's own error text.

What works cleanly,
 driving the same binary directly:

```sh
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}' | node server.mjs
```

- `server/discover` with or without `clientCapabilities` and `clientInfo`:
   answered.
- `initialize` followed by `server/discover`:
   the first is refused,
   the second answered.
- Time to first frame:
   39 ms,
   41 ms,
   41 ms across three runs.

## Verified workarounds

None restore the CLI health check.
Both options below are about what the server chooses to serve,
 and the first is the one this repo chose:
 the user was shown the cost and elected to stay modern-only rather than add a legacy
fallback or patch the client.

Accept it and verify elsewhere.
Drive the built binary over stdio in tests,
 which is what `package/mcp/stdio` and `package/mcp/mvm` do.
Tradeoff:
 `claude mcp list` shows a permanent red line for a healthy server,
 and a real outage in that server looks identical to this,
 so the CLI stops being a usable signal for it.

Serve both eras.
Answer `initialize` as well as `server/discover`,
 which makes the server work regardless of the gate.
Tradeoff:
 it reverses the modern-only scope decision recorded in
`doc/planning/mcp-stdio-2026-07-28-upgrade.md`,
 and dual-era support means maintaining a handshake path the spec calls legacy,
 including the older result shapes that lack `resultType`.

## What does not work

`MCP_PROTOCOL_NEGOTIATION=auto`.
The resolver reads it and the value is spelled correctly,
 yet the health check still sends `initialize`:

```sh
MCP_PROTOCOL_NEGOTIATION=auto claude mcp get mvm
#   Status: ✘ Failed to connect
#   Issue: -32601: MCP error -32601: Method not found: initialize...
```

No warning is printed,
 so the value was accepted as valid rather than rejected.
Either `V` is not `process.env`,
 or the health check reaches a different construction site than the one that calls `$qa`
with the mapped transport kind.
Note that the other call site in the bundle passes `$qa("sdk-control")`,
 and `sdk-control` returns legacy in both the switch and the `auto` branch,
 because `xpS` is `new Set(["http","claudeai-proxy","ccr-proxy","stdio"])` and does not
contain it.

Re-registering the server.
`claude mcp remove` plus `claude mcp add` changes nothing:
 the registration in `~/.claude.json` is a plain command entry with no cached era,
 confirmed by reading it.

Waiting for a retry.
Repeated health checks give the same result;
 this is not a transient connection failure.

## Upstream filing decision

`.out-of-scope/claude-code-upstream-bugs.md` covers this tool:
 "This project does not file or track Claude Code bugs as GitHub issues",
 because upstream is unresponsive to reports and fixes take many releases.
Its prescribed handling is exactly this document plus a local workaround,
 and explicitly no tracking issue.

So the six-constraint check is not run and no draft issue is kept.
Recording it anyway would be drafting a filing this repo's own policy forbids sending.

It is also not clear this is a defect rather than a deliberate staged rollout:
 the probe machinery is complete and gated,
 which is what an incremental rollout looks like from outside.
