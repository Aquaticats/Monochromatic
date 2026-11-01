# opentype.js 2.0.0's CJS/UMD `main` drops named exports under Node's ESM interop,
and the workaround trips oxlint's `no-named-as-default-member`

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

`node src/build-font.ts` (this repo's `package/typeface/aquaticat` `build:font`
task) threw at the first constructor call:

```text
TypeError: opentype.Path is not a constructor
    at .../package/typeface/aquaticat/src/build-font.ts:87
```

with the import written as a namespace import:

```ts
import * as opentype from 'opentype.js';
// opentype.Path, opentype.Glyph, opentype.Font are all undefined
```

Switching to a real named import produces a harder failure,
 a `SyntaxError` at
module-load time rather than a runtime `TypeError`:

```text
$ node -e "import('opentype.js').then(() => {})" --input-type=module
SyntaxError: Named export 'Path' not found. The requested module 'opentype.js' is a CommonJS module, which may not support all module.exports as named exports.
CommonJS modules can always be imported via the default export, for example using:

import pkg from 'opentype.js';
const { Path } = pkg;
```

(reproduced directly against the installed `opentype.js@2.0.0` package with
`import { Path } from 'opentype.js';`).

Switching to a default import (`import opentype from 'opentype.js';`) fixes the
runtime problem,
 but then trips a different,
 unrelated tool:
 oxlint's
`import/no-named-as-default-member` warns on every `opentype.Path`,
`opentype.Glyph`,
 `opentype.Font` access:

```text
! import(no-named-as-default-member): "opentype" also has a named export "Path"
  help: Check if you meant to write `import { Path } from "opentype.js"`
```

## Root cause

### Node can't see opentype.js's real ESM build

`opentype.js@2.0.0`'s `package.json` declares:

```json
"main": "./dist/opentype.js",
"browser": "./dist/opentype.js",
"module": "./dist/opentype.mjs"
```

(`opentypejs/opentype.js` at tag `2.0.0`,
 commit
`e2eaedebfa6187c1b435a87e6b7b02e6f1ba1b48`,
 `package.json:23-25`.)

There is no `"exports"` field.
 `"module"` is a bundler-only convention (Rollup,
webpack,
 esbuild honor it);
 Node's native resolver does not read it at all.
 So
every `import ... from 'opentype.js'` in plain Node resolves to `main`,
 the
CommonJS/UMD bundle at `dist/opentype.js`,
 even though a correct ES module
build already exists at `dist/opentype.mjs` and is never reached.

`dist/opentype.mjs` (built from `src/opentype.mjs` via the `b:esm` script,
`package.json:35`) ends with a plain static export list:

```js
// dist/opentype.mjs (built from src/opentype.mjs:468-477)
export {
    Font,
    Glyph,
    Path,
    BoundingBox,
    parse as _parse,
    parseBuffer as parse,
    load,
    loadSync
};
```

If Node ever loaded this file,
 `import { Path } from 'opentype.js'` would work
with no interop involved.
 It never does,
 because nothing in `package.json`
routes the `import` condition there.

### The CJS bundle's UMD footer defeats `cjs-module-lexer`

`dist/opentype.js` (built by the `b:umd` script,
 `package.json:33`) is an IIFE
that assigns the real export object to a local `opentype` variable,
 then a UMD
footer copies it into `module.exports` through a spread expression:

```js
// dist/opentype.js (installed opentype.js@2.0.0, tail of file)
var opentype = (() => { /* ...__toCommonJS(opentype_exports)... */ })();
(function (root, factory) {
  if (typeof define === 'function' && define.amd) define(factory);
  else if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.opentype = factory();
}(typeof self !== 'undefined' ? self : this, () => ({...opentype, 'default': opentype})));
```

At runtime `module.exports` genuinely has `Path`,
 `Glyph`,
 `Font`,
 etc. as own
properties (the spread evaluates the getters `__toCommonJS` defined).
 But
