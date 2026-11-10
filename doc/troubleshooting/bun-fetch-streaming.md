# Bun 1.3.10 fetch streaming: `ReadableStream.cancel()` does not propagate to the HTTP layer, leaving `FetchTasklet.poll_ref` ref'd and the process hanging

## Symptom

A Bun process that consumes a streaming HTTP response (SSE or
chunked transfer) via `fetch` fails to exit after all work
completes.
 The process hangs indefinitely,
 or until the server
closes the TCP connection.
 Node.
js exits in ~4 ms under
identical conditions.

Affected patterns:

```ts
// Breaking from a streaming response
const response = await fetch(url,);
for await (const chunk of response.body!) {
  // process chunk
  break; // process hangs here -- Bun keeps the HTTP connection alive
}

// OpenAI SDK streaming (uses fetch internally)
const stream = await client.chat.completions.create({
  model,
  messages,
  stream: true,
},);
for await (const chunk of stream) {
  // process chunk
}
// process should exit here but may hang
```

For SSE streams (e.g. OpenAI/OpenRouter chat completions with
`stream: true`),
 the server may hold the connection open after
sending `[DONE]` due to HTTP keep-alive,
 causing the process to
hang for seconds to minutes.
 With infinite streams,
 the process
never exits.

## Root cause

Bun's `FetchTasklet` maintains a `poll_ref` that keeps the JS
event loop alive while a fetch request is in progress.
 The ref
is set when the fetch starts (`poll_ref.ref()`) and only cleared
when:

1.  The HTTP thread receives the complete response
    (`is_done == true` in `onProgressUpdate`);
     or
