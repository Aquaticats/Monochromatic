# Rolldown 1.0.0-rc.9: import-attributes not applied, ESTree-mismatched `parseSync` types, `platform: 'neutral'` empties `mainFields`, and `node:` subpath imports fail under non-Node platforms

This file groups four independent rolldown 1.0.0-rc.
9 issues
that bite client-side and isomorphic workspace builds.
 Each
has its own canonical section.

---

## Bug 1: `with { type: 'text' }` parsed but not used for module loading

Upstream issue:
[rolldown/rolldown#2758](https://github.com/rolldown/rolldown/issues/2758)
(open,
 assigned to `@shulaoda`,
 and no longer on hold).
 Status checked 2026-09-09.
 See
 [`doc/research/rolldown-import-attributes-upstream-status.md`](../research/rolldown-import-attributes-upstream-status.md)
 for the upstream timeline,
 capacity evidence,
 and donation assessment.

### Symptom

A static import using import attributes:

```ts
import sql from './schema.sql' with { type: 'text', };
```

emits `PARSE_ERROR` because rolldown parses `.sql` as
JavaScript.
 The `with` clause is stored but does not affect
how the module is loaded.

### Root cause

Rolldown's Rust AST scanner parses `with { type: 'text' }`
clauses and stores them in an `import_attribute_map` keyed by
`ImportRecordIdx`.
 The scanner does not set
`asserted_module_type` on the import record based on the
`type` attribute value.

What works:

- Import attributes are preserved in ESM output formatting
  (`crates/rolldown/src/ecmascript/format/esm.rs:261-290`).

What does not work:

- `resolveId` hook receives no `attributes` field in its
  options (unlike Rollup 4's `options.attributes`).
   The
  `ResolveIdExtraOptions` type only has `custom`,
   `isEntry`,
  and `kind`.
- `.sql` files imported with `with { type: 'text' }` are
  parsed as JavaScript,
   causing `PARSE_ERROR` because
  rolldown does not map the `type` attribute to its native
  `ModuleType::Text`.
- The only place `asserted_module_type` is set is in
  `new_url.rs` for `new URL()` patterns
  (`ModuleType::Asset`).

Source citations (rolldown `main` commit `9704b5650`,
 checked 2026-09-09):

- `crates/rolldown/src/ast_scanner/mod.rs:902-923`:
   stores
  attributes but passes no asserted module type.
- `crates/rolldown/src/ast_scanner/new_url.rs:69`:
   only
  assignment to `asserted_module_type` sets `ModuleType::Asset`.
- `crates/rolldown_common/src/types/import_record.rs:32`:
  `asserted_module_type: Option<ModuleType>`.
- `packages/rolldown/src/plugin/index.ts:223-246`:
  `ResolveIdExtraOptions` lacks `attributes`.

### Verification

Version under test:
 rolldown 1.0.0-rc.
9.
 Reproduce by
importing any non-JS asset with `with { type: 'text' }` in a
client-side bundle.

The original upstream issue's dynamic JSON case now works in `rolldown@1.2.7`.
A disposable 2026-09-09 check bundled
`import('./file.json', { with: { type: 'json' } })`,
rewrote it to the emitted JavaScript chunk,
and loaded the expected JSON value.
That does not test or fix this section's static `type: 'text'` failure.

### Verified workarounds

#### `@monochromatic-dev/rolldown-plugin-import-attributes`

Bridges the gap by using the `transform` hook to rewrite
specifiers (appending `?__importattr=<type>` query params)
and the `load` hook to serve the file content as a JS
module.
 For dynamic imports where rolldown's scanner
discovers dependencies before `transform` runs,
 the
`resolveId` hook re-parses the importer source with
`parseSync` to find the attribute type.

Tradeoff:
 requires a custom plugin;
 the specifier rewrite
shows up in the dependency graph as a query-suffixed module.

#### `moduleTypes` input option

Maps file extensions to module types globally:

```ts
inputOptions: { moduleTypes: { '.sql': 'text' } }
```

Tradeoff:
 extension-wide;
 does not respect per-import `with`
clauses.
 Use only when all files of an extension are the
same module type.

### What does not work

- Marking `.sql` as external:
   rolldown still tries to parse
  it before the external decision is made.
- Implementing a custom `resolveId` that returns module type:
  the `ResolveIdExtraOptions` type does not pass through
  attributes,
   so the plugin cannot tell which type the
  import requested.

### Why we do not file this upstream (already filed; now assigned)

Already represented by rolldown/rolldown#2758.
 5 constraints:

1. **Is it really upstream's fault?
   ** Yes;
    the scanner stops
   short of applying the parsed `type` attribute.
2. **Can upstream fix it?
   ** Yes;
    thread the attribute through
   to `asserted_module_type`.
3. **Are they supporting this use case?
   ** Documented goal.
4. **Will they likely fix it?
   ** The 2026 Q3 roadmap names import attributes as a Vite 8 migration blocker;
    `@shulaoda` is assigned,
    but the documented searches did not find a complete implementation PR.
5. **Have we prototyped a minimal fix?
   ** External plugin is
   the prototype.

Decision:
 no new upstream report;
 the existing issue is the
right venue.

---

## Bug 2: `parseSync` runtime AST uses ESTree node-type names but `@oxc-project/types` declares OXC-native names

Upstream issue:
[oxc-project/oxc#10139](https://github.com/oxc-project/oxc/issues/10139)
(closed,
 wontfix).
 Date 2026-03-15.

### Symptom

Rolldown re-exports `parseSync` from `rolldown/utils` and
ESTree types via
`export type * as ESTree from '@oxc-project/types'`.
 The
`@oxc-project/types` package declares OXC-native type names:

- `StringLiteral` with `type: "StringLiteral"`
- `IdentifierName` with `type: "Identifier"` (this one
  matches runtime)

The **runtime** AST from `parseSync` uses ESTree-compatible
names:

- String literals have `type: "Literal"` (not
  `"StringLiteral"`).
- All literal types (string,
   number,
   boolean,
   null,
   regexp)
  share `type: "Literal"`.

TypeScript discriminated-union narrowing does not work:

```ts
// Does NOT narrow; "Literal" is not a discriminant in the declared union
if (node.source.type === 'Literal')
  node.source.value; // TypeScript error: 'value' does not exist on type 'Expression'
```

### Root cause

The OXC team has stated this is intentional;
 they do not aim
to align `@oxc-project/types` with ESTree.
 Recommendation:
use types from another ESTree-compatible implementation
(Acorn or TS-ESLint) when ESTree compatibility is needed.

### Verification

Version under test:
 rolldown 1.0.0-rc.
9 (re-exports
`@oxc-project/types`).
 Reproduce by writing a plugin that
inspects literal values via narrowed types;
 observe the type
errors.

### Verified workaround

Use runtime `typeof` checks instead of type discrimination:

```ts
// Workaround: runtime checks
if ('value' in node && typeof node.value === 'string') {
  // safely access string value
}
```

Tradeoff:
 loses compile-time guarantees that type
discrimination would provide.
 For plugin code that operates
on small AST surfaces,
 the runtime check is acceptable.
 For
larger plugins,
 switch the type imports to Acorn or
TS-ESLint types.

### What does not work

- Casting to ESTree types:
   works at the type level but loses
  any guarantee about the actual runtime shape;
   OXC may
  diverge further over time.
- Filing the issue with OXC:
   already closed wontfix.

### Why we do not file this upstream (closed wontfix)

Already represented by oxc-project/oxc#10139.
 Decision:
adapt to upstream's choice by switching type sources if
needed.

---

## Bug 3: `platform: 'neutral'` defaults `mainFields` to empty, breaking package resolution

Date 2026-03-22.

### Symptom

When `platform: 'neutral'`,
 rolldown defaults
`resolve.mainFields` to `[]`.
 Neither `module` nor `main` is
consulted;
 packages relying on these fields silently fail
with `UNRESOLVED_IMPORT` warnings and are treated as
externals.

```text
Could not resolve '<package>'
"The 'main' field here was ignored. Main fields must be configured explicitly when using the 'neutral' platform."
```

At runtime in the browser:

```text
TypeError: The specifier "<package>" was a bare specifier, but was not remapped to anything.
```

Transitive dependencies fail too (bundling `jspdf` leaves
`fast-png`,
 `canvg`,
 `stackblur-canvas` unresolved).

### Root cause

Per-platform `mainFields` defaults (from
`rolldown/dist/shared/define-config-*.d.mts:3338-3347`):

- `node`:
   `['main', 'module']`
- `browser`:
   `['browser', 'module', 'main']`
- `neutral`:
   `[]`

With `[]`,
 rolldown only resolves packages that have an
`exports` field with matching conditions.
 Packages without
`exports` (or whose `exports` conditions don't match the
neutral platform) become unresolvable.

### Verification

Version under test:
 rolldown 1.0.0-rc.
9 (via tsdown).
Reproduce:
 set `platform: 'neutral'` in a tsdown config
without overriding `resolve.mainFields`;
 bundle a project
that depends on a package without `exports`.

### Verified workaround: set `resolve.mainFields` explicitly via `inputOptions`

In tsdown,
 `resolve` is a rolldown `InputOptions` property,
not a top-level tsdown config option:

```ts
export default defineConfig({
  platform: 'neutral',
  inputOptions: {
    resolve: {
      mainFields: ['module', 'main',],
    },
  },
},);
```

Tradeoff:
 must be remembered for every neutral-platform
config.
 The override duplicates the defaults of the
`browser` platform;
 consider whether `platform: 'browser'` is
actually wanted instead.

### What does not work

- Setting `resolve` at the top level of the tsdown config:
  tsdown's `UserConfig` type does not include `resolve`,
   so
  it is silently dropped.
- Adding every transitive dependency to `deps.alwaysBundle`:
  `alwaysBundle` controls whether a package is bundled vs
  externalised,
   but if the package cannot be **resolved** in
  the first place,
   it is treated as external regardless of
  `alwaysBundle`.

### Source locations

Rolldown 1.0.0-rc.
9:

- `dist/shared/define-config-*.d.mts:3338-3347`:
  `mainFields` type and per-platform defaults.
- `dist/shared/binding-*.d.mts:559`:
   native binding
  `mainFields` field.

tsdown:

- `dist/types-*.d.mts:880-882`:
   `inputOptions` pass-through
  to rolldown `InputOptions`.
- `dist/types-*.d.mts:763`:
   `platform` option definition.

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** Borderline;
    the
   neutral default is documented but the silent
   externalisation is a UX wart.
2. **Can upstream fix it?
   ** Yes;
    emit a stronger warning,
   or default `mainFields` to a sensible value.
3. **Are they supporting this use case?
   ** Yes;
    `neutral` is
   documented.
4. **Will they likely fix it?
   ** Worth proposing as a UX
   improvement.
5. **Have we prototyped a minimal fix?
   ** No.

Decision:
 no upstream report from us yet.

---

## Bug 4: `node:` subpath imports produce `UNRESOLVED_IMPORT` under non-Node platforms

Date 2026-04-04 (initial).
 Prototype audit 2026-05-17.

### Symptom

Static imports like
`import { parse } from 'node:path/posix'` or
`import { posix } from 'node:path/posix'` emit
`[UNRESOLVED_IMPORT] Could not resolve 'node:path/posix'`
when `platform` is `'neutral'` or `'browser'`.
 Subpath
specifiers affected include `node:path/win32`,
`node:stream/promises`,
 `node:stream/consumers`,
`node:stream/web`,
 `node:dns/promises`,
`node:readline/promises`,
 `node:timers/promises`,
 and any
other `node:` subpath export.

Bare parent specifiers (`node:path`,
 `node:assert`,
 etc.)
also produce the same warning under `platform: 'neutral'`
unless they are explicitly listed in `external`.
 The earlier
reading that "the bare parent specifier `node:path` resolves
fine under all platforms" was wrong;
 the workspace happened
to list bare names in `external`,
 which masked the warning
for those specifiers only.
 The prototype fixture
(`crates/rolldown/tests/rolldown/resolve/neutral_platform_node_subpath/`)
shows both bare and subpath specifiers failing identically
when no external list is set.

### Root cause

oxc-resolver's builtin-module detection is gated on a single
`builtin_modules` flag that controls both `node:`-prefixed
specifiers and bare names (`fs`,
 `path`).
 Rolldown only sets
the flag to `true` when `platform` is `Node`
(`crates/rolldown_resolver/src/resolver_config.rs:133`:
`builtin_modules: matches!(platform, Platform::Node)`).
 On
`neutral` or `browser`,
 `builtin_modules` is `false`,
 so the
resolver never short-circuits `node:` specifiers as
builtins.

Under `Platform::Neutral`,
 every `node:`-prefixed import
falls through to `resolve_utils.rs`'s NotFound branch and
gets the "Module not found,
 treating it as an external
dependency" warning plus the misleading "Main fields must be
configured explicitly when using the 'neutral' platform"
help text.
 The fall-through still externalises the specifier
in the output,
 so the bundle is correct at runtime;
 only the
diagnostic noise is wrong.

Source locations (rolldown HEAD `14f967a43`,
~19 commits past `v1.0.1`):

- `crates/rolldown_resolver/src/resolver_config.rs:133`:
  `builtin_modules` gated on `Platform::Node`.
- `crates/rolldown_plugin/src/utils/resolve_id_with_plugins.rs:122-131`:
  Auto-external pass handles http/data URLs but not `node:`
  prefixes;
   falls through to `resolve_id`.
- `crates/rolldown/src/module_loader/resolve_utils.rs:103-148`:
  NotFound branch externalises with the warning under
  `Platform::Neutral`.
- `crates/rolldown_testing/src/integration_test.rs:535-537`:
  test harness auto-injects `external: ["node:assert"]`,
  which is what hid the bare-vs-subpath asymmetry in earlier
  observations.
- `crates/rolldown/src/utils/prepare_build_context.rs:157-160`:
  ESM defaults to `Browser` platform.

oxc-resolver (v11.19.1,
 the version in rolldown's
`Cargo.toml`):

- `src/lib.rs:466-481`:
   `require_core()` checks
  `starts_with("node:")` and `BUILTINS.binary_search` but
  only when `self.options.builtin_modules` is `true`.
- `nodejs-built-in-modules` crate (v1.0.0);
   `BUILTINS`
  includes `path/posix`,
   `path/win32`,
   `stream/promises`,
  `dns/promises`,
   etc. as bare entries;
   subpaths resolve
  the same way as their parents once the flag is on.

### Verification

Version under test:
 rolldown HEAD `14f967a43`
(`v1.0.1-19-g14f967a43`).
 The doc was originally written
against `v1.0.0-rc.9`;
 the relevant code path is unchanged.

Reproduction fixture
(`crates/rolldown/tests/rolldown/resolve/neutral_platform_node_subpath/`):

```js
// main.js
import { posix, } from 'node:path';
import { parse, } from 'node:path/posix';

console.log(parse('/a/b',), posix.parse('/a/b',),);
```

```json
// _config.json
{
  "config": {
    "platform": "neutral"
  }
}
```

Pre-patch run (`INSTA_UPDATE=always cargo test -p rolldown
--test integration neutral_platform_node_subpath`) produces:

```text
[UNRESOLVED_IMPORT] Could not resolve 'node:path' in main.js
[UNRESOLVED_IMPORT] Could not resolve 'node:path/posix' in main.js
```

Both warnings carry the "Main fields must be configured
explicitly when using the 'neutral' platform" help message,
which is misleading:
 the issue is unrelated to main fields.

### Verified workaround: import from the parent module

```ts
// Before; breaks under platform: 'neutral'
import { parse, } from 'node:path/posix';
parse(somePath,);

// After; resolves on all platforms when the parent specifier
// is listed in `external` (or after the upstream fix lands)
import { posix, } from 'node:path';
posix.parse(somePath,);
```

Tradeoff:
 imports the entire parent module's surface;
 loses
the narrower import that subpath imports were designed for.
For tree-shaken bundles the cost is minimal.
 Note that the
parent specifier still needs to be in `external` for
`platform: 'neutral'` builds to suppress the
`UNRESOLVED_IMPORT` warning.

For code that must also run in browsers (where `node:path`
does not exist),
 use a computed specifier to prevent static
resolution:

```ts
const specifier = `node${':path'}`;
const nodePath = hasNodeRuntime
  ? (await import(specifier) as typeof import('node:path')).posix
  : undefined;
```

The workspace previously referenced
`package/module/es/src/path/index.ts` as a call site for
this pattern;
 that path no longer exists,
 so the pattern may
have moved or been removed.

### What does not work

- Setting `resolve.builtinModules` manually:
   this option is
  not exposed in rolldown's public `InputOptions`;
   it is
  derived internally from `platform`.

### Workaround that started not working but works now: `external` array with exact subpaths

Listing the exact subpath in `external` does suppress the
warning as of rolldown HEAD `14f967a43`:

```ts
inputOptions: {
  external: ['node:path/posix', 'node:path',],
},
```

`crates/rolldown_plugin/src/utils/resolve_id_check_external.rs:33-34`
calls `bundle_options.external.call(specifier, importer, false)`
**before** the resolver runs.
 `IsExternal::StringOrRegex` does a
literal string match,
 so `external: ['node:path/posix']` short
circuits and externalises before oxc-resolver ever sees the
specifier.
 (The earlier reading that "externalisation applies
after resolution,
 and the specifier fails during resolution"
was true for an older revision and no longer holds;
 the
prototype fixture re-runs cleanly when the same `_config.json`
is given an `external` list.
)

Tradeoff:
 must enumerate each subpath;
 missed entries still
trigger the warning.
 For a small fixed set of `node:`
subpaths this is workable,
 but it does not generalise (e.g.
to a glob covering every `node:*/promises` entry);
 a
function-form `external: (id) => id.startsWith('node:')`
generalises but loses the static analysis upstream consumers
might rely on.

### Prototype: the one-line gate flip is too coarse

Patch applied to a fresh upstream clone
(`git remote get-url origin` =
`https://github.com/rolldown/rolldown.git`,
`git rev-parse HEAD` = `14f967a4330465c6fb7c402a5067971d2f0f13d9`):

```diff
--- a/crates/rolldown_resolver/src/resolver_config.rs
+++ b/crates/rolldown_resolver/src/resolver_config.rs
@@ -130,7 +130,7 @@ impl ResolverConfig {
       restrictions: vec![],
       roots: vec![],
       symlinks: resolve_options.symlinks.unwrap_or(true),
-      builtin_modules: matches!(platform, Platform::Node),
+      builtin_modules: true,
       module_type: true,
       allow_package_exports_in_directory_resolve: false,
       yarn_pnp: resolve_options.yarn_pnp.unwrap_or(false),
```

Effect on the reproduction fixture (the warnings block
disappears,
 the emitted module is unchanged):

````diff
--- pre-patch/artifacts.snap
+++ post-patch/artifacts.snap
@@ -1,32 +1,3 @@
-# warnings
-
-## UNRESOLVED_IMPORT
-
-```text
-[UNRESOLVED_IMPORT] Could not resolve 'node:path' in main.js
-...
-```
-
-## UNRESOLVED_IMPORT
-
-```text
-[UNRESOLVED_IMPORT] Could not resolve 'node:path/posix' in main.js
-...
-```
-
 # Assets

 ## main.js
````

Broader integration-test impact
(`cargo test -p rolldown --test integration`,
1710 pass / 5 fail;
 the 5 failures are environmental
pre-existing baselines verified by re-running the same
fixtures with the patch stashed:
 `cjs-module-lexer` and
`util-deprecate` npm packages are not installed in the
clone,
 and the `test262` submodule is not present).
 The
patch updated 9 existing snapshots in the same run:

- 7 esbuild ports
  (`export_fs_browser`,
   `export_fs_node`,
  `export_fs_node_in_common_js_module`,
   `import_fs_browser`,
  `re_export_fs_node`,
   `require_fs_browser`,
  `lower_export_star_as_name_collision_no_bundle`)
  and `topics/generated_code/symbols_ns2` lose their
  pre-existing `UNRESOLVED_IMPORT` warnings for bare
  builtins (`fs`,
   `path`,
   `assert`) on default-Browser
  platform.
   These are the same bug surface,
   just hit by
  bare names rather than `node:`-prefixed ones.
- **1 genuine regression**:
  `package_json_browser_map_native_module_disabled`.
   The
  fixture's `node_modules/demo-pkg/package.json` carries
  `{ "browser": { "fs": false } }` to disable the bare `fs`
  import under default-Browser.
   After the patch,
   oxc-resolver
  intercepts `fs` as a builtin before the browser-field map
  applies,
   and the bundler emits a runtime `__require("fs")`
  instead of the disabled empty stub:

````diff
--- pre-patch/artifacts.snap
+++ post-patch/artifacts.snap
@@ -8,10 +8,7 @@ source: crates/rolldown_testing/src/integration_test.rs
 ```js
 // HIDDEN [\0rolldown/runtime.js]
-//#region (ignored) node_modules/demo-pkg
-var require_demo_pkg$1 = /* @__PURE__ */ __commonJSMin((() => {}));
-//#endregion
 //#region src/entry.js
 var import_demo_pkg = /* @__PURE__ */ __toESM((/* @__PURE__ */ __commonJSMin(((exports, module) => {
-	const fs = require_demo_pkg$1();
+	const fs = __require("fs");
 	module.exports = function() {
 		return fs.readFile();
````

The gate flip therefore solves the documented `node:`-prefix
symptom but bypasses browser-field disabling for bare
builtins;
 it is not the minimal fix.
 A correct upstream
change either (a) adds a `node:`-prefix-only flag in
oxc-resolver (e.g. `builtin_modules_node_prefix`) that is
separable from the bare-name builtin check,
 or
(b) intercepts `node:`-prefixed builtins inside rolldown
itself,
 in `resolve_id_with_plugins.rs:122-131` immediately
after the http/data-url auto-external block,
 so bare names
still flow through the browser-field map.
 Both options keep
`builtin_modules: matches!(platform, Platform::Node)`
intact for bare names.

### Why we do not file this upstream (audit, post-prototype)

1. **Is it really upstream's fault?
   ** Yes;
    the resolver's
   builtin check should fire for `node:` subpaths regardless
   of platform when the source code spells the prefix.
2. **Can upstream fix it?
   ** Yes,
    but **not** via the gate
   flip in `resolver_config.rs:133`.
    The prototype showed
   that toggling the single `builtin_modules` flag also
   bypasses the browser-field map for bare names,
    breaking
   `package_json_browser_map_native_module_disabled`.
    The
   correct fix is a `node:`-prefix-only flag in oxc-resolver
   (`builtin_modules_node_prefix`,
    evaluated even when
   `builtin_modules` is `false`),
    or rolldown-side `node:`
   interception in `resolve_id_with_plugins.rs:122-131`
   right after the http/data-url auto-external block.
    Both
   are still small changes;
    the latter is implementable
   entirely in rolldown without an oxc-resolver release.
3. **Are they supporting this use case?
   ** Yes;
    cross-runtime
   bundles are a documented goal.
4. **Will they likely fix it?
   ** Plausible;
    the targeted fix
   is two file-local changes.
5. **Have we prototyped a minimal fix?
   ** Yes,
    the one-line
   gate flip.
    The prototype is the evidence that the naive
   fix is wrong,
    and pinpoints the surgical fix above.
    The
   diff and fixture above are sufficient to reproduce and
   re-validate the next attempt.

Decision:
 worth filing upstream;
 the audit and prototype
both inform the report.

### Draft upstream issue (do not file as-is)

````md
## Title

resolver: `node:` subpath imports fail under `platform: 'neutral' | 'browser'`
because `builtin_modules` is gated on `Platform::Node`

## Description

Under `platform: 'neutral'` or `platform: 'browser'`, importing any `node:` subpath specifier
emits `UNRESOLVED_IMPORT` with the misleading "Main fields must be configured explicitly
when using the 'neutral' platform" help text:

```text
[UNRESOLVED_IMPORT] Could not resolve 'node:path/posix' in main.js
   ╭─[ main.js:1:23 ]
   │
 1 │ import { parse } from 'node:path/posix';
   │                       ────────┬────────
   │                               ╰────────── Module not found, treating it as an external dependency
   │
   │ Help: The "main" field here was ignored. Main fields must be configured explicitly when using the "neutral" platform.
───╯
```

Affected specifiers include `node:path/posix`, `node:path/win32`, `node:stream/promises`,
`node:stream/consumers`, `node:stream/web`, `node:dns/promises`,
`node:readline/promises`, `node:timers/promises`, and every other entry in
`nodejs_built_in_modules::BUILTINS` that has a `/` in its name. Bare parent specifiers
(`node:path`, `node:assert`) hit the same warning. Users can suppress per-specifier
warnings by enumerating each one in `external` (the literal-string match in
`IsExternal::StringOrRegex` short-circuits before the resolver runs), but that does not
scale to a project importing many `node:*/promises`-style subpaths.

## Reproduction

Fixture under `crates/rolldown/tests/rolldown/resolve/`:

```json
// _config.json
{ "config": { "platform": "neutral" } }
```

```js
// main.js
import { posix, } from 'node:path';
import { parse, } from 'node:path/posix';
console.log(parse('/a/b',), posix.parse('/a/b',),);
```

Run: `cargo test -p rolldown --test integration <fixture-name>`. The snapshot
captures both `node:path` and `node:path/posix` `UNRESOLVED_IMPORT` warnings.

## Root cause

`crates/rolldown_resolver/src/resolver_config.rs:133` sets
`builtin_modules: matches!(platform, Platform::Node)`. oxc-resolver's
`require_core` (`src/lib.rs:466-481` in `oxc-resolver` v11.19.1) only intercepts
`node:` specifiers when `builtin_modules` is `true`. Under non-Node platforms the
specifier falls through to `crates/rolldown/src/module_loader/resolve_utils.rs:103-148`'s
NotFound branch, which externalises with the wrong warning + help text.

## Suggested fix

The naive `builtin_modules: true` fix is too coarse: oxc-resolver's `require_core`
also intercepts bare names (`fs`, `path`, `assert`) without a `node:` prefix, so flipping
the flag bypasses the browser-field map for bare builtins. The existing
`crates/rolldown/tests/esbuild/packagejson/package_json_browser_map_native_module_disabled`
fixture exercises exactly this: `{ "browser": { "fs": false } }` disables bare `fs`,
and the naive flip emits a runtime `__require("fs")` instead of the disabled stub.

Two minimal alternatives keep bare-name handling unchanged:

1. **Rolldown-side `node:` interception** (no oxc-resolver release needed).
   In `crates/rolldown_plugin/src/utils/resolve_id_with_plugins.rs:122-131`,
   right after the http/data-url auto-external block, add:

   ```rust
   if specifier.starts_with("node:") && is_nodejs_builtin_module(specifier) {
     return Ok(Ok(ResolvedId {
       is_external_without_side_effects: true,
       id: ModuleId::new(specifier),
       external: true.into(),
       ..Default::default()
     }));
   }
   ```

   `is_nodejs_builtin_module` is already imported at the top of the file.

2. **Distinct flag in oxc-resolver**. Split `builtin_modules` into
   `builtin_modules_bare_names` (the existing semantic) and
   `builtin_modules_node_prefix` (always on when honoured), then update
   `crates/rolldown_resolver/src/resolver_config.rs` to set the prefix flag
   unconditionally and keep the bare-name flag gated on `Platform::Node`.

Option (1) is preferred for being entirely a rolldown change. Either fix passes
the reproduction fixture above and leaves
`package_json_browser_map_native_module_disabled` snapshot unchanged.

## Notes

- The naive fix
  (`builtin_modules: matches!(platform, Platform::Node) -> builtin_modules: true`)
  was prototyped and rejected; see
  [rolldown.md Bug 4](./rolldown.md#bug-4-node-subpath-imports-produce-unresolved_import-under-non-node-platforms)
  for the full snapshot deltas (1 regression + 8 incidental fixes for the
  bare-name surface of the same bug).
- The misleading "Main fields" help text in `resolve_utils.rs:123-125` could also
  be tightened so it only fires for specifiers that look like package names, not
  ones prefixed with `node:`.
````
