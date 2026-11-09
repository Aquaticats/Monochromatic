/**
 What each name a file declares stands for, so a start's argument list can be read through it.

 A start spreads what the file bound under its own name (`...bounds`, `...container.limits`) or what a builder function
 in it returns (`...runtimeImageBuildArguments({ repoRoot })`), so the words that show where the limits come from sit
 one or two declarations away from the argument list.

 The index is built once per file, because a large task file is read by dozens of starts.
 */

import {
  bracketPairEnd,
  endOfQuotedString,
  isNameCharacter,
  isQuote,
  nameMatchesAt,
} from './positions-in-source-text.ts';

/**
 Keywords that declare a name this scan reads.
 */
const BINDING_KEYWORDS: readonly string[] = [
  'const',
  'let',
  'var',
];

/**
 The keyword that declares a function, whose body holds the arguments a start spreads.
 */
const FUNCTION_KEYWORD = 'function';

/**
 Characters that separate a name from an operator rather than continuing it.
 */

const SPACES = ' \t';

/**
 Newline character, which ends a statement that carries no semicolon.
 */
const NEWLINE = '\n';

/**
 A function the text declares: its name, the body that holds what a start spreading it takes, and where it ends.
 */
type DeclaredFunction = {
  /**
   The function's name, as the text writes it.
   */
  readonly name: string;
  /**
   The function's body.
   */
  readonly text: string;
  /**
   Index just past the function's closing brace.
   */
  readonly end: number;
};

/**
 Index just past the initializer of a binding whose `=` sits before an index.

 A one-line initializer ends at its semicolon or newline; a bracketed one runs over its own lines.

 @param characters - the file's characters
 
 @param from - index just after the `=`
 
 @param to - end of the range to read

 @returns the index of the first character after the initializer

 @example
 ```ts
 const end = initializerEnd({ characters, from: 12, to: characters.length });
 ```
 */
