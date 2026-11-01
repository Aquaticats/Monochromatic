/**
 String-returning hyperscript factory for server-side HTML generation.
 
 Same API shape as the DOM `h()` (`t object/t htmlElement/.../$`) but returns
 an HTML string instead of a live element. Useful in server contexts (e.g.
 Bun request handlers) where `document` is unavailable.
 
 Replaces manual template literal HTML like:
 ```ts
 const html = `<div class="card"><p>hello</p></div>`;
 ```
 with a declarative, composable, XSS-safe call:
 ```ts
 const html = $({
   tag: 'div',
   class: 'card',
   children: [$({ tag: 'p', text: 'hello' })],
 });
 ```
 
 @param options - Named parameters describing the element
 @param options.tag - HTML tag name
 @param options.class - CSS class name(s)
 @param options.text - Text content (HTML-escaped automatically)
 @param options.html - Raw inner HTML (NOT escaped; caller is responsible for safety)
 @param options.attrs - Record of attributes (values are HTML-escaped)
 @param options.style - Record of inline style properties
 @param options.children - Child HTML strings to concatenate inside the element
 @returns HTML string
 
 @example Standard element
 ```ts
 const html = $({
   tag: 'button',
   class: 'primary',
   text: 'Click me',
 });
 // '<button class="primary">Click me</button>'
 ```
 
 @example Void element
 ```ts
 const html = $({ tag: 'br' });
 // '<br>'
 ```
 
 @example Nested children
 ```ts
 const html = $({
   tag: 'ul',
   children: items.map(item =>
     $({ tag: 'li', text: item })
   ),
 });
 ```
 
 @example Text escaping prevents XSS
 ```ts
 const html = $({ tag: 'span', text: '<script>alert("xss")</script>' });
 // '<span>&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;</span>'
 ```
 */

/**
 HTML void elements that must not have a closing tag.
 */
export const VOID_ELEMENTS: Set<string> = new Set<string>([
  'area',
  'base',
  'br',
  'col',
  'embed',
  'hr',
  'img',
  'input',
  'link',
  'meta',
  'param',
  'source',
  'track',
  'wbr',
],);

/**
 Escapes characters that have special meaning in HTML attribute values and text content.
 
 @param raw - unescaped string
 
 @returns HTML-safe escaped string
 
 @example
 ```ts
 escapeHtml('<script>alert("xss")</script>');
 // '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
 ```
 */
export function escapeHtml(raw: string,): string {
  return raw
    .replaceAll(
    '&',
    '&amp;',
    )
    .replaceAll(
    '<',
    '&lt;',
    )
    .replaceAll(
    '>',
    '&gt;',
    )
    .replaceAll(
    '"',
    '&quot;',
    )
    .replaceAll(
    "'",
    '&#39;',
    );
}

/**
 Named parameters for string-based element creation
 */
type HOptions = {
  /**
   HTML tag name
   */
  readonly tag: string;
  /**
   CSS class name(s)
   */
  readonly class?: string;
  /**
   Text content (HTML-escaped automatically)
   */
  readonly text?: string;
  /**
   Raw inner HTML (NOT escaped)
   */
  readonly html?: string;
  /**
   Attributes set as key="value" pairs (values are HTML-escaped)
   */
  readonly attrs?: Readonly<Record<string, string>>;
  /**
   Inline style properties (camelCase or kebab-case keys)
   */
  readonly style?: Readonly<Record<string, string>>;
  /**
   Child HTML strings to concatenate inside the element
   */
  readonly children?: readonly string[];
};

/**
 Converts a camelCase CSS property name to kebab-case for inline style attributes.
 
 @param property - camelCase CSS property name
 
 @returns kebab-case CSS property name
 
 @example
 ```ts
 camelToKebab('flexDirection') // 'flex-direction'
 camelToKebab('backgroundColor') // 'background-color'
 camelToKebab('display') // 'display'
 ```
 */
function camelToKebab(property: string,): string {
  /**
   Per-character kebab fragments, joined once at the end so the accumulator is never rebuilt each step (single linear pass: O(n) time, O(1) stack, no recursion).
   */
  const fragments: string[] = [];

  for (const c of property) {
    /**
     True when `c` is an ASCII uppercase letter (`A`-`Z`).
     */
    const isUpper = (c >= 'A') && (c <= 'Z');
    fragments.push(isUpper ? `-${c.toLowerCase()}` : c,);
  }

  return fragments.join('',);
}

/**
 Builds an HTML element string from declarative options. Text, attribute
 values, and serialized styles all pass through {@link escapeHtml}; `style`
 property names convert via {@link camelToKebab}.
 
 @param tag - HTML tag name
 
 @param text - text content (HTML-escaped automatically)
 
 @param html - raw inner HTML (NOT escaped)
 
 @param attrs - HTML attributes as key-value pairs
 
 @param style - inline CSS styles as camelCase property-value pairs
 
 @param children - child HTML strings to concatenate
 
 @returns well-formed HTML element string
 
 @example
 ```ts
 $({ tag: 'div', class: 'card', text: 'hello' });
 // '<div class="card">hello</div>'
 ```
 */
/* @__NO_SIDE_EFFECTS__ */ export function $(
  {
    tag,
    class: className,
    text,
    html,
    attrs,
    style,
    children,
  }: HOptions,
): string {
  /**
   Accumulates serialization fragments so they can be joined once at the end without intermediate string concatenations.
   */
  const parts: string[] = [`<${tag}`,];

  if (className !== undefined)
    parts.push(` class="${escapeHtml(className,)}"`,);

  if (attrs !== undefined) {
    for (const [key, value,] of Object.entries(attrs,))
      parts.push(` ${key}="${escapeHtml(value,)}"`,);
  }

  if (style !== undefined) {
    /**
     Holds the serialized inline style value so it can be HTML-escaped as a single attribute before insertion.
     */
    const declarations = Object
      .entries(style,)
      .map(function toDecl([property, value,],) {
        return `${camelToKebab(property,)}:${value}`;
      },)
      .join(';',);
    parts.push(` style="${escapeHtml(declarations,)}"`,);
  }

  parts.push('>',);

  if (VOID_ELEMENTS.has(tag,))
    return parts.join('',);

  if (text !== undefined)
    parts.push(escapeHtml(text,),);

  if (html !== undefined)
    parts.push(html,);

  if (children !== undefined) {
    for (const child of children)
      parts.push(child,);
  }

  parts.push(`</${tag}>`,);

  return parts.join('',);
}
