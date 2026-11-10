# `@mdx-js/mdx` 3.1.1 emits literal-string tag names for author-written lowercase JSX, bypassing `useMDXComponents()` and rendering custom-element tags as inert HTML

## Symptom

A custom component is registered in the components map under a
kebab-case key like `'callout-alert'`.
 The MDX source uses
`<callout-alert>...</callout-alert>`.
 The component function
is never invoked;
 the tag passes through to the output HTML as
a plain element with its attributes copied verbatim.

Example output when the bug manifests:

```html
<callout-alert data-type='warning'><p>
    Never use Bootstrap in production.
  </p></callout-alert>
```

Expected output (if the component function had been invoked):

```html
<callout-alert data-is><blockquote data-type='warning'>
    <alert-indicator data-is>...</alert-indicator><alert-content data-is><p>
        ...
      </p></alert-content>
  </blockquote></callout-alert>
```

## Root cause

MDX intentionally distinguishes three JSX shapes in
`recma-jsx-rewrite`:

1.  Capitalised identifier (`<Foo>`):
     treated as a component
    reference.
2.  Explicit lowercase JSX written by the author
    (`<callout-alert>`):
     preserved as a literal tag name;
     no
    components lookup.
3.  Tags generated from markdown (`# x` -> `<h1>`):
     rewritten
    to look up `_components.h1`.

The discriminator is the `data._mdxExplicitJsx` flag set
during the remark phase.

Source trace (paths relative to the installed
`@mdx-js/mdx@3.1.1` package):

- `lib/plugin/remark-mark-and-unravel.js:88-94`:
   sets
  `data._mdxExplicitJsx = true` on every `mdxJsxFlowElement`
  and `mdxJsxTextElement` node:

  ```js
  if (
    node.type === 'mdxJsxFlowElement'
    || node.type === 'mdxJsxTextElement'
  ) {
    const data = node.data || (node.data = {});
    data._mdxExplicitJsx = true;
  }
  ```

- `lib/plugin/recma-jsx-rewrite.js:160`:
   treats any JSX
  whose tag is a valid identifier and does not start with a
  lowercase letter as a component (so `<Foo>`,
   `<$foo>`,
  `<_bar>` bind to a destructured reference).
- `lib/plugin/recma-jsx-rewrite.js:177-180`:
   the branch
  that fires for author-written lowercase hyphenated JSX;
   its
  body is an explanatory comment only:

  ```js
  } else if (node.data && node.data._mdxExplicitJsx) {
    // Do not turn explicit JSX into components from `_components`.
    // As in, a given `h1` component is used for `# heading` (next case),
    // but not for `<h1>heading</h1>`.
  }
  ```

- The fall-through `else` on the same file (lines 181-210)
  rewrites markdown-generated lowercase tags (which lack
  `_mdxExplicitJsx`) into `_components.tagname` lookups.
  Hyphenated non-identifier names get aliased to
  `_componentN` bindings there.
   Explicit JSX never reaches
  that branch.

The behaviour is documented by the comment above and mirrors
the stated design goal:
 overriding the `h1` component should
customise markdown-produced headings without also hijacking
any literal `<h1>` the author wrote.

## Verification

Version under test:
 `@mdx-js/mdx@3.1.1`.

Reproduce:

```ts
// repro.ts
import { compile, } from '@mdx-js/mdx';

const src = '# Hi\n\n<callout-alert>text</callout-alert>\n\n<Foo>x</Foo>';
console.log(String(await compile(src,),),);
```

Run with `bun repro.ts`.
 The relevant fragment:

```js
const _components = {
    h1: 'h1',
    ...props.components,
  }, { Foo, } = _components;
