/**
 Internal builder functions for the CSS hyperscript factory.
 
 Separated from the main index to stay within the line budget.
 */

import type {
  AtRuleOptions,
  RuleOptions,
} from './index.types.ts';
import type { CssValue, } from './values.ts';

export type {
  AtRuleOptions,
  CssOptions,
  RuleOptions,
} from './index.types.ts';

//region Helpers

/**
 At-rule fields needed only to decide whether block syntax is present.
 */
type AtRuleBlockPresence = {
  readonly decls?: unknown;
  readonly raw?: unknown;
  readonly children?: readonly unknown[];
};

/**
 Serializes a declarations record into a CSS declaration string.
 
 Accepts any declarations object: {@link StrictCssDeclarations}, at-rule descriptor
 interfaces, or plain `Record<string, string>`. Skips `undefined` values since
 csstype interfaces mark all properties as optional.
 
 @param decls - property-value pairs (typed at the API boundary, loose here for internal flexibility)
 
 @returns semicolon-separated declarations (e.g. `'display:flex;gap:1rem'`)
 
 @example
 ```ts
 serializeDecls({ display: 'flex', gap: cssRem(1) })
 // 'display:flex;gap:1rem'
 ```
 */
function serializeDecls(
  decls: NonNullable<AtRuleOptions['decls'] | RuleOptions['decls']>,
): string {
  /**
   Accumulates property:value fragments so they can be joined with `;` once at the end.
   */
  const parts: string[] = [];

  for (const [property, value,] of Object.entries(decls,) as readonly [
    string,
    CssValue | string | number,
  ][]) {
    if (value === undefined)
      continue;

    parts.push(`${property}:${value}`,);
  }

  return parts.join(';',);
}

/**
 Checks whether an at-rule has any block content (declarations, raw, or children).
 
 Statement at-rules like `@layer tokens;` have no block. Note that empty
 `decls: {}` or empty `raw: ''` still count as block content (producing
 `@layer{}` rather than `@layer;`); see {@link AtRuleOptions} for details.
 
 @param options - at-rule options to inspect
 
 @returns `true` when the at-rule should produce a `{ }` block
 */
function hasBlock(options: AtRuleBlockPresence,): boolean {
  if (options.decls
    !== undefined)
    return true;

  if (options.raw
    !== undefined)
    return true;

  if ((options.children
    !== undefined) && (options.children
      .length
      > 0))
    return true;

  return false;
}

/**
 Renders the inside of a `{ }` block from declarations, raw, and children.
 
 Inserts a `;` separator between any two non-empty segments so declarations
 never run into following raw content or children. Empty segments
 (`decls: {}`, `raw: ''`, omitted, or empty `children`) contribute nothing
 and emit no separator.
 
 Children concatenate without separators since each child is already a complete
 rule or at-rule string; CSS tolerates the absence of separators between
 adjacent block-form rules.
 
 @param decls - serialized declarations (any object), may be `undefined`
 
 @param raw - raw CSS string, may be `undefined`
 
 @param children - already-built child CSS strings, may be `undefined`
 
 @returns block body string with proper separators (no surrounding braces)
 
 @example
 ```ts
 renderBody({ decls: { display: 'flex' }, raw: 'background:url(a)' });
 // 'display:flex;background:url(a)'
 ```
 */
function renderBody(
  {
    decls,
    raw,
    children,
  }: {
    readonly decls?: AtRuleOptions['decls'] | RuleOptions['decls'];
    readonly raw?: string;
    readonly children?: readonly string[];
  },
): string {
  /**
   Accumulates declaration and raw segments so they can be joined with `;` separators.
   */
  const parts: string[] = [];

  if (decls !== undefined) {
    /**
     Captures the joined declarations once so the empty check and push read from the same value.
     */
    const serialized = serializeDecls(decls,);
    if (serialized !== '')
      parts.push(serialized,);
  }

  if ((raw !== undefined) && (raw !== ''))
    parts.push(raw,);

  /**
   Holds the concatenated children so the empty case can be detected before composing with the rest of the body.
   */
  const childrenStr = children !== undefined
    ? children.join('',)
    : '';

  /**
   Holds the declarations-and-raw section so the join with `childrenStr` can insert a `;` only when both halves are non-empty.
   */
  const innerStr = parts.join(';',);

  if ((innerStr !== '') && (childrenStr !== ''))
    return `${innerStr};${childrenStr}`;

  return `${innerStr}${childrenStr}`;
}

//endregion

//region Builder functions

/**
 Builds a CSS style rule string. Renders the block body via {@link renderBody}.
 
 @param options - selector plus optional declarations, raw CSS, and child rules
 
 @returns CSS rule string (e.g. `'.card{display:flex}'`)
 
 @example
 ```ts
 buildRule({ rule: '.card', decls: { display: 'flex' } });
 // '.card{display:flex}'
 ```
 */
export function buildRule(
  options: RuleOptions,
): string {
  /**
   Pulls the selector out so the block body renders from the remaining option fields.
   */
  const { rule, } = options;
  return `${rule}{${
    renderBody(options,)
  }}`;
}

/**
 Builds a CSS at-rule string.
 
 Produces either a block at-rule (`@media (...) { ... }`) or a statement
 at-rule (`@layer tokens;`) depending on whether {@link hasBlock} reports
 block content; the block body itself renders via {@link renderBody}.
 
 @param options - at-rule configuration with keyword, params, and optional block content
 
 @returns CSS at-rule string
 
 @example
 ```ts
 buildAtRule({ at: 'layer', params: 'tokens' });
 // '\@layer tokens;'
 ```
 */
export function buildAtRule(
  options: AtRuleOptions,
): string {
  /**
   Pulls the at-rule options into named locals so the head and body sections can read each field directly.
   */
  const {
    at,
    params,
  } = options;
  /**
   Captures the at-rule keyword and optional params as a single prefix shared by both the statement and block forms.
   */
  const head = params !== undefined
    ? `@${at} ${params}`
    : `@${at}`;

  //region Statement at-rule
  // At-rules with no block content are statements terminated by a semicolon.
  //endregion
  if (!hasBlock(options,))
    return `${head};`;

  return `${head}{${
    renderBody(options,)
  }}`;
}

//endregion
