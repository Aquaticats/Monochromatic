/**
 The argument text a container program receives, read from the command the file writes.

 A start passes its arguments in one of the shapes the repository writes: an argument array after the program name
 (`execFileSync('podman', ['run', ...])`), a program name as the array's first element (`['podman', 'build', ...]`), a
 call that returns the arguments (`spawn('podman', [...containerRunArguments({})])`), a shell command line
 (`podman run --rm ...`, with its `\` continuations joined), or, in a Rust source, the calls chained after the call that
 named the program (`Command::new("podman").args(["run", ...])`).
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
 Characters that can sit between a call and the method call chained after it.
 */
const CHAIN_SEPARATORS = ' \t\r\n';

/**
 The methods of `std::process::Command` that pass an argument to the program the command runs.
 */
const RUST_ARGUMENT_METHODS: ReadonlySet<string> = new Set([
  'arg',
  'args',
],);

/**
 The characters that open the macro a Rust source builds an argument vector with.
 */
const RUST_VECTOR_MACRO = 'vec!';

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

/**
 Index of the first character of a range that is neither a space nor a line break.

 @param characters - the file's characters
 
 @param from - index to start reading at
 
 @param to - end of the range to read

 @returns the index of the first character no chain separator holds, or the end of the range

 @example
 ```ts
 const start = chainStart({ characters, from: 12, to: characters.length });
 ```
 */