Node's CJS-to-ESM interop does not evaluate the module to find this out;
 it
uses `cjs-module-lexer` to *statically* scan the source text for assignment
patterns it recognizes (`exports.x = ...`,
 `module.exports = { a, b, c }`
object literals,
 etc.).
 `module.exports = factory()` is an opaque function
call whose return value is a spread expression,
 not a form the lexer
recognizes,
 so it detects zero named exports.

Reproduced directly (`/tmp/claude-*/scratchpad/opentype-repro`,
 Node
`v26.4.0`),
 a minimal file reproducing only the getter-object-plus-UMD-footer
shape:

```js
// pkg.cjs
var opentype = (() => {
  var __defProp = Object.defineProperty;
  var exportsObj = {};
  __defProp(exportsObj, "Path", { get: () => class Path {}, enumerable: true });
  __defProp(exportsObj, "__esModule", { value: true });
  return exportsObj;
})();
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
}(this, () => ({...opentype, 'default': opentype})));
```

```js
// test-ns.mjs
import * as ns from './pkg.cjs';
console.log('namespace keys:', Object.keys(ns));
console.log('ns.Path:', ns.Path);
console.log('ns.default.Path:', ns.default ? ns.default.Path : undefined);
```

```text
$ node test-ns.mjs
namespace keys: [ 'default', 'module.exports' ]
ns.Path: undefined
ns.default.Path: [class Path]
```

The synthetic namespace only carries `default` and the always-present
`'module.exports'` fallback key.
 `Path` is missing,
 matching the original
`TypeError` exactly.

A default import bypasses the lexer entirely:
 Node always binds a default
import to the literal runtime value of `module.exports`,
 no static export
list needed.

```js
// test-default.mjs
import def from './pkg.cjs';
console.log('default keys:', Object.keys(def));
console.log('def.Path:', def.Path);
```

```text
$ node test-default.mjs
default keys: [ 'Path', 'default' ]
def.Path: [class Path]
```

This is the mechanism the suggested fix in Monochromatic issue #267 relies on,
and it is why a real *named* import (`import { Path } from 'opentype.js'`)
fails harder (a load-time `SyntaxError`,
 shown in Symptom) than a namespace
import:
 named-import bindings are also resolved from the lexer's static list,
which is empty here.

### The default-import workaround collides with oxlint's `no-named-as-default-member`

Once the fix switches to `import opentype from 'opentype.js';`,
 oxlint's
`import/no-named-as-default-member` (`crates/oxc_linter/src/rules/import/no_named_as_default_member.rs:77-104`
in `oxc-project/oxc`) flags every `opentype.Path`/`opentype.Glyph`/`opentype.Font`
access.
 The rule resolves the imported module's *declared* named exports
(`module_record.get_loaded_module(specifier)` -> `exported_bindings`) and
warns whenever a default-imported binding's property access matches one of
those names,
 on the theory that the author meant to write
`import { Path } from '...'` instead.

In this repo,
 `package/typeface/aquaticat/src/env.d.ts` is the ambient module
declaration that supplies opentype.js's types (the package ships none).
 It
necessarily declares `Path`,
 `Glyph`,
 `Font` as named exports too,
 because two
sibling files (`build-font-paths.ts`,
 `build-font-paths-stroked.ts`) use
`import type * as opentype from 'opentype.js';` and reference `opentype.Path`
as a type.
 oxlint sees those declared named exports and,
 correctly per its own
logic,
 flags the default-import member access as the exact footgun the rule
exists to catch.
 It has no way to know the declared named exports are
runtime-dead under Node's CJS interop.

The oxlint configuration schema confirms this rule takes no options:

```text
$ python3 -c "..." # search node_modules/oxlint/configuration_schema.json
"import/no-named-as-default-member": {
  "$ref": "#/definitions/RuleNoConfig"
}
```

(`node_modules/oxlint/configuration_schema.json`,
 `oxlint@1.72.0`.)
 There is
