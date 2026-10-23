//region Lexical helpers of the source scan
// What the scan of a source file's text (`claude-schema-code.test-fixture.ts`, `claude-schema-member.test-fixture.ts`
// and `claude-schema-import.test-fixture.ts`) shares: what a name is, where a line is, and what a value that holds a
// function looks like at its start.

/**
 What `indexOf` answers when the text it looks for is not there.
 */
export const NOT_FOUND = -1;

/**
 The characters a name can hold, so a name found inside a longer name is no use of it.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 How many characters of a text are read for a name, which no name in the family's source outgrows.
 */
const NAME_LIMIT = 120;

/**
 Whether a character can be part of a name.

 @param character - one character, empty at the edge of a line

 @returns Whether a name can hold it

 @example
 ```ts
 const holds = isIdentifierCharacter('a');
 // => true
 ```
 */
export function isIdentifierCharacter(character: string,): boolean {
  return (character !== '') && IDENTIFIER_CHARACTERS.includes(character,);
}

/**
 The name a text starts with, empty when it starts with a character no name holds.

 @param text - text read from its start

 @returns Leading characters that a name can hold

 @example
 ```ts
 const name = leadingNameOf({ text: 'build({});', },);
 // => 'build'
 ```
 */
export function leadingNameOf({ text, }: { readonly text: string; },): string {
  /**
   Where the first character no name holds stands, among the first characters of the text.
   */
  const end = Array.from(text.slice(
    0,
    NAME_LIMIT,
  ),)
    .findIndex(function endsName(character,): boolean {
      return !isIdentifierCharacter(character,);
    },);
  return (end === NOT_FOUND) ? text.slice(
    0,
    NAME_LIMIT,
  ) : text.slice(
    0,
    end,
  );
}

/**
 The positions of the lines that satisfy a test.

 @param lines - lines searched

 @param test - what a line must satisfy

 @returns Positions, in order

 @example
 ```ts
 const positions = positionsWhere({ lines: ['a', 'b', 'a',], test: (line,) => line === 'a', },);
 // => [0, 2]
 ```
 */
export function positionsWhere(
  {
    lines,
    test,
  }: {
    readonly lines: readonly string[];
    readonly test: (line: string,) => boolean;
  },
): readonly number[] {
  return lines.flatMap(function positionOf(
    line,
    index,
  ): readonly number[] {
    return test(line,) ? [index,] : [];
  },);
}

/**
 Whether a value starts the way a function does: `function`, a parameter list, type parameters, or a lone parameter
 and the arrow, any of them after `async`, or a call whose first argument starts the same way (`memoize((a) => ...)`).
 A parameter list or type parameters count only when the value holds an arrow somewhere.

 @param value - text after the `=` of a constant, or after the `:` of an object's property

 @returns Whether the value starts as a function does

 @example
 ```ts
 const starts = startsAsFunctionValue({ value: 'memoize((a: string,) => ({ a, }));', },);
 // => true
 ```
 */
export function startsAsFunctionValue({ value, }: { readonly value: string; },): boolean {
  /**
   The word the value starts with, empty when it starts with a character no name holds.
   */
  const word = leadingNameOf({ text: value, },);
  /**
   What follows that word.
   */
  const afterWord = value.slice(word.length,)
    .trimStart();
  /**
   What follows the parenthesis that opens a call's arguments, when the word is a call's.
   */
  const firstArgument = afterWord.slice(1,)
    .trimStart();
  if (word === 'function')
    return true;
  if (word === 'async')
    return startsAsFunctionValue({ value: afterWord, },);
  if (word !== '')
    return afterWord.startsWith('=>',)
      || (afterWord.startsWith('(',) && startsAsFunctionValue({ value: firstArgument, },));
  return (value.startsWith('(',) || value.startsWith('<',)) && value.includes('=>',);
}

//endregion Lexical helpers of the source scan