2.  `ignoreRemainingResponseBody()` is called (e.g. when the
    `Response` object is GC'd without body consumption).

The bug:
 calling `ReadableStream.cancel()` on the response body
(which happens when breaking from `for await`,
 calling
`reader.cancel()`,
 or `stream.cancel()`) does not propagate to
the HTTP layer.
 The `FetchTasklet` never learns the JS side is
done with the stream,
 so `poll_ref` stays ref'd until the server
finishes sending or closes the connection.

Why the hang is intermittent:
 it depends on server-side TCP
connection teardown timing.
 After the SSE `[DONE]` sentinel,
 if
the server closes the connection immediately,
 `poll_ref` unrefs
via the `onProgressUpdate(is_done=true)` path and the process
exits.
 If the server holds the connection (HTTP/1.1 keep-alive),
`poll_ref` stays ref'd.

Upstream issue:
 [oven-sh/bun#17048](https://github.com/oven-sh/bun/issues/17048).

Upstream fix:
 [oven-sh/bun#27232](https://github.com/oven-sh/bun/pull/27232)
adds a `cancel_handler` callback that propagates
`ReadableStream.cancel()` to
`FetchTasklet.ignoreRemainingResponseBody()`.
 Not merged as of
March 2026.

## Verification

Versions under test:

- Bun:
   all versions up to and including 1.3.10
- OpenAI SDK v5+ (uses native fetch;
   v4 had a separate
  `agentkeepalive` hang bug)
- OpenRouter SDK:
   all versions (uses native fetch)
- Node.
  js:
   not affected;
   undici unrefs idle sockets via
  `socket.unref()` in `resumeH1`

Reproduce against any SSE endpoint that holds keep-alive after
`[DONE]`:

```ts
const response = await fetch('https://example.com/sse-endpoint',);
for await (const chunk of response.body!) {
  if (someCondition(chunk,))
    break;
}
// Process does not exit; SIGINT required.
```

## Verified workarounds

### Watchdog with diagnostics

The inference-canary uses an unref'd watchdog timer that dumps
active async handles via `why-is-node-running` before
force-exiting:

```ts
import whyIsNodeRunning from 'why-is-node-running';

// ... after all work completes ...

const WATCHDOG_TIMEOUT_SECONDS = 5;
const watchdog = setTimeout(() => {
  console.error('process did not exit naturally, dumping active handles:',);
  whyIsNodeRunning();
  process.exit(0,);
}, WATCHDOG_TIMEOUT_SECONDS * 1000,);
watchdog.unref(); // the watchdog itself must not prevent exit
```

Normal case:
 event loop drains,
 unref'd timer never fires,
 clean
exit.
 Hang case:
 after 5 seconds the watchdog logs which
resources are alive,
 then force-exits.

Tradeoff:
 trades a clean exit for a force-exit on the
hang path.
 The 5-second window adds latency on the genuine-hang
case;
 reduce or raise depending on how quickly the process must
turn around.
 Logging makes the hang diagnosable so the next
investigator does not re-derive the cause.

### AbortController

`AbortController.abort()` on the fetch request bypasses the
`ReadableStream.cancel()` path and tears down the HTTP
connection directly:

```ts
const controller = new AbortController();
const response = await fetch(url, { signal: controller.signal, },);
for await (const chunk of response.body!) {
  // process chunk
}
controller.abort(); // tears down HTTP connection at the transport level
```

The OpenAI SDK accepts a `signal` option on `.create()` calls,
but aborting after successful completion produces an
`AbortError` that must be caught.

Tradeoff:
 every streaming caller must create a controller and
remember to abort;
 missing the abort restores the hang.
Functional but invasive.

## Alternative transports that avoid the bug

Existing transport options for streaming inference:

- **SSE via fetch (OpenRouter,
   OpenAI default)**:
   uses fetch
  ReadableStream;
   affected.
- HTTP chunked via fetch:
   uses fetch ReadableStream;
  affected.
- WebSocket via `ws` (OpenAI Responses API only):
   does not
  use fetch ReadableStream;
   not affected.
- WebSocket via Bun native:
   does not use fetch
  ReadableStream;
   not affected.
- undici fetch (bypasses Bun's native fetch):
   unclear;
  Bun polyfills Node APIs;
   untested.

OpenRouter's Responses API (`/api/v1/responses`) supports
`stream: true` but returns SSE over HTTP POST,
 the same fetch
ReadableStream path as Chat Completions.
 No WebSocket transport
is available.

OpenAI's Responses API supports WebSocket via
`openai/resources/responses/ws` (requires `ws` npm package,
 beta
feature,
 header `OpenAI-Beta: responses_websockets=2026-02-06`),
but this is only available when calling OpenAI directly,
 not
through OpenRouter.

## What does not work

- `response.body.cancel()` or `reader.cancel()` alone:
   hits the
  exact `ReadableStream.cancel()` path that does not propagate.
- Setting `keepAlive: false` headers on the request:
   server may
  honour or ignore;
   even when honoured the hang persists if the
  server holds the socket briefly after `[DONE]`.
- Polling `process._getActiveHandles()` to force exit:
   the
  fetch's poll_ref does not appear in active-handles output the
  way a libuv timer does;
   the diagnostic is unhelpful here.
- Waiting for upstream PR #27232 to merge:
   as of March 2026 it
  is still open;
   no estimated merge date.

## Why we do not file this upstream (again)

Already filed.
 Walking the 5 constraints for completeness:

1.  **Is it really upstream's fault?
    ** Yes;
     missing cancel
    propagation in `FetchTasklet`.
2.  **Can upstream fix it?
    ** Yes;
     PR #27232 implements the fix
    in roughly the right shape.
3.  **Are they supporting this use case?
    ** Yes;
     native fetch
    streaming is an explicit Bun feature.
4.  **Will they likely fix it?
    ** The PR exists but is not merged.
    Adding a duplicate report would not advance the PR.
5.  **Have we prototyped a minimal fix?
    ** Upstream has a fix
    prototype already;
     nothing for us to add.

Decision:
 no new upstream report.
 Track #17048 and #27232.

## Draft (kept as reference)

Already represented by oven-sh/bun#17048 and PR #27232.
 No
separate draft.