no allow-list or per-import escape hatch,
 only whole-rule severity.

## Verification

Versions under test:
 `opentype.js@2.0.0` (installed via pnpm catalog
`'opentype.js': '>=2.0.0'`),
 Node.js `v26.4.0`,
 oxlint `1.72.0`.

Patterns and their outcome against the installed `opentype.js@2.0.0`:

- `import * as opentype from 'opentype.js'; opentype.Path` -> `opentype.Path`
  is `undefined`;
   `new opentype.Path()` throws `TypeError: opentype.Path is
  not a constructor` (the original bug).
- `import { Path } from 'opentype.js';` -> `SyntaxError: Named export 'Path'
  not found` at module load (harder failure,
   shown in Symptom).
- `import opentype from 'opentype.js'; opentype.Path` -> works;
   `opentype` is
  bound to the real `module.exports` object,
   which does have `Path`.

## Verified workarounds

### Default import plus a type-only named import (applied in this repo)

`package/typeface/aquaticat/src/build-font.ts`:

```ts
// before
import * as opentype from 'opentype.js';
// ...
): opentype.Glyph[] {

// after
import opentype from 'opentype.js';
import type { Glyph, } from 'opentype.js';
// ...
): Glyph[] {
```

`package/typeface/aquaticat/src/env.d.ts` gains a default export matching the
real runtime shape of `module.exports`:

```ts
declare module 'opentype.js' {
  export class Path { /* ... */ }
  export class Glyph { /* ... */ }
  export class Font { /* ... */ }
  export function parse(buffer: ArrayBuffer,): Font;

  const opentype: {
    Path: typeof Path;
    Glyph: typeof Glyph;
    Font: typeof Font;
    parse: typeof parse;
  };
  export default opentype;
}
```

Tradeoff:
 this makes the ambient declaration describe two shapes for the same
module (real named exports for type-only consumers,
 a default object for
value consumers),
 which is exactly what trips the oxlint rule below.
 It is
accurate to the module's actual runtime behavior,
 just not to what a
"clean" ESM package would look like.

### Scoped oxlint suppression for the resulting `no-named-as-default-member` warnings

No allow-list exists for this rule (see Root cause),
 so each flagged line in
`build-font.ts` needs a scoped disable comment naming the reason,
 per this
repo's lint-suppression convention (`AGENTS.md` `LN3`):

```ts
// oxlint-disable-next-line import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection (see doc/troubleshooting/opentype-js-cjs-esm-interop.md); the default import's .Path/.Glyph/.Font members are the only ones that work under Node's CJS/ESM interop
const path = new opentype.Path();
```

Tradeoff:
 this is a per-call-site annotation tax (six sites in
`build-font.ts`),
 and it locally silences a rule that is genuinely useful
elsewhere in the codebase,
 so it must stay scoped to these exact lines rather
than disabling the rule package-wide.

## What does not work

- **Namespace import** (`import * as opentype`):
   the original bug;
   `.Path`
  etc. are `undefined` at runtime (Verification).
- **Real named import** (`import { Path } from 'opentype.js'`):
   fails harder,
  a load-time `SyntaxError`,
   because named bindings go through the same empty
  lexer-detected export list as the namespace import (Verification).
- **Declaring only a default export in `env.d.ts` and dropping the named
  exports entirely**:
   breaks the two sibling files
  (`build-font-paths.ts`,
   `build-font-paths-stroked.ts`) that use
  `import type * as opentype from 'opentype.js';` for `opentype.Path` as a
  type;
   they would need to switch to `import type { Path }` too,
   which is a
  larger,
   unrelated diff than this issue's scope.
- **Disabling `import/no-named-as-default-member` package-wide**:
   the rule is
  a genuine footgun-catcher elsewhere;
   only this module's ambient declaration
  creates the false positive.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    Yes.
    `opentype.js` already builds a
   correct,
    working ESM file (`dist/opentype.mjs`,
    confirmed by the direct
   prototype below) but never routes Node's native `import` resolution to it,
   because `package.json` has no `"exports"` field and relies on the
   bundler-only `"module"` convention instead.
