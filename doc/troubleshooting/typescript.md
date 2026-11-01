# TypeScript aggregator (native tsc 7.0.1-rc through 7.0.2 and classic tsc6 6.0.x): nine failure modes from dprint baseUrl warnings through `using` downlevel helpers

This file aggregates nine distinct TypeScript-related failure modes
encountered across the workspace.
 Each section follows the
troubleshooting-doc canonical structure (Symptom / Root cause /
Verification / Workaround / What does not work / 5-constraint
upstream-filing audit / Draft issue if warranted).
 Sections written
before the canonicalization push use "Problem" / "Solution" / "Root
Cause" headings,
 kept as-is to preserve git history;
 the content
matches the canonical shape.

## TypeScript Path Warnings with dprint

### Problem

You see warnings when running dprint or other tools:

```txt
warn: Non-relative path "package/config/oxlint/src/index.ts" is not allowed when "baseUrl" is not set (did you forget a leading "./"?)
```

### Solution

Set `baseUrl` to `"./"` in your root `tsconfig.json`:

```json
{
  "compilerOptions": {
    "baseUrl": "./"
  }
}
```

This tells TypeScript to resolve non-relative paths from the project root,
 which is necessary when using path mappings in a monorepo structure.

### Note

Setting `baseUrl` may or may not completely resolve the warnings,
 but it helps TypeScript understand that non-relative paths
in the `paths` mapping should be resolved from the project root.

### Verification

Reproduced under tsc 6.0.
x and dprint 0.
x.
 Trigger:
 any `paths` map
entry without a leading `./` when `baseUrl` is unset.

### Why we do not file this upstream

5-constraint walk:

1. **Upstream's fault?
   ** No. TypeScript / dprint correctly require
   `baseUrl` to be set when `paths` contains non-relative entries;
    the
   warning is informative,
    not a bug.
2. **Can upstream fix it?
   ** Not applicable;
    this is documented behavior.
3. **Supporting this use case?
   ** Yes;
    `paths` + `baseUrl` is a
   first-class TypeScript feature.
4. **Will they fix it?
   ** Not applicable.
5. **Minimal-fix prototype?
   ** Not applicable.

**Decision:
 no upstream report.
** Workspace config fix is the
correct response.

## Type Predicate Assignment Errors

### Problem

You encounter TypeScript error TS2677:
 "A type predicate's type must be assignable to its parameter's type"
when using complex conditional types in type predicates:

```ts
export function maybeAsyncSchemaIsSchemaAsync<
  const MyMaybeAsyncSchema extends MaybeAsyncSchema = MaybeAsyncSchema,
>(
  maybeAsyncSchema: MyMaybeAsyncSchema,
): maybeAsyncSchema is MyMaybeAsyncSchema extends
  SchemaAsync<infer Input, infer Output> ? SchemaAsync<Input, Output>
  : Schema & MyMaybeAsyncSchema // TS2677 error here
{
  return ('parseAsync' in maybeAsyncSchema);
}
```

### Root Cause

TypeScript cannot verify that complex conditional types in type predicates are assignable to the parameter type.
The compiler struggles with conditional types that depend on generic parameters,
 especially when trying to preserve the original type information.

### Solution

Use intersection types instead of conditional types in the type predicate:

```ts
export function maybeAsyncSchemaIsSchemaAsync<const Input = unknown,
  const Output = unknown,
  const MyMaybeAsyncSchema extends MaybeAsyncSchema<Input, Output> =
    MaybeAsyncSchema<
      Input,
      Output
    >,>(
  maybeAsyncSchema: MyMaybeAsyncSchema,
): maybeAsyncSchema is SchemaAsync<Input, Output> & MyMaybeAsyncSchema {
  return ('parseAsync' in maybeAsyncSchema);
}
```

### Why This Works

- The intersection type `SchemaAsync<Input, Output> & MyMaybeAsyncSchema`
  is always assignable to `MyMaybeAsyncSchema` (since it includes it)
- It preserves the specific type information of the input parameter
- It avoids the conditional type complexity that TypeScript cannot verify
- The type guard remains useful for narrowing types in calling code

### Common Pitfall to Avoid

Don't simplify by removing generic parameters entirely:

```ts
// BAD: Loses type precision
function maybeAsyncSchemaIsSchemaAsync<Input, Output,>(
  maybeAsyncSchema: MaybeAsyncSchema<Input, Output>,
): maybeAsyncSchema is SchemaAsync<Input, Output>;
```

This throws away the specific schema type information,
 making the type guard less useful for preserving types in calling code.

### Verification

Reproduced under tsc 6.0.
x (tsgo 7.0.0-dev exhibits identical
behavior).
 Trigger:
 type predicate with conditional type body
referencing the function's generic parameter.

### Why we do not file this upstream

5-constraint walk:

1. **Upstream's fault?
   ** Partial.
    TS2677 is a known limitation of
   conditional-type assignability in type predicates,
    documented in
   the TypeScript handbook as a constraint of the type system rather
   than a bug.
2. **Can upstream fix it?
   ** Possibly,
    but the fix would require
   extending conditional-type variance reasoning,
    a structural-core
   change touching the type relation algorithm.
    Not a small change.
3. **Supporting this use case?
   ** Conditional types in type predicates
   are not a documented first-class feature;
    the recommendation is
   intersection types as shown above.
4. **Will they fix it?
   ** Unlikely.
    Multiple long-standing tracker
   entries on conditional-type-in-predicate produce no movement.
5. **Minimal-fix prototype?
   ** Not feasible without touching the type
   relation core.

**Decision:
 no upstream report.
** The intersection-type workaround
satisfies the use case;
 the limitation is accepted as a tradeoff of
the type system's design.

## JSX.IntrinsicElements Missing in Astro MDX Files

### Problem

In VS Code,
 MDX files in Astro projects show TypeScript error ts-plugin(7026):

```txt
JSX element implicitly has type 'any' because no interface 'JSX.IntrinsicElements' exists.
```

This affects HTML elements like `<abbr>`,
 `<sub>`,
 `<sup>`,
 `<kbd>`,
 `<mark>`,
 etc. in MDX content.

### Root Cause

The `@types/mdx` package expects a global `JSX.IntrinsicElements` interface,
 which is normally provided by `@types/react`.
Astro defines its JSX types under `astroHTML.JSX` namespace,
 not the global `JSX` namespace.

From the MDX documentation:

> "For types to work,
>  the `JSX` namespace must be typed.
>  This is done by installing and using the types of your framework,
>  such as `@types/react`.
> "

This creates an incompatibility when using MDX with Astro without React.

### Solution

Create `src/env.d.ts` in your Astro project that bridges the namespaces:

```ts
/// <reference types="astro/client" />

declare namespace JSX {
  type Element = astroHTML.JSX.Element;
  type IntrinsicElements = astroHTML.JSX.IntrinsicElements;
}
```

This maps Astro's JSX types to the global namespace that `@types/mdx` expects.

### Note

- This is an IDE/editor type-checking issue;
   `skipLibCheck: true` in tsconfig prevents this from blocking builds
- Each Astro project using MDX with TypeScript needs this `env.d.ts` file
- The Astro-generated `.astro/types.d.ts` includes `astro/client` but doesn't bridge to the global `JSX` namespace

### References

