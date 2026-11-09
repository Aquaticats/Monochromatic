/**
 AST-based extraction of import attribute types.
 
 Extracts `type` values from import attributes on static import/export
 declarations and from dynamic import options, using the parsed AST
 instead of fragile regex patterns.
 
 OXC's runtime AST uses `type: "Literal"` for all literals while the
 `@oxc-project/types` declarations use `"StringLiteral"`. This module
 uses `typeof value === 'string'` checks for safe narrowing.
 
 @module
 */


import type { ESTree, } from 'rolldown/utils';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { HANDLERS, } from './handlers.ts';

//region Sentinels

/**
 Sentinel returned by {@link getStringLiteralValue} and
 {@link getPropertyKeyName} when an AST node carries no usable string value
 (a computed key, or a non-string-literal expression). A `unique symbol` so
 an empty string can never be mistaken for "not a string"; callers narrow
 with `=== NON_STRING_NODE`.
 */
export const NON_STRING_NODE: unique symbol = Symbol('import-attributes/non-string-node',);

/**
 Sentinel returned by {@link extractTypeFromAttributes} and
 {@link extractTypeFromOptions} when no supported `type` import attribute is
 present. A `unique symbol` so it can never collide with a real handler-type
 string; callers narrow with `=== NO_ATTR_TYPE`.
 */
export const NO_ATTR_TYPE: unique symbol = Symbol('import-attributes/no-attr-type',);

//endregion Sentinels

//region Helpers

/**
 Extracts a string property key name from an AST property key node.
 Handles both identifier keys and string literal keys.
 
 @param key - AST property key node
 
 @returns key name as string, or {@link NON_STRING_NODE} for computed/non-string keys
 */
function getPropertyKeyName(
  key: ForeignBorrowed<ESTree.PropertyKey>,
): string | typeof NON_STRING_NODE {
  if (key.type
    === 'Identifier')
    return key.name;
  if (('value' in key) && ((typeof key.value) === 'string'))
    return key.value;
  return NON_STRING_NODE;
}

/**
 Extracts a string value from an AST expression node if it is a string literal.
 
 @param node - AST expression node
 
 @returns string value if the node is a string literal, {@link NON_STRING_NODE} otherwise
 
 @example
 ```ts
 // Given AST node for string literal "text"
 getStringLiteralValue(stringNode); // "text"
 getStringLiteralValue(identifierNode); // NON_STRING_NODE
 ```
 */
export function getStringLiteralValue(
  node: ForeignBorrowed<ESTree.Expression>,
): string | typeof NON_STRING_NODE {
  if (('value' in node) && ((typeof node.value) === 'string'))
    return node.value as string;
  return NON_STRING_NODE;
}

//endregion Helpers

/**
 Extracts the attribute type value from a static import/export declaration's
 `attributes` array.
 
 Looks for an attribute with key `type` whose value is a supported handler
 type registered in {@link HANDLERS}.
 
 @param attributes - import attribute nodes from the AST
 
 @returns supported attribute type string, or {@link NO_ATTR_TYPE} if none found
 
 @example
 ```ts
 // For: import x from './file.sql' with { type: 'text' }
 extractTypeFromAttributes(node.attributes); // 'text'
 ```
 */
export function extractTypeFromAttributes(
  attributes: ForeignBorrowed<readonly ESTree.ImportAttribute[]>,
): string | typeof NO_ATTR_TYPE {
  for (const attr of attributes) {
    /**
     Resolved attribute key name covering both identifier and string-literal AST forms.
     */
    const key = attr.key
      .type
      === 'Identifier'
      ? attr.key
        .name
      : attr.key
        .value;
    if ((key === 'type') && (HANDLERS[attr.value
      .value]
      !== undefined))
      return attr.value
        .value;
  }
  return NO_ATTR_TYPE;
}

/**
 Extracts the attribute type of one static import or export declaration.
 
 This owns the shape every static-declaration visitor in this package repeats:
 a declaration carrying no attributes has no type, and a set whose `type` value
 is not a registered handler has none either. A visitor therefore reads
 {@link NO_ATTR_TYPE} once, from here, instead of re-testing both conditions.
 
 @param attributes - import attribute nodes of a static import or export declaration
 
 @returns supported attribute type string, or {@link NO_ATTR_TYPE} when the declaration carries none
 
 @example
 ```ts
 // For: import x from './file.sql' with { type: 'text' }
 staticDeclAttrType(node.attributes); // 'text'
 // For: import x from './file.ts'
 staticDeclAttrType(node.attributes); // NO_ATTR_TYPE
 ```
 */
export function staticDeclAttrType(
  attributes: ForeignBorrowed<readonly ESTree.ImportAttribute[]>,
): string | typeof NO_ATTR_TYPE {
  return attributes.length
    === 0
    ? NO_ATTR_TYPE
    : extractTypeFromAttributes(attributes,);
}

/**
 Extracts the attribute type from a dynamic import's options expression.
 
 Handles the `{ with: { type: '...' } }` pattern used in dynamic `import()`,
 accepting only types registered in {@link HANDLERS}.
 
 @param options - options expression from `ImportExpression.options`
 
 @returns supported attribute type string, or {@link NO_ATTR_TYPE} if the options
 do not contain a recognized `with.type` value
 
 @example
 ```ts
 // For: import('./file.sql', { with: { type: 'text' } })
 extractTypeFromOptions(node.options); // 'text'
 ```
 */
export function extractTypeFromOptions(
  options: ForeignBorrowed<ESTree.Expression>,
): string | typeof NO_ATTR_TYPE {
  if (options.type
    !== 'ObjectExpression')
    return NO_ATTR_TYPE;

  for (const prop of options.properties) {
    if (prop.type
      !== 'Property')
      continue;

    /**
     Outer property name; only the `with` clause is relevant.
     */
    const key = getPropertyKeyName(prop.key,);
    if ((key !== 'with') || (prop.value
      .type
      !== 'ObjectExpression'))
      continue;

    for (const innerProp of prop.value
      .properties) {
      if (innerProp.type
        !== 'Property')
        continue;

      /**
       Inner property name; narrowed to `type` below.
       */
      const innerKey = getPropertyKeyName(innerProp.key,);
      /**
       String value paired with the `type` entry.
       */
      const innerValue = getStringLiteralValue(innerProp.value,);
      if ((innerKey === 'type')
        && (innerValue !== NON_STRING_NODE)
        && (HANDLERS[innerValue]
          !== undefined))
        {
        return innerValue;
      }
    }
  }

  return NO_ATTR_TYPE;
}
