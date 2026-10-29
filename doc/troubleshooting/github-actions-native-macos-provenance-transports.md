# Node 26.11.1 provenance fetch and artifact action 4.6.2 time out after native macOS consumers

## Symptom

GitHub run `39028181199`,
first-attempt macOS job `117383212664`,
completed its source consumers but failed to retain paired provenance.
These are distinct incidents with different emitting tools and endpoints.
A shared transport cause has not been established.

The after-provenance task used Node.js `v26.11.1` and reported at `2026-10-17T12:01:44Z`:

```text
TypeError: fetch failed
ConnectTimeoutError: Connect Timeout Error (attempted address: api.github.com:443, timeout: 10000ms)
code: UND_ERR_CONNECT_TIMEOUT
```

The artifact action separately exhausted its CreateArtifact requests:

```text
Failed to CreateArtifact: Failed to make request after 5 attempts:
Request timeout: /twirp/github.actions.results.api.v1.ArtifactService/CreateArtifact
```

It found the available before-provenance file but published no macOS archive.
The Windows row retained its own complete provenance pair.
Source Clippy and nextest failures are separate from both transport incidents.
Repository issues `#1175` and `#1176` retain those distinct operational boundaries.

## Source trace and attribution limits

### The provenance helper has separate request and connection bounds

At frozen harness `467dc0002d6080f8aa951f488430450debf3df3c`,
`package/monorepo-task/core-exec/mise.toml:219` defines `readContentsApi`.
Its call at line `223` supplies a 30-second request signal:

```javascript
// package/monorepo-task/core-exec/mise.toml:223, frozen native harness
const response = await fetch(url, {
  headers: {
    Accept: 'application/vnd.github.object+json',
    'User-Agent': 'Monochromatic-Depot-Trial',
    ...(token ? { Authorization: 'Bearer ' + token } : {}),
  },
  signal: AbortSignal.timeout(30000),
});
```

The excerpt expands the unchanged object expression across lines.
It contains credential variable names without credential values.
The authenticated fetch,
decoded-byte checks,
and common-manifest binding checks precede the phase-record write.
The record write is at `package/monorepo-task/core-exec/mise.toml:265`.
The failed fetch therefore leaves no after record.
Further source assertions follow the record write.

The source clone at Node tag `v26.11.1`,
commit `364593d0631def13e3043dd84f77ebca2c767ce5`,
is unmodified.
Its `deps/undici/src/lib/core/connect.js:69` defines the connector default:

```javascript
// deps/undici/src/lib/core/connect.js:69, nodejs/node v26.11.1
timeout = timeout == null ? 10e3 : timeout;
```

`deps/undici/src/lib/core/util.js:871` constructs the connection diagnostic
and destroys the socket with that error:

```javascript
// deps/undici/src/lib/core/util.js:884, nodejs/node v26.11.1
message += ` timeout: ${opts.timeout}ms)`;
destroy(socket, new ConnectTimeoutError(message));
```

The error class assigns its stable code at `deps/undici/src/lib/core/errors.js:26`:

```javascript
// deps/undici/src/lib/core/errors.js:26, nodejs/node v26.11.1
this.code = 'UND_ERR_CONNECT_TIMEOUT';
```

The fetch boundary wraps its network error at `deps/undici/src/lib/web/fetch/index.js:272`:

```javascript
// deps/undici/src/lib/web/fetch/index.js:272, nodejs/node v26.11.1
p.reject(new TypeError('fetch failed', { cause: response.error }));
```

The observed ten-second connection bound is distinct from the helper's 30-second signal.
The diagnostic does not identify DNS,
routing,
TLS,
or a provider outage as the cause.

### The artifact client exhausts its own retry loop

The provider setup log resolves the actual action to
`actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`.
The unmodified `v4.6.2` source clone resolves to that same commit.
The shipped bundle sets the default attempt count at `dist/upload/index.js:2731`:

```javascript
// dist/upload/index.js:2731, actions/upload-artifact v4.6.2
this.maxAttempts = 5;
```

The request timeout message comes from `dist/upload/index.js:12587`:

```javascript
// dist/upload/index.js:12587, actions/upload-artifact v4.6.2
handleResult(new Error(`Request timeout: ${info.options.path}`));
```

The retry loop raises its final error at `dist/upload/index.js:2811`:

```javascript
// dist/upload/index.js:2811, actions/upload-artifact v4.6.2
if (attempt + 1 === this.maxAttempts) {
  throw new Error(`Failed to make request after ${this.maxAttempts} attempts: ${errorMessage}`);
}
```

The retained provider log contains the failed attempts and backoff messages.
It does not establish artifact quota exhaustion,
authentication rejection,
or a defect in the uploaded source inputs.

## Verification

The following historical-log command was executed successfully on October 17.
It returned 599,998 bytes containing both exact diagnostics:

```sh
# Repository workflow log retrieval, no provider mutation
gh run view 39028181199 --repo Aquaticats/Monochromatic \
  --attempt 1 --job 117383212664 --log-failed
```

The first-attempt workflow-log archive was downloaded and hashed before the retry.
The [GitHub gate evidence](../audit/depot-coherent-github-gates-2026-10-17.evidence.json)
retains the two failed step logs,
frozen source/common context,
and retry request.

Working observations include the same run's Windows source/common pair,
the desktop macOS pair,
and every scanner and Scorecard gate pair in the cohort.
The failing catalog contains the native macOS after fetch and the independent artifact creation timeout.
No synthetic outage or unsupported assertion about the historical network path is substituted for those observations.

## Recovery and tradeoffs

Only native macOS was requested again with its unchanged frozen harness,
source,
and common inputs:

```sh
# Executed unchanged-input recovery request
gh run rerun 39028181199 --repo Aquaticats/Monochromatic --job 117383212664
```

The request was accepted and attempt two completed.
macOS job `117393630494` verified equal source/common before and after records
and published artifact `12101426954` with a matching provider digest.
All 1382 observed test identities equal the first attempt's identity set,
with a dropped-identity sensitivity control.
The retry recorded 1356 passes,
26 known failures,
and one summary skip whose identity was not printed.
The decode-cache case tracked by `#1178` passed on this unchanged-source repeat;
the first-attempt failure remains valid evidence and the issue stays open.
Re-execution consumes another full job allocation and adds its actual orchestration gap to the critical path.
The original attempt remains retained and has no paired-provenance completion credit.
The successful transport repeat establishes recovery for attempt two.
It does not establish the cause or permanent resolution of either transport incident.
Issues `#1175` and `#1176` remain open.

The second-attempt jobs API also lists a Windows row under new ID `117393631745`.
Its start and completion times and every step record equal original job `117383212796`.
Its provenance artifact is the original `12099718026`,
and the attempt-two log archive contains only macOS logs.
This Windows row carries the original execution forward.
Allocation accounting must preserve that alias without counting another Windows execution.

## What does not work

The 30-second AbortSignal did not prevent the separately observed connection timeout.
Treating the successful before record as an after record would omit the consumer boundary.
Treating a successful source immutability step as a published artifact would omit the evidence transport boundary.
Combining the two endpoint errors into a single provider-outage claim is unsupported.
The workflow's existing artifact retries were already exhausted;
this incident is not evidence that the action made no retry.

## Upstream filing decision

The `.out-of-scope/` search for GitHub,
Undici,
and artifact topics found no matching exemption.
No upstream submission is proposed:

1.  Fault attribution remains unknown.
    The logs establish transport failures,
    not an upstream software defect.
2.  Fixability of a supposed upstream defect is not established.
    The repository can preserve failed-attempt evidence and exercise unchanged-input recovery.
3.  Both authenticated Contents API access and artifact publication are intended workflow operations.
4.  Contribution eligibility was not investigated because no upstream defect or submission is established.
5.  No maintainer willingness or refusal is inferred.
6.  No upstream patch is claimed.
    The actual repository recovery request and verified attempt-two source/common pair are retained separately.

There is no substantiated upstream defect report or additive upstream comment to file from this evidence.
The repository incidents stay separate and open after the verified transport recovery.
