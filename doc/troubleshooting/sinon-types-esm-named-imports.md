# `@types/sinon` 22.0.1 `index.d.mts` breaks named `sinon` imports for ESM TypeScript consumers

`@types/sinon` 22.0.1 added an `exports` map whose `import` condition points at a
three-line `index.d.mts` that only default-exports the sinon namespace,
so every
named import from `sinon` fails under `tsc` while the runtime module and oxlint's
resolver both expose the named surface.

## Symptom

Any named import from `sinon` in an ESM TypeScript program fails at the import
site with TS2614,
whether the member is a value or a type:

```ts
// package/module/test-sandbox/src/sinon.ts
import {
  createSandbox,
  type SinonSandbox,
  type SinonSandboxConfig,
} from 'sinon';
```

```text
src/sinon.ts(2,3): error TS2614: Module '"sinon"' has no exported member 'createSandbox'. Did you mean to use 'import createSandbox from "sinon"' instead?
src/sinon.ts(3,8): error TS2614: Module '"sinon"' has no exported member 'SinonSandbox'. Did you mean to use 'import SinonSandbox from "sinon"' instead?
src/sinon.ts(4,8): error TS2614: Module '"sinon"' has no exported member 'SinonSandboxConfig'. Did you mean to use 'import SinonSandboxConfig from "sinon"' instead?
```

Trigger patterns and their error variants:

- `import { createSandbox, } from 'sinon';` (value) produces TS2614.
- `import type { SinonSpy, } from 'sinon';` (type) produces TS2614.
- `import { type SinonStub, } from 'sinon';` (inline type) produces TS2614.
- Lost type inference then cascades:
  `spy.getCalls().map((call,) => ...)`
  produces `error TS7006: Parameter 'call' implicitly has an 'any' type`
  at each callback site.
- Default imports are unaffected:
  `import sinon from 'sinon';` compiles.

The same TS2614 wording appears for every named member;
the count varies with
how many names the import statement lists.

## Root cause

The call chain from import site to failure,
step by step:

### Step 1: `sinon` ships no types of its own

`sinon@22.1.0` declares no `types` field and its `exports` map names no `types`
condition,
so the declaration files come from `@types/sinon`:

```text
$ node -p "require('sinon/package.json').exports"
{
  browser: './pkg/sinon-esm.js',
  require: './lib/sinon.js',
  import: './pkg/sinon-esm.js'
}
```

### Step 2: `@types/sinon` 22.0.1 routes ESM importers at `index.d.mts`

```jsonc
// node_modules/.pnpm/@types+sinon@22.0.1/node_modules/@types/sinon/package.json:50-55
"types": "index.d.ts",
"exports": {
  ".": {
    "import": "./index.d.mts",
    "require": "./index.d.ts"
  },
  "./package.json": "./package.json"
},
```

`tsc --traceResolution` confirms the match under the workspace's
`moduleResolution: bundler` (`package/config/typescript/tsconfig.options.json`):

```text
Using 'exports' subpath '.' with target './index.d.mts'.
======== Module name 'sinon' was successfully resolved to '.../@types/sinon/index.d.mts' with Package ID '@types/sinon/index.d.mts@22.0.1'. ========
```

### Step 3: `index.d.mts` exposes only `default`

```ts
// node_modules/.pnpm/@types+sinon@22.0.1/node_modules/@types/sinon/index.d.mts:1-3
import sinon = require("./index.js");

export default sinon;
```

All named members live in `index.d.ts`,
which declares namespace `Sinon` and ends
with:

```ts
// node_modules/.pnpm/@types+sinon@22.0.1/node_modules/@types/sinon/index.d.ts:1468-1471
declare const Sinon: Sinon.SinonStatic;

export = Sinon;
export as namespace sinon;
```

A module whose only export is `default` has no named members,
so every
`import { X } from 'sinon'` raises TS2614 and the hint names the default import
as the only available shape.

### Why this is new

