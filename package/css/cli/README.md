# @monochromatic-dev/css-cli

CSS build tool that bundles `@import` statements across monorepo packages and
processes custom `@mixin`/`@apply` syntax into expanded CSS.

## Why this exists

No single CSS tool handles all three requirements at once:

1.  **Monorepo-aware `@import` resolution**:
    off-the-shelf CSS tools resolve relative paths only,
    not `node_modules` or package.json `exports`
2.  **Custom `@mixin`/`@apply` processing**:
    no standard plugin provides the mixin semantics this monorepo needs
3.  **Browser-compatible**:
    the mixin pipeline runs in both Node.js and browser environments
    (no native binary dependencies,
     no process globals)

Parsing sits on `@monochromatic-dev/css-core-edit`,
the workspace's byte-preserving CSS CST over the `@csstools/css-tokenizer`
spec tokenizer.
Untouched CSS survives byte-exactly,
 comments and author formatting included.

See [doc/troubleshooting/css-tooling.md](../../../doc/troubleshooting/css-tooling.md)
for the full tooling chronicle,
 including the 2026-07 parser survey that led
here from postcss.

## Usage

### CLI

```bash
build-css src/main.css dist/bundle.css
```

### Build a file (Node)

```ts
import { buildCss, } from '@monochromatic-dev/css-cli';

const css = await buildCss({
  input: 'src/main.css',
  output: 'dist/bundle.css',
},);
```

### Expand mixins in memory (browser-safe)

For consumers that already have CSS text in memory,
such as web components with Shadow DOM styles defined as JavaScript strings:

```ts
import { expandCssMixins, } from '@monochromatic-dev/css-cli/ts';
import mixinSource from './mixins.css' with { type: 'text', };

const expanded = expandCssMixins({
  css: `
    .close { @apply --reset-button; @apply --touch-target; }
    .pill  { @apply --pill; }
  `,
  mixinCss: mixinSource,
},);
```

`expandCssMixins` collects definitions from `mixinCss` and from `css` itself
(inline definitions win on name collision),
 expands nested references,
replaces every `@apply`,
 and strips the definitions from the output.
No filesystem access,
 no process globals.

## CSS syntax

### Mixin definitions

```css
@mixin --flex-center {
  display: flex;
  align-items: center;
  justify-content: center;
}
```

Definitions are removed from the output.
A definition without a name or without structural body content is an error.

### Mixin application

```css
.component {
  @apply --flex-center;
  background: var(--primary);
}
```

`@apply` rules are replaced with the referenced mixin's body.
Bodies may contain declarations,
 nested rules
(with or without `&`),
 and further `@apply` references.

### Nested mixins

Mixins can reference other mixins via `@apply`.
Each mixin expands exactly once;
circular references throw `CircularCssMixinError`
naming the exact reference chain
(for example `--a -> --b -> --a`).

```css
@mixin --card {
  @apply --flex-center;
  padding: 1rem;
  border: 1px solid gray;
}
```

### Cross-package imports

Imports resolve through package.json `exports` mappings,
`node_modules` lookup,
 and direct file paths:

```css
/* Via exports field */
@import '@monochromatic-dev/style-monochromatic/index.css';

/* Via direct file path (package has no exports field) */
@import '@some-package/src/tokens.css';
```

Specifiers are read from parsed tokens,
so trailing conditions such as `layer(base)` or media queries
never corrupt the target.
Each file inlines once;
 circular imports resolve to nothing on revisit.

## Build pipeline

1.  **Resolve and bundle**:
    `@import` at-rules are resolved
    (relative paths,
     package.json `exports`,
     bare `node_modules` specifiers)
    and replaced by the parsed contents of their files,
     recursively
2.  **Collect mixins**:
    `@mixin` definitions move into a registry and leave the tree
3.  **Expand mixin bodies**:
    nested `@apply` between definitions resolve by memoized recursion
    with an explicit trail (exact cycle reporting)
4.  **Inline `@apply`**:
    document references splice in registry bodies,
    shared by reference thanks to the immutable CST
5.  **Write output**:
    final CSS string written to disk

## Module structure

- `index.ts`:
   public surface (`buildCss`,
   `expandCssMixins`,
   error classes)
- `build.ts`:
   file pipeline orchestration
- `expand.ts`:
   in-memory mixin pipeline
- `import.ts`:
   `@import` inlining with monorepo-aware resolution
- `mixin.ts`:
   mixin collection and expansion engine (internal)
- `errors.ts`:
   `UnknownCssMixinError`,
   `CircularCssMixinError`
- `package-resolver.ts`:
   `node_modules` and `exports` resolution.
   Its upward walk stays hand-rolled and synchronous:
   it runs inside the synchronous `transformStylesheet` visitor from `css-edit`,
   which has no async form,
   and it probes through this package's in-memory registry,
   so the async `findRoot` walker of `@monochromatic-dev/fs-path-core` cannot replace it.
- `specifier.ts`:
   specifier classification helpers
- `fs.ts`:
   adaptive file reader (in-memory registry with `node:fs` fallback)
- `fs-registry.ts`:
   in-memory `Map` for browser-side file storage
- `cli.ts`:
   `build-css` binary entry point

## Testing

```bash
# Build, then run all module-test files in this package
mise run //package/css/cli:buildAndTest
```

Integration tests exercise two CSS import resolution strategies using fixture
packages:

- **`exports` field**:
  `test-css-importing` imports from `test-css-imported`
  (has `exports` in package.json)
- **Direct file path**:
  `test-css-importing-filepath` imports from `test-css-imported-no-exports`
  (no `exports` field)

Both strategies run identical assertions:
import resolution,
 mixin removal,
 `@apply` expansion,
nested mixin inlining,
 and output file writing.
Error paths and import dedup run on disposable temp directories.
