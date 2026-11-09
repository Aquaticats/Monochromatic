/**
 The argument text a container program receives, read from the command the file writes.

 A start passes its arguments in one of the shapes the repository writes: an argument array after the program name
 (`execFileSync('podman', ['run', ...])`), a program name as the array's first element (`['podman', 'build', ...]`), a
 call that returns the arguments (`spawn('podman', [...containerRunArguments({})])`), or a shell command line
 (`podman run --rm ...`, with its `\` continuations joined).
 */

import {
  CONTAINER_START_VERBS,
  type ContainerStartVerb,
} from './container-start-vocabulary.ts';
import {
  bracketPairEnd,
  isNameCharacter,
  isQuote,
  lineEnd,
} from './positions-in-source-text.ts';

/**
 Space characters that separate a program name from its arguments.
 */
const SPACES = ' \t\r\n';

/**
 Spaces and commas between a program name and its argument text.
 */
const SEPARATORS = ' \t\r\n,';

/**
 The spread marker that puts a builder's arguments in an array literal.
 */

const SPREAD_MARKER = '...';

/**
 Newline character, which a command line ends at.
 */
const NEWLINE = '\n';

/**
 Index of the first character that can belong to the argument text of a program mention.

 Skips the whitespace and commas between a program name and its arguments, and a named property such as `args:` of an
 options object, which is how a spawn call writes the arguments.

 @param characters - the file's characters
 
 @param from - index after the program name
 
 @param to - end of the range to read

 @returns the index of the first character of the argument text

 @example
 ```ts
 const start = argumentStart({ characters, from: 10, to: characters.length });
 ```
 */

