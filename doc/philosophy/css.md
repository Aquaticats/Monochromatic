# CSS

## h-css: hyperscript-style CSS generation

h-css is a string-returning function that constructs CSS from declarative options objects.
Same pattern as h-xml and h-html:
 call `$()` with named parameters,
 get a string back.

```ts
$({ rule: '.card', decls: { display: 'flex', gap: '1rem', }, },);
// '.card{display:flex;gap:1rem}'
```

### Why not template literals

Template literals in CSS-in-JS (Lit's `css`,
 styled-components,
 Emotion) treat CSS as opaque strings.
Editors see them as plain text:
 no property name autocomplete,
 no value validation,
 no type checking.
Composition requires string interpolation,
 which breaks structure and makes refactoring fragile.

h-css uses object keys for property names and object values for property values.
TypeScript types the keys via `csstype`'s `PropertiesHyphen` (auto-generated from MDN browser compatibility data),
giving editor intellisense for every standard CSS property in kebab-case and value autocomplete for each property.
Custom properties (`--*`) are also accepted via a template literal index signature.

### Why not external CSS files

External CSS files (CSS Modules,
 vanilla-extract,
 Tailwind,
 UnoCSS) generate stylesheets
that attach to the document's global scope.
Shadow DOM encapsulation prevents these global stylesheets from reaching shadow roots.
Injecting external CSS into each shadow root requires either duplicating the file content
or using `adoptedStyleSheets` with constructable stylesheets.
 Both add machinery
that h-css eliminates by generating CSS strings directly where they're needed.

### Why not runtime CSS-in-JS

Runtime CSS-in-JS libraries (Emotion,
 styled-components,
 Goober) inject styles into `<style>` elements
at runtime in the browser:
 the browser parses JavaScript,
 generates CSS strings,
 creates DOM elements,
and inserts them into the document.
h-css generates the same CSS strings at build time (or SSR time) as part of the JS bundle.
The browser receives pre-built CSS strings that need no generation step.

### Why not build-time CSS tooling

PostCSS and Lightning CSS process `.css` files through a build pipeline with custom syntax
(`@mixin`,
 `@apply`,
 nesting transforms,
 vendor prefixes).
This adds a build dependency,
 a separate compilation step,
 and a custom syntax that editors
may not fully support.
h-css needs no build step:
 it's a pure function that runs wherever TypeScript runs.

## Mixins as functions

`@mixin`/`@apply` is replaced by plain TypeScript functions returning `CssDeclarations` records.
Composition uses object spread:

```ts
function buttonOutlined(): CssDeclarations {
  return { ...flexCenter(), ...minTouchTarget(), gap: '0.5rem',
    cursor: 'pointer', };
}
```

Advantages over CSS `@mixin`/`@apply`:

- Parameterized:
   `focusOutline({ offset: '0.25rem' })` (function arguments replace mixin variables)
- Type-checked:
   return type `CssDeclarations` ensures property names and values are valid CSS
- Composable:
   object spread merges declarations;
   later properties override earlier ones
- Refactorable:
   rename a mixin function and TypeScript reports every call site
- No custom syntax:
   standard TypeScript that any editor,
   linter,
   and formatter understands

## Shadow DOM style injection

Each web component generates its styles as a CSS string (via h-css calls joined together)
and injects them into a `<style>` element in its shadow root.
Global styles are generated as a TypeScript module exporting a CSS string,
bundled into the client JS entry point,
 and injected via `injectCSS()`.

No separate CSS build step,
 no CSS file output,
 no `<link>` elements.
CSS lives alongside the component code that uses it.

## Design token system

All colors use CSS custom properties defined in a token layer.
Light and dark themes use `@media (prefers-color-scheme: dark)` at-rules
generated via h-css's `at`/`params` split:

```ts
$({
  at: 'media',
  params: '(prefers-color-scheme: dark)',
  children: [
    $({ rule: ':root', decls: { '--color-fg': 'oklch(0.9 0 0)', }, },),
  ],
},);
```

No `var()` fallbacks:
 the token system guarantees every custom property is defined.