// ...
return _jsxs(_Fragment, {
  children: [
    _jsx(_components.h1, { children: 'Hi', },), // markdown  -> lookup
    '\n',
    _jsx('callout-alert', { children: 'text', },), // explicit JSX -> literal string
    '\n',
    _jsx(Foo, { children: 'x', },), // capitalized -> component reference
  ],
},);
```

Three tag shapes,
 three different emission rules.
 Only the
markdown-derived and capitalised tags consult the components
map.
 The author-written lowercase hyphenated tag becomes a
literal string;
 the JSX runtime renders it as a plain HTML
element.

## Verified workaround: author the MDX source with a capitalised identifier; emit the kebab tag from the component function

In the barrel:

```ts
// src/components/index.ts
export { CalloutAlert, } from './callout-alert.ts';
export { QuizQuestion, } from './quiz-question.ts';
```

In the MDX content:

```mdx
<CalloutAlert data-type="warning">
Never use Bootstrap in production.
</CalloutAlert>
```

In the component:

```ts
export function CalloutAlert(props: CalloutAlertProps,): SafeHtml {
  return jsx('callout-alert', {
    'data-is': true,
    children: blockquote,
  },);
}
```

The MDX author-facing identifier stays capitalised so MDX
dispatches through the components map;
 the rendered DOM
element is the hyphenated custom element because the function
emits it.

Tradeoff:
 introduces an indirection between the MDX source
("CalloutAlert") and the rendered output
("`<callout-alert>`").
 Authors need to know both names.
 The
project chose this approach because the alternative
(`mdxJsx` runtime wrapper,
 below) hides the dispatch entirely.

## What does not work

- Re-exporting under a string key:
  `export { CalloutAlert as 'callout-alert' } from './callout-alert.ts'`
  combined with `import * as mdxComponents from './index.ts'`
  does not fix it.
   The problem is not that
  `_components['callout-alert']` is missing;
   it is that MDX
  never emits that lookup for explicit JSX.
- Providing `providerImportSource`:
   setting
  `compile(src, { providerImportSource: '@mdx-js/react' })`
  wires `useMDXComponents` into the compile output for
  capitalised identifiers only;
   the explicit JSX branch is
  unchanged.
   The compile output still emits
  `_jsx("callout-alert", ...)` as a literal string tag.
- Intercepting at the jsx runtime boundary:
   a wrapper that
  dispatches string tag names through a components map does
  work,
   but adds a layer that must be kept in sync with the
  registry and makes the transformation invisible at the MDX
  source level:

  ```ts
  function mdxJsx(type, props, key,) {
    if (typeof type === 'string' && type in components)
      return jsx(components[type], props, key,);
    return jsx(type, props, key,);
  }
  ```

  Avoid unless MDX source authors cannot be asked to use
  capitalised identifiers.
   The capitalised-identifier solution
  has no runtime cost and no registry indirection.

## Why we would file this upstream (documentation only)

1.  **Is it really upstream's fault?
    ** No (behaviour);
     yes
    (documentation gap).
     The behaviour is intentional and
    correct;
     the user-facing docs do not call out the
    distinction.
2.  **Can upstream fix it?
    ** Yes;
     the fix is a documentation
    change to the "Components" section of mdxjs.
    com.
3.  **Are they supporting this use case?
    ** Yes;
     custom
    components are explicitly supported via capitalised
    identifiers.
4.  **Will they likely fix it?
    ** Plausible;
     documentation PRs
    are routinely accepted.
5.  **Have we prototyped a minimal fix?
    ** Yes;
     the draft below
    proposes the exact text.

Decision:
 worth filing as a docs improvement (not a code
change).

## Draft upstream issue (kept as reference; revise before filing)

````md
**Title**: Documentation improvement: call out that lowercase JSX tags bypass the components map

**Labels**: documentation

**Description**:

The behaviour that `<h1>heading</h1>` in MDX source is preserved as a literal tag and does NOT resolve through `useMDXComponents()` (while `# heading` DOES) is intentional and explained by the comment at `package/mdx/lib/plugin/recma-jsx-rewrite.js:177-180`.

This is not clearly called out in the user-facing documentation at <https://mdxjs.com/docs/using-mdx/#components>. The current docs imply any element can be overridden via the components map, without distinguishing markdown-produced elements from author-written JSX.

Authors reaching for custom elements (e.g. `<callout-alert>`) hit this as a pit of failure: registering `{ 'callout-alert': CalloutAlert }` in the components map has no effect, the tag renders as a plain HTML element, and there is no diagnostic. The fix (use a capitalised identifier in MDX source and emit the kebab tag from the component function) is obvious once the constraint is known but not discoverable from the docs.

Suggested documentation change: add a short note to the "Components" section explaining that lowercase JSX tags in MDX source are treated as intrinsic HTML elements and will not be routed through the components map, and that only markdown-syntax-produced elements and capitalised JSX identifiers are.

Reproduction:

```ts
import { compile, } from '@mdx-js/mdx';
const src = '# Hi\n\n<callout-alert>x</callout-alert>\n\n<Foo>y</Foo>';
console.log(String(await compile(src,),),);
```

Output shows `_jsx(_components.h1, ...)` for the markdown heading, `_jsx("callout-alert", ...)` for the explicit JSX, and `_jsx(Foo, ...)` for the capitalised identifier.
````