function argumentStart({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Index past the separators and the property name, which is where the argument text starts.
   */
  let index = from;
  while ((index < to) && SEPARATORS.includes(characters[index] ?? '',))
    index += 1;
  /**
   Index past the name of a property that holds the arguments.
   */
  let nameEnd = index;
  while ((nameEnd < to) && isNameCharacter(characters[nameEnd],))
    nameEnd += 1;
  if ((nameEnd > index) && (characters[nameEnd] === ':'))
    return argumentStart({
      characters,
      from: nameEnd + 1,
      to,
    },);
  return index;
}

/**
 Index past the last continuation of the command line that holds an index.

 A shell command continues on a line whose last character is `\`.

 @param characters - the file's characters
 
 @param from - index the command starts at
 
 @param to - end of the range to read

 @returns the index of the newline that ends the command, or the end of the range

 @example
 ```ts
 const end = commandLineEnd({ characters, from: 0, to: characters.length });
 ```
 */
function commandLineEnd({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Index of the newline that ends the last line of the command read so far.
   */
  let index = lineEnd({
    characters,
    from,
    to,
  },);
  while (index < to) {
    /**
     Last character before the newline that is not a space.
     */
    let last = index - 1;
    while ((last >= from) && SPACES.includes(characters[last] ?? '',))
      last -= 1;
    if (characters[last] !== '\\')
      return index;
    index = lineEnd({
      characters,
      from: index + 1,
      to,
    },);
  }
  return index;
}

/**
 The argument text that follows a program name.

 The forms read: `['run', ...]` and `args: ['run', ...]` as an array literal, `(['run', ...])` through its parentheses,
 and a shell command line with its `\` continuations joined.

 @param characters - the file's characters
 
 @param from - index after the program name
 
 @param to - end of the range to read

 @returns the argument text, and whether an array literal held it

 @example
 ```ts
 const argument = argumentTextNear({ characters, from: 8, to: characters.length });
 ```
 */
export function argumentTextNear({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): {
  readonly text: string;
  readonly inArrayLiteral: boolean
} {
  /**
   First character of the argument text.
   */
  const start = argumentStart({
    characters,
    from,
    to,
  },);
  if (start >= to)
    return {
      text: '',
      inArrayLiteral: false,
    };
  /**
   Character that opens the argument text, which tells an array literal or a call from a command line.
   */
  const opening = characters[start] ?? '';
  if (opening === '[') {
    /**
     Index just past the array literal's closing bracket.
     */
    const end = bracketPairEnd({
      characters,
      opening: start,
      to,
    },);
    return {
      text: characters.slice(
        start + 1,
        end - 1,
      )
        .join('',),
      inArrayLiteral: true,
    };
  }
  if (opening === '(') {
    /**
     Index just past the call's closing parenthesis.
     */
    const end = bracketPairEnd({
      characters,
      opening: start,
      to,
    },);
    return argumentTextNear({
      characters,
      from: start + 1,
      to: end - 1,
    },);
  }
  return {
    text: characters.slice(
      start,
      commandLineEnd({
        characters,
        from: start,
        to,
      },),
    )
      .join('',),
    inArrayLiteral: false,
  };
}

/**
 The first word of an argument text, past the separators that follow a program name and any quote around it.

 @param argvText - the argument text that follows a program name

 @returns the word, or the empty string when the text holds none

 @example
 ```ts
 const word = firstWordOf("'run', '--rm'");
 ```
 */

function firstWordOf(argvText: string,): string {
  for (let index = 0; index < argvText.length; index += 1) {
    if (SEPARATORS.includes(argvText[index] ?? '',))
      continue;
    /**
     Index of the first character of the word, past the quote that may wrap it.
     */
    const wordStart = isQuote(argvText[index],) ? index + 1 : index;
    /**
     Index past the word itself.
     */
    let wordEnd = wordStart;
    while ((wordEnd < argvText.length) && isNameCharacter(argvText[wordEnd],))
      wordEnd += 1;
    return argvText.slice(
      wordStart,
      wordEnd,
    );
  }
  return '';
}

/**
 The verb an argument text starts with, when a container start is one of the three verbs.

 @param argvText - the argument text that follows a program name

 @returns the verb, and nothing when the first word is not one

 @example
 ```ts
 const { verb } = startVerbOf("'run', '--rm'");
 ```
 */
export function startVerbOf(argvText: string,): { readonly verb?: ContainerStartVerb; } {
  /**
   The first word of the argument text, which is the verb unless the text names a variable.
   */
  const word = firstWordOf(argvText,);
  for (const verb of CONTAINER_START_VERBS) {
    if (word === verb)
      return { verb, };
  }
  return {};
}

/**
 Index just past the bracket that opens when the name at an index is a bracket pair's first element.

 The repository writes a program's arguments as an array that starts with the program name
 (`['podman', 'build', ...limits, '--tag', ...]`), so the argument text then holds every later element of it.

 @param characters - the file's characters
 
 @param index - index of the first character of the mention, which is its opening quote when it is quoted

 @returns the index of the first character inside the array, and nothing when no array opens right before

 @example
 ```ts
 const { startsAt } = arrayContentsStart({ characters, index: 4 });
 ```
 */
export function arrayContentsStart({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): { readonly startsAt?: number; } {
  for (let back = index - 1; back >= 0; back -= 1) {
    if (SEPARATORS.includes(characters[back] ?? '',))
      continue;
    if (characters[back] === '[')
      return { startsAt: back + 1, };
    return {};
  }
  return {};
}

/**
 The spread an argument text starts with, when its first element spreads a name or a call.

 `['podman', ...containerRunArguments({ image })]` and `['podman', ...pre, 'sign']` write a start whose arguments come
 from somewhere other than this line, so the verb is not in the text at all.

 @param argvText - the argument text of a start read from an array literal

 @returns the spread name and whether it is a call, and nothing when the text starts with something else

 @example
 ```ts
 const { name, isCall } = spreadAtStart('...containerRunArguments({ image })');
 ```
 */
export function spreadAtStart(argvText: string,): {
  readonly name?: string;
  readonly isCall: boolean
} {
  for (let index = 0; index < argvText.length; index += 1) {
    if (SEPARATORS.includes(argvText[index] ?? '',))
      continue;
    if (!argvText.startsWith(
      SPREAD_MARKER,
      index,
    ))
      return { isCall: false, };
    /**
     Index of the spread name's first character.
     */
    const nameStart = index + SPREAD_MARKER.length;
    /**
     Index past the spread name.
     */
    let nameEnd = nameStart;
    while ((nameEnd < argvText.length) && isNameCharacter(argvText[nameEnd],))
      nameEnd += 1;
    if (nameEnd === nameStart)
      return { isCall: false, };
    /**
     First character after the name, which opens a call's parentheses when the spread is one.
     */
    let after = nameEnd;
    while ((after < argvText.length) && SPACES.includes(argvText[after] ?? '',))
      after += 1;
    return {
      name: argvText.slice(
        nameStart,
        nameEnd,
      ),
      isCall: argvText[after] === '(',
    };
  }
  return { isCall: false, };
}
