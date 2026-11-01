# PostCSS traversal and node methods need audited receiver and callback effects

## Symptom

Type-aware Oxlint reported opaque effects in `package/css/cli` for PostCSS `8.5.16` calls,
including `Container.walkAtRules`,
`Node.remove`,
`Node.replaceWith`,
`Node.error`,
`Node.toString`,
and `AtRule.clone`.

The declarations identify callable types,
but declarations and runtime probes do not establish caller-observable effects.

## Source audit

The audit used PostCSS tag `8.5.16`,
commit `92ccc93ff15bd193491d67fad9763e62d489dfad`.

Audited files:

- [`lib/container.js`][postcss-container],
  digest `0f8aaa013a910e142be706c3d6f54a3ce04751a08df3f17ed3a61bb91f863c39`;
- [`lib/node.js`][postcss-node],
  digest `52c8d992e881f0d40d3dc4610039d3cc19b1dfdf0fc32aef8923b5537d161eae`;
- [`lib/stringify.js`][postcss-stringify],
  digest `99229cdda200513f88f7fee46004d5c39c1492a73451509e66164d26dff34abc`.

`Container.walkAtRules` delegates to `walk` and `each`.
`each` allocates iterator state on the receiver,
updates it while invoking the selected callback with receiver children,
and deletes the active index afterward.
The `lastEach` counter remains changed.
A two-argument selector can be a `RegExp`;
its `test` call can change `lastIndex`,
so that selector position remains opaque rather than being classified as proven mutation for every selector type.

`Node.remove` changes its parent container and receiver parent link.
`Node.replaceWith` also adopts supplied nodes through parent insertion methods.
`Node.error` reads receiver-reachable input state and options while constructing an error,
but does not directly change the receiver.
`Node.toString` invokes the selected stringifier over receiver state,
but does not directly change the receiver.

`Node.clone` walks enumerable receiver and override state,
invokes constructors and property reads,
recursively copies nested values,
and preserves the original `source` reference in the result.
It does not directly change the receiver.
Constructor calls,
accessors,
and proxy traps remain caller-observable uncertainty.
The implementation intentionally preserves only `source` identity,
but current result-origin modeling is not property-sensitive,
so the clone result conservatively retains receiver-derived provenance.

## Implementation

`postcss-package-effect-catalog.ts` records exact package-major and declaration-owner entries:

- `Container_.walkAtRules` affects the receiver and invokes a callback with receiver children;
- `AtRule_.clone` has opaque receiver and override observation,
  and returns a value retaining receiver provenance;
- `Node_.error` has opaque receiver and options observation;
- `Node_.toString` has opaque receiver observation and invokes a supplied stringifier;
- `Node_.remove` changes its receiver;
- `Node_.replaceWith` changes the receiver and supplied nodes.

PostCSS overloads place the callback at argument `0` for a one-argument call
and argument `1` for a two-argument call.
`IntrinsicCallbackEffect.callArgumentCount` selects the matching relation,
so a selector string is not misclassified as a callback while the callback-only overload remains covered.

## Verification

The package effect tests verify exact PostCSS declaration identities,
major-version gating,
clone result provenance,
and callback positions for each supported arity.

After rebuilding the semantic plugin,
this command reported no diagnostics:

```text
OXLINT_THREADS=1 mise run //package/css/cli:lint:oxlint
```

## Upstream filing decision

No upstream report is warranted.
PostCSS behavior matches its implementation and documented mutation-oriented API.
The missing information belonged in this project's semantic effect catalog.

[postcss-container]: https://github.com/postcss/postcss/blob/92ccc93ff15bd193491d67fad9763e62d489dfad/lib/container.js
[postcss-node]: https://github.com/postcss/postcss/blob/92ccc93ff15bd193491d67fad9763e62d489dfad/lib/node.js
[postcss-stringify]: https://github.com/postcss/postcss/blob/92ccc93ff15bd193491d67fad9763e62d489dfad/lib/stringify.js
