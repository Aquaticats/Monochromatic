# pi-plugin-neuralwatt loses a resolved credential when one sops store read times out

The NeuralWatt pi extension resolved its API key through a fresh
sops/age decryption of `.env.local.json` on every request.
Under host pressure one decryption hit the reader's 15 s bound,
the resolution answered "no credential",
and pi ended a working session with `Provider is not configured: neuralwatt`.
The fix is a per-process memory of the store read in
`package/pi-plugin/neuralwatt/src/credential-memory.ts`.

Versions under test:
`@earendil-works/pi-ai` 1.5.0,
`@earendil-works/pi-coding-agent` 1.5.0,
sops from the repository toolchain (the `sops` file in
`~/.cache/toolchain-bin`),
`@monochromatic-dev/env-local-json-core` at commit `3414461d99`,
and `@monochromatic-dev/pi-plugin-neuralwatt` at the commit that added
`src/credential-memory.ts`.

## Symptom

An agent 142 turns into a run ended with these records in its `stderr.log`:

```text
[warn] [pi-plugin-neuralwatt] [createNeuralWattApiKeyAuth] [resolveNeuralWattApiKey] encrypted store read failed, falling through to the environment: sops could not decrypt env store /var/mnt/encrypted/repository/Monochromatic/.env.local.json (timeout)
[info] [pi-plugin-neuralwatt] [createNeuralWattApiKeyAuth] [resolveNeuralWattApiKey] No NeuralWatt credential resolved, so requests will fail until one lands.
```

The model call then failed with `Provider is not configured: neuralwatt`,
and the run ended with `endReason: model-error`.
The same process had already resolved the credential hundreds of times.
Sixteen agents shared the host.

## Root cause

The chain has four links:
pi asks for auth per request,
the extension re-reads the store per resolution,
the reader bounds one sops invocation at 15 s,
and a failed read counted as "no credential in this process".

### pi resolves auth once per request and twice per availability refresh

`@earendil-works/pi-ai` 1.5.0
`node_modules/@earendil-works/pi-ai/dist/models.js:346` resolves auth in
`getAuth`,
which every streamed request reaches:

```js
    async getAuth(providerOrModel, overrides) {
        ...
        const result = await resolveProviderAuth(provider, this.credentials, this.authContext, { ...overrides, signal });
```

`dist/models.js:253` resolves again for a provider catalog refresh,
and `dist/auth/resolve.js:126` is the provider-facing call:

```js
        return await apiKey.resolve({ ctx: authContext, credential, signal });
```

`checkAuth` resolves too when the provider carries no `check` callback,
and the model runtime runs both `getAvailable` and `checkAuth` for every
provider on an availability pass,
at
`node_modules/@earendil-works/pi-coding-agent/dist/core/model-runtime.js:192-197`:

```js
        const [available, checks, credentials] = await Promise.all([
            this.models.getAvailable(undefined, { signal }),
            Promise.all(providers.map(async (provider) => [
                provider.id,
                await this.models.checkAuth(provider.id, { signal }),
            ])),
```

Measured with `createModels` from the same pi-ai build and the package's
provider over a counting store-reader seam:
each of `checkAuth`,
`getAuth`,
and `getAvailable` cost one store read,
a prompt preflight plus one request cost two,
and one availability pass cost two.
A 142-turn session therefore ran several hundred decryptions.

### The reader bounds one decryption at 15 s

`package/env-local-json/core/src/constants.ts:51`:

```ts
export const SOPS_TIMEOUT_MS = 15_000;
```

`package/env-local-json/core/src/sops.ts:295` turns that into an
`AbortSignal.timeout` and a `timeout` option for the child process,
and the failure class reaches the caller as
`sops could not decrypt env store <path> (timeout)`.

One warm decryption of the repository store costs 17.6 to 23.4 ms
(eleven reads across two runs),
and the first read in a fresh process 1.1 s.
A 15 s window is therefore roughly seven hundred times the warm cost:
reaching it means the host was badly starved,
which is the normal condition this repository already documents
(`AGENTS.md` rule `NHR`).

### A failed read was a lost credential

`package/pi-plugin/neuralwatt/src/credential.ts` at the failing commit
caught the read failure and answered with an empty store result,
so the resolution chain fell through to the environment,
found nothing,
and returned `source: 'none'`.
`resolve` then answered `undefined`,
and pi treats an undefined api-key resolution as an unconfigured provider.

## Verification

The harness drives the two halves separately,
so no run needs a real secret to reproduce the loss.

Counting resolutions through the store-reader seam
(`package/pi-plugin/neuralwatt/src/credential.unit.test.ts`,
suite `createNeuralWattApiKeyAuth`):

- A key resolved once,
 then a store read that rejects:
 before the fix the resolution answered `undefined`
 (`expected undefined to equal 'store-key-1'`),
 and after the fix it answers the remembered key with one read.
- A first read that rejects,
 then answers:
 before the fix the resolution failed outright,
 and after the fix it retries and resolves the key.
- Two concurrent resolutions:
 before the fix the seam made two store reads,
 and after the fix one.

Timing the real decryption,
with the built `@monochromatic-dev/env-local-json-core`.
Run this file as `measure-decrypt.ts` with
`node --experimental-strip-types measure-decrypt.ts 6`;
it prints durations,
the loaded flag,
and whether a value came back,
never the value.

