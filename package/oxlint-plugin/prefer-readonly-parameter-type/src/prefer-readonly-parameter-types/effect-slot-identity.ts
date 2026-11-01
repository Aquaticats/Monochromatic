/**
 Brands separating a parameter position from an effect slot, and the property keys slots
 are named by.
 
 Both used to be plain `number`, and an effect fact recorded against one was read against
 the other with no diagnostic anywhere. Under per-property attribution the two stop
 coinciding: a write through a destructured binding records a property slot, so a surviving
 `mutated.has(parameterIndex)` silently answers `false` and the write is lost. Losing a
 write is what offers `readonly` for state something mutates, and nothing in the type system
 or the test suite would have caught it. The brands turn every such site into a compile
 error instead.
 
 Property keys stay plain strings. A brand there would need an assertion function with
 nothing to assert, since every string is a possible property name, and a vacuous check
 reads as a proof that was never performed.
 
 @module
 */

import type { PropertyName, } from 'typescript/ast';
import {
  isComputedPropertyName,
  isIdentifier,
  isNoSubstitutionTemplateLiteral,
  isNumericLiteral,
  isPrivateIdentifier,
  isStringLiteral,
} from 'typescript/ast/is';

/**
 Declared position of one formal parameter.
 */
export type ParameterIndex = number & { readonly __brand: 'ParameterIndex'; };

/**
 One target effects are attributed to: a whole parameter, or one property of one.
 
 Slots below a callable's parameter count are its whole parameters, numbered exactly as
 parameter positions are, so a fact recorded before per-property attribution keeps its
 meaning. Slots at or above it are properties.
 */
export type EffectSlot = number & { readonly __brand: 'EffectSlot'; };

/**
 Sentinel for a property name no static key can be derived from.
 */
export const NOT_A_STATIC_KEY: unique symbol = Symbol(
  'property name resolves to no static slot key',
);

/**
 Narrows a number to a position usable as a parameter index or an effect slot.
 
 @param value - Number to check.
 
 @returns nothing after narrowing value to a usable position.
 
 @throws RangeError when value is not a nonnegative integer.
 
 @example
 ```ts
 assertPosition(0);
 ```
 */
function assertPosition(value: number,): asserts value is ParameterIndex & EffectSlot {
  if ((!Number.isInteger(value,)) || (value < 0))
    throw new RangeError(
      `Effect positions are nonnegative integers, and ${String(value,)} is not one.`,
    );
}

/**
 Marks a number as a declared parameter position.
 
 @param value - Position to brand.
 
 @returns same number, as a parameter position.
 
 @throws RangeError when value is not a nonnegative integer.
 
 @example
 ```ts
 asParameterIndex(0);
 ```
 */
export function asParameterIndex(value: number,): ParameterIndex {
  assertPosition(value,);
  return value;
}

/**
 Marks a number as an effect slot.
 
 @param value - Slot number to brand.
 
 @returns same number, as an effect slot.
 
 @throws RangeError when value is not a nonnegative integer.
 
 @example
 ```ts
 asEffectSlot(0);
 ```
 */
export function asEffectSlot(value: number,): EffectSlot {
  assertPosition(value,);
  return value;
}

/**
 Derives the canonical key one property name reads, when it reads a static one.
 
 Canonical means the spellings that name one property agree: `{ 1: x }` and `{ "1": y }`
 both read property `1`, so both have to reach the same slot or a caller writing one form
 would miss a callee written in the other. Resolved text does that, since the parser has
 already normalized the literal.
 
 A computed name gets no key. Which property it reads is a runtime question, so the binding
 widens to its whole parameter and a caller supplying one contributes to every property
 slot.
 
 A private identifier gets no key either. It names a class-private field rather than a
 property any caller can write, so no caller-side literal can ever match it.
 
 @param name - Property name from a binding element or object-literal property.
 
 @returns canonical key, or sentinel when the name is not statically known.
 
 @example
 ```ts
 canonicalPropertyKey({ name });
 ```
 */
export function canonicalPropertyKey(
  { name, }: { readonly name: PropertyName; },
): string | typeof NOT_A_STATIC_KEY {
  if (isComputedPropertyName(name,) || isPrivateIdentifier(name,))
    return NOT_A_STATIC_KEY;
  if (isIdentifier(name,)
    || isStringLiteral(name,)
    || isNumericLiteral(name,)
    || isNoSubstitutionTemplateLiteral(name,))
    return name.text;
  /* A bigint literal is the remaining `PropertyName` form. `{ 1n: x }` is not valid property
   * syntax any caller can match, so it takes no slot rather than a key that nothing reads. */
  return NOT_A_STATIC_KEY;
}
