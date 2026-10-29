import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Sentinel returned by every classifier below when a node matches no banned
 fake-optional form. A unique `Symbol` keeps the "no match" signal out of a
 `string | undefined` union (which {@link noNullishUnion} bans) and
 dogfoods the real-sentinel pattern this rule prescribes as the fix for
 fake optionality.
 */
const NO_MATCH = Symbol('node matches no banned fake-optional form',);

/**
 Keyword `type` names whose presence in a union widens the slot to accept
 everything (including nullish): `T | unknown` and `T | any` both collapse to
 the wide type. A `Set` keeps the membership test a single call.
 */
const WIDENING_KEYWORD_TYPES: ReadonlySet<string> = new Set([
  'TSUnknownKeyword',
  'TSAnyKeyword',
],);

/**
 Utility-type names that produce an empty object when their second type
 argument is `never`: `Record<K, never>` and `Pick<T, never>`.
 */
const EMPTY_OBJECT_UTILITIES: ReadonlySet<string> = new Set([
  'Record',
  'Pick',
],);

/**
 Allowed-alternative guidance appended to every diagnostic message. Names the
 four sanctioned ways to express "may be absent" without a type-level escape.
 */
const ALLOWED = 'exactOptionalPropertyTypes is on; faking optionality at the type level is banned. Use `foo?: T` for an optional property; an `if`-guard so the value is always present where typed; throw via `nonNullishOrThrow` (@monochromatic-dev/or-throw-core); or a real sentinel: a unique `Symbol`, or a distinct non-empty domain value. Never null, undefined, the empty string, zero, negative one, `false`, `void`, an empty tuple, an empty object, or `Partial`. Genuine external-boundary mirrors use a scoped `oxlint-disable-next-line no-restricted-syntax/no-optional-escape` with a justification.';

//endregion Constants

//region Predicates

/**
 Checks whether a union member is not a literal type.
 
 Falsy-literal sentinels (`T | 0`, `T | ""`) are flagged only when a union has
 such a member, so a pure literal domain (`0 | 1 | 2`, `"a" | "b"`) is left
 alone: there the falsy literal is a genuine member of the finite domain, not
 a sentinel bolted onto a wider type.
 
 @param member - union member to test
 
 @returns whether member is not a `TSLiteralType`
 
 @example
 ```ts
 isNonLiteralMember(member); // true for `string`, false for `0`
 ```
 */
function isNonLiteralMember(member: ForeignBorrowed<ESTree.TSType>,): boolean {
  return member.type
    !== 'TSLiteralType';
}

/**
 Checks whether a tuple element is an optional element (`T?`).
 
 @param element - tuple element to test
 
 @returns whether element is a `TSOptionalType`
 
 @example
 ```ts
 isOptionalTupleElement(element); // true for the `T?` in `[T?]`
 ```
 */
function isOptionalTupleElement(element: ForeignBorrowed<ESTree.TSTupleElement>,): boolean {
  return element.type
    === 'TSOptionalType';
}

//endregion Predicates

//region Classifiers

/**
 Classifies a literal-type union member as a falsy sentinel form.
 
 Covers the empty string (`""`), an empty template literal type, numeric zero
 (`0`), a negative numeric literal (`-1`), and `false`. A non-empty literal
 (`42`, `"pending"`) is a real domain value and returns {@link NO_MATCH}.
 
 @param member - literal-type union member to classify
 
 @returns message id for the matched falsy form, or {@link NO_MATCH}
 
 @example
 ```ts
 falsyLiteralMessageId(member); // 'emptyStringUnion' for `T | ""`
 ```
 */
function falsyLiteralMessageId(member: ForeignBorrowed<ESTree.TSLiteralType>,): string | typeof NO_MATCH {
  /**
   Literal node carried by this union member.
   */
  const { literal, } = member;
  if (literal.type
    === 'Literal') {
    if (literal.value
      === '') {
      return 'emptyStringUnion';
    }
    if (literal.value
      === 0) {
      return 'falsyNumberUnion';
    }
    if (literal.value
      === false) {
      return 'falseUnion';
    }
    return NO_MATCH;
  }
  if (literal.type
    === 'TemplateLiteral') {
    /**
     First template quasi, present on every template literal.
     */
    const [firstQuasi,] = literal.quasis;
    if (firstQuasi === undefined) {
      return NO_MATCH;
    }
    if (literal.quasis
      .length
      !== 1) {
      return NO_MATCH;
    }
    if (firstQuasi.value
      .cooked
      === '') {
      return 'emptyStringUnion';
    }
    return NO_MATCH;
  }
  if (literal.type
    === 'UnaryExpression') {
    if (literal.operator
      !== '-') {
      return NO_MATCH;
    }
    if (literal.argument
      .type
      !== 'Literal') {
      return NO_MATCH;
    }
    if ((typeof literal.argument
      .value) === 'number') {
      return 'falsyNumberUnion';
    }
    return NO_MATCH;
  }
  return NO_MATCH;
}

