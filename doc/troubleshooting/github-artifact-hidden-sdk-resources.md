# GitHub `upload-artifact` at `043fb46d` omits compiled SDK dot resources by default

## Symptom

The frozen-source native host build succeeded,
but its published artifact omitted a resource explicitly indexed by the producer.
The boundary was measured in GitHub Actions run `39119985290`,
using `actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`.

The producer indexed
`artifacts/package/pi-plugin/auto-mode/dist/native-release-E6DmsJ/pi/ai/dist/providers/data/.manifest.json`,
3,762 bytes with SHA-256 `a4c0a81f2cc4323b81d4c5d3886b343dc226939168f33e8eb731c928a81cfce9`.
That exact path was absent from the actual ZIP.
Every one of its 33,912 actual members was independently decompressed and hashed;
a deliberately corrupted extracted fixture was rejected.

This establishes an output-membership gap.
It does not establish that a runtime consumer needs that resource or fails without it.
The evidence and verification implementation are retained in
[`depot-complete-coverage-node-host-receipts.json.gz`](../planning/depot-complete-coverage-node-host-receipts.json.gz).

## Root cause

The exact uploader source was fetched in a disposable checkout of
[`actions/upload-artifact`](https://github.com/actions/upload-artifact)
at commit `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`.
The integration omitted the hidden-file input.
The action's `action.yml:43` declares its default:

```yaml
# action.yml:43
include-hidden-files:
  description: >
    If true, hidden files will be included in the artifact.
    If false, hidden files will be excluded from the artifact.
  default: 'false'
```

`src/upload/input-helper.ts:12` reads that boolean input:

```typescript
// src/upload/input-helper.ts:12
const includeHiddenFiles = core.getBooleanInput(Inputs.IncludeHiddenFiles)
```

`src/upload/upload-artifact.ts:27` passes it to file discovery:

```typescript
// src/upload/upload-artifact.ts:27
const searchResult = await findFilesToUpload(
  inputs.searchPath,
  inputs.includeHiddenFiles
)
```

`src/shared/search.ts:82` creates the globber with the selected options:

```typescript
// src/shared/search.ts:82
export async function findFilesToUpload(
  searchPath: string,
  includeHiddenFiles?: boolean
): Promise<SearchResult> {
  const searchResults: string[] = []
  const globber = await glob.create(
    searchPath,
    getDefaultGlobOptions(includeHiddenFiles || false)
  )
```

`src/shared/search.ts:14` explicitly selects the exclusion:

```typescript
// src/shared/search.ts:14
function getDefaultGlobOptions(includeHiddenFiles: boolean): glob.GlobOptions {
  return {
    followSymbolicLinks: true,
    implicitDescendants: true,
    omitBrokenSymbolicLinks: true,
    excludeHiddenFiles: !includeHiddenFiles
  }
}
```

An explicitly named hidden ancestor and a genuinely hidden child are different boundaries.
Successful upload from a receipt directory inside a dot directory does not prove that
its descendant `.manifest.json` files are included.
No unsupported-hidden-files or uploader-defect claim follows from this result.

## Verification

The failing catalog is the indexed SDK resource omitted from the actual original ZIP.
The working catalog is the original ZIP's 33,912 independently verified actual members.
The archive verifier also rejects an altered extracted file and accepts its restored bytes.
These controls validate membership and byte comparison,
not a restored application's runtime behavior.

A separate immutable follow-up completed as run `39124269347`,
harness `28bf38b0281f3bcace40b70d8b989d6e0ef7f875`,
against the same application source `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
It explicitly included hidden files.
Its producer passed all 44 owning invocations;
the fresh consumer verified all 1,036 declared compiled file hashes,
including the previously absent 3,762-byte resource,
and restored all 33,802 ZIP members without rebuilding application source.
The actual first domain import then failed with:

```text
SyntaxError: The requested module '.../runner.js' does not provide an export named 'nativeAssertActive'
```

This is a distinct restored topology boundary;
it does not retract the verified hidden-resource inclusion.

### Materialized symlinks change module identity and dependency resolution

The frozen application source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Its `package/pi-plugin/auto-mode/native/build-activation.ts:43` activates an immutable release by symlink:

```typescript
// package/pi-plugin/auto-mode/native/build-activation.ts:43
const link=`${output}.link`;
await symlink(basename(output),link,'dir');
```

The pinned uploader's `followSymbolicLinks: true`,
quoted in the root-cause section,
collects regular bytes through that link.
The measured ZIP extraction restores `dist/native` as a directory.
The frozen `package/pi-plugin/auto-mode/native/contract/integration/current-sdk/paths.ts:28`
derives the loader root from the loaded module URL:

```typescript
// package/pi-plugin/auto-mode/native/contract/integration/current-sdk/paths.ts:28
export const nativeRoot: string = fileURLToPath(new URL(
  '../../../',
  import.meta.url
));
```

`package/pi-plugin/auto-mode/native/loader.ts:51` maps exact original module URLs,
and `loader.ts:145` looks up the resolved URL:

```typescript
// package/pi-plugin/auto-mode/native/loader.ts:145
const hostModule = hostModules.get(resolved.url);
```

The flattened loader root produces a `dist/native/pi/coding-agent/.../runner.js` key,
while the generated emitter requests `dist/native-release-rIbI0D/pi/coding-agent/.../runner.js`.
The corrected measured probe records unequal URLs before activation and equal URLs afterward.
The same runner bytes then expose the maintained `nativeAssertActive` export through the loader.
This excludes SDK runner byte drift as the cause of the changed namespace.

Activation alone is insufficient.
In run `39126088565`,
it advances the actual owner to `Error: Cannot find module 'which'` from `cross-spawn`.
The frozen `package/pi-plugin/auto-mode/native/build-emission.ts:176`
links each emitted package to its installed dependency graph:

```typescript
// package/pi-plugin/auto-mode/native/build-emission.ts:176
await linkDependencies({
  target: join(target, 'node_modules'),
  source: dirname(dirname(installed)),
});
```

The artifact materializes those dependency trees too.
Run `39126304854`,
harness `bb659f0f7fea71de9a2ba850bd04c5d0919fcb8e`,
preserves those directories,
restores the exact source-owned dependency-link formula for coding-agent,
agent,
and AI,
and invokes the actual source activation function.
No application source is built and no unit suite runs in that consumer.
Permission and prohibition then pass with zero HTTP/provider requests;
`none` reaches the retained source assertion,
actual `unresolved` versus expected `unspecified`.
The observed namespace has `nativeAssertActive`,
`nativeCreateContext`,
and `nativeEmitError` only after the corrected topology.

## Verified workarounds

Explicit hidden-file inclusion is membership-qualified by run `39124269347`.
For this native artifact,
restoring the active-release and installed-dependency symlinks is additionally runtime-qualified
through permission and prohibition by run `39126304854`.
The source's `none` assertion remains red.
The immutable control workflow records the runnable restoration and actual owner invocation;
its six commands retain the flattened negative control and activated positive control.
Generated absolute import locations restrict this result to the measured GitHub workspace layout.
This does not qualify arbitrary relocation,
another OS,
or a dependency-free artifact.
Broad hidden-file inclusion also requires keeping the upload path restricted to reviewed outputs;
enabling it for a whole checkout could include unrelated files.

## What does not work

- Treating artifact-created metadata as proof that every indexed output is present.
- Treating an explicit hidden ancestor as proof that hidden descendants are included.
- Silently dropping missing producer declarations from a verifier.
- Calling the omitted resource a runtime requirement before exercising that consumer boundary.
- Restoring regular bytes alone while dropping source-owned activation and dependency-link topology.
- Treating activation alone as sufficient:
  it removes the missing-export boundary but exposes the independent missing-dependency boundary.
- Crediting the first topology run's namespace probes:
  their `.mjs` spelling for `paths.ts` was a harness error,
  corrected and rerun in `39126304854`.

## Upstream filing decision

Nothing to add upstream for the observed default exclusion:
the integration omitted an input whose false default is explicit in the pinned action source.

- Upstream fault is not established;
  the measured boundary matches the declared input behavior.
- Upstream can change the filter,
  but the integration already has a documented control.
- No upstream change proposal is necessary for the measured omission.
- The failure is reproducible as an artifact-membership comparison,
  while the separate restored topology controls qualify the stated native scenarios.
- The repository-side explicit-input and topology controls run without changing production workflows.
- No issue or comment was filed,
  and no defect-only duplicate search is represented as completed.

## A hidden receipt root also needs explicit inclusion

Current-source public run `39141147245` used the same pinned uploader
with the path `.coverage-host-evidence` and omitted `include-hidden-files`.
It emitted `No files were found with the provided path: .coverage-host-evidence`
and uploaded no artifacts.
The actual pinned deployed bundle, SHA-256 `eea594941d8ee535974e0fbc03bbdf567f3abc78194f224b93f2df9a887ee2e9`,
was fetched from its immutable official commit.
A bounded local Node26.12 control exposed its unchanged `findFilesToUpload` function
by removing only the upload startup and exporting that function.
It found zero files in the hidden root with default exclusion,
and four files including the child `.manifest.json` with explicit inclusion.
No upload or network request ran in this control.
The correction restricts uploads to that reviewed receipt directory,
sets `include-hidden-files: true`, and makes missing files an upload error.
The first public run remains failed and has no case-level artifact credit.
