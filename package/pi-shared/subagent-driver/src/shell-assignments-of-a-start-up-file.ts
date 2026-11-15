/**
 What a shell start-up file exports, read the way a shell reads it: quotes hold what they hold, a `#` that starts a word begins a
 comment, and a value the shell computes is not a value this code can know.

 The distinction that matters is the one `export STAND_IN_API_KEY="a value" # a note` draws: the shell exports `a value`, so that
 is what has to be armed, and neither the quotes nor the note may end up in the value. A value the shell concatenates
 (`"a"b`, `$OTHER`, `$(command)`) is skipped as a whole rather than guessed at, because a wrong guess arms text that is not a
 credential and leaves the real one unmasked.

 @module
 */

import { carriesCredentialName, } from './environment-without-credentials.ts';

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
 What a shell takes a backslash to escape inside double quotes.
 */
const ESCAPED_IN_DOUBLE_QUOTES: ReadonlySet<string> = new Set([
  '"',
  '\\',
  '$',
  '`',
],);

/**
 What marks the rest of a value as computed by the shell when it loads, which this code cannot know.
 */
const COMPUTED_MARKERS: ReadonlySet<string> = new Set([
  '$',
  '`',
],);

/**
 The characters a shell takes as the end of an unquoted word.
 */
const WORD_BREAKS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
],);

//endregion Constants

//region Reading one value

/**
 What reading one written value found.
 */
export type WrittenValue =
  | { readonly value: string }
    | { readonly computed: true }
    | { readonly malformed: true };

/**
 Reads the value of one assignment the way the shell would, up to the end of the word.

 @param written - the text after the `=`, as the line wrote it

 @returns the value the shell exports, that it is computed when the shell loads, or that the line is not a value this code can read

 @example
 ```ts
 const value = writtenValue({ written: '"a value" # a note', },);
 ```
 */
export function writtenValue({ written, }: { readonly written: string },): WrittenValue {
  /**
   The characters of the value the shell exports, which is what the quotes do not swallow.
   */
  const characters: string[] = [];
  /**
   Which quoting the reader is inside, or none, in a holder so the loop carries its own state.
   */
  const quoting: { inside: 'none' | 'single' | 'double' } = { inside: 'none', };
  for (let index = 0; index < written.length; index += 1) {
    /**
     The character being read.
     */
    const character = written[index] ?? '';
    if (quoting.inside === 'single') {
      if (character === '\'') {
        quoting.inside = 'none';
        continue;
      }
      characters.push(character,);
      continue;
    }
    if (quoting.inside === 'double') {
      if ((character === '\\') && (ESCAPED_IN_DOUBLE_QUOTES.has(written[index + 1] ?? '',))) {
        characters.push(written[index + 1] ?? '',);
        index += 1;
        continue;
      }
      if (character === '"') {
        quoting.inside = 'none';
        continue;
      }
      if (COMPUTED_MARKERS.has(character,)) {
        return { computed: true, };
      }
      characters.push(character,);
      continue;
    }
    // Outside quotes: a word break ends the value, a `#` that starts a word begins a comment, and a quote opens one.
    if (WORD_BREAKS.has(character,)) {
      return { value: characters.join('',), };
    }
    // A `#` begins a comment only where a word begins, and a word break already ended the value above, so the only place a
    // comment can start here is the value's own first character.
    if ((character === '#') && (index === 0)) {
      return { value: characters.join('',), };
    }
    if ((character === '\\') && ((written[index + 1] ?? '') !== '')) {
      characters.push(written[index + 1] ?? '',);
      index += 1;
      continue;
    }
    if (character === '\'') {
      quoting.inside = 'single';
      continue;
    }
    if (character === '"') {
      quoting.inside = 'double';
      continue;
    }
    if (COMPUTED_MARKERS.has(character,)) {
      return { computed: true, };
    }
    characters.push(character,);
  }
  // A value whose quoting never closed is a line this code does not understand, and guessing at it would arm text a shell never
  // exports.
  return (quoting.inside === 'none') ? { value: characters.join('',), } : { malformed: true, };
}

/**
 Reads a shell start-up file's lines and returns the value each credential-named variable is exported with.

 @param text - the file's text

 @param fileName - what a marker names the file as

 @returns the assignments a marker names, each with the value the shell exports

 @example
 ```ts
 const exported = exportedCredentialValues({ text, fileName: '.bashrc', },);
 ```
 */
export function exportedCredentialValues({
  text,
  fileName,
}: {
  readonly text: string;
  readonly fileName: string;
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
  for (const line of text.split('\n',)) {
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
    if ((!('value' in read)) || (read.value === '')) {
      continue;
    }
    assignments.push({
      name: `${fileName} ${name}`,
      value: read.value,
    },);
  }
  return assignments;
}

//endregion Reading one value
