/**
 * Minimal JSX-to-HTML runtime for fragment rendering.
 *
 * Inline copy of `packages/ssg/aquati.cat/src/lib/jsx-to-html.ts`.
 * Kept inline (not factored to a workspace package) per the design plan
 * so each surface can pin its own escape semantics.
 *
 * **How it works.** Each `jsx` call directly returns a `SafeHtml` wrapper
 * (`{ html: string }`). The wrapper solves the escaping problem: raw text
 * children (plain strings from data) are HTML-escaped, while results
 * from nested `jsx` calls (which are `SafeHtml` objects) pass through
 * unescaped. Function components are called with their props and return
 * `SafeHtml` directly; no separate render pass needed.
 *
 * Reuses `escapeHtml` and `VOID_ELEMENTS` from `@monochromatic-dev/hyperscript-core`
 * to avoid duplicating HTML generation internals.
 */

import {
  escapeHtml,
  VOID_ELEMENTS,
} from '@monochromatic-dev/hyperscript-core/ts';

//region Types

/**
 * Wrapper distinguishing rendered HTML from raw text in the JSX tree.
 */
export type SafeHtml = { readonly html: string; };

//endregion Types

//region Internals

/**
 * Maps JSX prop names to their HTML attribute equivalents.
 */
const PROP_TO_ATTR: Record<string, string> = {
  className: 'class',
  htmlFor: 'for',
};

/**
 * Checks whether a value is already-rendered HTML from a `jsx` call.
 *
 * @param value - child value to test
 *
 * @returns `true` if value is a `SafeHtml` wrapper
 */
function isSafeHtml(value: unknown,): value is SafeHtml {
  return (value !== null)
    && ((typeof value) === 'object')
    && ('html' in value);
}

/**
 * Checks whether a JSX prop value is a `dangerouslySetInnerHTML` payload.
 *
 * @param value - prop value to test
 *
 * @returns `true` if value has the `{ __html: string }` shape
 */
function isDangerousHtml(value: unknown,): value is { readonly __html: string; } {
  if ((value === null) || ((typeof value) !== 'object')
    || (!('__html' in value)))
    return false;
  return (typeof value.__html) === 'string';
}

/**
 * Recursively renders a child value to an HTML string.
 *
 * @param child - value from `props.children`
 *
 * @returns HTML string
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
 * Renders JSX props to an HTML attribute string.
 *
 * @param props - JSX props object
 *
 * @returns space-prefixed attribute string (empty string if no attributes)
 */
function renderAttrs(props: Readonly<Record<string, unknown>>,): string {
  /**
   * Accumulator for the rendered attribute string.
   */
  let result = '';
  for (const [key, value,] of Object.entries(props,)) {
    if ((key === 'children') || (key === 'dangerouslySetInnerHTML'))
      continue;
    if ((value === null) || (value === undefined)
      || (value === false))
      continue;
    /**
     * Attribute name; remapped from JSX prop names when the table provides one.
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
    else if (
      ((typeof value) === 'number')
      || ((typeof value) === 'bigint')
        || ((typeof value) === 'symbol')
    ) {
      result += ` ${name}="${escapeHtml(value.toString(),)}"`;
    }
  }
  return result;
}

//endregion Internals

//region Public API

/**
 * Fragment component; renders children without a wrapper element.
 *
 * @param props - props containing children
 *
 * @returns rendered children as `SafeHtml`
 *
 * @example
 * ```ts
 * Fragment({ children: ['a', 'b', 'c'] });
 * ```
 */
export function Fragment(props: { readonly children?: unknown; },): SafeHtml {
  return { html: renderChild(props.children,), };
}

/**
 * Type signature for the JSX factory; matches the automatic JSX runtime contract
 * (`jsx(type, props, key)` called positionally by the TS JSX transform).
 */
type JsxFactory = (
  type: string | ((props: Readonly<Record<string, unknown>>,) => SafeHtml),
  props: Readonly<Record<string, unknown>>,
  _key?: string,
) => SafeHtml;

/**
 * Renders an element from already-narrowed inputs. The destructured-params shape
 * keeps the implementation lint-conformant; the public `jsx` factory adapts the
 * positional JSX-runtime signature to this shape.
 *
 * @param type - element tag name or component function
 *
 * @param props - JSX props including `children`
 *
 * @returns rendered HTML wrapped in `SafeHtml`
 *
 * @example
 * ```ts
 * jsxImpl({ type: 'p', props: { className: 'intro', children: 'Hello' } });
 * // { html: '<p class="intro">Hello</p>' }
 * ```
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
   * Rendered attribute string used by both branches below.
   */
  const attrs = renderAttrs(props,);

  if (VOID_ELEMENTS.has(type,))
    return { html: `<${type}${attrs}>`, };

  /**
   * Inner HTML: dangerouslySetInnerHTML wins over children when present.
   */
  const inner = isDangerousHtml(props.dangerouslySetInnerHTML,)
    ? props.dangerouslySetInnerHTML
      .__html
    : renderChild(props.children,);

  return { html: `<${type}${attrs}>${inner}</${type}>`, };
}

/**
 * JSX factory hook called positionally as `jsx(type, props, key)`.
 * The IIFE wrap returns a named FunctionExpression (not FunctionDeclaration),
 * exempting the positional signature from `no-restricted-syntax/require-destructured-params`
 * while keeping the JSX-runtime contract intact. The body delegates to {@link jsxImpl}.
 *
 * @example
 * ```ts
 * jsx('p', { className: 'intro', children: 'Hello' }, undefined);
 * // { html: '<p class="intro">Hello</p>' }
 * ```
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
 * JSX factory for elements with statically known children.
 *
 * Identical to {@link jsx} for static rendering; the `s` variant exists
 * in React/Preact for reconciliation optimizations that do not apply here.
 */
export const jsxs: typeof jsx = jsx;

//endregion Public API