function chainStart({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): number {
  /**
   Index past the separators, which is where the next call or the end of the statement starts.
   */
  let index = from;
  while ((index < to) && CHAIN_SEPARATORS.includes(characters[index] ?? '',))
    index += 1;
  return index;
}

/**
 The argument text one `arg` or `args` call passes, and whether an array literal held it.

 A Rust source writes an argument array as a literal, behind a borrow (`&["run", ...]`) or through the vector macro
 (`vec!["run", ...]`), and writes one argument as a plain string.

 @param text - the text between the call's parentheses

 @returns the argument text past the borrow, the macro or the brackets, and whether brackets held it

 @example
 ```ts
 const { text, inArrayLiteral } = argumentTextOfOneCall('&["run", "--rm"]');
 ```
 */
function argumentTextOfOneCall({ text, }: {
  readonly text: string;
},): {
  readonly text: string;
  readonly inArrayLiteral: boolean
} {
  /**
   The call's argument, past the spaces around it.
   */
  const trimmed = text.trim();
  /**
   The argument past the borrow a Rust source may write before its array.
   */
  const borrowed = trimmed.startsWith('&',)
    ? trimmed.slice(1,)
      .trim()
    : trimmed;
  /**
   The argument past the macro that builds the vector it names.
   */
  const unwrapped = borrowed.startsWith(RUST_VECTOR_MACRO,)
    ? borrowed.slice(RUST_VECTOR_MACRO.length,)
      .trim()
    : borrowed;
  if (unwrapped.startsWith('[',) && unwrapped.endsWith(']',))
    return {
      text: unwrapped.slice(
        1,
        -1,
      ),
      inArrayLiteral: true,
    };
  return {
    text: unwrapped,
    inArrayLiteral: false,
  };
}

/**
 The argument text a Rust program passes to the container program whose name a call holds.

 A Rust source writes a start as a call that names the program, followed by the calls that hand the program its
 arguments: `Command::new("podman").args(["run", ...])` or
 `Command::new("podman").arg("run").arg("--rm")`. The arguments therefore sit in the chained calls, not beside the name.
 Only `arg` and `args` pass an argument to the program, so the text of every other call the chain makes (`output`,
 `expect`, `env`) is not part of what the program receives.

 @param characters - the file's characters
 
 @param from - index just past the quote that closes the program name
 
 @param to - end of the range to read

 @returns the argument text the chained calls pass, joined in call order, and whether an array literal held any of it

 @example
 ```ts
 const argument = argumentTextOfTheChainedCalls({ characters, from: 26, to: characters.length });
 ```
 */
/**
 One call a Rust chain makes, with the argument text it handed the program.
 */
type ChainedCall = {
  /**
   The argument text the call passed, past its brackets and borrow.
   */
  readonly text: string;
  /**
   Whether an array literal held that text.
   */
  readonly inArrayLiteral: boolean;
};

/**
 The argument text a Rust program passes to the container program whose name a call holds.

 A Rust source writes a start as a call that names the program, followed by the calls that hand the program its
 arguments: `Command::new("podman").args(["run", ...])` or
 `Command::new("podman").arg("run").arg("--rm")`. The arguments therefore sit in the chained calls, not beside the name.
 Only `arg` and `args` pass an argument to the program, so the text of every other call the chain makes (`output`,
 `expect`, `env`) is not part of what the program receives.

 @param characters - the file's characters
 
 @param from - index just past the quote that closes the program name
 
 @param to - end of the range to read

 @returns the argument text the chained calls pass, joined in call order, and whether an array literal held any of it

 @example
 ```ts
 const argument = argumentTextOfTheChainedCalls({ characters, from: 26, to: characters.length });
 ```
 */
export function argumentTextOfTheChainedCalls({
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
   Index past the parenthesis that closes the call naming the program, when the name was that call's only argument:
   anything else is a name the caller passes on, as `spawn('podman', args)` does.
   */
  const opening = chainStart({
    characters,
    from,
    to,
  },);
  if (characters[opening] !== ')')
    return {
      text: '',
      inArrayLiteral: false,
    };
  /**
   Each call the chain makes that handed the program an argument, in the order the source writes them.
   */
  const calls = chainedCalls({
    characters,
    from: opening + ')'.length,
    to,
  },);
  return {
    text: calls.map(argumentTextOfAChainedCall,)
      .join(', ',),
    inArrayLiteral: calls.some(argumentTextHeldByAnArrayLiteral,),
  };
}

/**
 The argument text one call of a chain passed the program.

 @param call - the call

 @returns the text the call passed

 @example
 ```ts
 const text = argumentTextOfAChainedCall(call);
 ```
 */
function argumentTextOfAChainedCall(call: ChainedCall,): string {
  return call.text;
}

/**
 Whether an array literal held the argument text one call of a chain passed the program.

 @param call - the call

 @returns true when brackets held the text

 @example
 ```ts
 const held = argumentTextHeldByAnArrayLiteral(call);
 ```
 */
function argumentTextHeldByAnArrayLiteral(call: ChainedCall,): boolean {
  return call.inArrayLiteral;
}

/**
 Each call a chain of method calls makes that handed the program an argument, in source order.

 @param characters - the file's characters
 
 @param from - index just past the parenthesis that closed the call naming the program
 
 @param to - end of the range to read

 @returns one entry per `arg` or `args` call, in the order the chain makes them

 @example
 ```ts
 const calls = chainedCalls({ characters, from: 26, to: characters.length });
 ```
 */
function chainedCalls({
  characters,
  from,
  to,
}: {
  readonly characters: readonly string[];
  readonly from: number;
  readonly to: number;
},): readonly ChainedCall[] {
  /**
   One entry per call that passed the program an argument.
   */
  const calls: ChainedCall[] = [];
  for (let index = chainStart({
    characters,
    from,
    to,
  },); index < to;) {
    if (characters[index] !== '.')
      break;
    /**
     Index of the first character of the method's name.
     */
    const nameStart = index + '.'.length;
    /**
     Index past the method's name.
     */
    let nameEnd = nameStart;
    while ((nameEnd < to) && isNameCharacter(characters[nameEnd],))
      nameEnd += 1;
    if (nameEnd === nameStart)
      break;
    /**
     The method the chain calls here.
     */
    const method = characters.slice(
      nameStart,
      nameEnd,
    )
      .join('',);
    /**
     Index of the parenthesis that opens the call's arguments.
     */
    const parentheses = chainStart({
      characters,
      from: nameEnd,
      to,
    },);
    if (characters[parentheses] !== '(')
      break;
    /**
     Index past the call's closing parenthesis.
     */
    const end = bracketPairEnd({
      characters,
      opening: parentheses,
      to,
    },);
    if (RUST_ARGUMENT_METHODS.has(method,)) {
      calls.push(argumentTextOfOneCall({ text: characters.slice(
        parentheses + '('.length,
        end - ')'.length,
      )
        .join('',), },),);
    }
    index = chainStart({
      characters,
      from: end,
      to,
    },);
  }
  return calls;
}
