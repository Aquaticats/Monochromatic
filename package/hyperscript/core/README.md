# module-hyperscript

Type-safe hyperscript factories for declarative HTML,
 CSS,
 DOM,
 and XML generation.

Each factory function builds strings (or DOM elements) from a named-parameter
options object,
 replacing manual template literals with composable,
 type-checked calls.

## Exports

The package provides two entry points:

### Built JavaScript (`.`)

Bundled and minified JavaScript.

### TypeScript source (`./ts`)

Raw TypeScript source for workspace consumers.

### Shared exports

Both entry points expose the same named exports:

<table>
<thead>
<tr>
<th>Export</th>
<th>Returns</th>
<th>Environment</th>
<th>Description</th>
</tr>
</thead>
<tbody>
<tr>
<td>`hHtml`</td>
<td>`string`</td>
<td>Any JS runtime</td>
<td>Server-side HTML with automatic XSS escaping</td>
</tr>
<tr>
<td>`hCss`</td>
<td>`string`</td>
<td>Any JS runtime</td>
<td>CSS rules and at-rules with strict property/value types via `csstype`</td>
</tr>
<tr>
<td>`hDom`</td>
<td>`HTMLElement`</td>
<td>Browser only</td>
<td>Live DOM elements via `document.createElement`</td>
</tr>
<tr>
<td>`hXml`</td>
<td>`string`</td>
<td>Any JS runtime</td>
<td>Well-formed XML with namespace support and self-closing tags</td>
</tr>
</tbody>
</table>

All `css*` value constructors (`cssRem`,
 `cssVar`,
 `cssOklch`,
 etc.) are also
top-level named exports.

## Usage

```ts
// TypeScript source (workspace)
import {
  cssRem,
  cssVar,
  hCss,
  hHtml,
} from '@monochromatic-dev/hyperscript-core/ts';

// Built JavaScript (external consumers)
import {
  cssRem,
  cssVar,
  hCss,
  hHtml,
} from '@monochromatic-dev/hyperscript-core';
```

```ts
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

const card = h({
  tag: 'div',
  class: 'card',
  children: [h({ tag: 'p', text: 'hello', },),],
},);
// '<div class="card"><p>hello</p></div>'
```

```ts
import {
  cssRem,
  cssVar,
  hCss as $,
} from '@monochromatic-dev/hyperscript-core/ts';

const styles = $({
  rule: '.card',
  decls: { display: 'flex', gap: cssRem(1,), color: cssVar('fg',), },
},);
// '.card{display:flex;gap:1rem;color:var(--fg)}'
```

## CSS value constructors

Branded value constructors replace raw strings,
preventing invalid units and disallowed color functions at the type level:

### Lengths

`cssRem`,
 `cssEm`,
 `cssCh`,
 `cssLh`,
 `cssVi`,
 `cssVb`,
 `cssCqi`,
 `cssCqb`,
 `cssDvi`,
 `cssDvb`,
`cssFr`,
 `cssPercent`

### Time

`cssS`

### Angle

`cssTurn`

### Color

`cssOklch`,
 `cssColorFn`

### Reference

`cssVar`,
 `cssCalc`,
 `cssMin`,
 `cssMax`,
 `cssClamp`

### Number

`cssNum`,
 `cssInt`

### Transform

`cssTranslateX`,
 `cssTranslateY`,
 `cssRotate`,
 `cssScale`

### Anchor

`cssAnchor`

### Composition

`cssCubicBezier`,
 `cssCommaList`,
 `cssCompounded`

## Design decisions

### No cross-dependencies

Each factory module is fully self-contained with zero imports from sibling
modules or external packages,
 except `csstype` for h-css type definitions.

### Split from module-es

These factories were originally subpath exports of
`@monochromatic-dev/module-es`.
They were extracted because they have zero coupling to the type utilities and
general-purpose helpers in that package.

### Named factory exports

Each module's `$` function is re-exported with a distinct name (`hCss`,
 `hDom`,
`hHtml`,
 `hXml`) so all four coexist in a single namespace.
 Consumers typically
alias on import:
 `import { hHtml as h } from '...'`.

### Prior art

Our `hHtml` crowds out
[`create-html-element`](https://github.com/sindresorhus/create-html-element)
as the HTML-string factory here.
 Thanks to
[Sindre Sorhus](https://github.com/sindresorhus) for the prior art.

### Children carry the output type, not the options type

Each factory's `children` field accepts pre-built fragments of whatever the
factory produces:

- `hHtml` returns `string`;
   `children` is `readonly string[]`
- `hXml` returns `string`;
   `children` is `readonly string[]`
- `hCss` returns `string`;
   `children` is `readonly string[]`
- `hDom` returns `HTMLElement`;
   `children` is `readonly (Node | string)[]`

Rationale,
 especially for `hCss`,
 where `children: readonly CssOptions[]` was
considered and rejected:

#### Uniform output channel

Every CSS-producing thing in a consuming package,
 such as a literal `$()` call,
helper function,
 imported constant,
 output of an external tool,
 or hand-written
block,
 speaks the same type:
 `string`.
 Modules export `STYLES: string` and splice
them into parent `children` arrays without commitment to "built" vs "unbuilt"
form.

#### Late binding for non-factory sources

Programmatically built children,
 such as a `for`-loop generating `@keyframes`
stops,
 and externally sourced CSS,
 such as minifier output or vendored snippets,
flow through the same channel as factory output.
 Restricting `children` to
options would force every non-factory source through a `{ raw: string }` wrapper,
duplicating the per-node `raw` escape hatch at the structural level.

#### One-node-per-call mental model

Each `$()` call serializes its own declarations and concatenates already-built
child strings,
 with no recursion into child option trees.
 The same shape applies
to `hHtml`,
 `hXml`,
 and `hCss`,
 keeping the four factories structurally
identical.

#### Local error sites

Type errors fire on the leaf literal (`$({ rule: 'x', decls: { foo: 'bar' } })`),
not on the outermost call after an entire nested tree fails to validate.

`hDom` diverges to `Node | string` because `element.append(...children)` accepts
live Node references and text;
 string entries become text nodes.
 There is no
analogous "string that gets parsed" path for the string-producing factories.