```typescript
// measure-decrypt.ts
import {
  readEnvLocalJson,
} from '<repo>/package/env-local-json-core/dist/final/node/index.mjs';

const KEY = 'NEURALWATT_API_KEY';
const storePath = '<repo>/.env.local.json';
const runs = Number(process.argv[2] ?? '5');

const samples = [];
for (let index = 0; index < runs; index += 1) {
  const started = process.hrtime.bigint();
  const result = await readEnvLocalJson({
    keys: [KEY],
    path: storePath,
    env: process.env,
  });
  samples.push({
    ms: Number(process.hrtime.bigint() - started) / 1e6,
    loaded: result.loaded,
    gotValue: typeof result.values[KEY] === 'string',
  });
}

const round = function round(value) { return Math.round(value * 10) / 10; };
const sorted = samples.map(function duration(sample) { return sample.ms; }).sort(function ascending(a, b) { return a - b; });
console.log(JSON.stringify({
  runs,
  ok: samples.length,
  loadedAll: samples.every(function loaded(sample) { return sample.loaded; }),
  gotValueAll: samples.every(function valued(sample) { return sample.gotValue; }),
  perRunMs: samples.map(function perRun(sample) { return round(sample.ms); }),
  minMs: round(sorted[0]),
  medianMs: round(sorted[Math.floor(sorted.length / 2)]),
  maxMs: round(sorted[sorted.length - 1]),
}, null, 2));
```

That run,
on 2026-11-09,
printed:

```json
{ "runs": 6, "ok": 6, "loadedAll": true, "gotValueAll": true,
  "perRunMs": [23.4, 19.6, 18.9, 18.5, 17.6, 18.6],
  "minMs": 17.6, "medianMs": 18.9, "maxMs": 23.4 }
```

The first read of that harness in a fresh process measured 1125.8 ms,
against 20.4 to 21.4 ms for the following four reads.

Driving the built extension through pi-ai's own `Models` class against the
real store,
after the fix,
six operations (an availability pass,
a prompt preflight,
three requests)
caused exactly one decryption,
and every later resolution took 0.0 to 0.1 ms:

```json
{ "decryptions": 1,
  "report": [
    { "step": "availability pass: getAvailable", "decryptions": 1, "ms": 129.6 },
    { "step": "availability pass: checkAuth", "decryptions": 0, "ms": 0.1 },
    { "step": "prompt preflight: checkAuth", "decryptions": 0, "ms": 0 },
    { "step": "request 1: getAuth", "decryptions": 0, "ms": 0.1 },
    { "step": "request 2: getAuth", "decryptions": 0, "ms": 0 },
    { "step": "request 3: getAuth", "decryptions": 0, "ms": 0 } ] }
```

## Verified fix

`package/pi-plugin/neuralwatt/src/credential-memory.ts` wraps the store read:

- `CREDENTIAL_REVALIDATION_INTERVAL_MS` (60 s) is the window one read stands
 for,
 and inside it a resolution decrypts nothing.
- Past the window the remembered key is still served,
 and the store is read behind the request,
 so no request waits on sops.
- A failed read keeps the last good key,
 and logs `keeping the NeuralWatt credential resolved earlier`.
- Concurrent resolutions share one in-flight read.
- A cold read is attempted `STORE_READ_ATTEMPTS` (2) times inside
 `STORE_READ_DEADLINE_MS` (32 s),
 which is two 15 s sops windows plus a 250 ms retry delay,
 so one stalled window delays a request instead of ending the session.

Tradeoffs,
stated:
a key rotated in the store reaches the process within the revalidation
window rather than at the next request,
and a key deleted from the store keeps being served until a window passes
with a failed read.

## What does not work

- Reacting to a rejected key through pi's `onResponse` callback.
 `node_modules/@earendil-works/pi-ai/dist/api/openai-completions.js:198-203`
 calls the callback only after `withResponse()` returns,
 and a 401 rejects that call,
 so the callback never sees the status.
 A test asserting a rejection report over a 401 failed with
 `expected [] to deeply equal [ 'unauthorized' ]`,
 and the callback is removed rather than kept as dead code.
- Raising `SOPS_TIMEOUT_MS`.
 A warm decryption measures about 20 ms,
 so 15 s is already hundreds of times the work,
 and a starved host can starve a longer window as easily.
- Caching inside `@monochromatic-dev/env-local-json-core`.
 Its documented contract is a stateless reader that returns requested keys
 and never reads a store whose keys the environment already carries,
 so a cache there would make every consumer serve stale secrets.
- A lock or a shared cache file across the sixteen agents.
 The cost is per process,
 and the process that resolved the key is the one that can remember it.

## Upstream filing decision

Nothing to file:
the defect is in this repository's extension,
not in sops or pi-ai.

Walked through the six constraints:

1.  Is it upstream's fault?
 No.
 The extension asked for a decryption per request,
 while sops timing out under host starvation and pi-ai resolving per
 request are both documented behavior.
2.  Can upstream fix it?
 Not applicable.
3.  Does upstream support this use case?
 Not applicable,
 since `ApiKeyAuth.resolve` is documented as request-time work,
 which the extension was performing too literally.
4.  Would the repository welcome a contribution?
 Not applicable.
5.  Will they fix it?
 Not applicable.
6.  Minimal prototype?
 The prototype is the shipped fix in this repository.

`.out-of-scope/` was not consulted,
because no upstream filing is proposed.
