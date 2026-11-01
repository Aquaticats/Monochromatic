# Node 26 built-in `fetch` (undici 8.11.2) ends a slow reply at 300 s, whatever the abort signal says

## Symptom

A long non-streamed request to an LLM provider made with Node's global `fetch` fails after almost exactly 300 s:

```text
TypeError: fetch failed
  cause: HeadersTimeoutError: Headers Timeout Error (code UND_ERR_HEADERS_TIMEOUT)
```

A streamed request whose server sends headers at once but then goes silent fails after 300 s of silence:

```text
TypeError: terminated
  cause: BodyTimeoutError: Body Timeout Error
```

Both happen even when the request carries `signal: AbortSignal.timeout(900_000)`.
A longer abort signal cannot lengthen the wait,
because these timers belong to the HTTP client underneath `fetch`,
not to the signal.

Seen on 2026-10-22 in the translation-repair Synthetic effort probe 4
(`~/temp/agent/tr-fan/w287-xingz18/synthetic-effort-probe-4.out`):
all 39 non-streamed calls to `https://api.synthetic.new/openai/v1/chat/completions` on a hard reasoning question
ended at 300.0 s with `TypeError: fetch failed`.
An earlier probe's single GLM call ended the same way,
reported as an uncaught `HeadersTimeoutError`.

## Root cause

Node's global `fetch` is undici.
This Node reports its bundled version:

```sh
# shell
node --print "process.versions.undici"
# 8.11.2
```

In undici v8.11.2 every client gets a 300 s default for both the headers wait and the gap between body chunks,
at `lib/dispatcher/client.js:316-317`:

```js
// lib/dispatcher/client.js (undici v8.11.2)
    this[kBodyTimeout] = bodyTimeout != null ? bodyTimeout : 300e3
    this[kHeadersTimeout] = headersTimeout != null ? headersTimeout : 300e3
```

While a request has no status line yet,
the parser arms the headers timer,
at `lib/dispatcher/client-h1.js:1140-1143`:

```js
// lib/dispatcher/client-h1.js (undici v8.11.2)
        const headersTimeout = request.headersTimeout != null
          ? request.headersTimeout
          : client[kHeadersTimeout]
        socket[kParser].setTimeout(headersTimeout, TIMEOUT_HEADERS)
```

When a timer fires,
the socket is destroyed with the matching error,
at `lib/dispatcher/client-h1.js:848-856`:

```js
// lib/dispatcher/client-h1.js (undici v8.11.2)
  if (timeoutType === TIMEOUT_HEADERS) {
    if (!socket[kWriting] || socket.writableNeedDrain || client[kRunning] > 1) {
      assert(!paused, 'cannot be paused while waiting for headers')
      util.destroy(socket, new HeadersTimeoutError())
    }
  } else if (timeoutType === TIMEOUT_BODY) {
    if (!paused) {
      util.destroy(socket, new BodyTimeoutError())
    }
```

The default is documented,
at `docs/docs/api/Client.md:49-51`:

```text
* `headersTimeout` {number|null} The timeout, in milliseconds, the parser
  waits to receive the complete HTTP headers before the request times out. Use
  `0` to disable it entirely. **Default:** `300e3`.
```

A non-streamed chat completion sends its status line and headers only once the whole answer is generated,
so any answer that takes longer than 300 s to generate hits the headers timer.
A streamed completion sends headers at once and then one chunk per token or reasoning delta;
only a silence of 300 s between chunks hits the body timer.

## Verification

Node v26.12.0,
bundled undici 8.11.2 (source read at tag `v8.11.2`,
cloned to `~/temp/agent/undici-2026-10-22`).

### Harness: headers never sent

