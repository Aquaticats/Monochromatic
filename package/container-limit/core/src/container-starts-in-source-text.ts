/**
 Where a file starts a container or builds an image, and the argument text that reaches the program there.

 The ledger in `container-start-sites.unit.test.ts` needs each start separately: a file that imports
 `containerLimitArguments` and never calls it in a start's own argument list used to read as covered, because the scan
 looked for the helper's name anywhere in the file (`doc/handover/container-limits-everywhere.md`,
 `package/terminal/app/mise.toml` before `bbadfac560`).
 Reading the start itself closes that gap.

 A mention whose first argument is not one of the verbs is not a start: `podman image exists` inspects an image, and
 `spawn('podman', args)` starts whatever its caller's arguments say, which the paired ledger role covers.
 */

import {
  bracketPairEnd,
  isNameCharacter,
  isQuote,
  nameMatchesAt,
} from './positions-in-source-text.ts';
import {
  stripSourceComments,
  type SourceLanguage,
} from './source-text-without-comments.ts';

import {
  argumentTextNear,
  arrayContentsStart,
  spreadAtStart,
  startVerbOf,
} from './argument-text-of-a-container-start.ts';
import {
  CONTAINER_PROGRAMS,
  type ContainerProgram,
  type ContainerStartInvocation,
} from './container-start-vocabulary.ts';
import { definedTextByName, } from './defined-text-in-source-text.ts';


/**
 Verbs after a program name that start a container or build an image.
 */

/**
 One place a file starts a container or builds an image.
 */

/**
 Container programs the scan looks for.
 */

/**
 Verbs that make a program mention a start.
 */

/**
 Characters that cannot stand between a command and whatever precedes it on the line.
 */

const COMMAND_BREAKS = ',<>`#';

/**
 Characters before a command that start one: the quote that opens the script it runs in, and the shell's separators.
 */
const COMMAND_SEPARATORS = '\'";|&';

/**
 Punctuation a shell word can hold, which the walk back over a command passes.
 */
const COMMANDWORD_PUNCTUATION = '-=./:@$~^%+*?!';

/**
 Newline character, which the line count reads.
 */

const NEWLINE = '\n';

/**
 The container program a text names at an index, when a whole word there is one.

 A path or a dotted name (`./podman`, `com.podman`) is not a program mention.

 @param characters - the file's characters
 
 @param index - the index to read at

 @returns the program, and nothing when no program name starts here

 @example
 ```ts
 const { program } = programAt({ characters, index: 0 });
 ```
 */