function initializerEnd({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Depth of the brackets opened so far.
   */
  let depth = 0;
  /**
   Index past the character the walk reads, which ends past the initializer.
   */
  let index = from;
  while (index < to) {
    /**
     Character at the scan's index, which opens a bracket, closes one, or ends the initializer.
     */
    const character = characters[index];
    if (isQuote(character,)) {
      index = endOfQuotedString({
        characters,
        quote: index,
        to,
      },);
      continue;
    }
    if ((character === '[') || (character === '(')
      || (character === '{'))
      depth += 1;
    else if ((character === ']') || (character === ')')
      || (character === '}')) {
      depth -= 1;
      if (depth < 0)
        return index;
    }
    else if ((depth === 0) && ((character === ';') || (character === NEWLINE)))
      return index;
    index += 1;
  }
  return index;
}

/**
 The `=` that starts the initializer of a binding, past the type annotation that may stand before it.

 A type annotation's brackets hold no assignment: what looks like one there is a function type's `=>` or a comparison,
 which the checks beside the `=` exclude.

 @param characters - the file's characters
 
 @param from - index just past the bound name

 @returns the `=`'s index when the name is being assigned here, and nothing when it is not

 @example
 ```ts
 const { assignment } = assignmentAt({ characters, from: 12 });
 ```
 */
function assignmentAt({
  characters,
  from,
}: {
  readonly characters: readonly string[];
  readonly from: number;
},): { readonly assignment?: number; } {
  for (let index = from; index < characters.length; index += 1) {
    /**
     Character at the scan's index, which starts the assignment or ends the search.
     */
    const character = characters[index];
    if (isQuote(character,)) {
      index = endOfQuotedString({
        characters,
        quote: index,
        to: characters.length,
      },) - 1;
      continue;
    }
    if ((character === '=') && (characters[index + 1] !== '>')
      && (characters[index + 1] !== '='))
      return { assignment: index, };
    if ((character === ';') || (character === NEWLINE))
      return {};
  }
  return {};
}

/**
 The function a text declares at an index, when `function` stands there.

 @param characters - the file's characters
 
 @param index - index of the `function` keyword

 @returns the name, the body and the end of the declaration, and nothing when no function starts here

 @example
 ```ts
 const { declared } = functionDeclarationAt({ characters, index: 40 });
 ```
 */
function functionDeclarationAt({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): { readonly declared?: DeclaredFunction; } {
  if (!nameMatchesAt({
    characters,
    index,
    name: FUNCTION_KEYWORD,
  },))
    return {};
  for (let nameStart = index + FUNCTION_KEYWORD.length; nameStart < characters.length; nameStart += 1) {
    /**
     Character at the scan's index, which may be the generator star or the first character of the name.
     */
    const character = characters[nameStart];
    if ((character === '*') || SPACES.includes(character ?? '',))
      continue;
    if (!isNameCharacter(character,))
      return {};
    /**
     Index past the function's name.
     */
    let nameEnd = nameStart;
    while (isNameCharacter(characters[nameEnd],))
      nameEnd += 1;
    /**
     Index of the parameter list that follows the name.
     */
    let parameters = nameEnd;
    while ((parameters < characters.length) && (characters[parameters] !== '('))
      parameters += 1;
    if (parameters >= characters.length)
      return {};
    /**
     Index of the body's opening brace, which follows the parameter list and its return type.
     */
    let body = bracketPairEnd({
      characters,
      opening: parameters,
      to: characters.length,
    },);
    while ((body < characters.length) && (characters[body] !== '{'))
      body += 1;
    if (body >= characters.length)
      return {};
    /**
     Index just past the body's closing brace.
     */
    const bodyEnd = bracketPairEnd({
      characters,
      opening: body,
      to: characters.length,
    },);
    return {
      declared: {
        name: characters.slice(
          nameStart,
          nameEnd,
        )
          .join('',),
        text: characters.slice(
          body + 1,
          bodyEnd - 1,
        )
          .join('',)
          .trim(),
        end: bodyEnd,
      },
    };
  }
  return {};
}

/**
 The text each name a file declares stands for: a binding's initializer, or a function's body.

 @param blankedSourceText - the file's text with its comments already blanked

 @returns one entry per declared name, mapped to the text of its initializer or body

 @example
 ```ts
 const definitions = definedTextByName({ blankedSourceText });
 ```
 */
/**
 The binding keyword a text declares at an index, when one stands there.

 @param characters - the file's characters
 
 @param index - index the declaration starts at

 @returns the keyword, and nothing when no binding keyword stands there

 @example
 ```ts
 const { keyword } = bindingKeywordAt({ characters, index: 4 });
 ```
 */
function bindingKeywordAt({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): { readonly keyword?: string; } {
  for (const keyword of BINDING_KEYWORDS) {
    if (nameMatchesAt({
      characters,
      index,
      name: keyword,
    },))
      return { keyword, };
  }
  return {};
}

/**
 The text each name a file declares stands for: a binding's initializer, or a function's body.

 @param blankedSourceText - the file's text with its comments already blanked

 @returns one entry per declared name, mapped to the text of its initializer or body

 @example
 ```ts
 const definitions = definedTextByName({ blankedSourceText });
 ```
 */
export function definedTextByName({ blankedSourceText, }: {
  readonly blankedSourceText: string;
},): ReadonlyMap<string, string> {
  /**
   The file's characters, whose declarations this scan reads.
   */
  const characters = blankedSourceText.split('',);
  /**
   The text each declared name stands for, collected as the scan meets each declaration.
   */
  const definitions = new Map<string, string>();
  for (let index = 0; index < characters.length;) {
    /**
     Keyword that declares a binding at the scan's index, when one does.
     */
    const { keyword, } = bindingKeywordAt({
      characters,
      index,
    },);
    if (keyword !== undefined) {
      /**
       Index of the declared name, past the keyword and the spaces after it.
       */
      let nameStart = index + keyword.length;
      while (SPACES.includes(characters[nameStart] ?? '',))
        nameStart += 1;
      if (isNameCharacter(characters[nameStart],)) {
        /**
         Index past the declared name.
         */
        let nameEnd = nameStart;
        while (isNameCharacter(characters[nameEnd],))
          nameEnd += 1;
        /**
         The `=` that starts the value this binding gives its name, when it assigns one.
         */
        const { assignment, } = assignmentAt({
          characters,
          from: nameEnd,
        },);
        if (assignment !== undefined) {
          /**
           Index past the binding's initializer.
           */
          const end = initializerEnd({
            characters,
            from: assignment + 1,
            to: characters.length,
          },);
          definitions.set(
            characters.slice(
              nameStart,
              nameEnd,
            )
              .join('',),
            characters.slice(
              assignment + 1,
              end,
            )
              .join('',)
              .trim(),
          );
          index = end;
          continue;
        }
      }
    }
    /**
     The function this index declares, when `function` stands there.
     */
    const { declared, } = functionDeclarationAt({
      characters,
      index,
    },);
    if (declared !== undefined) {
      definitions.set(
        declared.name,
        declared.text,
      );
      index = declared.end;
      continue;
    }
    index += 1;
  }
  return definitions;
}
