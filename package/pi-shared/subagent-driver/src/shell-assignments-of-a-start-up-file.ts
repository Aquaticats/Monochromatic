/**
 What a shell start-up file exports, read by a reader that refuses every line it does not fully understand.

 A second reader of a shell's language keeps differing from the shell: a `;` after a value, a `#`, a computed value, a
 concatenation. So this reader accepts only the shapes it can read exactly (one quoted or unquoted word, nothing after it but a
 comment), and every other line is "could not" rather than a guess: a value it cannot read is taken from the environment the driver
 itself was started with, by name, or the run does not start. A wrong text is never armed.

 @module
 */

import { carriesCredentialName, } from './environment-without-credentials.ts';
import type { EnvironmentOfThisMachine, } from './secret-values-in-the-files-of-this-machine.ts';
import { SecretEnvironmentUnreadableError, } from './secret-environment-unreadable-error.ts';

//region Constants

/**
 An assignment of a shell start-up file: the variable's name, then its value as written.

 A regular expression is what reads one line of a shell file into a name and the rest of the line: the alternative is a shell
 parser, and a value may hold spaces, quotes and `=` of its own. The pattern is anchored at the line's start, has no nested
 quantifier, and each line is one line of a file this machine owns, so matching it is linear in the line and cannot backtrack.
 */
const ASSIGNMENT_PATTERN =
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- reading `NAME=value` from one line needs a pattern; see the comment above.
  /^\s*(?:export\s+|declare\s+-[a-zA-Z]*x[a-zA-Z]*\s+)?(?<name>[A-Za-z_][A-Za-z0-9_]*)=(?<written>.*)$/u;

/**
 The characters that end an unquoted word in a shell, or that begin a command of their own: a value that holds one of them unquoted
 is a line this reader will not guess at.
 */
const UNQUOTED_OPERATORS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  ';',
  '&',
  '|',
  '<',
  '>',
  '(',
  ')',
],);

/**
 The characters a shell takes as the start of something it computes when it loads, which this code cannot know.
 */
const COMPUTED_MARKERS: ReadonlySet<string> = new Set([
  '$',
  '`',
],);

/**
 What a shell takes a backslash to escape inside double quotes.
 */
const ESCAPED_IN_DOUBLE_QUOTES: ReadonlySet<string> = new Set([
  '"',
  '\\',
  '$',
  '`',
],);

//endregion Constants

//region Reading one value

/**
 What reading one written value found.
 */
export type WrittenValue =
  | { readonly value: string }
  // The shell computes it when it loads, so this code cannot know it.
    | { readonly computed: true }
  // The line is not one this reader can read exactly: a control operator, a concatenation, a redirection, an unclosed quote.
    | { readonly notUnderstood: string };

/**
 Says whether a text is the end of an assignment: nothing but spaces, or a comment.

 @param text - what follows the value

 @returns whether the line holds nothing but the value and, perhaps, a comment
 */
function onlySpacesOrAComment({ text, }: { readonly text: string },): boolean {
  /**
   The text without its leading spaces.
   */
  const rest = trimmedAtTheStart({ text, },);
  return rest.startsWith('#',) || (rest.trim() === '');
}

/**
 Takes the leading spaces off a text, which an assignment never puts in its value.

 @param text - the text

 @returns the text from its first character that is not a space or a tab
 */
function trimmedAtTheStart({ text, }: { readonly text: string },): string {
  /**
   The place the leading spaces end, in a holder so the loop names its own state.
   */
  const place: { start: number } = { start: 0, };
  while ((place.start < text.length) && ((text[place.start] === ' ') || (text[place.start] === '\t'))) {
    place.start += 1;
  }
  return text.slice(place.start,);
}

/**
 Reads the value of one assignment the way the shell would, and refuses every line it cannot read exactly.

 @param written - the text after the `=`, as the line wrote it

 @returns the value the shell exports, that the shell computes it, or why the line is not one this reader reads

 @example
 ```ts
 const read = writtenValue({ written: '"a value" # a note', },);
 ```
 */
