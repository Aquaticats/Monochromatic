# Node writeFile keeps data effects opaque across VFS and native boundaries

## Symptom

`prefer-readonly-parameter-type/prefer-readonly-parameter-types` reported opaque effects when private Git policy
storage passed `Uint8Array` values to `node:fs/promises` `writeFile` and `FileHandle.writeFile`.

The JavaScript implementation does not visibly assign into the supplied bytes.
That observation is insufficient to classify the calls as observational.

## Source audit

The audit used `Node.js` `v26.5.0` embedded module `internal/fs/promises`.
The embedded source digest is
`783a0f44a0689f4d4773f7202e3d0425defe9fe8b900ee125174ceba7fbb0bdf`.

The public `writeFile(path, data, options)` implementation first checks the configured VFS handler.
When present,
it calls `h.writeFile(path, data, options)` with the original values.
That dynamically installed handler is an opaque capability.

The default path calls `writeFileHandle`.
For custom iterables,
that function invokes asynchronous iteration and consumes each yielded value.
For an `ArrayBufferView`,
it creates successive `Uint8Array` views over the same buffer and passes them through the internal `write` boundary.
The JavaScript loop changes only local view variables,
but the native write implementation receives caller-owned storage.

`FileHandle.writeFile(data, options)` delegates through `fsCall` to the same `writeFile` implementation with the
handle as its path argument.
The handle receiver has its own reference lifecycle,
and data still crosses the VFS or native boundary.

## Resolution

The calls remain fail-closed.
Boundary contracts name `writeFile` or `handle.writeFile` and the exact byte provenance exposed to the boundary.
Generated snapshot callbacks and their enclosing preparation functions propagate the same uncertainty rather than
promoting native access to proven mutation.

This does not claim that Node normally changes the supplied bytes.
It records that shipped JavaScript alone cannot prove absence of caller-observable effects for every supported VFS,
iterable,
and native path.

## Verification

The audited implementation is `Node.js` `v26.5.0` (`internal/fs/promises`,
digest above).
The rule that reports these boundaries is `prefer-readonly-parameter-type` from
`@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type` `0.0.1`,
run by Oxlint `1.92.0` on `Node.js` `v26.13.0`,
against this repository at commit `ca82b5f39772a2a12e5893c228f3b9555e065bf3`.

The harness is the plugin's own fixture project,
`package/test-fixture/oxlint-no-restricted-syntax`,
whose `.oxlintrc.readonly.fixture.json` loads the built plugin and enables
`prefer-readonly-parameter-types`,
`no-opaque-parameter-effects`,
`no-readonly-parameter-mutations`,
and `no-invalid-parameter-effect-contracts`.
Rebuild the plugin with
`mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build`
so the bundle under test matches its source,
put each pattern below in a file under that fixture package's `src/`,
then lint it from the repository root:

```sh
oxlint --threads 1 --format json \
  -c package/test-fixture/oxlint-no-restricted-syntax/.oxlintrc.readonly.fixture.json \
  package/test-fixture/oxlint-no-restricted-syntax/src/<file>
```

The TypeScript call sites this record covers are no longer in the tree:
commit `642b6d72cb` (2026-10-13) deleted the TypeScript Git policy wrapper,
and the boundary contracts it carried went with it.
The rule and its fixtures remain,
so both outcomes below are still reproducible.

### Patterns that work cleanly

A boundary whose input is marked as runtime-owned host input and documents its effects returns
`"diagnostics": []`:

```ts
import { writeFile } from 'node:fs/promises';

import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Persists bytes owned by the runtime.
 *
 * @param path - destination path.
 *
 * @param bytes - runtime-owned storage whose write implementation stays uninspectable.
 *
 * @mutates bytes - The VFS handler or native write receives the storage and may change it.
 */
export async function persistBounded(
  path: string,
  bytes: ForeignHostCapability<Uint8Array>,
): Promise<void> {
  await writeFile(path, bytes,);
}
```

Reading primitive leaves from a deeply readonly object never reaches a callable boundary,
and reports nothing either;
the fixture's `readonly-valid.ts` is the shipped case.

### Patterns that fail

Each pattern below reports one `prefer-readonly-parameter-type(no-opaque-parameter-effects)`
diagnostic.
Its first line names the unresolved call and its site,
for example
`The function input named "bytes" is exposed to these unresolved calls: handle.writeFile [.../handle.ts:13].`
The closing paragraph is identical in every variant:
`An @mutates block alone documents known effects but cannot make an unresolved implementation safe.`

- `writeFile(path, bytes)` with a caller-owned `Uint8Array` and no contract names `writeFile`.
- `handle.writeFile(bytes)` through an open `FileHandle` names `handle.writeFile`.
- `writeFile(path, chunks)` with a caller-owned `AsyncIterable<Uint8Array>` names `writeFile`
  and marks the `chunks` parameter.
- The first two patterns with an `@mutates bytes` block alone still report the same diagnostic,
  which is the documented-uncertainty outcome this record rests on.

## Upstream filing decision

No upstream issue was filed.
Node's behavior and supported extension points are intentional.
The fail-closed effect requirement belongs to this repository's semantic policy.

## Sources

- [Node `fs/promises` implementation][node-fs-promises]
- [Node `fs/promises` API][node-fs-api]

[node-fs-promises]: https://github.com/nodejs/node/blob/v26.5.0/lib/internal/fs/promises.js
[node-fs-api]: https://nodejs.org/docs/latest-v26.x/api/fs.html#fspromiseswritefilefile-data-options