```ts
// headers-timeout-probe.ts: run with `node headers-timeout-probe.ts`, no network, no key
import { createServer, } from 'node:http';

const server = createServer(function neverAnswer() {},);
await new Promise<void>(function listen(resolve,) {
  server.listen(0, '127.0.0.1', resolve,);
},);
const address = server.address();
if (address === null || typeof address === 'string') {
  throw new Error('server address is not a port');
}
const started = performance.now();
try {
  await fetch(`http://127.0.0.1:${address.port}/`, { signal: AbortSignal.timeout(900_000,), },);
} catch (error) {
  const cause = error instanceof Error ? error.cause : undefined;
  console.log(((performance.now() - started) / 1000).toFixed(1,), String(error,), String(cause,),);
}
server.closeAllConnections();
server.close();
```

### Harness: headers at once, body after 310 s

The same file with the server replaced by one that calls `response.writeHead(200)` and `response.flushHeaders()`
at once and `response.end()` after 310 s,
reading the body with `await response.text()`.

### Cases that fail

- Headers never sent,
  900 s abort signal:
  `300.7 s`,
  `TypeError: fetch failed`,
  cause `HeadersTimeoutError: Headers Timeout Error`,
  code `UND_ERR_HEADERS_TIMEOUT`.
- Headers at once,
  no body bytes for 310 s,
  900 s abort signal:
  `300.8 s`,
  `TypeError: terminated`,
  cause `BodyTimeoutError: Body Timeout Error`.
- Synthetic,
  non-streamed,
  digit-sum question with a 1,000,000 range,
  900 s abort signal,
  four models:
  39 of 39 calls `TypeError: fetch failed` at 300.0 s.

### Cases that work

- Synthetic,
  non-streamed,
  an easier question answered within a few hundred reasoning tokens:
  36 of 36 calls returned in 3.3 s to 82.9 s (`synthetic-effort-probe-3.out`).
- Streamed requests whose server sends a chunk at least every 300 s:
  the body timer restarts on every chunk.
- Synthetic,
  streamed (`synthetic-effort-probe-5.ts`),
  the same hard question,
  1,200 s abort signal,
  four models:
  26 of 39 calls returned HTTP 200 after 336.3 s to 1,135.2 s,
  every one past the 300 s limits that ended the non-streamed calls
  (`synthetic-effort-probe-5.out`).
  The other 13 calls are not undici timer cases:
  five ended at the 1,200 s abort signal,
  and eight in one cluster ended with cause `Error: read ECONNABORTED`
  (`TypeError: terminated` mid-stream,
  or `TypeError: fetch failed` before headers),
  a dropped connection rather than a timer.

## Verified workarounds

### Stream the reply

Send `stream: true` (with `stream_options: { include_usage: true }` on OpenAI-compatible APIs to keep token counts)
and read the server-sent events.
Headers arrive at once,
so the headers timer cannot fire,
and a reasoning model streams its reasoning deltas,
so the body timer keeps restarting.

Tradeoffs:
the caller must parse server-sent events and reassemble content,
reasoning and usage itself;
a provider that buffers its reasoning and sends nothing for 300 s still ends at the body timer;
the abort signal must stay as the overall bound,
because the 300 s limit no longer bounds the whole call.
The translation-repair provider clients already stream (`stream: true` in each client under
`package/translation-repair/provider/src/`),
with their own first-content window
(`STREAM_FIRST_CONTENT_MS = 180_000` at `package/translation-repair/provider-stream/src/stream-idle-guard.ts:89`,
below both undici limits),
so production calls are not affected.

## What does not work

- A longer `AbortSignal.timeout(...)`:
  measured at 900 s,
  the call still ends at 300 s.
  The signal only shortens a request;
  it does not reach undici's parser timers.
- Assuming the 300 s failure is the provider's:
  the local never-answering server reproduces it with no provider involved.
- Not tried:
  passing `dispatcher: new Agent({ headersTimeout, bodyTimeout })` from the npm `undici` package in the
  `fetch` init.
  It needs the npm package (not installed in this repository),
  and its pairing with Node's bundled
  copy was not tested here.

## Upstream filing decision

Do not file.

1. Is it really upstream's fault?
   No. The 300 s defaults are documented behaviour
   (`docs/docs/api/Client.md:49-51`),
   with `0` documented as the way to disable them.
2. Can upstream fix it?
   Not applicable:
   there is no defect to fix.
3. Are they supporting this use case?
   Yes,
   through the documented `headersTimeout` and `bodyTimeout` options.
4. Would the repo welcome our contribution?
   Not checked,
   since no contribution is proposed.
5. Will they likely fix it?
   Not applicable.
6. Prototyped a fix?
   Not applicable.

`.out-of-scope/` has no entry for Node or undici (listing of 2026-10-22).
Duplicate search:
`gh search issues --repo nodejs/undici "headersTimeout fetch"` returned no results,
and `gh search issues --repo nodejs/node "fetch HeadersTimeoutError 300"` returned one unrelated 2015 meeting issue.
Nothing to file or add.