/**
 Classifies a single union member as a fake-optional escape.
 
 Handles the keyword widening forms (`void`, `never`, `unknown`, `any`) and
 the empty-object form (`{}`) directly; delegates falsy literals to
 {@link falsyLiteralMessageId}, gated on `hasNonLiteral`. `undefined` and
 `null` are intentionally not matched here: {@link noNullishUnion} owns them.
 
 @param member - union member to classify
 
 @param hasNonLiteral - whether the union has a non-literal member
 
 @returns message id for the matched form, or {@link NO_MATCH}
 
 @example
 ```ts
 unionMemberMessageId({ member, hasNonLiteral }); // 'voidUnion' for `T | void`
 ```
 */
function unionMemberMessageId(
  {
    member,
    hasNonLiteral,
  }: ForeignBorrowed<{
    readonly member: ESTree.TSType;
    readonly hasNonLiteral: boolean;
  }>,
): string | typeof NO_MATCH {
  if (member.type
    === 'TSVoidKeyword') {
    return 'voidUnion';
  }
  if (member.type
    === 'TSNeverKeyword') {
    return 'neverUnion';
  }
  if (WIDENING_KEYWORD_TYPES.has(member.type,)) {
    return 'wideningUnion';
  }
  if (member.type
    === 'TSTypeLiteral') {
    if (member.members
      .length
      === 0) {
      return 'emptyObjectUnion';
    }
    return NO_MATCH;
  }
  if (member.type
    === 'TSLiteralType') {
    if (hasNonLiteral) {
      return falsyLiteralMessageId(member,);
    }
    return NO_MATCH;
  }
  return NO_MATCH;
}

/**
 Classifies a tuple type as a fake-optional escape.
 
 Flags an empty tuple (`[]`), a tuple with an optional element (`[T?]`), and
 a rest-only tuple (`[...T[]]`, functionally `T[]` dressed as 0-or-many). A
 fixed non-empty tuple (`[number, string]`) and a leading-element variadic
 tuple (`[T, ...U[]]`) are legitimate and return {@link NO_MATCH}. An optional
 named member (`[foo?: T]`) is handled by the `TSNamedTupleMember` visitor.
 
 @param node - tuple type to classify
 
 @returns message id for the matched form, or {@link NO_MATCH}
 
 @example
 ```ts
 tupleMessageId(node); // 'emptyTuple' for `[]`
 ```
 */
function tupleMessageId(node: ForeignBorrowed<ESTree.TSTupleType>,): string | typeof NO_MATCH {
  if (node.elementTypes
    .length
    === 0) {
    return 'emptyTuple';
  }
  if (node.elementTypes
    .some(isOptionalTupleElement,)) {
    return 'optionalTupleElement';
  }
  if (node.elementTypes
    .length
    !== 1) {
    return NO_MATCH;
  }
  /**
   Sole tuple element, checked for the rest-only `[...T[]]` shape.
   */
  const [firstElement,] = node.elementTypes;
  if (firstElement === undefined) {
    return NO_MATCH;
  }
  if (firstElement.type
    === 'TSRestType') {
    return 'restOnlyTuple';
  }
  return NO_MATCH;
}

/**
 Classifies a type reference as a fake-optional escape.
 
 Flags `Partial<T>` (makes every property optional) and the empty-object
 utility forms `Record<K, never>` and `Pick<T, never>` (second type argument
 `never`). A real `Record<K, V>` or `Pick<T, K>` returns {@link NO_MATCH}.
 
 @param node - type reference to classify
 
 @returns message id for the matched form, or {@link NO_MATCH}
 
 @example
 ```ts
 typeReferenceMessageId(node); // 'partial' for `Partial<T>`
 ```
 */