function programAt({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): { readonly program?: ContainerProgram; } {
  if ((characters[index - 1] === '.') || (characters[index - 1] === '/')
    || (characters[index - 1] === '-'))
    return {};
  for (const program of CONTAINER_PROGRAMS) {
    if (nameMatchesAt({
      characters,
      index,
      name: program,
    },))
      return { program, };
  }
  return {};
}

/**
 Whether a bare program name sits where a shell command starts.

 Prose names a program too (`podman build has no --cpus`, in a sentence or inside a backtick-quoted phrase), so the
 walk back from the name passes the words of one command and stops at anything a command cannot follow.

 @param characters - the file's characters
 
 @param index - index of the program name

 @returns true when the name can start a command

 @example
 ```ts
 const startsCommand = isCommandPosition({ characters, index: 4 });
 ```
 */

function isCommandPosition({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): boolean {
  for (let back = index - 1; back >= 0; back -= 1) {
    /**
     Character the walk back reads.
     */
    const character = characters[back] ?? '';
    if ((character === NEWLINE) || COMMAND_SEPARATORS.includes(character,))
      return true;
    if (character === '(')
      return characters[back - 1] === '$';
    if (COMMAND_BREAKS.includes(character,) || isQuote(character,))
      return false;
    if ((character !== ' ') && (character !== '\t')
      && (!isNameCharacter(character,))
      && (!COMMANDWORD_PUNCTUATION.includes(character,)))
      return false;
  }
  return true;
}

/**
 The line an index sits on, counted from one.

 @param characters - the file's characters
 
 @param index - index of the character to place

 @returns the line the character sits on

 @example
 ```ts
 const line = lineOf({ characters, index: 420 });
 ```
 */

function lineOf({
  characters,
  index,
}: {
  readonly characters: readonly string[];
  readonly index: number;
},): number {
  /**
   Lines found before the index, which the character's own line adds one to.
   */
  let lines = 1;
  for (let read = 0; read < index; read += 1) {
    if (characters[read] === NEWLINE)
      lines += 1;
  }
  return lines;
}

/**
 Every place a text starts a container or builds an image, in the order the text holds them.

 @param text - the file's text
 
 @param language - the language whose comment syntax the text holds
 
 @param importedDefinitions - the text each name stands for in a module the file imports from

 @returns one entry per start, with the argument text that reaches the program

 @example
 ```ts
 const starts = containerStartInvocations({ text, language: 'typescript' });
 ```
 */
export function containerStartInvocations({
  text,
  language,
  importedDefinitions,
}: {
  readonly text: string;
  readonly language: SourceLanguage;
  readonly importedDefinitions?: ReadonlyMap<string, string>;
},): readonly ContainerStartInvocation[] {
  /**
   The text with its comments blanked, so a mention inside one is not read as a start.
   */
  const blankedSourceText = stripSourceComments({
    text,
    language,
  },);
  /**
   The blanked text's characters, whose mentions this scan reads.
   */
  const characters = blankedSourceText.split('',);
  /**
   The text each name the file declares stands for, which tells whether a spread names a builder.
   */
  const definitions = definedTextByName({ blankedSourceText, },);
  /**
   One entry per start found so far.
   */
  const invocations: ContainerStartInvocation[] = [];
  for (let index = 0; index < characters.length; index += 1) {
    /**
     Program the text names at the index, when it names one.
     */
    const { program, } = programAt({
      characters,
      index,
    },);
    /**
     Whether the name is a string literal, as the argument-array form writes it (`['podman', 'run']`), which is a
     quoted name on both sides; a name after a template's backtick is not one.
     */
    const quoted = (program !== undefined) && isQuote(characters[index - 1],)
      && isQuote(characters[index + program.length],);
    if ((program !== undefined) && (quoted || isCommandPosition({
      characters,
      index,
    },))) {
      /**
       Index past the program name, and past the quote that ended a quoted name.
       */
      let programEnd = index + program.length;
      if (isQuote(characters[programEnd],))
        programEnd += 1;
      /**
       Index of the character that starts the mention's element: the opening quote of a quoted name.
       */
      const elementStart = quoted ? index - 1 : index;
      /**
       First character of an array literal whose first element is the program name, when the mention is one.
       */
      const { startsAt, } = arrayContentsStart({
        characters,
        index: elementStart,
      },);
      /**
       Argument text that reaches the program, and whether an array literal held it.
       */
      const argument = startsAt === undefined
        ? argumentTextNear({
          characters,
          from: programEnd,
          to: characters.length,
        },)
        : {
          text: characters.slice(
            programEnd,
            bracketPairEnd({
              characters,
              opening: startsAt - 1,
              to: characters.length,
            },) - 1,
          )
            .join('',),
          inArrayLiteral: true,
        };
      /**
       Verb the start passes, when the mention is a start at all.
       */
      const { verb, } = startVerbOf(argument.text,);
      /**
       Spread the argument text starts with, when its arguments come from a builder instead of a verb written here.
       */
      const {
        name,
        isCall,
      } = argument.inArrayLiteral
        ? spreadAtStart(argument.text,)
        : { isCall: false, };
      /**
       Whether that spread's builder is a call, or a name this text or the module it imports tells.
       */
      const argumentsFromCall = (name !== undefined)
        && (isCall || definitions.has(name,)
          || (importedDefinitions?.has(name,) === true));
      /**
       Line the mention sits on, counted from one in the text the scan read.
       */
      const line = lineOf({
        characters,
        index,
      },);
      if (verb !== undefined)
        invocations.push({
          program,
          verb,
          line,
          argvText: argument.text,
        },);
      else if (argumentsFromCall)
        invocations.push({
          program,
          line,
          argvText: argument.text,
        },);
    }
  }
  return invocations;
}
