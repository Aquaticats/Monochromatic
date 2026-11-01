# WHATWG `TextEncoder.encode` is observational while `TextDecoder.decode` updates decoder state

## Symptom

`prefer-readonly-parameter-type/prefer-readonly-parameter-types` initially treated
`encoder.encode(text)` as an unresolved method effect.
The call is observational,
but a full `Readonly<TextEncoder>` projection remains unsupported because it also retains mutating `encodeInto`.
`new TextDecoder(...).decode(bytes)` also initially remained unresolved even though it does not change `bytes`.
Unlike encoder `encode`,
 decoder `decode` operates on decoder state,
so its exact effect belongs on the receiver rather than the input buffer.

## Root cause

TypeScript 7 declares `TextEncoder.encode(input?: string): Uint8Array<ArrayBuffer>` in the platform-specific
`lib.dom.d.ts` bundled by `@typescript/typescript-<platform>@7.0.2`.
The declaration alone does not describe receiver effects,
so the semantic rule correctly failed closed until the exact platform method was audited.

The [WHATWG Encoding Standard][encoding-standard] commit
`a985b62a9b45c17da3e17a9f0a0b4e30c34c4a8a` specifies that `encode`:

- converts primitive string input to a scalar-value queue;
- runs a fresh UTF-8 encoder instance;
- creates and returns a new `Uint8Array`;
- does not update an associated state slot on the `TextEncoder` receiver.

This differs from `encodeInto(source, destination)`,
which writes into caller-supplied `destination` and therefore cannot share an empty-target catalogue entry.

The same Encoding Standard specifies `TextDecoder.decode` through the receiver's decoder,
I/O queue,
BOM-seen flag,
and do-not-flush state.
The operation does not write the supplied `AllowSharedBufferSource`,
but it does update receiver-owned decoding state.
A fresh decoder expression therefore carries no caller origin,
while a decoder received from a caller is an affected capability.

## Verified resolution

`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/intrinsic-effect-catalog.ts`
now records exact DOM provenance:

- Owner:
  `TextEncoder`.
- Member:
  `encode`.
- Mutation targets:
  none.
- Evidence:
  WHATWG Encoding Standard commit `a985b62a`.

The same catalogue records:

- Owner:
  `TextDecoder`.
- Member:
  `decode`.
- Mutation target:
  receiver.
- Evidence:
  WHATWG Encoding Standard commit `a985b62a`.

Semantic tests resolve the actual TypeScript 7 declaration to that owner and member.
Effect-summary coverage proves exact `TextEncoder.encode` plus primitive text produces no caller-observable mutation.
The TOML conformance decoder now accepts `new TextDecoder('utf-8', { fatal: true }).decode(bytes)` because the
receiver is fresh and only the receiver effect is catalogued.
Classifier coverage separately proves a full `Readonly<TextEncoder>` remains unsupported while
`Pick<TextEncoder, 'encode'>` is sound.
Same-named methods on other owners do not inherit this treatment.

## Authoring guidance

Use `Pick<TextEncoder, 'encode'>` when a function only needs fresh byte arrays.
Do not use `Readonly<TextEncoder>` as a shortcut because it retains `encodeInto`.
Do not catalogue `TextEncoder.encodeInto` as observational:
its destination argument is deliberately mutated and needs an exact argument-target effect.
Instantiate `TextDecoder` inside the ownership-known boundary when decoder state must not escape.
A caller-supplied decoder requires a capability contract because `decode` can change its streaming state.

## Verification

The installed TypeScript declaration source is TypeScript `7.0.2`.
The focused verification commands are:

```sh
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
mise run //package/toml-edit/core:lint:oxlint
```

The plugin type and Oxlint tasks pass.
The complete unit suite passes after the callback-effect assertion update,
including the intrinsic catalogue checks.
The TOML lint no longer reports `TextDecoder.decode(bytes)`.

## What does not work

- An empty target list for `TextDecoder.decode` hides caller-owned streaming decoder state.
- An argument target for `bytes` falsely claims the input buffer changes.
- `Readonly<TextDecoder>` preserves behavioral methods and does not prove observational semantics.
- Matching the member name `decode` without exact `TextDecoder` owner and DOM provenance can bless unrelated methods.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    No. TypeScript declarations do not encode effects,
   and WHATWG decoder state transitions are intended behavior.
2. **Can upstream fix it?**
    No specification defect was identified.
3. **Are they supporting this use case?**
    WHATWG specifies runtime decoding,
   not static caller-effect metadata.
4. **Would the repo welcome our contribution?**
    Not applicable because no upstream defect exists.
5. **Will they likely fix it?**
    Not applicable.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    The consumer-side exact-owner catalogue
   entry and tests are the verified fix.

Nothing should be filed upstream.

[encoding-standard]: https://encoding.spec.whatwg.org/#dom-textencoder-encode
