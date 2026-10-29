# rolldown 1.2.9: an unresolvable bare import is emitted as an external import with only a warning, and `onLog` escalation fails the build after the output is written

Three related behaviours met while investigating
[issue #570](https://github.com/Aquaticat/Monochromatic/issues/570)
(`doc/handover/config-oxlint-stale-plugin-bundle-issue-570.md`),
where `config-oxlint` was built before `pnpm install` linked a workspace dependency:

- Bug 1:
  a bare specifier the resolver cannot find is kept as an `import` in the output,
  the build prints `UNRESOLVED_IMPORT` as a warning,
  and exits 0.
  This is deliberate Rollup-compatible behaviour.
- Bug 2:
  `docs/in-depth/external-modules.md` says an unresolved module becomes external
  only when the `external` option matches it,
  which contradicts the source and Bug 1.
- Bug 3:
  the documented escalation,
  `onLog` calling `defaultHandler('error', log)`,
  makes the build exit 1,
  but only after every output file has been written.
  Rollup aborts before writing.

## Bug 1: unresolved bare imports become externals with a warning

### Symptom

Input `bare.js`:

```js
// bare.js
import { x } from 'nonexistent-package-570';
console.log(x);
```

`rolldown -c` with no `external` option prints on stderr
(ANSI colour codes removed):

```text
bare.js (1:18) [UNRESOLVED_IMPORT] Could not resolve 'nonexistent-package-570' in bare.js
   ╭─[ bare.js:1:19 ]
 1 │ import { x } from 'nonexistent-package-570';
   │                               ╰──── Module not found, treating it as an external dependency
```

It exits 0 and writes:

```js
// out-bare.mjs
import { x } from "nonexistent-package-570";
//#region bare.js
console.log(x);
//#endregion
```

Variants:

- Bare specifier (`'pkg'`,
   `'@scope/pkg/ts'`):
  warning,
  externalized,
  exit 0.
- Path-like specifier (`'./missing-570.js'`,
   absolute paths):
  `[UNRESOLVED_IMPORT] ... Module not found.` as an error,
  no output file,
  exit 1.
- Either kind inside a `require` in a `try` block:
  externalized with no diagnostic at all.

In issue #570 the unresolved bare specifier was `@monochromatic-dev/logger-core/ts`,
and the resulting `import` failed at lint time with `ERR_MODULE_NOT_FOUND`.

### Root cause

`crates/rolldown/src/module_loader/resolve_utils.rs:98-147`
(tag `v1.2.9`,
 unchanged on `main` at `b3b93d950575f7fea39880e4892355a113ebc640`)
handles `ResolveError::NotFound`:

```rust
// crates/rolldown/src/module_loader/resolve_utils.rs:98-146 (v1.2.9, abridged)
ResolveError::NotFound(..) => {
  // NOTE: IN_TRY_CATCH_BLOCK meta if it is a `require` import
  // record
  if !dep.meta.contains(ImportRecordMeta::InTryCatchBlock) {
    // https://github.com/rollup/rollup/blob/49b57c2b30d55178a7316f23cc9ccc457e1a2ee7/src/ModuleLoader.ts#L643-L646
    if ecmascript::is_path_like_specifier(specifier) {
      // Unlike rollup, we also emit errors for absolute path
      build_errors.push(BuildDiagnostic::resolve_error(/* ... */ "Module not found.".into(), /* ... */));
    } else {
      // ... help text for `virtual:` and the neutral platform ...
      warnings.push(
        BuildDiagnostic::resolve_error(
          /* ... */
          "Module not found, treating it as an external dependency".into(),
          EventKind::UnresolvedImport,
          help,
        )
        .with_severity_warning(),
      );
    }
  }
  ret.push(ResolvedId {
    id: ModuleId::new(specifier.as_str()),
    external: true.into(),
    ..Default::default()
  });
}
```

The `external: true` push at lines 142-146 runs for every `NotFound`,
whatever the `external` option says.

### Intent: deliberate Rollup compatibility

- The code links Rollup's `ModuleLoader.ts` directly (line 102).
- Rollup does the same today:
  `src/ModuleLoader.ts:680-697` (rollup/rollup `main` at `299f95c`)
  errors for relative sources and otherwise calls
  `onLog(LOGLEVEL_WARN, logUnresolvedImportTreatedAsExternal(...))`
  and returns `{ external: true, id: source }`.
- Rollup documents it:
  `docs/troubleshooting/index.md:71-79`,
  "Warning:
   'Treating [module] as external dependency'",
  says Rollup only resolves relative ids by default,
  so a bare import becomes an external dependency.
- A rolldown maintainer chose warning over error on purpose:
  in [rolldown/rolldown#1174](https://github.com/rolldown/rolldown/issues/1174)
  (closed),
  hyfdev wrote
  "We used to throw errors for implicit external modules.
  And for better DX,
   I changed to throw warnings rather than errors".

### What exists to make it an error

- `checks.unresolvedImport` is a boolean that only silences the warning
  (`packages/rolldown/src/utils/validator.ts:351-354`:
  "Whether to emit warnings when an import cannot be resolved").
- `onLog` escalation works for the exit status but see Bug 3.
- A declarative severity (`checks.unresolvedImport: 'error'`) was requested in
  [rolldown/rolldown#9362](https://github.com/rolldown/rolldown/issues/9362) (open,
   no comments),
  implemented by PR [#9388](https://github.com/rolldown/rolldown/pull/9388),
  merged 2026-05-18,
  and reverted the same day by PR [#9438](https://github.com/rolldown/rolldown/pull/9438)
  with the reason "need through discussion".
  That PR's `unresolvedImport` fast path failed at resolve time,
  so it would also have avoided Bug 3.

### Assessment

Defensible design,
not a bug:
it matches Rollup's documented default,
and the maintainer choice is on record.
The gap is the missing opt-in severity,
already tracked by #9362.

## Bug 2: the external-modules guide contradicts the resolver

### Symptom

`docs/in-depth/external-modules.md:25`
(tag `v1.2.9` and `main` at `b3b93d950575f7fea39880e4892355a113ebc640`):

```md
3. **Unresolved modules** — if no plugin or the internal resolver can find a module and the `external` option matches the specifier, Rolldown treats it as external rather than throwing an error.
```

A reader concludes that an unresolved import not listed in `external` fails the build.
The Bug 1 harness has no `external` option at all and still externalizes.

### Root cause

The sentence was added with the page in commit `8a28ad6b5`
("docs:
 explain how external modules work in rolldown (#8457)",
 2026-03-01)
and not changed by the only later commit to the file (`cc5825f42`,
 #10215).
The source it describes is Bug 1's `resolve_utils.rs:98-147`,
which never consults `external` in this branch.
The same page's step 1 already says a matching `external` pattern skips resolution entirely,
so an unresolved import can never reach the resolver while also matching `external`.

### Verification

Bug 1's `bare.config.mjs` (`{ input: 'bare.js', output: { file: 'out-bare.mjs' } }`)
externalizes `nonexistent-package-570` and exits 0,
with no `external` option.

### Proposed docs fix (prototype)

```diff
--- a/docs/in-depth/external-modules.md
+++ b/docs/in-depth/external-modules.md
@@ -23,5 +23,5 @@ There are three ways a module can be marked as external:
 2. **A plugin's `resolveId` hook** — a plugin can return `{ id, external: true }` (or `"relative"` / `"absolute"`) to explicitly mark a module as external. A plugin can also `return false` to mark the raw specifier as external with the same normalization as the `external` option.
 
-3. **Unresolved modules** — if no plugin or the internal resolver can find a module and the `external` option matches the specifier, Rolldown treats it as external rather than throwing an error.
+3. **Unresolved bare imports** — if no plugin or the internal resolver can find a bare specifier (e.g. `'lodash'`), Rolldown treats it as external and emits an `UNRESOLVED_IMPORT` warning instead of throwing an error, as Rollup does. Unresolved relative or absolute paths are errors. To make unresolved bare imports fail the build, escalate the warning with [`onLog`](/reference/InputOptions.onLog).
 
 ## The Full Resolution Flow
```

## Bug 3: `onLog` escalation fails the build after writing the output

### Symptom

```js
// onlog.config.mjs
export default {
  input: 'bare.js',
  output: { file: 'out-onlog.mjs' },
  onLog(level, log, handler) {
    if (log.code === 'UNRESOLVED_IMPORT') { handler('error', log); return; }
    handler(level, log);
  },
};
```

`rolldown -c onlog.config.mjs` prints
`ERROR  Build failed with 1 error: ... RolldownError: [UNRESOLVED_IMPORT] Could not resolve 'nonexistent-package-570'`
and exits 1,
yet `out-onlog.mjs` exists afterwards with the bare import in it,
freshly written
(verified by deleting it first and listing it after the failed run).

The same config under Rollup 4.63.5 exits 1 with
`[!] RollupError: "nonexistent-package-570" is imported by "bare.js", but could not be resolved – treating it as an external dependency.`
and writes no file.

For issue #570 this matters twice:
a failed build still leaves a broken sidecar on disk,
and it is newer than every input,
so the `lint:oxlint` freshness check in `mise.toml` (`shouldBuildOxlintConfig`) treats it as current.

### Root cause

Rolldown collects warnings in Rust and hands them to `onLog` only after the whole write finishes.

`Bundle::write` scans and then calls `bundle_write`
(`crates/rolldown/src/bundle/bundle.rs:45-58`,
 v1.2.9).
`bundle_write` generates chunks and writes every file before returning the warnings:

```rust
// crates/rolldown/src/bundle/bundle.rs:186-233 (v1.2.9, abridged)
let mut output = self.bundle_up(scan_stage_output, /* is_write */ true).await?;
self.fs.create_dir_all(&dist_dir)/* ... */?;
for chunk in &output.assets {
  // ...
  self.fs.write(&dest, chunk.content_as_bytes())/* ... */?;
}
self.plugin_driver.write_bundle(&mut output.assets, &self.options, &mut output.warnings).await?;
output.warnings.append(&mut self.warnings);
Ok(output)
```

Only then does the N-API binding deliver them:

```rust
// crates/rolldown_binding/src/binding_bundler.rs:115-131 (v1.2.9)
let mut bundle_output = match bundle.write().await {
  Ok(output) => output,
  Err(errs) => { /* ... */ }
};

if let Err(err) = handle_warnings(std::mem::take(&mut bundle_output.warnings), &options).await
{
  let error = to_binding_error(&err.into(), cwd.clone());
  return Ok(napi::Either::A(BindingErrors::new(vec![error])));
}
```

`handle_warnings` (`crates/rolldown_binding/src/utils/mod.rs:69-105`) calls `on_log` for each warning
and returns the first error,
which is how `defaultHandler('error', log)` becomes a failed build,
after the files exist.
The watcher has a mirror copy with the same ordering
(`crates/rolldown_watcher/src/watch_task.rs:130` writes,
 `:160` emits warnings).

Rollup instead calls `onLog` synchronously inside `ModuleLoader.handleInvalidResolvedId`
(`src/ModuleLoader.ts:690`),
so a throwing handler rejects `rollup()` before `bundle.write` is reachable.

The `onLog` reference text
(`packages/rolldown/src/options/docs/on-log.md:3`)
says the `"error"` level
"will turn the log into a thrown error",
with no mention of output already being on disk.

### Verification

Version under test:
rolldown 1.2.9 from this repository's `node_modules/.bin/rolldown`
(`rolldown --version` prints `rolldown v1.2.9`);
source read at tag `v1.2.9` (`5b4746e442989d770c606ce08d2737e6aafbd25d`)
and `main` (`b3b93d950575f7fea39880e4892355a113ebc640`),
where `resolve_utils.rs` lines 95 to 150 are byte-identical
and `binding_bundler.rs` keeps the write-then-`handle_warnings` order (lines 115 and 127).
Rollup 4.63.5 installed with `pnpm add rollup@4` in a throwaway directory.

Harness files:
`bare.js` (Bug 1),
`rel.js` (`import { x } from './missing-570.js';`),
and one config per case.

```sh
rolldown -c bare.config.mjs; echo "exit=$?"
rm --force out-onlog.mjs && rolldown -c onlog.config.mjs; echo "exit=$?"; ls -la out-onlog.mjs
```

Results:

- `bare.config.mjs` (rolldown):
  warning,
  exit 0,
  `out-bare.mjs` written with the bare import.
- `rel.config.mjs` (rolldown):
  error `Module not found.`,
  exit 1,
  no output file.
- `onlog.config.mjs` (rolldown):
  error,
  exit 1,
  `out-onlog.mjs` written with the bare import.
- `bare.config.mjs` (Rollup):
  `(!) Unresolved dependencies` warning,
  exit 0,
  output written.
- `onlog.config.mjs` (Rollup):
  `RollupError`,
  exit 1,
  no output file.

## Verified workarounds

### Fail in a `resolveId` plugin, before anything is written

```js
// plugin.config.mjs
const failUnresolvedBare = {
  name: 'fail-unresolved-bare',
  async resolveId(source, importer, options) {
    if (!importer || source.startsWith('.') || source.startsWith('/') || source.startsWith('node:')) return null;
    const resolved = await this.resolve(source, importer, { ...options, skipSelf: true });
    if (resolved === null || resolved.external) {
      this.error(`Could not resolve '${source}' from ${importer}; refusing to externalize`);
    }
    return resolved;
  },
};
export default { input: 'bare.js', output: { file: 'out-plugin.mjs' }, plugins: [failUnresolvedBare] };
```

Verified with rolldown 1.2.9:
`Build failed with 1 error: [plugin fail-unresolved-bare] RolldownError: Could not resolve 'nonexistent-package-570' ...`,
and `out-plugin.mjs` does not exist afterwards.

Tradeoffs:
every bare import pays an extra `this.resolve` round trip through JS.
Specifiers the `external` option matches never reach the hook (guide step 1),
so intended externals are unaffected,
but a plugin that deliberately returns `{ external: true }` for a specifier
trips the `resolved.external` check;
narrow the check to a pattern
(in this repository,
 the `alwaysBundle` globs such as `@monochromatic-dev/**`)
when such plugins are present.

### `onLog` escalation plus removing the stale output

`onLog` with `defaultHandler('error', log)` for `UNRESOLVED_IMPORT` gives a correct exit status.
Pair it with deleting the output on failure,
or write to a temporary directory and move it into place only on success.

Tradeoffs:
the delete or move is the caller's responsibility,
and until it runs,
a concurrent reader can load the broken file.
Any mtime-based freshness check must not treat a file written by a failed build as current.

## What does not work

- `checks.unresolvedImport`:
  boolean,
  only silences the warning.
- `logLevel`:
  filters logs,
  cannot raise a warning to an error.
- `external`:
  it is tested before resolution (guide step 1),
  so it cannot express "external only if the resolver fails".
- `onLog` escalation alone:
  correct exit status,
  output still written (Bug 3).

## Upstream filing decision

### Out-of-scope check

`.out-of-scope/` has no entry for rolldown or unresolved imports
(listing checked 2026-09-25).

### Duplicate search

`gh search issues --repo rolldown/rolldown` for
`unresolved import`,
`UNRESOLVED_IMPORT`,
`unresolved import external`,
`treating it as an external dependency`,
`external-modules docs`,
`onLog`,
`onLog error output written`,
`onwarn throw still writes`,
`warnings emitted after write`,
`output written despite error`;
`gh search prs` for `unresolvedImport`.

- Bug 1 severity option:
  duplicate of open [#9362](https://github.com/rolldown/rolldown/issues/9362).
- Bug 2 docs line:
  no issue found.
- Bug 3 write-then-fail:
  no issue found.
  Neighbours,
   none a duplicate:
  [#9922](https://github.com/rolldown/rolldown/issues/9922) (errors thrown from `onLog` swallowed
for native-plugin warnings,
   closed),
  [#8640](https://github.com/rolldown/rolldown/issues/8640) (circular-dependency severity;
   maintainer recommends `onLog`,
   closed),
  [#3606](https://github.com/rolldown/rolldown/issues/3606) (diagnostic severity control,
   closed).

### Bug 1: comment on #9362, not a new issue

The warn-and-externalize default is deliberate (#1174) and matches Rollup,
so constraint 1 fails for a "make it an error by default" report.
The opt-in severity is already requested in #9362.
Additive content for that thread:
the `onLog` workaround the issue proposes still writes the output (Bug 3),
which is a concrete reason the reverted #9388 resolve-time fast path mattered.
Kept as a draft comment,
to post only after the Bug 3 issue exists so it can link to it.

~~~md
One data point for the discussion that #9438 asked for: the `onLog` workaround in this issue gives the
right exit status but does not prevent output. With rolldown 1.2.9, `onLog` escalating
`UNRESOLVED_IMPORT` via `defaultHandler('error', log)` exits 1 *after* `bundle.write()` has written
every chunk, because warnings reach `onLog` only after `Bundle::bundle_write` returns
(`crates/rolldown_binding/src/binding_bundler.rs` write → `handle_warnings`). Rollup 4 aborts
before writing. Details and a prototype fix: #<bug 3 issue>. The reverted #9388 failed at resolve
time, so `unresolvedImport: 'error'` would not have this problem.

(Prepared with AI assistance; reproduction and source references verified by me.)
~~~

### Bug 2: docs fix

1.  Upstream's fault:
    yes,
     wording contradicts `resolve_utils.rs:98-147`.
2.  Can fix:
    yes,
     one line.
3.  Supported:
    yes,
     the page exists to explain exactly this flow.
4.  Welcome:
    yes with disclosure
    (`docs/contribution-guide/index.md:11-21` in rolldown/rolldown asks for AI-usage disclosure and human review;
    no ban found).
5.  Likely to fix:
    no negative signal.
6.  Prototype:
    the diff under "Proposed docs fix (prototype)".

Fileable as a docs PR,
or folded into the Bug 3 issue as a note.

### Bug 3: write-then-fail

1.  Upstream's fault:
    yes.
    Rollup-compatible `onLog` semantics abort before output;
    rolldown's own docs describe the result as a thrown error;
    the ordering is rolldown's.
2.  Can fix:
    yes,
     see "Prototype".
3.  Supported:
    yes.
    `onLog` escalation is documented (`packages/rolldown/src/options/input-options.ts:480-501` example)
    and recommended by a maintainer in #8640.
4.  Welcome:
    yes with disclosure,
     as for Bug 2.
5.  Likely to fix:
    no negative signal;
    #9922 shows maintainers treat "a throwing `onLog` must fail the build" as a bug.
6.  Prototype:
    see "Prototype".

#### Prototype

Disposable clone of `rolldown/rolldown` at `v1.2.9`
(`5b4746e442989d770c606ce08d2737e6aafbd25d`,
 push URL disabled).
Diff:
[`rolldown-unresolved-import-external.patch`](rolldown-unresolved-import-external.patch).
It and the Bug 2 docs diff also apply cleanly (`git apply --check`)
to rolldown `main` at `b3b93d950575f7fea39880e4892355a113ebc640`.

The change adds `crates/rolldown/src/utils/deliver_warnings.rs`
(a copy of the binding's `handle_warnings` logic placed where `Bundle` can call it;
the binding and watcher copies stay,
and deduplicating the three is a follow-up cleanup,
 not part of the minimal fix)
and,
in `Bundle::bundle_write`,
delivers scan and generate warnings to `onLog` after `bundle_up` and before the first file write,
only when an `onLog` handler exists.
Warnings are taken out of the output as they are delivered,
so the binding's and watcher's later delivery sees only `writeBundle` warnings and none are duplicated.

Build,
secret-free and bounded
(only the disposable clone mounted,
 2 CPUs,
 2 GiB;
the first attempt failed because the image lacks `cmake` for `libmimalloc-sys2`):

```sh
podman run --rm --memory=2g --cpus=2 --volume "${PWD}:/work:Z" --workdir /work/rolldown \
  --env CARGO_HOME=/work/cargo-home --env CARGO_BUILD_JOBS=2 \
  --env CARGO_PROFILE_DEV_DEBUG=0 --env CARGO_INCREMENTAL=0 \
  docker.io/library/rust:latest \
  sh -c 'apt-get update -qq && apt-get install -y -qq cmake > /dev/null && cargo build -p rolldown_binding --lib'
# Finished `dev` profile [unoptimized] target(s)
```

Harness:
the installed rolldown 1.2.9 CLI with its native binding swapped through
`NAPI_RS_NATIVE_LIBRARY_PATH=<clone>/target/debug/librolldown_binding.so`
(copied to a `.node` name;
the override is read in `dist/shared/binding-BbrDfv1x.mjs:93-94`),
same configs as "Verification".

- `onlog.config.mjs`,
  output deleted before each run:
  installed binding exits 1 and writes `out-onlog.mjs`;
  prototype binding exits 1 and writes nothing.
- `bare.config.mjs`:
  unchanged,
  warning printed once,
  exit 0,
  output written.
- `rel.config.mjs`:
  unchanged,
  error,
  exit 1,
  nothing written.
- A pass-through `onLog` that counts calls:
  one `UNRESOLVED_IMPORT` call with both bindings,
  exit 0,
  output written,
  so delivery is not duplicated.

Not run:
upstream `cargo test`,
`just` recipes,
clippy,
watch mode,
and a `writeBundle` hook that warns.

#### Decision

Not filed by this session (the task was read-only research).
All six constraints hold,
so the draft is fileable after human review of the reproduction and prototype,
as rolldown's AI policy requires.
File it before posting the #9362 comment,
 which links to it.

#### Draft upstream issue (do not file as-is)

~~~md
Title: [Bug]: `onLog` escalating a warning to an error fails the build only after output files are written

AI assistance disclosure: prepared with an AI assistant; the reproduction, source trace, and
prototype verification were run and reviewed by me.

### Reproduction (rolldown 1.2.9)

bare.js:
    import { x } from 'nonexistent-package-570';
    console.log(x);
rolldown.config.mjs:
    export default {
      input: 'bare.js',
      output: { file: 'out.mjs' },
      onLog(level, log, handler) {
        if (log.code === 'UNRESOLVED_IMPORT') { handler('error', log); return; }
        handler(level, log);
      },
    };

    rm -f out.mjs; npx rolldown -c; echo $?; cat out.mjs
    # exit 1, "Build failed with 1 error: [UNRESOLVED_IMPORT] ...", but out.mjs exists
    # and contains `import { x } from "nonexistent-package-570";`

The same config with Rollup 4.63.5 exits 1 and writes nothing.

### What is expected?

A warning escalated to an error in `onLog` aborts the build before any file is written,
as in Rollup (which calls `onLog` from `ModuleLoader` during the build phase). The
`onLog` docs describe the `"error"` level as turning the log into a thrown error.

### Root cause

`Bundle::bundle_write` (`crates/rolldown/src/bundle/bundle.rs`) writes every asset and then
returns warnings on `BundleOutput`; `BindingBundler::write`
(`crates/rolldown_binding/src/binding_bundler.rs`) only then calls `handle_warnings`, which
invokes `onLog`. The watcher (`crates/rolldown_watcher/src/watch_task.rs`) has the same order.

### Suggested fix

Deliver scan and generate warnings to `onLog` inside `Bundle::bundle_write` after
`bundle_up` and before the first `fs.write`, and let the existing post-write delivery handle
only `writeBundle` warnings. Patch attached: it adds a copy of `handle_warnings` to the
`rolldown` crate so `Bundle` can call it; the binding and watcher copies could then call the
shared one.

### Related

- #9362 / #9388 / #9438: `checks.unresolvedImport: 'error'` (the reverted PR failed at resolve
  time, which avoided this).
- `docs/in-depth/external-modules.md` item 3 says unresolved modules become external only when
  `external` matches; the resolver externalizes every unresolved bare specifier.
~~~

## Related

- `doc/troubleshooting/rolldown.md` Bug 4
  (`node:` subpath imports hit the same `NotFound` branch under non-Node platforms).
- `doc/troubleshooting/oxlint-config-load-failure-exit-code.md`
  (the lint-time half of issue #570).