function typeReferenceMessageId(node: ForeignBorrowed<ESTree.TSTypeReference>,): string | typeof NO_MATCH {
  if (node.typeName
    .type
    !== 'Identifier') {
    return NO_MATCH;
  }
  /**
   Referenced type name, e.g. `Partial`, `Record`, `Pick`.
   */
  const { name, } = node.typeName;
  if (name === 'Partial') {
    return 'partial';
  }
  if (!EMPTY_OBJECT_UTILITIES.has(name,)) {
    return NO_MATCH;
  }
  /**
   Type arguments supplied to the reference, if any.
   */
  const { typeArguments, } = node;
  if (typeArguments === null) {
    return NO_MATCH;
  }
  /**
   Second type argument; `never` here marks an empty-object utility type.
   */
  const [, secondArg,] = typeArguments.params;
  if (secondArg === undefined) {
    return NO_MATCH;
  }
  if (secondArg.type
    === 'TSNeverKeyword') {
    return 'emptyUtilityObject';
  }
  return NO_MATCH;
}

//endregion Classifiers

/**
 Bans every statically-detectable type-level encoding of "optional / absent /
 empty-as-absent" except the two nullish keywords (`undefined`, `null`), which
 {@link noNullishUnion} already owns.
 
 `tsconfig` sets `exactOptionalPropertyTypes: true`. Agents repeatedly invent
 new encodings to dodge that setting: once `| undefined` and `| null` were
 banned, the next dodge was `| void`, then tuple-as-Maybe (`[]`, `[T?]`), then
 literal sentinels in unions (`T | ""`, `T | 0`), then `Partial<T>`. This rule
 enumerates and bans the whole detectable space in one pass.
 
 Banned forms (each its own diagnostic):
 
 - Union members: `void`, `never`, `unknown`/`any` (widening dodge: the union
   collapses to the wide type), an empty object `{}`, and falsy literals
   (`""`, an empty template literal type, `0`, a negative number, `false`).
   Falsy literals are flagged only when the union also has a non-literal
   member, so a finite literal domain like `0 | 1 | 2` is left alone.
 - Tuples: empty `[]`, optional element `[T?]`, optional named member
   `[foo?: T]`, rest-only `[...T[]]`.
 - Type references: `Partial<T>`, `Record<K, never>`, `Pick<T, never>`.
 - Mapped types that add optionality: `{ [K in keyof T]?: ... }` (a hand-rolled
   `Partial`). The `Required` form `{ [K in keyof T]-?: ... }` is not flagged.
 
 Allowed (not flagged): a bare `(): void` return (only `void` inside a union
 is banned); `T | null` / `T | undefined` (owned by {@link noNullishUnion});
 a fixed non-empty tuple (`[number, string]`); a leading-element variadic
 tuple (`[T, ...U[]]`); a real `Symbol` sentinel via `typeof MY_SYMBOL`; a
 non-empty literal member (`T | 42`, `T | "pending"`); a real
 `Record<K, V>` / `Pick`.
 
 Statically undetectable (documented blind spots, addressed by review): a
 field honestly typed `string` but defaulted to `""` at runtime; a `T[]` whose
 emptiness encodes absence; `0`/`-1` used as absent on a plain `number`. In all
 three the type annotation itself is honest and carries no syntactic marker, so
 there is nothing to detect. `T | typeof CONST` where `CONST` resolves to a
 falsy literal is detectable only by resolving the binding's value: the
 `typeof` node is identical whether `CONST` is a real `Symbol` or an empty
 string, so the dodge is indistinguishable at the type node without scope or
 type-checker analysis the JS plugin lacks.
 
 @example
 ```ts
 // Bad
 type A = string | void;
 type B = string | "";
 type C = [string?];
 type D = Partial<{ a: string }>;
 
 // Good
 type Opt = { foo?: string };
 const NOT_FOUND = Symbol('requested key not found in store');
 type Result = string | typeof NOT_FOUND;
 type Pair = [number, string];
 ```
 */
