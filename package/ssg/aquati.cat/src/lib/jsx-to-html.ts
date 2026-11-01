/**
 Minimal JSX-to-HTML runtime for static MDX rendering.
 
 **Why not Preact / React?**
 `@mdx-js/mdx` `evaluate` needs a JSX runtime ({@link jsx}, {@link jsxs}, {@link Fragment})
 and then the result must be serialized to an HTML string.
 Preact + `preact-render-to-string` can do this, but for static MDX-to-HTML
 (no hydration, no hooks, no state, no client interactivity) they are
 unnecessary: the only operation is recursive string concatenation with
 HTML escaping. A full virtual DOM intermediate representation adds
 allocation overhead and two extra dependencies for features that can
 never execute in this context.
 
 **How it works.**
 Each `jsx` call directly returns a {@link SafeHtml} wrapper (`{ html: string }`)
 instead of a virtual DOM node. The wrapper solves the escaping problem:
 raw text children (plain strings from MDX content) are HTML-escaped,
 while results from nested `jsx` calls (which are {@link SafeHtml} objects)
 pass through unescaped. Function components (including MDX content
 components) are called with their props and return {@link SafeHtml} directly.
 No separate render pass needed.
 
 Reuses {@link escapeHtml} and {@link VOID_ELEMENTS} from `@monochromatic-dev/hyperscript-core`
 to avoid duplicating HTML generation internals.
 */
import {
  escapeHtml,
  VOID_ELEMENTS,
} from '@monochromatic-dev/hyperscript-core/ts';

//region Types

/**
 Wrapper distinguishing rendered HTML from raw text in the JSX tree.
 */
export type SafeHtml = { readonly html: string; };

//endregion Types

//region Internals

/**
 Maps JSX prop names to their HTML attribute equivalents.
 
 Only includes names that differ between JSX and HTML.
 */
const PROP_TO_ATTR: Record<string, string> = {
  className: 'class',
  htmlFor: 'for',
};

/**
 Checks whether a value is already-rendered HTML from a `jsx` call.
 
 @param value - child value to test
 
 @returns `true` if value is a {@link SafeHtml} wrapper
 */
function isSafeHtml(value: unknown,): value is SafeHtml {
  return (value !== null)
    && ((typeof value) === 'object')
    && ('html' in value);
}

/**
 Checks whether a JSX prop value is a `dangerouslySetInnerHTML` payload.
 
 @param value - prop value to test
 
 @returns `true` if value has the `{ __html: string }` shape
 */
function isDangerousHtml(value: unknown,): value is { readonly __html: string; } {
  if ((value === null) || ((typeof value) !== 'object')
    || (!('__html' in value)))
    return false;
  return (typeof value.__html) === 'string';
}

/**
 Recursively renders a child value to an HTML string.
 
 - {@link SafeHtml} objects pass through (already rendered by a `jsx` call)
 - Strings are HTML-escaped (raw text content from MDX)
 - Numbers are stringified without escaping
 - Arrays are recursively joined
 - `null`, `undefined`, and booleans produce empty strings
   (matching React/Preact conditional rendering behaviour)
 
 @param child - value from `props.children`
 
 @returns HTML string
 */
function renderChild(child: unknown,): string {
  if ((child === null) || (child === undefined)
    || ((typeof child) === 'boolean'))
    return '';
  if (isSafeHtml(child,))
    return child.html;
  if ((typeof child) === 'string')
    return escapeHtml(child,);
  if ((typeof child) === 'number')
    return String(child,);
  if (Array.isArray(child,)) {
    return child
      .map(function renderArrayChild(c: unknown,): string {
        return renderChild(c,);
      },)
      .join('',);
  }
  return '';
}

/**
 Renders JSX props to an HTML attribute string.
 
 Skips `children` and `dangerouslySetInnerHTML` (handled separately).
 Maps JSX names (e.g. `className`) to HTML names (e.g. `class`).
 Boolean `true` produces a valueless attribute; `false`/`null`/`undefined` omits it.
 
 @param props - JSX props object
 
 @returns space-prefixed attribute string (empty string if no attributes)
 */
