# Pi 1.5.0 agents ended with `terminated` when Cloudflare cut provider responses mid-stream

## Symptom

A Pi run ended part way through a model response,
and the driver recorded `endReason: model-error` with `modelError: terminated`.
The assistant message in the session file had `stopReason: error` and `errorMessage: terminated`.
The text `terminated` is all Pi keeps,
so a closed connection,
a reset and a timeout cannot be told apart afterwards.

On 2026-11-09 Pi's session logs recorded 21 such responses in 10 bursts,
from 15:05:04Z to 17:08:36Z,
on `neuralwatt` and `opencode-go`.
OpenAI responses in flight across the same bursts finished.
No response was cut among the 816 recorded from 17:09Z to 17:31:11Z.

## Root cause

The owner relayed the provider's answer on 2026-11-09:
"The provider confirmed it was CF and told us to turn on client retries for terminated errors.
Do it for pi permanently."
Cloudflare's status page carried "Network Performance Issues in Eastern North America",
investigating from 16:34:35Z,
identified at 16:56:39Z and monitoring from 17:21:50Z.
The bursts at 15:05Z and 15:08Z came before that incident's first post.
The confirmation came from `neuralwatt`;
`opencode-go` sits behind the same Cloudflare edge,
measured from its address and headers,
and its operator was not asked.

`terminated` is the error Node's `fetch` raises when a response body ends early.
A local test on Node v26.13.0 with undici 8.12.2 printed:

```json
{ "cause": "other side closed", "causeCode": "UND_ERR_SOCKET", "message": "terminated", "name": "TypeError" }
```

Pi retries a failed response only when `retry.enabled` is on.
`~/.pi/agent/settings.json` had it off,
so one cut ended the run.
Pi's retryable pattern,
in the installed bundle `dist/bundle/chunks/chunk-62VOPMLK.js` of `@earendil-works/pi-coding-agent` 1.5.0,
includes the alternative:

```text
"timeout","terminated","websocket.?closed"
```

`package/pi-shared/subagent-driver/src/retries-of-a-model-error.ts` has its own list of transient phrases,
and `terminated` is not on it,
so the driver's retry did not cover this failure either.

## Verification

Tested on Pi 1.5.0,
running from a throwaway home with a local server speaking the OpenAI streaming format.
The server sent one chunk of its first response,
then destroyed the socket,
and answered every later request in full:

```ts
// a stand-in provider: the first response is cut mid-stream
response.write(chunk({ content: 'partial', role: 'assistant' }, null));
if (attempt <= cutsBeforeSuccess) {
  response.destroy();
}
```

With `retry.enabled` true the server saw two requests and the run finished:

```text
assistant_end:terminated
auto_retry_start:terminated
assistant_end:(no error)
auto_retry_end:true
```

With `retry.enabled` false the server saw one request and the run stopped at the cut:

```text
assistant_end:terminated
```

## Verified workarounds

Set `retry.enabled` to `true` in `~/.pi/agent/settings.json`.
This is the setting's documented default,
and the provider's advice.
A retried response is requested again from the start,
so the partial output is discarded and the attempt is billed again.
Pi makes three attempts by default,
and after the last one the run ends as it did before.

`retry.provider.maxRetries` stays at `0`,
as Pi's `docs/settings.md` recommends,
because provider-level retries can delay Pi from handling quota errors itself.

## What does not work

The Wi-Fi driver was the owner's first suspect and the evidence pointed away from it:
`ath12k_wifi7_pci` had been connected for 287,180 s with no radio lines in the kernel journal from 14:55Z to 17:15Z.
That does not rule it out on its own,
because this driver may not report retry and beacon counters.

The WireGuard tunnel cannot explain the earlier bursts:
`fr-par-fr1` appeared at 17:06:43Z,
after every burst but the last.

## Upstream filing decision

Nothing to file.

- Is it upstream's fault?
  No.
  Pi behaved as configured,
  and the cut happened at the network.
- Can upstream fix it?
  Pi already does,
  with the retry this setting turns on.
- Are they supporting this use case?
  Yes,
  `retry.enabled` is a documented setting that defaults to `true`.
- Would the repository welcome a contribution?
  Not checked,
  because there is nothing to contribute.
- Will they likely fix it?
  Does not apply,
  since nothing is broken in Pi.
- Have we prototyped a minimal fix?
  Does not apply for the same reason.
  The change was to this machine's setting.

`.out-of-scope/` and the tracker were not searched,
because no filing is proposed.