export const noOptionalEscape: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description:
        'Disallow every statically-detectable type-level encoding of fake optionality except `| null`/`| undefined` (owned by no-nullish-union): `| void`, `| never`, `| unknown`/`| any`, falsy literal members (`""`, `0`, `-1`, `false`), `| {}`, empty/optional/rest-only tuples, `Partial<T>`, `Record<K, never>`, and added-optionality mapped types.',
      recommended: true,
    },
    messages: {
      voidUnion: `\`T | void\` widens a slot to also accept \`undefined\` (\`void\` is assignable from \`undefined\`). ${  ALLOWED}`,
      neverUnion: `\`T | never\` collapses to \`T\`; a hand-written \`| never\` is a stubbed-out absence branch masquerading as a type. ${  ALLOWED}`,
      wideningUnion: `\`T | unknown\` and \`T | any\` collapse to \`unknown\`/\`any\`, widening the slot to accept everything (including nullish) and defeating the type. ${  ALLOWED}`,
      emptyStringUnion: `An empty-string literal in a union (\`T | ""\`, or an empty template literal type) is a falsy sentinel encoding absence. ${  ALLOWED}`,
      falsyNumberUnion: `A zero or negative numeric literal in a union (\`T | 0\`, \`T | -1\`) is a falsy/not-found sentinel encoding absence. ${  ALLOWED}`,
      falseUnion: `\`T | false\` uses \`false\` as an off/absent sentinel. ${  ALLOWED}`,
      emptyObjectUnion: `An empty object type in a union (\`T | {}\`) widens the slot to any non-nullish value, encoding "anything or nothing". ${  ALLOWED}`,
      emptyTuple: `An empty tuple type \`[]\` encodes absence as a zero-length tuple. ${  ALLOWED}`,
      optionalTupleElement: `An optional tuple element (\`[T?]\`) encodes a possibly-absent slot. ${  ALLOWED}`,
      namedOptionalTupleMember: `An optional named tuple member (\`[foo?: T]\`) encodes a possibly-absent slot. ${  ALLOWED}`,
      restOnlyTuple: `A rest-only tuple (\`[...T[]]\`) is \`T[]\` dressed as 0-or-many; its empty case encodes absence. Use \`T[]\`, or \`[T, ...T[]]\` for one-or-more. ${  ALLOWED}`,
      partial: `\`Partial<T>\` makes every property optional, reopening the holes exactOptionalPropertyTypes closes. Mark individual properties \`?:\` only where genuinely optional. ${  ALLOWED}`,
      emptyUtilityObject: `A utility type producing an empty object (\`Record<K, never>\`, \`Pick<T, never>\`) encodes "no value". ${  ALLOWED}`,
      mappedOptional: `A mapped type that adds optionality (\`{ [K in keyof T]?: ... }\`) is a hand-rolled \`Partial\`, reopening the holes exactOptionalPropertyTypes closes. ${  ALLOWED}`,
    },
  },
  /**
   Handles foreign Oxlint callback.
   
   @param context - Foreign rule context receiving diagnostics.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    return {
      TSUnionType(node: ForeignBorrowed<ESTree.TSUnionType>,): void {
        /**
         Whether the union has a non-literal member, gating falsy-literal sentinels.
         */
        const hasNonLiteral = node.types
          .some(isNonLiteralMember,);
        for (const member of node.types) {
          /**
           Matched fake-optional form for this member, or the no-match sentinel.
           */
          const messageId = unionMemberMessageId({
            member,
            hasNonLiteral,
          },);
          if (messageId !== NO_MATCH) {
            context.report({
              node: member,
              messageId,
            },);
          }
        }
      },
      TSTupleType(node: ForeignBorrowed<ESTree.TSTupleType>,): void {
        /**
         Matched fake-optional tuple form, or the no-match sentinel.
         */
        const messageId = tupleMessageId(node,);
        if (messageId !== NO_MATCH) {
          context.report({
            node,
            messageId,
          },);
        }
      },
      TSNamedTupleMember(node: ForeignBorrowed<ESTree.TSNamedTupleMember>,): void {
        if (node.optional) {
          context.report({
            node,
            messageId: 'namedOptionalTupleMember',
          },);
        }
      },
      TSTypeReference(node: ForeignBorrowed<ESTree.TSTypeReference>,): void {
        /**
         Matched fake-optional utility form, or the no-match sentinel.
         */
        const messageId = typeReferenceMessageId(node,);
        if (messageId !== NO_MATCH) {
          context.report({
            node,
            messageId,
          },);
        }
      },
      TSMappedType(node: ForeignBorrowed<ESTree.TSMappedType>,): void {
        if (node.optional
          === true) {
          context.report({
            node,
            messageId: 'mappedOptional',
          },);
          return;
        }
        if (node.optional
          === '+') {
          context.report({
            node,
            messageId: 'mappedOptional',
          },);
        }
      },
    };
  },
};
