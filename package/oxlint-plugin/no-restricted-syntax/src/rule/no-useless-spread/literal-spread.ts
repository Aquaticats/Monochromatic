/**
 Spreads of array and object literals into a list: `[a, ...[b]]`, `f(...[a])`,
 and `{ ...{ a } }`. The literal's items can be written in place, so the spread
 only builds and discards a temporary.

 @module
 */

import type {
  Context,
  ESTree,
  Fix,
  Fixer,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Literal item: array element (possibly a hole), or object property or spread.
 */
type LiteralItem = ESTree.ArrayExpressionElement | ESTree.ObjectPropertyKind;

/**
 Sentinel for a literal spread whose textual inline could change behavior or drop comments.
 */
const NO_SAFE_INLINE: unique symbol = Symbol('inlining this literal spread could change behavior or drop comments',);

/**
 Sentinel for a spread literal whose container is not an array, object, or argument list.
 */
const NOT_A_LIST_SPREAD: unique symbol = Symbol('spread literal container is not an array, object, or argument list',);

/**
 Whether text between two tokens holds only whitespace, so a rewrite drops no comment.

 @param text - Source slice to inspect.

 @returns Whether every character is whitespace or a comma.

 @example
 ```ts
 isBlankGap(' ,\n'); // true
 ```
 */
function isBlankGap(text: string,): boolean {
  return text.trim()
    .replaceAll(
    ',',
    '',
    )
    .length
    === 0;
}

/**
 Whether inlining an object literal's properties would change behavior.
 Accessors run eagerly under spread but stay accessors when inlined, and a
 `__proto__` key sets the prototype instead of copying a property.

 @param items - Properties of the spread object literal.

 @returns Whether a textual inline is unsafe.

 @example
 ```ts
 hasSpreadSensitiveProperty([getterProperty]); // true
 ```
 */
function hasSpreadSensitiveProperty(items: ForeignBorrowed<readonly LiteralItem[]>,): boolean {
  return items.some(function sensitive(item,): boolean {
    if ((item === null) || (item.type !== 'Property'))
      return false;
    if ((item.kind !== 'init') || item.method)
      return true;
    return (!item.computed) && (!item.shorthand)
      && (item.key
        .type
        === 'Identifier')
      && (item.key
        .name
        === '__proto__');
  },);
}

/**
 Builds the fix that replaces a literal spread with the literal's items, or
 removes an empty literal spread together with its separating comma.

 @param context - Rule context supplying source text.

 @param spread - Spread element wrapping the literal.

 @param literal - Array or object literal being spread.

 @param items - Items of the literal.

 @returns Fix factory, or {@link NO_SAFE_INLINE} when the rewrite could change behavior or drop comments.

 @example
 ```ts
 inlineFix({ context, spread, literal, items: literal.elements });
 ```
 */
function inlineFix(
  {
    context,
    spread,
    literal,
    items,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly spread: ESTree.SpreadElement;
    readonly literal: ESTree.ArrayExpression | ESTree.ObjectExpression;
    readonly items: readonly LiteralItem[];
  }>,
): ((fixer: Fixer) => Fix) | typeof NO_SAFE_INLINE {
  /**
   Whole source text of the linted file.
   */
  const { text, } = context.sourceCode;
  /**
   First literal item.
   */
  const [first,] = items;
  /**
   Last literal item.
   */
  const last = items.at(-1,);
  if ((first === undefined) || (last === undefined)) {
    /**
     Source following the spread up to the next comma or closing token.
     */
    const following = text.slice(spread.end,);
    /**
     Offset of the first non-whitespace character after the spread.
     */
    const nextOffset = following.length
      - following.trimStart()
        .length;
    /**
     End of the removal: through a directly following comma when present.
     */
    const removalEnd = following.charAt(nextOffset,) === ',' ? spread.end + nextOffset
      + 1 : spread.end;
    if (!isBlankGap(text.slice(
      literal.start + 1,
      literal.end - 1,
    ),))
      return NO_SAFE_INLINE;
    return function removeEmptySpread(fixer,): Fix {
      return fixer.removeRange([
        spread.start,
        removalEnd,
      ],);
    };
  }
  if (items.includes(null,) || (first === null)
    || (last === null))
    return NO_SAFE_INLINE;
  if ((!isBlankGap(text.slice(
    literal.start + 1,
    first.start,
  ),)) || (!isBlankGap(text.slice(
    last.end,
    literal.end - 1,
  ),)))
    return NO_SAFE_INLINE;
  /**
   Items' source text, keeping any comments between them.
   */
  const inlined = text.slice(
    first.start,
    last.end,
  );
  return function inlineItems(fixer,): Fix {
    return fixer.replaceText(
      spread,
      inlined,
    );
  };
}

/**
 Names the diagnostic for a literal spread into its container.

 @param literal - Spread array or object literal.

 @param container - Node holding the spread element.

 @returns Message id, or {@link NOT_A_LIST_SPREAD} when the pairing is not reported.

 @example
 ```ts
 literalSpreadMessageId({ literal, container }); // 'spreadArrayInArguments' for f(...[a])
 ```
 */
function literalSpreadMessageId(
  {
    literal,
    container,
  }: ForeignBorrowed<{
    readonly literal: ESTree.ArrayExpression | ESTree.ObjectExpression;
    readonly container: ESTree.Node;
  }>,
): 'spreadArrayInArguments' | 'spreadArrayInArray' | 'spreadObjectInObject' | typeof NOT_A_LIST_SPREAD {
  if (literal.type === 'ObjectExpression')
    return container.type === 'ObjectExpression' ? 'spreadObjectInObject' : NOT_A_LIST_SPREAD;
  if (container.type === 'ArrayExpression')
    return 'spreadArrayInArray';
  if ((container.type === 'CallExpression') || (container.type === 'NewExpression'))
    return 'spreadArrayInArguments';
  return NOT_A_LIST_SPREAD;
}

/**
 Reports an array or object literal that is spread straight into an enclosing
 array, object, or argument list.

 @param context - Rule context receiving the diagnostic.

 @param literal - Literal being visited.

 @returns Whether a diagnostic was reported, so later checks skip this literal.

 @example
 ```ts
 if (reportLiteralSpread({ context, literal: node })) return;
 ```

 @mutates context - Emits diagnostics through the foreign rule context.
 */
export function reportLiteralSpread(
  {
    context,
    literal,
  }: ForeignBorrowed<{
    readonly context: Context;
    readonly literal: ESTree.ArrayExpression | ESTree.ObjectExpression;
  }>,
): boolean {
  /**
   Spread element directly wrapping the literal.
   */
  const spread = literal.parent;
  if (spread.type !== 'SpreadElement')
    return false;
  /**
   List the spread sits in.
   */
  const container = spread.parent;
  /**
   Items the literal contributes.
   */
  const items: readonly LiteralItem[] = literal.type === 'ArrayExpression' ? literal.elements : literal.properties;
  /**
   Diagnostic kind for this literal and container pairing.
   */
  const messageId = literalSpreadMessageId({
    literal,
    container,
  },);
  if (messageId === NOT_A_LIST_SPREAD)
    return false;
  /**
   Rewrite, withheld for accessor or `__proto__` object properties.
   */
  const fix = (literal.type === 'ObjectExpression') && hasSpreadSensitiveProperty(items,)
    ? NO_SAFE_INLINE
    : inlineFix({
      context,
      spread,
      literal,
      items,
    },);
  context.report({
    node: spread,
    messageId,
    ...fix === NO_SAFE_INLINE ? {} : { fix, },
  },);
  return true;
}