export function writtenValue({ written, }: { readonly written: string },): WrittenValue {
  /**
   The trimmed text, whose leading spaces an assignment does not put in its value.
   */
  const text = trimmedAtTheStart({ text: written, },);
  if (text === '') {
    return { value: '', };
  }
  /**
   The first character, which says which of the shapes this is.
   */
  const first = text[0] ?? '';
  if ((first === '\'') || (first === '"')) {
    /**
     The characters of the value, which is what the quotes do not swallow.
     */
    const characters: string[] = [];
    for (let index = 1; index < text.length; index += 1) {
      /**
       The character being read.
       */
      const character = text[index] ?? '';
      if (first === '\'') {
        if (character === first) {
          return onlySpacesOrAComment({ text: text.slice(index + 1,), },)
            ? { value: characters.join('',), }
            : { notUnderstood: 'the line holds something after its quoted value', };
        }
        characters.push(character,);
        continue;
      }
      if ((character === '\\') && (ESCAPED_IN_DOUBLE_QUOTES.has(text[index + 1] ?? '',))) {
        characters.push(text[index + 1] ?? '',);
        index += 1;
        continue;
      }
      if (character === first) {
        return onlySpacesOrAComment({ text: text.slice(index + 1,), },)
          ? { value: characters.join('',), }
          : { notUnderstood: 'the line holds something after its quoted value', };
      }
      if (COMPUTED_MARKERS.has(character,)) {
        return { computed: true, };
      }
      characters.push(character,);
    }
    return { notUnderstood: 'the quoted value does not close', };
  }
  /**
   The characters of an unquoted value.
   */
  const characters: string[] = [];
  for (let index = 0; index < text.length; index += 1) {
    /**
     The character being read.
     */
    const character = text[index] ?? '';
    if (UNQUOTED_OPERATORS.has(character,)) {
      // A space ends the word, so what follows is a comment or another command; anything else is a command of its own.
      return onlySpacesOrAComment({ text: text.slice(index,), },)
        ? { value: characters.join('',), }
        : { notUnderstood: `the line holds ${character} after its value`, };
    }
    if (character === '\'') {
      return { notUnderstood: 'the line quotes part of its value and leaves the rest unquoted', };
    }
    if (character === '"') {
      return { notUnderstood: 'the line quotes part of its value and leaves the rest unquoted', };
    }
    if ((character === '\\') && ((text[index + 1] ?? '') !== '')) {
      characters.push(text[index + 1] ?? '',);
      index += 1;
      continue;
    }
    if (COMPUTED_MARKERS.has(character,)) {
      return { computed: true, };
    }
    characters.push(character,);
  }
  return { value: characters.join('',), };
}

/**
 Reads a shell start-up file's lines and returns the value each credential-named variable is exported with.

 A value the reader cannot give is taken from the environment the driver itself was started with, by the variable's name; when that
 environment holds nothing either, the run does not start, and the message names the file and the line.

 @param text - the file's text

 @param fileName - what a marker names the file as

 @param environment - the environment the driver itself runs with, which is where a value the shell computes can be found

 @returns the assignments a marker names, each with the value the shell exports

 @throws SecretEnvironmentUnreadableError when a credential line is one this reader cannot give and the environment does not hold it

 @example
 ```ts
 const exported = exportedCredentialValues({ text, fileName: '.bashrc', environment: process.env, },);
 ```
 */
export function exportedCredentialValues({
  text,
  fileName,
  environment,
}: {
  readonly text: string;
  readonly fileName: string;
  readonly environment?: EnvironmentOfThisMachine;
},): readonly {
  readonly name: string;
  readonly value: string
}[] {
  /**
   The assignments read.
   */
  const assignments: {
    name: string;
    value: string
  }[] = [];
  /**
   The line this is, counting from one.
   */
  let lineNumber = 0;
  for (const line of text.split('\n',)) {
    lineNumber += 1;
    /**
     The assignment this line is, if it is one.
     */
    const assignment = ASSIGNMENT_PATTERN.exec(line,);
    if (assignment === null) {
      continue;
    }
    /**
     The variable's name and the rest of the line.
     */
    const {
      name,
      written,
    } = assignment.groups ?? {};
    if ((name === undefined) || (written === undefined)
      || (!carriesCredentialName({ name, },))) {
      continue;
    }
    /**
     The value the shell exports.
     */
    const read = writtenValue({ written, },);
    if ('value' in read) {
      if (read.value !== '') {
        assignments.push({
          name: `${fileName} ${name}`,
          value: read.value,
        },);
      }
      continue;
    }
    /**
     What the environment this driver was started with holds under that name.
     */
    const fromTheEnvironment = (environment ?? {})[name];
    if (((typeof fromTheEnvironment) === 'string') && (fromTheEnvironment.trim() !== '')) {
      assignments.push({
        name: `${fileName} ${name}`,
        value: fromTheEnvironment,
      },);
      continue;
    }
    throw new SecretEnvironmentUnreadableError({
      source: `${fileName} line ${String(lineNumber,)}`,
      cause: `the value of ${name} is one this reader cannot give (${('computed' in read) ? 'the shell computes it when it loads' : read.notUnderstood}), and the environment this driver was started with does not hold ${name} either; write the credential out in the file, or export it before the run starts`,
    },);
  }
  return assignments;
}

//endregion Reading one value