- [Astro GitHub Issue #5061](https://github.com/withastro/astro/issues/5061)
- [MDX Getting Started - Types](https://mdxjs.com/docs/getting-started/#types)
- [Astro TypeScript - Extending global types](https://docs.astro.build/en/guides/typescript/#extending-global-types)

### Verification

Reproduced under Astro 4.
x with `@types/mdx@2.x` and VS Code's
TypeScript language service (any tsc 5.
x or later).
 Trigger:
 any MDX
file containing native HTML elements like `<abbr>`,
 `<sub>`,
 `<kbd>`
inside an Astro project that does not also depend on `@types/react`.

### Why we do not file this upstream

5-constraint walk:

1. **Upstream's fault?
   ** Distributed across Astro and @types/mdx.
    MDX
   documentation explicitly says "the JSX namespace must be typed";
   Astro chose to define JSX types under `astroHTML.JSX` namespace
   rather than the global `JSX` namespace.
    Neither side considers it
   a bug.
2. **Can upstream fix it?
   ** Yes,
    either by Astro adding a global
   `JSX` re-export or by @types/mdx accepting a different namespace
   prefix.
    Both are tractable but require coordination.
3. **Supporting this use case?
   ** Astro+MDX without React is a
   documented configuration,
    but the namespace bridging step is
   left to the user.
4. **Will they fix it?
   ** Astro issue #5061 has been open for years
   without movement.
    Low signal of a fix.
5. **Minimal-fix prototype?
   ** The workspace-side bridge
   (`src/env.d.ts`) is the minimal fix;
    no upstream code change is
   strictly required.

**Decision:
 no upstream report.
** Existing Astro issue #5061 already
tracks the namespace question;
 the workspace-side bridge satisfies
the use case.

## All packages must extend `config-typescript/dom`

### Problem

`tsgo --build` reports errors like `Cannot find name 'FileSystemWritableFileStream'` or
`Property 'storage' does not exist on type 'Navigator'` in a package that never uses browser APIs directly.

```txt
../../module/es/src/types/.../t opfs/p p/index.ts(8,15): error TS2304: Cannot find name 'FileSystemWritableFileStream'.
../../module/es/src/types/.../t opfs/p p/index.ts(24,38): error TS2339: Property 'storage' does not exist on type 'Navigator'.
```

### Root cause

`module-es` exports raw `.ts` source files via its `exports` map (e.g. `"./logger": "./src/..."`).
When another package imports from `module-es`,
 tsgo checks those source files under the **consumer's** tsconfig,
 not module-es's.
If the consumer extends the base config (`config-typescript`) which only has `"lib": ["ESNext"]`,
DOM types like `FileSystemWritableFileStream` and `navigator.storage` are missing.

Non-browser runtimes may adopt browser APIs over time,
so separating into `/dom` vs non-`/dom` configs provides no future-proofing benefit
and causes false positives instead.

### Solution

Every package tsconfig must extend `@monochromatic-dev/config-typescript/dom` (not the base export).
This adds `"lib": ["ESNext", "DOM", "WebWorker"]` so all standard platform types are available
regardless of the package's target runtime.

```json
{
  "extends": "@monochromatic-dev/config-typescript/dom"
}
```

### Verification

Reproduced under tsgo 7.0.0-dev `--build`,
 tsc 6.0.
x exhibits the
same.
 Trigger:
 package extends the base `@monochromatic-dev/config-typescript`
(lib:
 ESNext only) and imports a workspace package that re-exports
raw `.ts` from a module touching `FileSystemWritableFileStream`,
`navigator.storage`,
 or other DOM/WebWorker types.

### Why we do not file this upstream

5-constraint walk:

1. **Upstream's fault?
   ** No. TypeScript checks consumed `.ts` files
   under the consumer's tsconfig by design;
    the lib setting is
   inherited from the consumer,
    not the package.
    This is documented
   compiler behavior.
2. **Can upstream fix it?
   ** Yes (in principle:
    per-package lib
   resolution),
    but it would be a structural change to lib resolution.
3. **Supporting this use case?
   ** Workspace packages re-exporting raw
   `.ts` sources is a non-standard pattern;
    standard practice ships
   `.d.ts` declarations.
4. **Will they fix it?
   ** Not on the roadmap.
5. **Minimal-fix prototype?
   ** Workspace-side fix (extending `/dom`
   variant) is the minimal solution;
    no upstream change needed.

**Decision:
 no upstream report.
** The workspace convention (extend
`config-typescript/dom` everywhere) satisfies the constraint at the
workspace boundary.

## Narrowing not preserved inside function declarations

### Problem

A `const` variable narrowed by a null check before a function declaration
still reports the nullable type inside the function body:

```ts
const el = document.querySelector<HTMLDivElement>('#app',);
if (el === null)
  throw new Error('missing',);

// TS18047: 'el' is possibly 'null'.
function setup(): void {
  console.log(el.clientWidth,);
}
```

Replacing the function declaration with a function expression or arrow eliminates the error:

```ts
const setup = function(): void {
  console.log(el.clientWidth,); // OK
};
```

### Root cause

TypeScript's control flow analysis extends narrowing across closure boundaries
only for certain node kinds.
The `while` loop in `checker.ts` (around line 31181 in the tsc 6.0 source) checks:

```ts
// checker.ts: getTypeOfSymbolAtLocation, inner narrowing loop
while (
  flowContainer !== declarationContainer && (
    flowContainer.kind === SyntaxKind.FunctionExpression
    || flowContainer.kind === SyntaxKind.ArrowFunction
    || isObjectLiteralOrClassExpressionMethodOrAccessor(flowContainer,)
  ) && (
    isConstantVariable(localOrExportSymbol,) && type !== autoArrayType
    || isParameterOrMutableLocalVariable(localOrExportSymbol,)
      && isPastLastAssignment(localOrExportSymbol, node,)
  )
) {
  flowContainer = getControlFlowContainer(flowContainer,);
}
```

`SyntaxKind.FunctionDeclaration` is intentionally absent.
Function declarations are hoisted,
so a call site can appear **before** the narrowing guard in source order:

```ts
const el = document.querySelector<HTMLDivElement>('#app',);

setup(); // runs before the null check below

if (el === null)
  throw new Error('missing',);

function setup(): void {
  // el is genuinely nullable here at runtime
  console.log(el.clientWidth,);
}
```

Because hoisting makes the call-before-guard pattern legal,
TypeScript conservatively refuses to narrow inside function declarations.
Function expressions and arrows are bound to a `const`,
so they cannot be invoked before their definition,
making narrowing safe to propagate.

This behavior is the same in tsc (6.0.1-rc) and tsgo (7.0.0-dev).

### Solutions

**Return non-null from a helper function.
**
The return type carries the narrowed type into all callers
regardless of declaration kind:

```ts
function requireElement<T extends Element,>(selector: string,): T {
  const element = document.querySelector<T>(selector,);
  if (element === null)
    throw new Error(`Missing required element: ${selector}`,);
  return element;
}

const el = requireElement<HTMLDivElement>('#app',);
// el is HTMLDivElement (non-null) everywhere
function setup(): void {
  console.log(el.clientWidth,); // OK
}
```

**Reassign to a new `const` with an explicit type annotation**
after the null check.
The explicit annotation becomes the variable's declared type,
which is non-null regardless of closure context:

```ts
const maybeEl = document.querySelector<HTMLDivElement>('#app',);
if (maybeEl === null)
  throw new Error('missing',);
const el: HTMLDivElement = maybeEl;

function setup(): void {
  console.log(el.clientWidth,); // OK
}
```

### What does not work

- Combining multiple null checks into one `if` guard:
  the same hoisting concern applies per-variable
- `asserts` functions;
   they narrow the **parameter**
  in the caller's flow,
   but the narrowed binding is still a `const`
  subject to the same closure rules
- Adding `as HTMLDivElement`:
  suppresses the error but is flagged by `no-unsafe-type-assertion`

### Why we do not file this upstream

5-constraint walk:

1. **Upstream's fault?
   ** No. The cited code in `checker.ts` (around
   line 31181) deliberately excludes `SyntaxKind.FunctionDeclaration`
   from the closure-narrowing loop because hoisting makes
   call-before-guard legal.
    The behavior is intentional safety,
    not
   a bug.
2. **Can upstream fix it?
   ** Possibly (via flow analysis that
   distinguishes "definitely called after the guard" from "could be
   called before"),
    but this would be a significant change to the
   control-flow analyzer that could regress soundness in edge cases.
3. **Supporting this use case?
   ** The recommended pattern is helper
   functions returning non-null or explicit reassignment with
   annotation;
    both are documented and idiomatic.
4. **Will they fix it?
   ** Not on the roadmap.
    The intentional
   exclusion has been in place across multiple TypeScript major
   versions (verified consistent in tsc 6.0.1-rc and tsgo 7.0.0-dev).
5. **Minimal-fix prototype?
   ** Not feasible without proving soundness
   of the new flow-analysis branch.

**Decision:
 no upstream report.
** Workarounds are well-understood
and idiomatic.
 The behavior is correctly defensive given hoisting
semantics.

## JSR packages ship `.ts` source files that `skipLibCheck` cannot skip

### Problem

`tsgo --build` reports type errors **inside `node_modules`**
from JSR packages like `@zod/zod`:

```txt
node_modules/.bun/@jsr+zod__zod@4.3.6/…/src/v4/core/schemas.ts(2088,19): error TS2532: Object is possibly 'undefined'.
node_modules/.bun/@jsr+zod__zod@4.3.6/…/src/v4/core/schemas.ts(2130,17): error TS2532: Object is possibly 'undefined'.
node_modules/.bun/@jsr+zod__zod@4.3.6/…/src/v4/core/util.ts(930,41): error TS2345: Argument of type 'number | undefined' is not assignable to parameter of type 'number'.
node_modules/.bun/@jsr+zod__zod@4.3.6/…/src/v4/locales/he.ts(44,17): error TS18048: 'TypeNames.unknown' is possibly 'undefined'.
```

These errors appear despite `skipLibCheck: true` in tsconfig.
The errors are all `| undefined` narrowing failures:
the library's code is correct but was not written for `noUncheckedIndexedAccess: true`.

### Root cause

Four things combine to create this problem:

**JSR ships `.ts` source files,
 not `.d.ts` declarations.
**
The `@jsr/zod__zod` package contains both `.ts` and `.js` for every module.
The `package.json` exports point to `.js` files,
but no `.d.ts` declaration files exist:

```jsonc
// node_modules/@jsr/zod__zod/package.json
{ "exports": { ".": { "default": "./src/index.js" } } }
```

```txt
src/v4/core/
  schemas.ts   schemas.js   schemas.js.map
  util.ts      util.js      util.js.map
  // no .d.ts files anywhere
```

**TypeScript's bundler resolution prefers `.ts` over `.js`.
**
When an export points to `./src/index.js`,
the resolver strips the `.js` extension and tries candidates in a fixed priority order.
From `typescript-go/internal/module/resolver.go` line 1471:

```go
case tspath.ExtensionTs, tspath.ExtensionDts, tspath.ExtensionJs, "":
    if extensions&extensionsTypeScript != 0 {
        r.tryExtension(tspath.ExtensionTs, …)   // 1st: .ts
        r.tryExtension(tspath.ExtensionTsx, …)  // 2nd: .tsx
    }
    if extensions&extensionsDeclaration != 0 {
        r.tryExtension(tspath.ExtensionDts, …)  // 3rd: .d.ts
    }
    if extensions&extensionsJavaScript != 0 {
        r.tryExtension(tspath.ExtensionJs, …)   // 4th: .js (never reached)
    }
```

The `.ts` sibling is found at step 1 and the `.js` export is never used.
This priority order is hardcoded:
 no tsconfig option changes it.
The `extensionsTypeScript` bit is always set for regular imports
(line 117:
 `state.extensions = extensionsTypeScript | extensionsJavaScript | extensionsDeclaration`).

**`skipLibCheck` only covers `.d.ts` files,
 not `.ts` files.
**
From `typescript-go/internal/compiler/program.go` line 562:

```go
func (p *Program) SkipTypeChecking(sourceFile *ast.SourceFile, ignoreNoCheck bool) bool {
    return (!ignoreNoCheck && p.Options().NoCheck.IsTrue()) ||
        p.Options().SkipLibCheck.IsTrue() && sourceFile.IsDeclarationFile ||
        // …
}
```

`IsDeclarationFile` is only true for `.d.ts` files.
The `.ts` source files from JSR packages are type-checked
under the **consumer's** tsconfig settings,
 not the library's.

**Zod was not written for `noUncheckedIndexedAccess: true`.
**
Array index access like `nonaborted[0]` returns `T | undefined` under this flag.
Zod's code assumes the index is valid after a `.length` check,
which TypeScript cannot prove:

```ts
// schemas.ts:2087-2088: TS2532 here
const nonaborted = results.filter(r => !util.aborted(r,));
if (nonaborted.length === 1)
  final.value = nonaborted[0].value; // Object is possibly 'undefined'

// util.ts:930: TS2345 here
binaryString += String.fromCharCode(bytes[i],); // Argument of type 'number | undefined'
```

### TypeScript team's position

The TypeScript team has closed multiple issues about this as **"Working as Intended"**:

- [microsoft/TypeScript#41883](https://github.com/microsoft/TypeScript/issues/41883):
  `skipLibCheck` ignored when `types` points to `.ts`.
  Ryan Cavanaugh:
   "skipLibCheck causes the 'check each top-level statement or declaration' step
  to not occur for `.d.ts` files.
   It has no other effect.
   It does nothing in `.ts` files.
  "
- [microsoft/TypeScript#44205](https://github.com/microsoft/TypeScript/issues/44205):
  request to not apply strict checks to `node_modules`.
   **Declined.
  **
  "The only correct path forward is to not have a .
  ts file in your node_modules.
  "
- [microsoft/TypeScript#48779](https://github.com/microsoft/TypeScript/issues/48779):
  `noUncheckedIndexedAccess` errors in `node_modules`.
  Closed as duplicate of #44205.
- [microsoft/TypeScript#40426](https://github.com/microsoft/TypeScript/issues/40426):
  "Disable type checking for node_modules entirely.
  "
  Still open,
   labeled "Awaiting More Feedback",
   no action.

tsgo inherits the same semantics.
 No planned fix.

### Solution: wrapper script that strips `node_modules` errors

Since no tsconfig option can suppress these errors,
the `lint:types` mise task wraps the native `tsc --build`
in a script that filters out diagnostics originating from `node_modules` paths.

The wrapper:

1. Runs `tsc --build` (or `tsc --build --noEmit`,
    etc.) with all original arguments
2. Captures stdout/stderr line by line
3. Drops any line whose file path contains `/node_modules/`
4. Drops continuation lines (indented lines following a dropped diagnostic)
5. Preserves the exit code:
    exits non-zero only if non-`node_modules` errors remain

This is the least invasive option because:

- It does not modify `node_modules` (unlike `bun patch`)
- It does not sacrifice type safety (unlike `declare module` with `any`)
- It does not introduce version-drift risk (unlike installing npm zod alongside JSR zod)
- It does not require maintaining generated `.d.ts` files across package updates
- It works for **any** JSR package that ships `.ts`,
   not just zod

### Alternatives considered

**Switch to npm zod.
**
The npm `zod` package ships `.d.ts` + `.js`,
 so `skipLibCheck` works.
This is the simplest fix but ties us to npm
and loses JSR's advantage of direct `.ts` source for editor go-to-definition.

**`bun patch` to convert `.ts` to `.d.ts`.
**
Patch the JSR package to generate `.d.ts` files and delete `.ts` sources.
Works in principle,
 but zod's complex types may fail declaration generation,
and the patch must be re-applied on every zod update.

**`paths` redirect with explicit `.d.ts` extension.
**
`resolver.go` line 1225 shows that `paths` substitutions with an explicit file extension
call `tryFile` directly,
 bypassing the `.ts`-sibling preference:

```go
if extension := tspath.TryGetExtensionFromPath(subst); extension != "" {
    if path, ok := r.tryFile(candidate, onlyRecordFailures); ok {
        return &resolved{path: path, extension: extension}
    }
}
```

So `"paths": { "zod": ["./typings/zod.d.ts"] }` would resolve directly to the `.d.ts`
and `skipLibCheck` would cover it.
 But you need a source for the `.d.ts` types:
either generating them (same fragility as the patch approach)
or installing npm zod in parallel (version drift).

**`declare module 'zod'` ambient override.
**
Ambient module declarations only take effect when normal resolution fails.
Since `zod` resolves fine through `node_modules`,
 the ambient declaration is ignored.

### References

- [JSR @zod/zod](https://jsr.io/@zod/zod):
   the source of the `.ts`-shipping package
- [TypeScript tsconfig: skipLibCheck](https://www.typescriptlang.org/tsconfig/skipLibCheck.html):
   only `.d.ts`
- [typescript-go resolver.go](https://github.com/microsoft/typescript-go/blob/main/internal/module/resolver.go):
   extension priority and `paths` bypass
- [typescript-go program.go](https://github.com/microsoft/typescript-go/blob/main/internal/compiler/program.go):
   `SkipTypeChecking` implementation

### Why we do not file this upstream

5-constraint walk:

1. **Upstream's fault?
   ** No,
    per the TypeScript team's repeated
   stance.
    Cited issues (#41883,
    #44205,
    #48779,
    #40426) all
   closed or stalled with "Working as Intended" or "The only correct
   path forward is to not have a .
   ts file in your node_modules.
   "
   `skipLibCheck` is documented as `.d.ts`-only.
2. **Can upstream fix it?
   ** Yes (extend `skipLibCheck` to cover `.ts`
   in `node_modules`,
    or add a separate `skipNodeModulesCheck`
   option),
    but they have explicitly declined to do so.
3. **Supporting this use case?
   ** JSR's `.ts`-shipping is a JSR
   convention;
    TypeScript does not endorse it.
    The TypeScript team's
   stated path is "don't ship .
   ts to node_modules.
   "
4. **Will they fix it?
   ** No. Multiple closed-as-WAI issues with the
   same request.
    Filing a new one would duplicate #44205,
    #48779,
   #41883 without changing the outcome.
5. **Minimal-fix prototype?
   ** Even with a prototype,
    the team's
   position is structural ("don't have .
   ts in node_modules"),
    not
   missing-implementation.

**Decision:
 no upstream report.
** Filing would duplicate already-
declined issues.
 The workspace wrapper script that strips
`node_modules` diagnostics is the correct boundary fix.

## tsgo LSP panics on non-source files (SVG, PNG, etc.)

### Problem

The tsgo LSP crashes with a panic when a non-source file
exists in a directory covered by a `tsconfig.json`:

```txt
panic: ScriptKind must be specified when parsing source file:
  /var/home/user/Monochromatic/package/test-harness/core/architecture.svg
```

### Root cause

The proximate cause is a missing `ScriptKind` guard in the LSP's `compilerHost.GetSourceFile`.
When a file with an unrecognized extension (like `.svg`) reaches the parser,
the parser panics because `ScriptKind` is `Unknown`.

All source references below are from commit `c0703e66` of `microsoft/typescript-go`.

**Step 1:
 `diskFile.Kind()` returns `ScriptKindUnknown` for `.svg`.
**

`internal/project/overlayfs.go:100-102`:

```go
func (f *diskFile) Kind() core.ScriptKind {
	return core.GetScriptKindFromFileName(f.fileName)
}
```

`internal/core/core.go:512-529` (the switch only handles TS/JS/JSON extensions):

```go
func GetScriptKindFromFileName(fileName string) ScriptKind {
	dotPos := strings.LastIndex(fileName, ".")
	if dotPos >= 0 {
		switch strings.ToLower(fileName[dotPos:]) {
		case tspath.ExtensionJs, tspath.ExtensionCjs, tspath.ExtensionMjs:
			return ScriptKindJS
		case tspath.ExtensionJsx:
			return ScriptKindJSX
		case tspath.ExtensionTs, tspath.ExtensionCts, tspath.ExtensionMts:
			return ScriptKindTS
		case tspath.ExtensionTsx:
			return ScriptKindTSX
		case tspath.ExtensionJson:
			return ScriptKindJSON
		}
	}
	return ScriptKindUnknown  // ← .svg lands here
}
```

**Step 2:
 `compilerHost.GetSourceFile` passes `Unknown` to the parse cache without checking.
**

`internal/project/compilerhost.go:95-102`:

```go
func (c *compilerHost) GetSourceFile(opts ast.SourceFileParseOptions) *ast.SourceFile {
	c.ensureAlive()
	if fh := c.sourceFS.GetFileByPath(opts.FileName, opts.Path); fh != nil {
		key := NewParseCacheKey(opts, fh.Hash(), fh.Kind())  // ← Kind() = ScriptKindUnknown
		return c.builder.parseCache.Acquire(key, fh)
	}
	return nil
}
```

**Step 3:
 the parse cache calls `parser.ParseSourceFile` with `ScriptKindUnknown`.
**

`internal/project/parsecache.go:30-38`:

```go
func NewParseCache(options RefCountCacheOptions) *ParseCache {
	return NewRefCountCache(
		options,
		func(key ParseCacheKey, fh FileHandle) *ast.SourceFile {
			file := parser.ParseSourceFile(key.SourceFileParseOptions, fh.Content(), key.ScriptKind)
			//                                                                       ^^^^^^^^^^^
			//                                                                       ScriptKindUnknown
			file.Hash = fh.Hash()
			return file
		},
	)
}
```

**Step 4:
 the parser panics.
**

`internal/parser/parser.go:288-291`:

```go
func (p *Parser) initializeState(opts ast.SourceFileParseOptions, sourceText string, scriptKind core.ScriptKind) {
	if scriptKind == core.ScriptKindUnknown {
		panic("ScriptKind must be specified when parsing source file: " + opts.FileName)
	}
```

**The extension guard in the file loader exists but is bypassed.
**

`internal/compiler/filesparser.go:68-95` has a guard in `parseTask.load()`:

```go
if tspath.HasExtension(t.normalizedFilePath) {
	compilerOptions := loader.opts.Config.CompilerOptions()
	allowNonTsExtensions := compilerOptions.AllowNonTsExtensions.IsTrue()
	if !allowNonTsExtensions {
		canonicalFileName := tspath.GetCanonicalFileName(t.normalizedFilePath, ...)
		if !loader.isSupportedExtension(canonicalFileName) {
			// ... add diagnostic and return early (line 93)
			return
		}
	}
}
```

This guard is bypassed for **inferred projects** because `NewInferredProject`
(`internal/project/project.go:100-121`) sets `AllowNonTsExtensions: core.TSTrue`
in its default compiler options:

```go
func NewInferredProject(
	currentDirectory string,
	compilerOptions *core.CompilerOptions,
	rootFileNames []string,
	builder *ProjectCollectionBuilder,
	logger *logging.LogTree,
) *Project {
	p := NewProject(inferredProjectName, KindInferred, currentDirectory, builder, logger)
	if compilerOptions == nil {
		compilerOptions = &core.CompilerOptions{
			AllowNonTsExtensions:       core.TSTrue,  // ← bypasses the guard
			// ...
		}
	}
```

When `AllowNonTsExtensions` is true,
 the guard at `filesparser.go:71` is skipped entirely,
and the file proceeds to `loader.parseSourceFile(t)` at line 108,
which calls `compilerHost.GetSourceFile` (step 2 above),
which passes the `ScriptKindUnknown` to the parser (step 4).

**The file scanning correctly filters by extension,
 but the SVG enters through a different path.
**

The `matchFiles` function (`internal/vfs/vfsmatch/vfsmatch.go:604-606`) checks extensions
during directory scanning:

```go
for _, file := range entries.Files {
	if len(v.extensions) > 0 && !tspath.FileExtensionIsOneOf(file, v.extensions) {
		continue  // ← SVG would be skipped here
	}
```

Confirmed:
 `tsgo --showConfig` resolves the `include` patterns correctly,
and the SVG does not match any include pattern.
The SVG enters the project through a path that bypasses this extension filter:
either through the inferred project's `AllowNonTsExtensions` override,
or through project reference resolution that feeds files directly to `compilerHost.GetSourceFile`
without checking `isSupportedExtension` first.

### Current status: partially mitigated, not fully resolved

The crash is triggered by editord forwarding non-source files to tsgo.
Two distinct paths lead to the panic:

1. **Spawn trigger**:
    when a non-source file is the first file opened
   for a project root,
    `pool.resolve({ type: 'tsgo' })` spawns tsgo
   with that file as the trigger.
    tsgo adds the file to the project
   during initialization and panics on the unsupported extension.

2. **Reuse + feature request**:
    when tsgo is already running
   (spawned earlier from a `.ts` file),
    and a non-source file is opened,
   feature request handlers (`withClient` for hover,
    inlayHints,
    etc.)
   call `pool.resolve()` which returns the existing client.
   The handler sends the request with the non-source file's URI.
   tsgo creates an inferred project for the unknown file,
   which triggers parsing and the ScriptKind panic.

tsgo does NOT crash from its own directory scanning:
`include`/`exclude` patterns work correctly during normal project loading
from a `.ts` trigger.

### Mitigations in place

**Include filter in `resolve()` gating ALL tsgo access** (`lsp-pool.ts`):
the `#resolveTsgoWithIncludeCheck` method runs the tsconfig include check
before returning ANY tsgo client:
 both reuse of existing clients and new spawns.
Files outside the project's declared include scope get `null`,
so tsgo never receives a non-source file URI through any code path:
`resolveAll` (didOpen lifecycle),
 `withClient` (feature requests),
 or direct calls.
The resolved include patterns are cached with a 2-minute TTL
via `resolveTsconfigIncludes` (`tsconfig-includes.ts`).

**Crash recovery with ScriptKind-aware retry** (`lsp-pool.ts`,
 `lsp-client.ts`):
on unexpected exit,
 editord parses stderr for the ScriptKind panic pattern.
For this specific crash,
 retry uses a flat 1 s interval (no backoff escalation)
since the crash resolves as soon as the user navigates away from the
non-source file.
 For other crashes,
 exponential backoff applies
(2 s base,
 doubling to 60 s cap).

**Base tsconfig `exclude` for non-source extensions** (`tsconfig.options.json`):
added as belt-and-suspenders for the CLI path.
Verified to work for `tsgo --build` but does not prevent the LSP crash.

### What does not work

- **tsconfig `include`/`exclude` patterns in LSP mode**:
  these work for CLI `tsgo --build` and for normal project loading
  when tsgo is spawned by a `.ts` trigger,
  but do not prevent the crash when tsgo is spawned by a non-source trigger.
  The LSP's `DidOpenFile` → `ensureConfiguredProjectAndAncestorsForFile`
  likely adds the trigger file as a root file name,
  bypassing `matchFiles` extension filtering.
- **Shadow root / symlink directory**:
  pnpm workspace `node_modules` resolution breaks because `${configDir}`
  resolves to the shadow path and the nested `node_modules` structure
  does not contain hoisted dependencies.
- **Restarting without any delay**:
  creates a crash loop;
   the 1 s flat retry gives time for the user
  to navigate away from the problematic file.

### Previously broken: filtering only spawns, not reuse

Two earlier approaches failed because they only gated part of the problem:

1. **Filtering only in `resolveAll`**:
   feature request handlers call `pool.resolve()` directly via `withClient`,
   bypassing the `resolveAll` include filter entirely.
2. **Filtering only before spawning in `resolve()`**:
   returning an existing tsgo client for a non-source file is just as
   dangerous as spawning a new one:
    the feature request sends the file URI
   to tsgo,
    which creates an inferred project and panics.

**Fixed** by checking tsconfig includes as the very first step in `resolve()`
for tsgo,
 before both pool cache lookup and spawn.
`#resolveTsgoWithIncludeCheck` returns `null` for non-matching files
regardless of whether a tsgo client exists for the project root.

### References

- [microsoft/typescript-go PR #437](https://github.com/microsoft/typescript-go/pull/437):
   original extension guard (later bypassed)
- [microsoft/typescript-go PR #2004](https://github.com/microsoft/typescript-go/pull/2004):
   removed "unsupported extensions" concept
- [microsoft/typescript-go PR #1556](https://github.com/microsoft/typescript-go/pull/1556):
   improved panic message to include filename
- [microsoft/typescript-go#2669](https://github.com/microsoft/typescript-go/issues/2669):
  same crash in the completions LSP path,
   fixed by [PR #2679](https://github.com/microsoft/typescript-go/pull/2679)
- [denoland/deno#31423](https://github.com/denoland/deno/issues/31423):
  CSS imports causing the same `ScriptKind` panic
- [neovim/nvim-lspconfig#4018](https://github.com/neovim/nvim-lspconfig/issues/4018):
  filetype mismatch triggering the same crash
- Source commit:
   `c0703e66` of `microsoft/typescript-go`
- `internal/core/core.go:512-529`:
   `GetScriptKindFromFileName`
- `internal/parser/parser.go:288-291`:
   panic site
- `internal/project/compilerhost.go:95-102`:
   `GetSourceFile` missing guard
- `internal/project/overlayfs.go:100-102`:
   `diskFile.Kind()` returning `Unknown`
- `internal/project/parsecache.go:30-38`:
   parse cache forwarding `Unknown` to parser
- `internal/project/project.go:100-121`:
   `NewInferredProject` setting `AllowNonTsExtensions`
- `internal/compiler/filesparser.go:68-95`:
   extension guard (bypassed by `AllowNonTsExtensions`)

### Why we would file this upstream (5 constraints)

5-constraint walk:

1. **Upstream's fault?
   ** Yes.
    `compilerHost.GetSourceFile` passes
   `ScriptKindUnknown` to the parse cache without guarding,
    leading
   to a deliberate panic in `parser.initializeState`.
    The parser
   panic site exists specifically to flag this misuse (line
   288-291),
    so the calling code is buggy by the parser's own
   contract.
2. **Can upstream fix it?
   ** Yes;
    the suggested fix is a 4-line guard
   in `compilerHost.GetSourceFile`.
    PR #2679 already implemented the
   equivalent fix for the completions code path (issue #2669);
    the
   same pattern applies here.
3. **Supporting this use case?
   ** Yes.
    LSP support for projects
   containing mixed file types (SVG icons alongside .
   ts source) is a
   first-class scenario;
    the panic crashes the entire LSP,
    which is
   never the intended response.
4. **Will they fix it?
   ** Likely yes given the existing PR #2679 fix
   pattern.
    The completions team accepted the same shape of fix.
5. **Minimal-fix prototype?
   ** Yes;
    the suggested patch below is
   the prototype,
    with related-issue evidence that the fix shape is
   accepted upstream.

**Decision:
 file upstream.
** All five constraints hold.
 The draft
below is ready;
 re-validate against current microsoft/typescript-go
HEAD before filing in case the path got fixed in the meantime.

### Draft upstream issue (do not file as-is; re-validate against current microsoft/typescript-go HEAD before filing)

````md
**Title:** LSP panics on ScriptKindUnknown when non-source file reaches parser via compilerHost.GetSourceFile

**Labels:** bug, LSP

**Summary:**

The LSP server panics when it receives a feature request (e.g. hover,
inlayHints) for a non-source file (e.g. `.svg`, `.png`, `.css`).
tsgo creates an inferred project for the file, which triggers parsing
with `ScriptKindUnknown` and panics in `parser.initializeState`.

**tsgo version:** 7.0.0-dev.20260404.1

**Reproduction:**

1. Create a directory with a `tsconfig.json`:

   ```json
   {
     "compilerOptions": { "strict": true },
     "include": ["src/**/*.ts"]
   }
   ```

2. Add `src/index.ts` (any valid TypeScript file).
3. Add `architecture.svg` (any SVG file) in the same directory.
4. Start `tsgo --lsp --stdio` and send an `initialize` request with
   `rootUri` pointing to this directory.
5. Open `src/index.ts` via `textDocument/didOpen`.

tsgo panics with:

```text
panic: ScriptKind must be specified when parsing source file: /path/to/architecture.svg
```

**Root cause analysis:**

`compilerHost.GetSourceFile` (`internal/project/compilerhost.go:95-102`)
passes `fh.Kind()` to the parse cache without checking for
`ScriptKindUnknown`. `diskFile.Kind()`
(`internal/project/overlayfs.go:100-102`) delegates to
`GetScriptKindFromFileName` (`internal/core/core.go:512-529`),
which returns `ScriptKindUnknown` for any unrecognized extension.
The parse cache forwards this to `parser.ParseSourceFile`, which
panics at `internal/parser/parser.go:289-290`.

The extension guard in `filesparser.go:68-95` exists but is bypassed
when `AllowNonTsExtensions` is true (as set by `NewInferredProject`
at `internal/project/project.go:119`).

The `include` patterns correctly exclude the SVG (confirmed by
`--showConfig` and by `matchFiles` in `vfsmatch.go:604-606` filtering
by supported extensions). The file enters the project through a path
that bypasses the `matchFiles` extension filter.

**Suggested fix:**

Add a `ScriptKindUnknown` check in `compilerHost.GetSourceFile`
before calling `parseCache.Acquire`:

```go
func (c *compilerHost) GetSourceFile(opts ast.SourceFileParseOptions) *ast.SourceFile {
    c.ensureAlive()
    if fh := c.sourceFS.GetFileByPath(opts.FileName, opts.Path); fh != nil {
        kind := fh.Kind()
        if kind == core.ScriptKindUnknown {
            return nil
        }
        key := NewParseCacheKey(opts, fh.Hash(), kind)
        return c.builder.parseCache.Acquire(key, fh)
    }
    return nil
}
```

This is consistent with how the CLI's file loader handles
unsupported extensions (returning early with a diagnostic rather than
panicking).

**Related issues:**

- microsoft/typescript-go#2669 and #2679: same crash fixed in the
  completions code path.
- denoland/deno#31423: CSS imports causing the same panic.
- neovim/nvim-lspconfig#4018: filetype mismatch in LSP.
````

## Type check of a test file is only as current as the last build

### Symptom

`mise run //package/<path>:lint:types` reports zero errors while test files in
that package carry real type errors.
The same command,
 run again after `mise run //package/<path>:build` and with no source change in
between,
 reports them.

Observed 2026-08-20 in `package/translation-repair`,
 adding two required fields to `ChunkRepairOutcome`:
 the type check named the source construction sites,
 stayed silent about five test files whose fixtures were missing the same
fields,
 and named all seven test-file errors on the next run after a build.

### Root cause

Test files in this package import the BUILT bundle,
 not the source:

```ts
// package/translation-repair/assembly/src/repair-round-record.unit.test.ts
import { describeJudgedRound, } from '../dist/final/node/index.mjs';
```

251 of the package's test files import `../dist/final/node/index.mjs` or
`../../dist/final/node/index.mjs`.
TypeScript resolves those imports to `dist/final/node/index.d.mts`,
 which rolldown emits during `build` and which `lint:types` never regenerates.

So a test file is always checked against the declarations of the last
successful build.
Change a type in `src/`,
 run `lint:types`,
 and every source file is checked against the change while every test file is
checked against the shape it had before.

### Verification

A positive control,
 with the same source state throughout and only the build between the two runs:

```bash
# One required field added to a widely constructed type, filled in at every
# source construction site so the source half is clean.
mise run //package/translation-repair/<package>:lint:types   # 0 errors
mise run //package/translation-repair/<package>:build
mise run //package/translation-repair/<package>:lint:types   # 7 errors, 5 test files
```

### Workaround

Build before type-checking whenever a change could reach a test file:

```bash
mise run //package/<path>:build && mise run //package/<path>:lint:types
```

`mise run //package/translation-repair/<package>:buildAndTest` already does this
for the test half,
 and its description says so:
 "Build the node bundle,
 then run unit tests against the built dist".
The same ordering is what the type check needs and nothing states it.

### What does not work

Deleting `dist/final/types/tsconfig.tsbuildinfo` does not surface the errors.
The incremental build info is not involved,
 and clearing it was the first cause guessed here.
Verified by clearing it against a stale `index.d.mts`:
 the test-file errors stayed hidden.
Only the rebuild changes the answer.

### Why we do not file this upstream

Not a TypeScript defect.
TypeScript resolves a declaration file that exists on disk and reports what it
says;
 the declarations being stale is a property of when this repo runs its own
build.
The fix,
 if the ordering is judged worth enforcing rather than documenting,
 belongs in the `lint:types` task definition.

## `using` downlevel helpers: 2381 bytes inlined per emitted file, and a `null` `Symbol.asyncDispose` guard that diverges from the spec

### Problem

Two symptoms share one emit path.
Both were measured with the installed `typescript@7.0.2` and with a compiler built from
`microsoft/TypeScript` main at `fed0bf24149fb1ed36039212648bafdafc1ea10e` (2026-10-08).

Symptom 1 is size and duplication.
Every `.js` file `tsc` emits from a source containing `using` or `await using` starts with both helper
definitions verbatim.
On a three-function fixture the helper block is 2381 bytes of a 3264 byte emit,
 and the same bytes repeat
in every other emitted file of the program that uses `using`:

```text
# node <repo>/node_modules/.pnpm/typescript@7.0.2/node_modules/typescript/bin/tsc \
#   --project tsconfig.json   (target es2025, two one-line `using` files)
out/a.js   bytes=2678   __disposeResources occurrences=2
out/b.js   bytes=2678   __disposeResources occurrences=2
```

In this workspace 7 of the 135 ignored stray `.js` files carry the helper,
 for example
`package/logger/core/src/create-logger.unit.test.js:1`.

Symptom 2 is behavior.
`await using` on an object whose `[Symbol.asyncDispose]` is `null` and whose `[Symbol.dispose]` is
callable throws in the downlevel output and completes natively.
The downlevel path reports `TypeError: Object not disposable.`

A third surprise frames both:
 `target: esnext` emits no helper at all and keeps native `using`,
 while
every named target downlevels,
 including `es2025` and (on upstream main) `es2026`,
 even though explicit
resource management is in the ES2026 edition (tc39/ecma262#3000,
 merged 2026-06-24).

### Root cause

Three separate mechanisms,
 cited against `microsoft/TypeScript` main at `fed0bf24149f`.

**1.
 Only the floating `esnext` target skips the transform.**
`tsc/internal/transformers/estransforms/definitions.go:10` builds the chain that owns `using`:

```go
// tsc/internal/transformers/estransforms/definitions.go:10
NewESNextTransformer = transformers.Chain(newUsingDeclarationTransformer, esDecoratorAndClassFields)
```

`tsc/internal/transformers/estransforms/definitions.go:24` selects that chain by target:

```go
// tsc/internal/transformers/estransforms/definitions.go:27-30
case core.ScriptTargetESNext:
	return esDecoratorAndClassFields(opts)
case core.ScriptTargetES2026, core.ScriptTargetES2025, core.ScriptTargetES2024, core.ScriptTargetES2023, core.ScriptTargetES2022, core.ScriptTargetES2021:
	return NewESNextTransformer(opts)
```

The `esnext` arm omits `newUsingDeclarationTransformer`,
 so `using` survives into the output.
Every named edition arm includes it,
 so `tsc/internal/transformers/estransforms/using.go:21` rewrites each
`using` into an `env` record plus `try`/`catch`/`finally` calling the two helpers.
The file's own comment at `tsc/internal/transformers/estransforms/definitions.go:11` records
`// 2026: no new downlevel syntax`,
 which is why `es2026` still routes through the `using` transform:
the gate is the floating target,
 not edition membership.

**2.
 Without `importHelpers`,
 the printer has nowhere to point but the file itself.**
The helper carries both an inline text body and an import name:

```go
// tsc/internal/printer/helpers.go:70-72
var addDisposableResourceHelper = &EmitHelper{
	Name:       "typescript:addDisposableResource",
	ImportName: "__addDisposableResource",
```

`tsc/internal/printer/printer.go:4635` skips per-file text only when helpers are external or disabled:

```go
// tsc/internal/printer/printer.go:4638
shouldSkip := p.Options.NoEmitHelpers || (sourceFile != nil && p.emitContext.HasRecordedExternalHelpers(sourceFile))
```

External helpers are recorded only when `importHelpers` is set for that file,
`tsc/internal/compiler/program.go:493` gated at `tsc/internal/compiler/program.go:496`:

```go
// tsc/internal/compiler/program.go:496
if !optionsForFile.ImportHelpers.IsTrue() {
```

`tsc` ships no runtime module of its own,
 so with neither option set the only choices are per-file text or
an unresolvable reference.
That is the mechanism behind "into every .js file".

**3.
 The inlined helper treats `null` as present,
 the spec treats it as absent.**

```js
// tsc/internal/printer/helpers.go:78-87 (emitted text)
if (async) {
    if (!Symbol.asyncDispose) throw new TypeError("Symbol.asyncDispose is not defined.");
    dispose = value[Symbol.asyncDispose];
}
if (dispose === void 0) {
    if (!Symbol.dispose) throw new TypeError("Symbol.dispose is not defined.");
    dispose = value[Symbol.dispose];
    if (async) inner = dispose;
}
if (typeof dispose !== "function") throw new TypeError("Object not disposable.");
```

The guard is `dispose === void 0`,
 so a `null` `Symbol.asyncDispose` skips the `Symbol.dispose` fallback
and reaches the `typeof` check,
 which throws.
The merged spec's `GetDisposeMethod` uses `GetMethod`,
 and `GetMethod` returns `undefined` for a property
value that is `undefined` **or** `null`,
 then falls back to `%Symbol.dispose%`:

```text
# tc39/ecma262 PR 3000, sec-getdisposemethod
1. If kind is sync-dispose, return ? GetMethod(value, %Symbol.dispose%).
1. Let asyncMethod be ? GetMethod(value, %Symbol.asyncDispose%).
1. If asyncMethod is not undefined, return asyncMethod.
1. Let syncMethod be ? GetMethod(value, %Symbol.dispose%).
```

Engines implement the fallback,
 which is why native output succeeds on the same input.
`tslib@2.8.1` carries the identical guard at
`node_modules/.pnpm/tslib@2.8.1/node_modules/tslib/tslib.es6.js:305`,
 so `importHelpers` does not avoid
symptom 2.

An earlier hypothesis in this workspace was that the stray `.js` files came from a bare `tsc <file>`
invocation.
That specific attribution is not established:
 TS 7 rejects file arguments in a directory containing a
`tsconfig.json` with `TS5112` unless `--ignoreConfig` is passed (measured),
 and
`package/logger/core` has one.
What the evidence does support is emit produced outside
`package/config/typescript/tsconfig.options.json`,
 whose `target: esnext` would have emitted no helper.
Finding the emitting task is tracked separately in `doc/handover/slopo-cluster-issue-triage.md`.

### Verification

Versions under test:
 `typescript@7.0.2` from `node_modules/.pnpm/typescript@7.0.2`,
 upstream
`microsoft/TypeScript` main `fed0bf24149fb1ed36039212648bafdafc1ea10e`,
 `node v26.10.0`,
 `tslib@2.8.1`,
`rolldown@1.2.12`.

Minimal reproduction of symptom 2,
 zero diagnostics under both targets:

```ts
// repro.ts
async function main(): Promise<void> {
  await using resource = {
    [Symbol.asyncDispose]: null,
    [Symbol.dispose]: () => { console.log('sync dispose ran'); },
  } as unknown as AsyncDisposable;
  console.log('body completed');
}

await main();

export {};
```

```sh
# tsc = node <repo>/node_modules/.pnpm/typescript@7.0.2/node_modules/typescript/bin/tsc
$tsc repro.ts --ignoreConfig --target es2025 --module esnext --lib ESNext,DOM --outDir out-downlevel
node out-downlevel/repro.js
# TypeError: Object not disposable.

$tsc repro.ts --ignoreConfig --target esnext --module esnext --lib ESNext,DOM --outDir out-native
node out-native/repro.js
# body completed
# sync dispose ran
```

Catalog of emit shapes that carry no helper,
 measured on the three-function fixture:

- `--target esnext`:
   native `using` and `await using`,
   293 bytes.
- `--target es2025 --importHelpers`:
   `import { __addDisposableResource, __disposeResources } from "tslib";`,
  952 bytes,
   with `tslib` resolvable from the output directory.
- `rolldown` `transformSync` with `target: ['firefox153']` or `['chrome154', 'firefox153', 'node26']`:
  native `using`,
   288 bytes.

Catalog of emit shapes that carry disposal machinery:

- `--target es2015`:
   4061 bytes,
   helper text inlined.
- `--target es2020` through `--target es2025`:
   3264 bytes each,
   helper text inlined.
- `--target es2025 --importHelpers` with `tslib` unresolvable:
   `error TS2354: This syntax requires an
  imported helper but module 'tslib' cannot be found.`
- `--target es2025 --noEmitHelpers`:
   no helper text,
   and running the output throws
  `ReferenceError: __disposeResources is not defined`.
- `rolldown` `transformSync` with `target: 'es2022'`,
   `'es2025'`,
   or `['firefox140']`:
   598 bytes,
  `import _usingCtx2 from "@oxc-project/runtime/helpers/usingCtx";`,
   one shared helper module instead of
  per-file text.

Catalog of disposal semantics that match between native emit and the inlined helper,
 measured by running a
twelve-case fixture compiled three ways (native `esnext`,
 inline `es2025`,
 `es2025` with `tslib`) and
diffing the event logs:

- Nested scopes dispose in reverse acquisition order.
- Early `return` disposes before returning.
- A body throw disposes,
   then the original error propagates.
- A body throw plus a disposer throw yields `SuppressedError` with `.error` holding the disposer's error,
  `.suppressed` holding the body error,
   `instanceof Error` true,
   and prototype name `SuppressedError`.
- Two failing disposers nest with the later acquisition as `.error`.
- `for...of` with `using` inside the body disposes per iteration,
   including on `break`.
- `for (using x of iterable)` disposes each yielded resource and the disposable iterable itself.
- `await using` with an async disposer awaits it.
- `await using null` is a no-op.
- `for await` combined with `await using` disposes per iteration across `continue`.
- A synchronous `[Symbol.dispose]` returning a thenable under `await using` is not awaited.

Catalog of semantics that differ:

- `null` `[Symbol.asyncDispose]` with a callable `[Symbol.dispose]`:
   native falls back and completes,
   the
  helper throws `TypeError: Object not disposable.`
- `SuppressedError` message text:
   native `An error was suppressed during disposal`,
   helper
  `An error was suppressed during disposal.`
  The spec creates "a newly created *SuppressedError* object" without passing a message,
   so this string is
  implementation-defined and the difference is cosmetic.

### Workarounds

1. **Compile with `target: esnext`.**
   No helper text,
    native syntax,
    and symptom 2 disappears because the engine implements the fallback.
   This workspace already sets `target: esnext` in `package/config/typescript/tsconfig.options.json`.
   Tradeoffs:
    `esnext` floats,
    so a compiler upgrade can start preserving other syntax with no config
   change;
    the output only runs where `using` parses,
    which per mdn/browser-compat-data means Chrome 134,
   Firefox 141,
    Node 24.0.0,
    Deno 1.37 for `using` and 2.2.10 for `await using`,
    Bun 1.0.23,
    with Safari
   recorded as preview;
    and the option governs all downleveling,
    not disposal alone.
2. **Set `importHelpers: true` with `tslib` available at runtime.**
   Fixture emit drops from 3264 to 952 bytes and one copy of the helper serves the whole dependency graph.
   `tslib@2.8.1` exports both helpers at `tslib.es6.js:305` and `tslib.es6.js:334`.
   Tradeoffs:
    adds a runtime dependency;
    the per-scope `try`/`catch`/`finally` scaffolding stays in every
   function;
    symptom 2 is unchanged because `tslib` carries the same guard;
    and `importHelpers` applies to
   module files.
3. **Let the bundler own the transform.**
   `rolldown` and oxc import one shared `@oxc-project/runtime/helpers/usingCtx` when the target lacks
   support and preserve native `using` when it does not.
   This workspace's shipped artifacts take the preserving path:
    `.browserslistrc` resolves to chrome 154,
   firefox 153,
    android 154,
    and_chr 154,
    and_ff 157,
    and node 26.x,
    all above the thresholds,
    and
   `package/cloudflare-worker/rand/dist/final/neutral/worker.mjs:1` contains
   `using flushAtExit=flushOnExit(ctx)`.
   Tradeoffs:
    the final emitter decides shipping syntax,
    not `tsc`;
    and
   `package/config/rolldown/src/browserslist-targets.ts` prefers a generated targets JSON over a fresh
   `browserslist` query when that file exists,
    so verify what the build actually reads.
4. **Hand-write `try`/`finally`.**
   No helpers and no target coupling.
   Tradeoffs:
    loses the `SuppressedError` composition and the reverse-order guarantee,
    and reintroduces the
   cleanup duplication `using` exists to remove.

### What does not work

- Reaching for a named edition target.
  `es2025` downlevels (measured),
   and upstream main routes `es2026` through the same transform
  (`tsc/internal/transformers/estransforms/definitions.go:29`),
   so the feature being in ES2026 does not
  move the gate.
- `lib: ESNext` on its own.
  `lib` selects type declarations;
   the target arm selects the transform.
- Polyfilling `Symbol.dispose` to make downleveling unnecessary.
  The helper itself throws `TypeError: Symbol.dispose is not defined.` when the symbol is missing,
   so
  downleveling buys parser compatibility only,
   never runtime compatibility.
- `noEmitHelpers: true` without `importHelpers`.
  The output references identifiers nothing defines,
   and executing it throws
  `ReferenceError: __disposeResources is not defined` (measured).
- Treating `Symbol.dispose` availability as syntax availability.
  Per mdn/browser-compat-data,
   `Symbol.dispose` and `Symbol.asyncDispose` exist in Node 18.18.0 and 20.4.0
  and in Chrome 125 and 127,
   well before the `using` syntax in Node 24.0.0 and Chrome 134.
  That gap is exactly the population where the 2381 bytes still earn their place.

### Upstream filing decision (6 constraints)

Symptom 2 is the only candidate;
 symptom 1 is documented,
 intended behavior of `importHelpers`.

1. **Upstream's fault?**
   Yes.
   The guard is TypeScript's own emitted text at `tsc/internal/printer/helpers.go:82`,
    and it contradicts
   `GetDisposeMethod` as merged in tc39/ecma262#3000.
   Not a wording issue and not an architectural restriction.
2. **Can upstream fix it?**
   Yes.
   One token,
    prototyped and built here.
3. **Are they supporting this use case?**
   Yes.
   `using` downlevel emit is a documented feature with conformance baselines under
   `tsc/testdata/baselines/reference/conformance/usingDeclarations*`,
    including
   `usingDeclarationsWithImportHelpers.js`.
4. **Would the repo welcome our contribution?**
   No for an agent-filed report or pull request.
   `CONTRIBUTING.md:15` ("Instructions for autonomous coding agents") states a pull request is acceptable
   only if a specific human operator chose that specific issue and will shepherd it through review,
    and
   `CONTRIBUTING.md:25` bans automated comments with an immediate block for inauthentic activity.
   No blanket ban on human-filed reports that disclose AI assistance was found in `CONTRIBUTING.md` or
   `.github/ISSUE_TEMPLATE/`.
   Per that section's own instruction to surface it and stop,
    this is handed to the operator.
5. **Will they likely fix it?**
   No signal either way,
    which meets the constraint.
   The helper text is actively maintained,
    a related emit bug is open at microsoft/TypeScript#63522
   ("Declarations that shadow global `Symbol` break emit for `using` declarations"),
    and duplicate searches
   over `gh search issues` and `gh search prs` for `asyncDispose null`,
    `Object not disposable`,
    and
   `addDisposableResource asyncDispose null` returned no matches in `microsoft/TypeScript` or
   `microsoft/tslib`.
6. **Minimal fix prototyped?**
   Yes,
    in a disposable clone at `${HOME}/temp/agent/upstream-prototype.Dqh0XXSR/ts`,
    origin
   `https://github.com/microsoft/TypeScript.git`,
    HEAD `fed0bf24149fb1ed36039212648bafdafc1ea10e`,
    built
   with `GOMAXPROCS=2 go build -p 2 -o ./tsc-patched ./cmd/tsc` (4m27s,
    go 1.27.1).

The prototyped diff:

```diff
--- a/tsc/internal/printer/helpers.go
+++ b/tsc/internal/printer/helpers.go
@@ -79,7 +79,7 @@ var addDisposableResourceHelper = &EmitHelper{
             if (!Symbol.asyncDispose) throw new TypeError("Symbol.asyncDispose is not defined.");
             dispose = value[Symbol.asyncDispose];
         }
-        if (dispose === void 0) {
+        if (dispose === void 0 || dispose === null) {
             if (!Symbol.dispose) throw new TypeError("Symbol.dispose is not defined.");
             dispose = value[Symbol.dispose];
             if (async) inner = dispose;
```

Verification command and output:

```sh
# Recompiles the twelve-case fixture with the patched binary at target es2025, then runs
# native, inline-downlevel, patched-downlevel, and tslib variants and diffs their event logs.
node ${HOME}/temp/agent/ts-dispose-probe/sem/runner.mjs
# native vs downlevel: 11 difference(s)
# native vs patched:    1 difference(s)   <- only the implementation-defined message text
# downlevel vs tslib:   IDENTICAL
```

`tslib` needs the same one-token change at `tslib.es6.js:305` for the `importHelpers` path to match.

Outcome:
 constraints 1,
 2,
 3,
 5,
 and 6 hold.
Constraint 4 fails for an agent filing,
 so the draft is kept below and is not fileable from a session.

### Draft upstream issue (do not file as-is; a human operator must choose and shepherd it per `CONTRIBUTING.md`)

~~~md
Title: `__addDisposableResource` helper throws on a `null` `Symbol.asyncDispose` instead of falling back to `Symbol.dispose`

Labels: Bug, Domain: JS Emit

The `__addDisposableResource` emit helper treats only `undefined` as an absent async disposer.
`GetDisposeMethod` in the merged explicit resource management text uses `GetMethod`, which returns
`undefined` for a property value that is `undefined` or `null`, and then falls back to `%Symbol.dispose%`.

Reproduction, TypeScript 7.0.2 and main at fed0bf24149fb1ed36039212648bafdafc1ea10e:

```ts
// repro.ts
async function main(): Promise<void> {
  await using resource = {
    [Symbol.asyncDispose]: null,
    [Symbol.dispose]: () => { console.log('sync dispose ran'); },
  } as unknown as AsyncDisposable;
  console.log('body completed');
}

await main();

export {};
```

```sh
tsc repro.ts --ignoreConfig --target es2025 --module esnext --lib ESNext,DOM --outDir out
node out/repro.js
```

Expected (matches V8 with `--target esnext`, and matches the spec): `body completed`, then
`sync dispose ran`.
Actual: `TypeError: Object not disposable.`

Cause, `tsc/internal/printer/helpers.go:82`:

```js
if (dispose === void 0) {
```

Suggested fix:

```diff
-        if (dispose === void 0) {
+        if (dispose === void 0 || dispose === null) {
```

Built and verified against a twelve-case disposal fixture: with this change the `es2025` output matches
`esnext` output on every case except the implementation-defined `SuppressedError` message text.
`tslib`'s copy of the helper needs the same change.
~~~

### References

- `tsc/internal/transformers/estransforms/definitions.go:10,24,27,29` (target gate)
- `tsc/internal/transformers/estransforms/using.go:21` (transform entry)
- `tsc/internal/printer/helpers.go:70,82,98` (helper text and the `null` guard)
- `tsc/internal/printer/printer.go:4635,4638` (per-file inline versus external helpers)
- `tsc/internal/compiler/program.go:493,496` (`importHelpers` gate)
- tc39/ecma262#3000,
   `sec-getdisposemethod` and `sec-disposeresources`,
   merged 2026-06-24
- mdn/browser-compat-data `javascript/statements.json` (`using`,
   `await_using`) and
  `javascript/builtins/Symbol.json` (`dispose`,
   `asyncDispose`)
- babel/babel#16409,
   which aligned Babel's helper to the same `GetDisposeMethod` fallback
- microsoft/TypeScript#63522,
   an open `using` emit bug involving shadowed `Symbol`
- `doc/handover/slopo-cluster-issue-triage.md`,
   which tracks the stray emitted `.js` files in this
  workspace

## Task-tsc 0.0.2 converted compiler process failures into successful checks

### Symptom and scope

During the Pi factory-scope migration on 2026-10-14,
the TypeScript subprocess printed `fatal error: newosproc`,
but the repository's `task-tsc` wrapper returned exit zero.
The [native incident record](pi-sdk-staging.md#overlapping-native-verification-and-oxlint-1870-stop-before-completing-checks)
retains the separate thread-allocation failure.
This section concerns the owned wrapper's incorrect result,
not the cause of the Go runtime's failed thread allocation.

The original log is `${HOME}/temp/agent/pi-factory-scopes-types-20261014.log`.
A zero wrapper status was not accepted as verification once that diagnostic was observed.
An isolated bounded retry completed successfully,
but could not repair the wrapper's error handling.

### Owned source cause

Before the repair,
`package/monorepo-task/core-exec/src/tsc-filter.ts:203`
only preserved a failed subprocess status when recognized TypeScript diagnostic headers survived filtering:

```ts
// Historical tsc-filter.ts condition, replaced on 2026-10-14
if (stdoutResult.hasRemainingErrors || stderrResult.hasRemainingErrors)
  process.exitCode = subprocessError.exitCode ?? 1;
```

`package/monorepo-task/core-exec/src/tsc-output-filter.ts:331`
sets that flag for retained recognized diagnostic headers:

```ts
// tsc-output-filter.ts:331,339
if (isDiagnosticLine(line)) {
  if (isSuppressedDiagnostic(line)) {
    droppingContinuation = true;
  } else {
    droppingContinuation = false;
    hasRemainingErrors = true;
    kept.push(line);
  }
}
```

A Go panic,
an unfamiliar diagnostic,
or an empty failed process has no matching `error TS` header.
The separate signal branch also does not cover ordinary nonzero process exits.
Consequently the wrapper could print a failure and leave its own default exit status at zero.

### Repair and compatibility boundary

`package/monorepo-task/core-exec/src/tsc-filter.ts:204`
now preserves failure unless `isFullySuppressedCompilerFailure` accounts for the output.
`package/monorepo-task/core-exec/src/tsc-suppressed-failure.ts:53`
requires actual removal of known diagnostics and rejects remaining stderr:

```ts
// tsc-suppressed-failure.ts:53
if ((stdout === filteredStdout) && (stderr === filteredStderr)) return false;
if (filteredStderr.trim().length > 0) return false;
return filteredStdout.split('\n').every(isEmissionMetadata);
```

Its stdout metadata recognizer accepts blank lines and `TSFILE: ` records with a nonblank path.
That preserves the configured compiler's emitted-file reporting:
`package/config/typescript/tsconfig.options.json:132` sets `"listEmittedFiles": true`.
Unknown output remains a failure instead of silently expanding the suppression policy.
This may surface unfamiliar future compiler metadata for explicit review;
it does not remove any diagnostic from the existing filter.

### Verification and rejected approaches

The fixture executes the built `dist/final/node/tsc-filter.mjs`
against a disposable Node executable named `tsc`.
Only the child process receives the modified executable search path.
No actual thread exhaustion is induced.
The source is `package/monorepo-task/core-exec/src/tsc-failure-fixture.ts`;
its per-case directory is removed after the child and output descriptors close.

The original built wrapper failed the expected-status assertions for:

- A Go panic with exit status two.
- A silent failure with exit status two.
- A non-TypeScript diagnostic with exit status seven.
- A panic beside otherwise suppressed dependency diagnostics.

The repaired artifact preserves those statuses while allowing successful output,
known suppressed diagnostics,
and emitted-file metadata accompanying known suppression.
Emission metadata alone cannot explain an unsuccessful process.
The current fixture also checks literal quotes,
backslashes,
newlines,
and statement delimiters when embedding output into its temporary executable.

```bash
# From the repository root; each check consumes rebuilt task-util output
mise run //package/monorepo-task/core-exec:build:js
mise run //package/monorepo-task/core-exec:test:tsc-failure
mise run //package/monorepo-task/core-exec:test:unit -- src/tsc-filter.unit.test.ts
```

The original failure log is `${HOME}/temp/agent/tsc-wrapper-failure-red-20261014.log`.
The repaired process and existing filter suite log is
`${HOME}/temp/agent/tsc-wrapper-failure-tests-final-20261014.log`.
The completed changed-file lint reports zero warnings and zero errors across four files;
the task-util and real Pi package type checks also pass.
The final process catalog is retained in `tsc-failure.unit.test.ts` and passes through `test:tsc-failure`.

The first success-control assertion incorrectly expected a trailing newline.
The initial report that this control passed is retracted.
Installed `nano-spawn` 2.1.0 removes the final newline at `node_modules/nano-spawn/source/result.js:73`:

```js
// nano-spawn/source/result.js:73
const getOutput = output => output.at(-1) === '\n'
  ? output.slice(0, output.at(-2) === '\r' ? -2 : -1)
  : output;
```

The expected successful wrapper output now matches that incumbent behavior.
The fixture captures wrapper output through separate disposable file descriptors,
so its own subprocess library does not normalize the observed bytes again.
The mistaken newline expectation does not invalidate the separate nonzero-status failures.

### Upstream filing decision

- Responsibility belongs to the repository-owned failure wrapper;
  no upstream defect is established by this false-success result.
- The repair is implemented in that owned wrapper.
- Upstream supported-use boundaries are not needed to change local failure handling.
- No external contribution or issue filing is proposed,
  so contribution-policy permission is not asserted.
- No prediction about an upstream maintainer's disposition is made.
- The built-wrapper process fixture supplies failing-before and passing-after evidence for the local repair.

There is nothing to file upstream from this wrapper defect.
The independent Go thread failure remains bounded by its existing incident evidence.

## Related Documentation

- [VSCode](vscode.md):
   VSCode extension configuration for TypeScript tools.