2. **Can upstream fix it?**
    Yes.
    The prototype below is a two-file,
   sub-20-line change:
    add an `"exports"` map and add a matching `export
   default` to the ESM source so the previously-documented default-import
   usage keeps working too.
3. **Are they supporting this use case?**
    Yes.
    The README documents all
   three import styles as supported npm usage
   (`README.md:56-60`,
    checked at commit
   `e2eaedebfa6187c1b435a87e6b7b02e6f1ba1b48`):

   ```js
   const opentype = require('opentype.js');
   import opentype from 'opentype.js'
   import { load } from 'opentype.js'
   ```

   Only the first of these three documented forms is reliable in plain Node
   today;
    the prototype fixes all three (see Verification of the prototype).
4. **Would the repo welcome our contribution?**
    No ban found.
    Checked
   `.github/ISSUE_TEMPLATE.md`,
    `.github/PULL_REQUEST_TEMPLATE.md`,
    and the
   README's `## Contribute` section (`README.md:65-77`):
    standard fork/PR
   workflow,
    no AI-disclosure restriction,
    no statement against external
   contributions.
5. **Will they likely fix it?**
    Soft yes.
    Upstream issue
   [opentypejs/opentype.js#836](https://github.com/opentypejs/opentype.js/issues/836)
   is open (filed 2026-05-04) and covers the same root defect from a
   different angle (a Rollup consumer's `"default" is not exported by
   ".../opentype.mjs"` build error).
    Maintainers (`ILOVEPIE`,
    `Jolg42`) and a
   reporter (`z3dev`) are actively discussing packaging/build-format tradeoffs
   there as of 2026-05-19,
    with no consensus yet and no stated won't-fix.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Yes,
    see below.

Decision:
 **do not open a new issue**;
 #836 already covers this defect class
and a second report would be a duplicate.
 Post an additive comment there
instead (drafted below),
 since #836's own thread has no root-cause trace,
 no
`package.json`/`src` diff,
 and no confirmation that the fix also resolves the
plain-Node named/namespace-import failure this doc investigates (only the
Rollup default-import failure is discussed there).

### Prototype

Cloned `https://github.com/opentypejs/opentype.js.git` at tag `2.0.0`
(commit `e2eaedebfa6187c1b435a87e6b7b02e6f1ba1b48`) into a disposable
`/tmp/agent/` clone (origin and commit confirmed before editing).
 `npm
install` was withheld by this environment's own guardrails (running a
third-party package's lifecycle scripts),
 so the fix was verified without
bundling:
 `dist/` was symlinked to `src/` and `src/opentype.mjs`'s relative
imports resolved directly,
 which is sufficient to prove the `exports` map and
export-statement shape without exercising esbuild's (separately well-tested)
bundling step.

```diff
--- a/package.json
+++ b/package.json
@@ -23,6 +23,14 @@
   "main": "./dist/opentype.js",
   "browser": "./dist/opentype.js",
   "module": "./dist/opentype.mjs",
+  "exports": {
+    ".": {
+      "import": "./dist/opentype.mjs",
+      "require": "./dist/opentype.js",
+      "default": "./dist/opentype.js"
+    },
+    "./package.json": "./package.json"
+  },
   "scripts": {
     "build": "npm run b:umd && npm run b:esm",
     "dist": " npm run d:umd && npm run d:esm",
--- a/src/opentype.mjs
+++ b/src/opentype.mjs
@@ -475,3 +475,14 @@ export {
     load,
     loadSync
 };
+
+export default {
+    Font,
+    Glyph,
+    Path,
+    BoundingBox,
+    _parse: parse,
+    parse: parseBuffer,
+    load,
+    loadSync
+};
```

Verification harness (`/tmp/claude-*/scratchpad/opentype-exports-fix-test`,
`node_modules/opentype.js` symlinked to the patched clone):

```text
$ node test-namespace.mjs   # import * as opentype from 'opentype.js'
namespace keys sample: [ 'BoundingBox', 'Font', 'Glyph', 'Path', '_parse', 'default' ]
opentype.Path: [Function: Path] { fromSVG: [Function (anonymous)] }
new opentype.Path() works: Path

$ node test-default.mjs     # import opentype from 'opentype.js'
opentype.Path: [Function: Path] { fromSVG: [Function (anonymous)] }
new opentype.Path() via default works: Path

$ node test-named.mjs       # import { Path, Glyph, Font } from 'opentype.js'
named Path: [Function: Path] { fromSVG: [Function (anonymous)] }
new Path() via named import works: Path
```

All three README-documented import styles work post-patch,
 including the
exact namespace-import pattern this repo originally used before hitting the
bug in issue #267.

Tradeoff worth flagging in the comment:
 adding an `"exports"` field is not
purely additive.
 Once present,
 Node (and exports-aware bundlers) stop
resolving any subpath not explicitly listed,
 so a consumer currently doing
`require('opentype.js/dist/opentype.min.js')` or similar deep imports would
break unless those subpaths are added to the map too.
 This prototype only
adds the `"."` and `"./package.json"` entries.

### Draft comment on opentypejs/opentype.js#836 (fileable as-is)

~~~md
This is the same root defect from a different angle: no `"exports"` field
means Node's native resolver never reaches `dist/opentype.mjs` at all
(`"module"` is bundler-only, not read by Node), so plain-Node consumers
always get `dist/opentype.js`, the UMD/CJS bundle. That bundle's
`module.exports = factory()` UMD footer (a spread expression) defeats
`cjs-module-lexer`'s static named-export detection, so `import * as opentype`
and `import { load } from 'opentype.js'` (both documented in the README) fail
under plain Node too: the namespace import silently gets `opentype.load ===
undefined`, and the named import throws `SyntaxError: Named export 'load' not
found` at load time. Root-cause trace, minimal reproduction, and a full
writeup: [link to this doc once published, or inline the Root cause section].

Prototyped the minimal fix: add an `"exports"` map routing the `import`
condition to `dist/opentype.mjs`, and add `export default {...}` to
`src/opentype.mjs` so it mirrors the UMD bundle's `{...opentype,
'default': opentype}` self-referencing default. Verified against a patched
clone (commit `e2eaedebfa6187c1b435a87e6b7b02e6f1ba1b48` + patch) that all
three README-documented import styles now work in plain Node, including this
repo's own default-import workaround for #836's Rollup error:

```diff
--- a/package.json
+++ b/package.json
@@ -23,6 +23,14 @@
   "main": "./dist/opentype.js",
   "browser": "./dist/opentype.js",
   "module": "./dist/opentype.mjs",
+  "exports": {
+    ".": {
+      "import": "./dist/opentype.mjs",
+      "require": "./dist/opentype.js",
+      "default": "./dist/opentype.js"
+    },
+    "./package.json": "./package.json"
+  },
   "scripts": {
     "build": "npm run b:umd && npm run b:esm",
     "dist": " npm run d:umd && npm run d:esm",
--- a/src/opentype.mjs
+++ b/src/opentype.mjs
@@ -475,3 +475,14 @@ export {
     load,
     loadSync
 };
+
+export default {
+    Font,
+    Glyph,
+    Path,
+    BoundingBox,
+    _parse: parse,
+    parse: parseBuffer,
+    load,
+    loadSync
+};
```

One tradeoff worth deciding on purpose: once `"exports"` exists, any subpath
not listed (e.g. `opentype.js/dist/opentype.min.js`) stops resolving. This
patch only adds `"."` and `"./package.json"`; anyone relying on a deep import
today would need an explicit entry added.
~~~
