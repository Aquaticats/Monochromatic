# Rolldown 1.2.13 rejects frozen import-attributes tests with a missing fixture directory

## Symptom

On application source `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`,
all four cases in `package/rolldown-plugin/import-attributes/src/import-attributes.unit.test.ts`
fail with Rolldown's `UNRESOLVED_ENTRY` diagnostic.
The requested entries are `src/fixtures/entry-static.ts`,
`src/fixtures/entry-dynamic.ts` and `src/fixtures/entry-reexport.ts`.
The static entry is used by two authored cases.

The diagnostic says `Cannot resolve entry module` followed by the affected entry path.
The baseline also emits this separate logger diagnostic:

```text
logger internal error: file sink verification failed: ENOENT: no such file or directory, mkdir '/workspace/node_modules/.monochromatic'
```

## Root cause

The first-party test at
`package/rolldown-plugin/import-attributes/src/import-attributes.unit.test.ts:20`
constructs the plural directory name:

```typescript
// package/rolldown-plugin/import-attributes/src/import-attributes.unit.test.ts
const FIXTURES_DIR = join(import.meta.dirname, 'fixtures',);
```

Its `buildFixture` function supplies that directory to Rolldown at line 31:

```typescript
// package/rolldown-plugin/import-attributes/src/import-attributes.unit.test.ts
input: join(FIXTURES_DIR, entryName,),
```

The frozen Git tree instead contains the singular `src/fixture` directory,
with the three entry files,
`sample.sql` and `text-modules.d.ts`.
The [counterfactual admission][admission] records their individual source hashes.
This is a first-party test-input mismatch.

Upstream source was read from the unchanged Rolldown `v1.2.13` clone,
commit `f7de35f45bcd4cb8e4b6d8d11e74fbfb20b03127`.
In that source,
`crates/rolldown/src/utils/load_entry_module.rs:40` maps the missing entry to its diagnostic:

```rust
//! Upstream diagnostic branch excerpt.
// crates/rolldown/src/utils/load_entry_module.rs
ResolveError::NotFound(_) => Err(BuildDiagnostic::unresolved_entry(id, None)),
```

`crates/rolldown_error/src/build_diagnostic/events/unresolved_entry.rs:20`
formats `Cannot resolve entry module {}.`.
`crates/rolldown_error/src/types/event_kind.rs:160` names the diagnostic:

```rust
//! Upstream event-name branch excerpt.
// crates/rolldown_error/src/types/event_kind.rs
EventKind::UnresolvedEntry => write!(f, "UNRESOLVED_ENTRY"),
```

The logger diagnostic has a separate input boundary.
At the frozen application source,
`package/logger/core/src/sink/file.ts:181` searches from `process.cwd()` for `node_modules`.
Its lines 198 to 205 choose and create the log directory:

```typescript
// package/logger/core/src/sink/file.ts
const LOG_DIR = directory ? directory : join(
  nodeModulesDir,
  '.monochromatic',
);
await mkdir(
  LOG_DIR,
  { recursive: true, mode: 0o700, },
);
```

The original container mounted `/workspace` read-only.
Making only the discovered log directory writable removes the observed logger diagnostic
while all four `UNRESOLVED_ENTRY` failures remain.
It does not repair the fixture input.

## Verification

The actual consumers use Node `v26.12.0` and Rolldown `1.2.13`.
Each container has two CPUs,
2 GiB RAM,
network disabled and a 150-second maximum.
No personal fixture is mounted.
The [baseline][baseline] retains the original failures.
The [terminal controls][controls] retain the exact source identities and process outputs.

The successful catalog is:

- All fifteen existing `transform-helpers.unit.test.ts` cases pass in every control leg.
- With a private writable logger directory and an untracked `fixtures -> fixture` alias,
  all four unchanged integration cases pass.
- Adding only `MONOCHROMATIC_VERBOSE=true` produces ten lifecycle debug lines
  in retained failed-child stderr.

The failing catalog is:

- Original fixture topology and read-only log directory:
  four entry-resolution failures and the separate file-sink diagnostic.
- Original fixture topology and writable log directory:
  the same four entry-resolution failures,
  without the observed file-sink diagnostic.
- Original fixture topology with writable logging and verbose output:
  the same four entry-resolution failures.

Successful-child stderr was discarded by the local wrapper.
The empty fields for those children are placeholders,
not evidence of empty stderr.
The terminal receipt explicitly corrects the original capture claim.
Successful leaf labels remain backed by captured stdout.
The subsequent [corrected capture][capture] retains both streams after a real channel-positive control.
It records the same four integration cases passing on the derived topology,
with 1994 bytes of successful-child stderr and no logger-internal diagnostic.
The old successful-stderr placeholders remain unqualified.

## Verified workarounds

In the owned disposable worktree only,
adding this relative alias qualifies the authored fixture consumer:

```typescript
// Disposable research topology only; run from its existing controller task.
symlinkSync('fixture', `${worktree}/package/rolldown-plugin/import-attributes/src/fixtures`, 'dir');
```

The tradeoff is an altered filesystem topology.
This is not a successful unchanged original owner and is not a production source fix.
The source assertion files and fixture bytes remain unchanged.

The logger workaround follows the recorded writable-directory boundary in
`pi-sdk-staging.md` and mounts only the private log directory writable:

```text
--volume <owned-worktree>:/workspace:ro
--volume <owned-private-logger-directory>:/workspace/node_modules/.monochromatic:rw
```

The mountpoint is created in the disposable worktree before the container starts.
The tradeoff is a deliberately writable diagnostic directory beside a read-only source mount.
It does not grant write access to the rest of the application source.

## What does not work

Writable logging and verbosity leave the fixture failures intact.
Treating the logger diagnostic as the cause of entry resolution is disproved by the separate control.
Successful plugin construction or a packed namespace import does not prove fixture transformations.

The authored case named `ignores imports without with clause` has a further assertion limitation.
At `import-attributes.unit.test.ts:73`,
it builds `entry-static.ts`,
which contains an import attribute.
At line 88 it only asserts `output.length > 0`.
Its spy returns `null` in both branches.
Even after the alias makes that case pass,
it does not qualify a plain-import input or absence of transformation.

This incident is separate from the older external-text parsing issue in [rolldown.md](rolldown.md).

## Upstream filing decision

Nothing to add upstream.

1.  Upstream fault fails:
    the consumer requests a directory absent from the frozen tree.
2.  An upstream change is not the remedy;
    the verified correction is at the first-party input boundary.
3.  The controls exercise ordinary entry loading;
    they establish no unsupported-use-case claim.
4.  Contribution policy is not assessed because no upstream defect or filing is proposed.
5.  Maintainer willingness is not assessed because no upstream change is requested.
6.  No upstream patch is appropriate;
    the unchanged consumer passes with the derived local alias.

No upstream issue,
comment or patch was sent or drafted.

[admission]: ../audit/depot-local-node-counterfactual-admission-2026-10-18.evidence.json
[baseline]: ../audit/depot-remaining-local-unit-2026-10-18.evidence.json
[controls]: ../audit/depot-local-node-counterfactual-2026-10-18.evidence.json
[capture]: ../audit/depot-local-node-capture-correction-2026-10-18.evidence.json