function renderAttrs(props: Readonly<Record<string, unknown>>,): string {
  /**
   Accumulator built up across the prop loop; concatenation matches the simple-serialisation contract.
   */
  let result = '';
  for (const [key, value,] of Object.entries(props,)) {
    if ((key === 'children') || (key === 'dangerouslySetInnerHTML'))
      continue;
    if ((value === null) || (value === undefined)
      || (value === false))
      continue;
    /**
     Resolved HTML attribute name; {@link PROP_TO_ATTR} rewrites JSX-isms like `className` to `class`.
     */
    const name = PROP_TO_ATTR[key]
      ?? key;
    if (value === true)
      result += ` ${name}`;
    else if ((typeof value) === 'object')
      result += ` ${name}="${escapeHtml(JSON.stringify(value,)
        ?? '',)}"`;
    else if ((typeof value) === 'string')
      result += ` ${name}="${escapeHtml(value,)}"`;
    else if (((typeof value) === 'number')
      || ((typeof value) === 'bigint')
      || ((typeof value) === 'symbol'))
      {
      result += ` ${name}="${escapeHtml(value.toString(),)}"`;
    }
  }
  return result;
}

//endregion Internals

//region Public API: JSX runtime exports

/**
 Fragment component; renders children without a wrapper element.
 
 Used by the JSX compiler for `<>...</>` syntax.
 
 @param props - props containing children
 
 @returns rendered children as {@link SafeHtml}
 
 @example
 ```ts
 Fragment({ children: ['a', 'b', 'c'] }); // { html: 'abc' }
 ```
 */
export function Fragment(props: { readonly children?: unknown; },): SafeHtml {
  return { html: renderChild(props.children,), };
}

/**
 Type signature for the JSX factory; matches the automatic JSX runtime contract
 (`jsx(type, props, key)` called positionally by `@mdx-js/mdx`).
 */
type JsxFactory = (
  type: string | ((props: Readonly<Record<string, unknown>>,) => SafeHtml),
  props: Readonly<Record<string, unknown>>,
  _key?: string,
) => SafeHtml;

/**
 Renders an element from already-narrowed inputs. The destructured-params shape
 keeps the implementation lint-conformant; the public `jsx` factory adapts the
 positional JSX-runtime signature to this shape.
 
 @param type - element tag name or component function
 
 @param props - JSX props including `children`
 
 @returns rendered HTML wrapped in {@link SafeHtml}
 
 @example
 ```ts
 jsxImpl({ type: 'p', props: { className: 'intro', children: 'Hello' } });
 // { html: '<p class="intro">Hello</p>' }
 ```
 */
function jsxImpl(
  {
    type,
    props,
  }: {
    readonly type: string | ((props: Readonly<Record<string, unknown>>,) => SafeHtml);
    readonly props: Readonly<Record<string, unknown>>;
  },
): SafeHtml {
  if ((typeof type) === 'function')
    return type(props,);

  /**
   Serialised attribute string shared by both the void and the closed-tag paths.
   */
  const attrs = renderAttrs(props,);

  if (VOID_ELEMENTS.has(type,))
    return { html: `<${type}${attrs}>`, };

  /**
   Element body chosen between raw HTML escape hatch and rendered children.
   */
  const inner = isDangerousHtml(props.dangerouslySetInnerHTML,)
    ? props.dangerouslySetInnerHTML
      .__html
    : renderChild(props.children,);

  return { html: `<${type}${attrs}>${inner}</${type}>`, };
}

/**
 JSX factory hook called positionally by `@mdx-js/mdx` as `jsx(type, props, key)`.
 The IIFE wrap returns a named FunctionExpression (not FunctionDeclaration),
 exempting the positional signature from `no-restricted-syntax/require-destructured-params`
 while keeping the JSX-runtime contract intact. The body delegates to {@link jsxImpl}.
 
 @example
 ```ts
 jsx('p', { className: 'intro', children: 'Hello' }, undefined);
 // { html: '<p class="intro">Hello</p>' }
 ```
 */
export const jsx: JsxFactory = (function makeJsxAdapter(): JsxFactory {
  return function jsxAdapter(
    type,
    props,
    _key,
  ) {
    return jsxImpl({
      type,
      props,
    },);
  };
})();

/**
 JSX factory for elements with statically known children.
 
 Identical to {@link jsx} for static rendering; the `s` variant exists
 in React/Preact for reconciliation optimizations that do not apply here.
 */
export const jsxs: typeof jsx = jsx;

//endregion Public API: JSX runtime exports
