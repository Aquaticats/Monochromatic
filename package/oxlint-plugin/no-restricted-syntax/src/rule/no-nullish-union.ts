import type {
  CreateOnceRule,
  ESTree,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { simpleBanRule, } from './_simple-ban-rule.ts';

//region Constants

/**
 AST `type` names of the two banned nullish keyword members: `undefined`
 (`TSUndefinedKeyword`) and `null` (`TSNullKeyword`). A `Set` keeps the
 membership test a single call, sidestepping a mixed-operator `||` chain.
 `TSNullKeyword` is the `null` type keyword, distinct from the `null` literal
 node `TSNullLiteral`.
 */
const NULLISH_KEYWORD_TYPES: ReadonlySet<string> = new Set([
  'TSUndefinedKeyword',
  'TSNullKeyword',
],);

/**
 Ranked decision-tree diagnostic for nullish-union reports.
 
 The sentinel branch must satisfy the sibling
 {@link noLowInformationSymbolDescription} rule, so examples use specific
 multi-word Symbol descriptions rather than vague labels like `not-found`.
 */
const NO_NULLISH_UNION_MESSAGE: string = [
  'Union type contains `null` or `undefined`. This repo models absence without nullish unions.',
  'exactOptionalPropertyTypes is on: for optional object properties, `?:` already means absent-or-`T`.',
  'Pivoting to `null`, or sliding to `| void`, tuple-as-Maybe, falsy literals, an empty object, or',
  '`Partial<T>`, is still fake optionality (the sibling no-optional-escape rule owns those forms).',
  'Take the first branch that fits:',
  '(1) optional object property/field -> `foo?: T` (never `foo?: T | undefined`, never `foo: T | undefined`).',
  '(2) value may be absent but presence is establishable here -> `if`-guard / early return so the',
  'typed slot receives only `T`.',
  '(3) absence should fail loud at this boundary -> throw via `nonNullishOrThrow` from',
  '`@monochromatic-dev/or-throw-core`.',
  '(4) absence must travel onward as a real value -> mint a domain-specific `unique symbol` sentinel',
  'for this exact absence condition, e.g.',
  "`const KEY_NOT_FOUND: unique symbol = Symbol('requested key not found in store')`,",
  'or carry a distinct non-empty domain value; consumers narrow symbols with',
  "`typeof value === 'symbol'` first, then identity (`value === KEY_NOT_FOUND`).",
  'Heaviest ordinary fix, reach for it last.',
  '(5) genuinely mirroring an external API typed `T | null`/`T | undefined` -> scoped',
  '`oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- <reason naming the external',
  'API and why the mirror is unavoidable>`.',
  'Never use `null`, `undefined`, empty string, zero, negative one, `false`, `void`, empty tuple,',
  'empty object, or `Partial` as a stand-in for absent.',
].join('\n',);

//endregion Constants

//region Predicates

/**
 Checks whether a union member is the `undefined` or `null` type keyword.
 
 @param member - union member to test
 
 @returns whether member is a nullish keyword type
 
 @example
 ```ts
 isNullishMember(member); // true for the `undefined` in `string | undefined`
 ```
 */
function isNullishMember(member: ForeignBorrowed<ESTree.TSType>,): boolean {
  return NULLISH_KEYWORD_TYPES.has(member.type,);
}

//endregion Predicates

/**
 Bans union types containing `null` or `undefined` (`T | undefined`,
 `undefined | T`, `T | null`, `null | T`, or either nullish keyword anywhere
 in a union).
 
 `tsconfig` sets `exactOptionalPropertyTypes: true`. Widening a slot to
 `T | undefined` skirts that setting instead of fixing the real problem: it
 lets `undefined` flow into a typed position the optional-property machinery
 was meant to keep absent. Pivoting that same slot to `T | null` is not a fix;
 it is the identical nullish escape with a different keyword.
 
 Take the first branch that fits:
 
 - Optional object property or field: write `foo?: T`, never
   `foo?: T | undefined` and never `foo: T | undefined`.
 - Value whose presence is establishable here: guard with `if` and return
   early so the typed slot receives only `T`.
 - Absence that should fail loud at this boundary: throw via
   {@link nonNullishOrThrow} from `@monochromatic-dev/or-throw-core`.
 - Absence that must travel onward as a real value: mint a domain-specific
   `unique symbol` sentinel for this exact absence condition, or carry a
   distinct non-empty domain value when the domain has one. Consumers narrow
   symbols by checking `typeof` first, then identity.
 - Genuine external API mirrors: use a scoped
   `oxlint-disable-next-line no-restricted-syntax/no-nullish-union` comment
   with a justification naming the external API and why the mirror is
   unavoidable.
 
 This rule only matches `TSUndefinedKeyword` and `TSNullKeyword` inside
 `TSUnionType`; the sibling {@link noOptionalEscape} rule owns `| void`,
 tuple encodings, `Partial<T>`, and other type-level fake-optionality
 escapes. A standalone `type X = undefined` or `type X = null` is not a
 union and is not flagged.
 
 @example
 ```ts
 // Bad
 let x: number | undefined;
 let y: number | null;
 type Opt = { foo?: string | undefined; };
 function find(): string | null {}
 function take(x: number | undefined): void {}
 const p: Promise<number | null> = load();
 
 // Good
 type Opt = { foo?: string; };
 const value = lookup(key);
 if (value === undefined) {
   return;
 }
 // value is now `T`, never `T | undefined`
 
 // Good; genuine Symbol sentinel instead of the union
 const KEY_NOT_FOUND: unique symbol = Symbol('requested key not found in store');
 type Result = string | typeof KEY_NOT_FOUND;
 ```
 */
export const noNullishUnion: CreateOnceRule = simpleBanRule({
  type: 'suggestion',
  nodeType: 'TSUnionType',
  description:
    'Disallow union types containing `null` or `undefined` (`T | null`, `T | undefined`). Ranked fixes: optional object property/field `foo?: T`; local `if` guard and early return; boundary `nonNullishOrThrow`; a domain-specific `unique symbol` sentinel for this exact absence condition or distinct non-empty domain value; or a justified scoped disable for genuine external API mirrors.',
  messageId: 'forbidden',
  message: NO_NULLISH_UNION_MESSAGE,
  shouldReport(node: ForeignBorrowed<ESTree.Node>,): boolean {
    if (node.type !== 'TSUnionType')
      return false;
    return node.types
      .some(isNullishMember,);
  },
},);
