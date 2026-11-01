import { namedUnderKey, } from './reask-role-call.test-fixture.ts';
import {
  afterBracket,
  propertiesOf,
  type Token,
  tokensOf,
} from './reask-role-tokens.test-fixture.ts';

//region Re-ask role in a literal
// WHAT AN OBJECT LITERAL NAMES UNDER A KEY WHEREVER IT IS WRITTEN, read from the source text (ledger B1163). A seat for the
// re-ask of a reasoning runaway is built in place (`reaskSeat: { ... }`) or in a variable (`const seat: ReaskSeat = { ... }`),
// and the mode it names decides whose answer the re-ask may give. This reads the keys of each such literal with the reader
// of a call's argument (`namedUnderKey`), so a seat built for a new caller cannot leave the mode out unseen.

/**
 Tells whether the tokens from a position are the given texts, in order.

 @param tokens - tokens of the file

 @param at - position of the first text

 @param texts - texts the tokens must have, one after another

 @returns True when every token from `at` has the text at the same offset

 @example
 ```ts
 const opens = startsWith({ tokens, at: 4, texts: ['reaskSeat', ':', '{',], },);
 ```
 */
function startsWith(
  {
    tokens,
    at,
    texts,
  }: {
    readonly tokens: readonly Token[];
    readonly at: number;
    readonly texts: readonly string[];
  },
): boolean {
  return texts.every(function matches(
    text,
    offset,
  ): boolean {
    return tokens[at + offset]
      ?.text
      === text;
  },);
}

/**
 Reads the keys of the object literal whose opening brace stands at a position.

 @param tokens - tokens of the file

 @param open - position of the opening brace

 @param key - key whose value is read

 @returns What the literal names under the key, as {@link namedUnderKey} reads it

 @example
 ```ts
 const named = namedInLiteralAt({ tokens, open: 7, key: 'reask', },);
 ```
 */
function namedInLiteralAt(
  {
    tokens,
    open,
    key,
  }: {
    readonly tokens: readonly Token[];
    readonly open: number;
    readonly key: string;
  },
): string {
  /**
   Index one past the closing brace of the literal.
   */
  const after = afterBracket({
    tokens,
    open,
    angled: false,
  },);
  return namedUnderKey({
    properties: propertiesOf({
      body: tokens.slice(
        open + 1,
        after - 1,
      ),
    },),
    key,
  },);
}

/**
 Reads what every object literal written under a property name names under a key, for a property such as
 `reaskSeat: { ... }` that builds a value in place wherever it is passed.

 @param source - text of the file

 @param holder - name of the property whose object literal is read

 @param key - key of the literal that is read

 @returns One entry per literal, in source order, as {@link namedUnderKey} reads a key: the value named, `none` where the
 literal names none, `only a spread` where it is a spread alone; empty where the property is written with no literal

 @example
 ```ts
 expect(keysNamedInLiteralsUnder({ source, holder: 'reaskSeat', key: 'reask', },),).toEqual(['reask',],);
 ```
 */
export function keysNamedInLiteralsUnder(
  {
    source,
    holder,
    key,
  }: {
    readonly source: string;
    readonly holder: string;
    readonly key: string;
  },
): readonly string[] {
  /**
   Tokens of the file.
   */
  const tokens = tokensOf({ source, },);
  return tokens.flatMap(function literalAt(
    token,
    at,
  ): readonly string[] {
    if ((token.kind !== 'word') || (!startsWith({
      tokens,
      at,
      texts: [
        holder,
        ':',
        '{',
      ],
    },)))
      return [];
    return [
      namedInLiteralAt({
        tokens,
        open: at + 2,
        key,
      },),
    ];
  },);
}

/**
 Reads what every object literal assigned to a variable of a type names under a key, for a value built in a variable
 before it is handed on, `const seat: ReaskSeat = { ... }`.

 @param source - text of the file

 @param type - name of the type the variable is annotated with

 @param key - key of the literal that is read

 @returns One entry per literal, in source order, as {@link keysNamedInLiteralsUnder} reads a key; empty where no literal
 is assigned to a variable of the type

 @example
 ```ts
 expect(keysNamedInLiteralsOfType({ source, type: 'ReaskSeat', key: 'reask', },),).toEqual(['none',]);
 ```
 */
export function keysNamedInLiteralsOfType(
  {
    source,
    type,
    key,
  }: {
    readonly source: string;
    readonly type: string;
    readonly key: string;
  },
): readonly string[] {
  /**
   Tokens of the file.
   */
  const tokens = tokensOf({ source, },);
  return tokens.flatMap(function literalAt(
    token,
    at,
  ): readonly string[] {
    if ((token.kind !== 'word') || (!startsWith({
      tokens,
      at: at - 1,
      texts: [
        ':',
        type,
        '=',
        '{',
      ],
    },)))
      return [];
    return [
      namedInLiteralAt({
        tokens,
        open: at + 2,
        key,
      },),
    ];
  },);
}

//endregion Re-ask role in a literal