`@types/sinon@22.0.0` published no `exports` map (`npm view @types/sinon@22.0.0
exports types` reports only `"index.d.ts"`),
so imports resolved `index.d.ts`
and the `export = ` namespace supported named imports.
PR
[DefinitelyTyped#75759][],
merged 2026-10-21 and published as 22.0.1,
added the
ESM entry to satisfy `@arethetypeswrong/cli`;
the file at DefinitelyTyped
`types/sinon/index.d.mts` (master `5ac70999087a3bd6ea4c70f80c1e3ea797a50c4d`)
still carries the three-line shape.

### An earlier reading was wrong

A first probe concluded that named imports still worked under TypeScript
7.1.0-beta,
which contradicted the observed failures.
The probe program was
contaminated:
its `augment.ts` declared `declare module 'sinon'` re-exports,
and that module augmentation supplied the missing names to every file in the
program.
Moving `augment.ts` out of the program made the same probe reproduce
TS2614 on every named import,
matching the workspace failures exactly.

## Verification

Versions under test:
`sinon@22.1.0`,
`@types/sinon@22.0.1`,
`@types/sinon-chai@4.0.1`,
`typescript@7.1.0-beta`,
`pnpm@12.14.0`.
The
`index.d.mts` content verified against DefinitelyTyped master
`5ac70999087a3bd6ea4c70f80c1e3ea797a50c4d` is byte-identical to the published
22.0.1 file,
so the prototype patch below applies to upstream source unchanged.

Runnable harness,
mirroring the workspace's compiler options
(`module: preserve`,
`moduleResolution: bundler`,
`esModuleInterop`,
`verbatimModuleSyntax`,
`erasableSyntaxOnly`,
`isolatedDeclarations`):

```sh
# throwaway fixture under ${HOME}/temp/agent/sinon-probe
mkdir -p "${HOME}/temp/agent/sinon-probe"
cd "${HOME}/temp/agent/sinon-probe"
printf '{"name":"sinon-probe","private":true,"type":"module"}' > package.json
pnpm add --save-dev sinon@22.1.0 @types/sinon@22.0.1 @types/sinon-chai@4.0.1
# tsconfig.json mirrors package/config/typescript/tsconfig.options.json
node <repo>/node_modules/typescript/bin/tsc --project .
```

### Patterns that fail

- `import { createSandbox, match, } from 'sinon';` (values):
  TS2614 per name.
- `import type { SinonSpy, } from 'sinon';` (type):
  TS2614.
- `import { type SinonSandbox, } from 'sinon';` (inline type):
  TS2614.
- Callback parameters typed through a failed `SinonSpy` import:
  TS7006.

### Patterns that work cleanly

- `import sinon from 'sinon';` with `sinon.createSandbox()`,
  `sinon.match`,
  and type access `sinon.SinonSpy`,
  `sinon.SinonSandbox`.
- `import type sinon from 'sinon';` for type-only sites.
- Runtime named imports,
  which is why the types are wrong rather than the
  code:
  `node --input-type=module -e "const m = await import('sinon');
  console.log(Object.keys(m).join(','))"` prints
  `addBehavior,assert,createSandbox,...,default,define,...,verifyAndRestore`,
  named exports and `default` together.

## Verified workarounds

### Restore named imports with a module declaration (applied in this repo)

`package/module/test-sandbox/src/sinon-named-exports.d.ts` declares the named
surface the runtime actually exports,
and `package/module/test-sandbox/src/sinon.ts:1`
carries `/// <reference path="./sinon-named-exports.d.ts" />` so every program
compiling the sandbox boundary receives the augmentation:

```ts
// package/module/test-sandbox/src/sinon-named-exports.d.ts
import sinon from 'sinon';

declare module 'sinon' {
  export const createSandbox: typeof sinon.createSandbox;
  export const match: typeof sinon.match;
  export type SinonMatcher = sinon.SinonMatcher;
  // ... one line per member the harness names
}
```

Tradeoffs:
only listed members become visible,
so new named uses need a shim
line;
the augmentation reaches only programs that include the file;
a `.d.ts`
keeps the repo's ambient-shape lint carve-outs
(`package/config/oxlint/src/overrides.ts`),
and call sites keep the named-import
style oxlint expects.

### Consume the default export (fixture-verified, not applied)

Rewrite each import site as `import sinon from 'sinon';` with local aliases
(`type SinonSpy = sinon.SinonSpy;`) and destructuring
(`const { createSandbox, } = sinon;`).
Compiles under the same options and keeps
working if upstream keeps `export default`.
Tradeoffs:
every call site leaves the
named-import style,
oxlint `no-named-as-default-member` warns at each destructure
("sinon also has a named export createSandbox"),
`tsdoc/require-tsdoc` demands
doc comments on the aliases (its inline suppression is banned by
`no-restricted-syntax/no-disable-require-tsdoc`),
and `isolatedDeclarations`
forces explicit annotations on re-exported values.

## What does not work

- Pinning `@types/sinon` back to 22.0.0.
  It restores the pre-22.0.1 resolution
  and named imports,
  but it was explicitly rejected (user directive "No
  pinning.",
  2026-10-23) and it fights the workspace catalog `>=` policy under
  `resolutionMode: highest`.
  Recorded so future sessions do not re-propose it.
- `export * from "./index.js";` in `index.d.mts` (prototype variant A).
  Star
  re-export of an `export = ` target yields no members;
  TS2614 persists.
- `export = sinon;` alone in `index.d.mts` (prototype variant G).
  Named imports
  recover but default imports break with
  `error TS1192: Module '.../index' has no default export`,
  which would break
  `@types/sinon-chai` itself (`import Sinon from "sinon";`).
- `skipLibCheck: true`.
  The errors are raised at the consumer's import
  statement,
  not inside the declaration files.
- Named `import type { X } from 'sinon'`.
  Type-only syntax changes nothing
  about module resolution.

## Upstream filing artifact

### Duplicate search

`gh search issues` and `gh search prs` over
DefinitelyTyped/DefinitelyTyped,
open and closed,
with
"sinon export default named",
"sinon index.d.mts",
"sinon named exports",
and
"sinon default export" found no matching issue or pull request.
The closest hit
is [DefinitelyTyped#75759][] itself,
the merged change that introduced the file.
`.out-of-scope/` holds no sinon or DefinitelyTyped exemption (checked 2026-10-23;
its entries cover bun,
cargo,
claude-code,
codex,
jsr,
lightningcss,
pi,
module-es-monolith,
low-impact-typescript-formatting,
terminal-title,
and
typescript-project-references topics).

### Upstream filing decision

Walking the 6 constraints:

#### 1. Is it really upstream's fault?

Yes.
The published `index.d.mts` names a module shape (default only) that
contradicts both the runtime ESM module (named exports plus default) and the
CJS declaration the same package serves to `require` callers.

#### 2. Can upstream fix it?

Yes.
Re-exporting the named surface from `index.d.mts` is a single-file change;
the prototype below is that change.

#### 3. Are they supporting this use case?

Yes.
The `exports` map exists specifically to give ESM consumers correct types
(the `resolves attw` goal of [DefinitelyTyped#75759][]),
and named imports are
the runtime-supported ESM surface.

#### 4. Would the repo welcome our contribution?

Yes,
with soft evidence.
`README.md`,
`.github/PULL_REQUEST_TEMPLATE.md`,
and a
code search for AI-contribution policy found no ban;
`CONTRIBUTING.md` does not
exist.

#### 5. Will they likely fix it?

Plausible.
No tracker signal against;
the file was touched two days before the
incident ([DefinitelyTyped#75759][]) and nothing indicates a won't-fix.

#### 6. Have we prototyped a minimal fix compatible with their architecture?

Yes.
The patch keeps the dual CJS/ESM entry layout and only gives the ESM entry
the surface the runtime has:
`export default sinon;` plus one
`export const`/`export type` re-export per member.
Verification command and
result:

```text
$ node <repo>/node_modules/typescript/bin/tsc --project .
TSFILE: /home/user/temp/agent/sinon-probe/.cache/typescript/tsbuildinfo
```

with `named.ts` (all named imports),
`full-surface.ts` (all 28 runtime values
and 38 namespace types re-exported),
`typeonly.ts`,
`default-ns.ts`,
and
`option2.ts` (default-import shapes) all compiling clean.
The full diff is
[sinon-types-esm-named-imports.patch](sinon-types-esm-named-imports.patch),
saved beside this doc because it exceeds inline size.

All six constraints hold,
so the draft below is fileable.
It stays unfiled:
the user chose to keep it as a record on 2026-10-23,
and opening an
upstream issue is an external action this repo takes only on request.

### Draft (fileable; kept unfiled by choice)

~~~md
Title: types/sinon: `index.d.mts` drops the named export surface that the
runtime and `index.d.ts` both provide

Labels: Critical package

## Description

`@types/sinon` 22.0.1 (from PR #75759) routes ESM importers at
`types/sinon/index.d.mts`:

```ts
import sinon = require("./index.js");

export default sinon;
```

so every named import from `sinon` fails under a TS program resolving the
`import` condition:

```text
error TS2614: Module '"sinon"' has no exported member 'createSandbox'. Did you mean to use 'import createSandbox from "sinon"' instead?
```

The runtime module (`sinon@22.1.0` `pkg/sinon-esm.js`) exports named members
(`createSandbox`, `match`, `spy`, `stub`, ...) and a default, and
`types/sinon/index.d.ts` serves the same named surface to `require` callers via
`export = Sinon`. Only the ESM declaration loses it. 22.0.0 had no `exports`
map and worked for named imports.

## Reproduction

```sh
mkdir -p /tmp/types-sinon-repro && cd /tmp/types-sinon-repro
printf '{"name":"repro","private":true,"type":"module"}' > package.json
pnpm add --save-dev sinon@22.1.0 @types/sinon@22.0.1 typescript
printf '{"compilerOptions":{"module":"preserve","moduleResolution":"bundler","esModuleInterop":true,"noEmit":true}}' > tsconfig.json
printf "import { createSandbox, type SinonSpy } from 'sinon';\nexport type Q = SinonSpy;\nexport const s = createSandbox();\n" > repro.ts
./node_modules/.bin/tsc --project .
```

Expected: compiles. Actual: TS2614 on both named members.

## Suggested fix

Rewrite `types/sinon/index.d.mts` to keep `export default sinon;` and add one
re-export per member of `Sinon.SinonStatic` and namespace `Sinon`, e.g.
`export const createSandbox: typeof sinon.createSandbox;` and
`export type SinonSpy = sinon.SinonSpy;`. The complete patch is attached to the
workspace record (sinon-types-esm-named-imports.patch) and passes a probe
importing every name.
~~~

[DefinitelyTyped#75759]: https://github.com/DefinitelyTyped/DefinitelyTyped/pull/75759
