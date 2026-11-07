# Showify 0.2.8 identifies Promise objects without revealing their internal state

## Affected teaching surface

The Promise lesson needs readable representations of actual objects,
arrays,
functions,
errors,
and Promise objects.
`JSON.stringify` is useful but is not a general debugger.
The user supplied `showify` through the workspace catalog for this work.

The tested unadorned native Promises stringify as `{}` whether pending,
fulfilled,
or rejected.
Showify 0.2.8 renders each as `Promise { <state unknown> }`.
That marker is a limitation of the formatter,
not an additional Promise state or evidence of pending work.
Ordinary enumerable properties added to a Promise remain separate from its internal outcome.

## Source boundary

[Showify release source][showify-source],
version `0.2.8`,
and commit `9a455119e0735ec2fc2a2de8ea14bb925fd52c38`
identify the read-only checkout at
`/var/home/user/temp/agent/promises-value-inspection/showify`.
The complete production source,
manifest,
Promise tests,
CI workflow,
and license files were inspected.

`src/index.ts:1267` explicitly supplies the marker:

```ts
// showify src/index.ts:1267
// Promise
if (value instanceof Promise) {
  extraEntries.push(text(c.special("<state unknown>")));
}
```

`test/promise.spec.ts:15` constructs a pending Promise;
line 17 expects `Promise { <state unknown> }`.
The consumer probe extends that check to fulfilled and rejected Promises.

The native Node inspector has a different boundary.
[Node `v26.8.1` source][node-source],
commit `7be6d3af31a65adea57c94c41e50c2b071ed0b3a`,
was inspected in a separate read-only checkout.
`src/node_util.cc:111` obtains engine-level state and result:

```cpp
// Node src/node_util.cc:111
int state = promise->State();
Local<Value> values[2] = { Integer::New(isolate, state) };
size_t number_of_values = 1;
if (state != Promise::PromiseState::kPending)
  values[number_of_values++] = promise->Result();
```

`lib/internal/util/inspect.js:2524`,
`formatPromise`,
uses that result:

```js
// Node lib/internal/util/inspect.js:2526
const { 0: state, 1: result } = getPromiseDetails(value);
if (state === kPending) {
  output = [ctx.stylize('<pending>', 'special')];
}
```

A browser formatter is not interchangeable with this privileged Node implementation.
The lesson must keep formatted text distinct from state observed by its own resolving functions and handlers.
Do not parse inspector text to drive application policy.

## Published entry versus development source

Directly bundling Showify's TypeScript source with Rolldown 1.2.7 failed in `proc_8dc2`:

```text
[PARSE_ERROR] Identifier `show` has already been declared
showify/src/index.ts:271:25
```

The source declares an ambient `show` function and separately exports its runtime constant.
The upstream `package.json` compile script erases declarations with `ts-blank-space` before publishing.
The normal published entry is JavaScript,
not this development-source entry.

The consumer probe therefore used the published `showify@0.2.8` artifact rather than patching upstream source.
`proc_69b7` fetched and checked its registry integrity:

- Archive:
  117,323 bytes;
  unpacked size:
  247,842 bytes.
- Registry `gitHead`:
  `9a455119e0735ec2fc2a2de8ea14bb925fd52c38`.
- Published `index.js` SHA-256:
  `a2e36462757588bfbdc6aae926dbc967bc91718fa2d8dbadb688c95faac2b745`.
- Published manifest has no runtime dependencies or lifecycle scripts.

The archive inventory contained ordinary files under `package/`,
with no links or traversal entries.
The published entry's Promise marker and hook handling match the inspected source.

## Verification

Harness:
`/var/home/user/temp/agent/promises-value-inspection/mise.toml`.
Tasks:
`fetch:showify`,
`build:showify-probe`,
and `probe:showify-browser`.
`proc_506a` passed Chromium 153.0.8010.12 and Firefox 155.0.
These are consumer-probe browser versions,
not a claim of Firefox ESR 140 integration coverage.

The generated IIFE measured 22,084 bytes.
It ran in a read-only,
network-disabled container with 2 GiB RAM,
2 CPUs,
256 process/thread entries,
bounded temporary filesystems,
and no ambient credentials.
No upstream build,
prepare,
benchmark,
or CI upload script was executed.

Working catalog:

- Objects and their selected string values retain distinguishable representations.
- Resolver bundles show Promise and resolving-function entries.
- Sparse arrays distinguish empty slots from JSON's `null` substitutions.
- Callback arrays identify functions instead of JSON's `null` entries.
- Errors retain their message and added fixture metadata.
- Circular values and BigInt have readable inspector representations.
- Pending,
  fulfilled,
  and rejected Promise objects all explicitly report unknown state.
- Formatting a rejected Promise does not attach a rejection handler;
  the probe still receives its expected `unhandledrejection` event.
- The probe's CSP forbids network access and unsafe evaluation;
  the exercised paths produced no CSP violations or page errors.

Limits and negative catalog:

- JSON serialization of a cycle and BigInt throws in the native probe.
- Top-level `JSON.stringify(undefined)` yields no JSON text.
- JSON cannot be used to infer these native Promises' state or value.
- `getters: 'none'` prevents ordinary getter expansion,
  but is not a blanket side-effect guarantee.
  The special `Symbol.toStringTag` getter was read once in both tested browsers.
- `callToJSON`,
  `callNodeInspect`,
  and `callCustomInspect` were explicitly disabled;
  the probe recorded zero calls to those custom hooks.

Results are preserved in `results/showify-browser-probe.json`.
The native comparison is in `native-inspection-probe.json`,
created by `proc_4981`.
That authored Node probe explicitly set `Error.stackTraceLimit = 0` to exclude host stack paths.

## Consumer configuration and execution environment

Use deliberate depth,
length,
and hook options in the teaching viewer.
Display strings as text rather than interpolating inspector output into HTML.
Use the program's existing observers and explicit public-property snapshots for outcome information.

The first container attempt reached Chromium but Firefox failed before library execution with
`Could not find profile folder.`
A private writable `HOME` and `XDG_CACHE_HOME` under the container's bounded `/tmp` resolved that launch failure.
The root filesystem remained read-only;
no host home directory was mounted.
This verifies a consumer-environment adjustment,
not a diagnosis of a Firefox engine defect.

## License and source notices

The published package is MPL-2.0.
`LICENSE` and `COMMERCIAL_LICENSE.md` were read.
Preserve its notices and provide the corresponding source when embedding the formatter.
It remains lesson infrastructure,
not a requirement for the learner's independent chat.

## Upstream filing decision

Nothing to file for the observed Promise marker or the use of the published JavaScript entry.

1.  Upstream fault:
    no. The Promise limitation is explicit;
    the public package entry works in the probe.
2.  Fixability:
    no upstream modification is required for the consumer route.
3.  Supported use:
    the README and Promise tests cover browser formatting and the unknown-state marker.
4.  Contribution welcome:
    not investigated because no report or patch is proposed.
5.  Likelihood of a fix:
    not applicable without an asserted upstream defect.
6.  Prototype:
    the bounded consumer probe demonstrates the integration boundary;
    no upstream patch was made.

No issue/comment draft or external filing was produced,
so no duplicate-report search was initiated.
This is a scoped API/integration investigation,
not a general ranking of formatter libraries.

[showify-source]: https://github.com/Snowflyt/showify/tree/9a455119e0735ec2fc2a2de8ea14bb925fd52c38
[node-source]: https://github.com/nodejs/node/tree/7be6d3af31a65adea57c94c41e50c2b071ed0b3a
