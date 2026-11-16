/**
 Splits a command into the words a shell would give it: at its separators and outside its quotes, with here-document bodies dropped
 because they are text a command writes rather than commands it runs.

 @module
 */

/**
 The wrapper options that take the word after them as their value, which is not the program.
 */
export const VALUE_OPTIONS: ReadonlySet<string> = new Set([
  '-s',
  '--signal',
  '-k',
  '--kill-after',
  '-u',
  '--unset',
  '-n',
  '-c',
  '-p',
  '-i',
  '-o',
  '-e',
  '-I',
  '-P',
  '-L',
  '--unit',
  '--description',
  '--slice',
  '--working-directory',
]);

/**
 Strips the body of every here-document, which is text a command writes and not a command it runs.

 @param command - the command as written

 @returns the command without its here-document bodies

 @example
 ```ts
 const stripped = withoutHereDocuments({ command: "cat > a.sh <<'EOF'\nprog\nEOF" });
 ```
 */
function withoutHereDocuments({ command, }: { readonly command: string },): string {
  /**
   The lines kept, and the delimiter the current here-document ends at.
   */
  const state = {
    kept: [] as string[],
    delimiter: '',
    stripLeadingTabs: false,
  };
  for (const line of command.split('\n',)) {
    if (state.delimiter !== '') {
      /**
       The line the here-document ends at, with the tabs `<<-` strips.
       */
      const closing = state.stripLeadingTabs ? withoutLeadingTabs({ line, }) : line;
      if (closing === state.delimiter) {
        state.delimiter = '';
      }
      continue;
    }
    /**
     Where this line opens a here-document.
     */
    const opening = hereDocumentOf({ line, },);
    if (opening === NO_HERE_DOCUMENT) {
      state.kept
        .push(line,);
      continue;
    }
    state.kept
      .push(line.slice(
        0,
        opening.at,
      ),);
    state.delimiter = opening.delimiter;
    state.stripLeadingTabs = opening.stripLeadingTabs;
  }
  return state.kept
    .join('\n',);
}

/**
 Reads a line without the tabs at its front.

 @param line - one line as written

 @returns the line with its leading tabs taken off

 @example
 ```ts
 const stripped = withoutLeadingTabs({ line: '\tEOF' });
 ```
 */
function withoutLeadingTabs({ line, }: { readonly line: string },): string {
  /**
   Where the first character that is no tab stands.
   */
  const cursor = { at: 0, };
  while (line[cursor.at] === '\t') {
    cursor.at += 1;
  }
  return line.slice(cursor.at,);
}

/**
 No here-document in a line, as a symbol so that no opening can stand for it.
 */
const NO_HERE_DOCUMENT: unique symbol = Symbol('line opens no here-document',);

/**
 Reads the here-document a line opens, when it opens one.

 @param line - one line of a command

 @returns where the operator stands and the delimiter, or nothing when the line opens none

 @example
 ```ts
 const opening = hereDocumentOf({ line: "cat > a.sh <<'EOF'" });
 ```
 */
function hereDocumentOf({ line, }: { readonly line: string },): {
  readonly at: number;
  readonly delimiter: string;
  readonly stripLeadingTabs: boolean
} | typeof NO_HERE_DOCUMENT {
  /**
   Where the operator stands.
   */
  const at = line.indexOf('<<',);
  if (at === (-1)) {
    return NO_HERE_DOCUMENT;
  }
  /**
   Where the delimiter's own text is read from.
   */
  const cursor = { at: at + 2, };
  /**
   Whether the operator strips leading tabs.
   */
  const stripLeadingTabs = (line[cursor.at] ?? '') === '-';
  if (stripLeadingTabs) {
    cursor.at += 1;
  }
  /**
   The quote the delimiter is written in, when it is quoted.
   */
  const quote = line[cursor.at];
  if ((quote === '\'') || (quote === '"')) {
    cursor.at += 1;
  }
  /**
   Where the delimiter ends.
   */
  const start = cursor.at;
  while (cursor.at < line.length) {
    /**
     The character this step reads.
     */
    const character = line[cursor.at] ?? '';
    if ((character === ' ') || (character === '\t')
      || (character === '\n')
      || ((quote !== undefined) && (character === quote))) {
      break;
    }
    cursor.at += 1;
  }
  /**
   The delimiter's text.
   */
  const delimiter = line.slice(
    start,
    cursor.at,
  );
  return (delimiter === '') ? NO_HERE_DOCUMENT : {
    at,
    delimiter,
    stripLeadingTabs,
  };
}

/**
 Splits a command at the shell's own separators, outside quotes.

 @param command - the command as written

 @returns its segments, in order, each with its words as written

 @example
 ```ts
 const segments = segmentsIn({ command: 'cd a && node b.ts | tee c' });
 ```
 */
export function segmentsIn({ command, }: { readonly command: string },): readonly string[] {
  /**
   The segments read so far, and the one being read.
   */
  const state = {
    segments: [] as string[],
    current: '',
    quote: '',
    escaped: false,
  };
  for (const character of withoutHereDocuments({ command, },)) {
    if (state.escaped) {
      state.current += character;
      state.escaped = false;
      continue;
    }
    if (character === '\\') {
      state.current += character;
      state.escaped = true;
      continue;
    }
    if (state.quote !== '') {
      state.current += character;
      if (character === state.quote) {
        state.quote = '';
      }
      continue;
    }
    if ((character === '\'') || (character === '"')
      || (character === '`')) {
      state.quote = character;
      state.current += character;
      continue;
    }
    if ((character === ';') || (character === '\n')
      || (character === '|')
      || (character === '&')) {
      state.segments
        .push(state.current,);
      state.current = '';
      continue;
    }
    state.current += character;
  }
  state.segments
    .push(state.current,);
  return state.segments;
}

/**
 Reads one segment's words, with their quotes taken off.

 @param segment - one segment of a command

 @returns its words, in order

 @example
 ```ts
 const words = wordsIn({ segment: 'node "a file.ts"' });
 ```
 */
export function wordsIn({ segment, }: { readonly segment: string },): readonly string[] {
  /**
   The words read so far, and the one being read.
   */
  const state = {
    words: [] as string[],
    current: '',
    quote: '',
    escaped: false,
    started: false,
  };
  for (const character of segment) {
    if (state.escaped) {
      state.current += character;
      state.escaped = false;
      state.started = true;
      continue;
    }
    if (character === '\\') {
      state.escaped = true;
      continue;
    }
    if (state.quote !== '') {
      if (character === state.quote) {
        state.quote = '';
      }
      else {
        state.current += character;
      }
      continue;
    }
    if ((character === '\'') || (character === '"')
      || (character === '`')) {
      state.quote = character;
      state.started = true;
      continue;
    }
    if ((character === ' ') || (character === '\t')
      || (character === '\r')) {
      if (state.started) {
        state.words
          .push(state.current,);
        state.current = '';
        state.started = false;
      }
      continue;
    }
    state.current += character;
    state.started = true;
  }
  if (state.started) {
    state.words
      .push(state.current,);
  }
  return state.words;
}

/**
 Says whether a word is a `NAME=value` assignment written before a program.

 @param word - one word of a segment

 @returns whether the shell reads it as an assignment

 @example
 ```ts
 const assignment = isAssignment('HOME=/tmp');
 ```
 */
export function isAssignment(word: string,): boolean {
  /**
   Where the equals sign stands.
   */
  const at = word.indexOf('=',);
  if (at <= 0) {
    return false;
  }
  /**
   The name before it.
   */
  const name = word.slice(
    0,
    at,
  );
  for (const [index, character,] of name.split('')
    .entries()) {
    /**
     Whether this character may stand in a name.
     */
    const isName = ((character >= 'A') && (character <= 'Z'))
      || ((character >= 'a') && (character <= 'z'))
      || ((character >= '0') && (character <= '9'))
      || (character === '_');
    if ((!isName) || ((index === 0) && ((character >= '0') && (character <= '9')))) {
      return false;
    }
  }
  return true;
}

/**
 Says whether a word is an option rather than a program or a value.

 @param word - one word of a segment

 @returns whether it begins with a dash

 @example
 ```ts
 const option = isAnOption('--quiet');
 ```
 */
export function isAnOption(word: string,): boolean {
  return word.startsWith('-',);
}

/**
 Reads a word's own name, without the folders before it.

 @param word - a word as written

 @returns its last path segment

 @example
 ```ts
 const name = basenameOf('/usr/bin/node');
 ```
 */
export function basenameOf(word: string,): string {
  /**
   Where the last separator stands.
   */
  const at = word.lastIndexOf('/',);
  return (at === (-1)) ? word : word.slice(at + 1,);
}

/**
 Skips the options at the cursor, with the values of the options that take one.

 @param words - the segment's words

 @param cursor - where the next word is read from, advanced past the options

 @param stopAtDoubleDash - whether a `--` ends the options and leaves the word after it to the caller

 @returns where the next word that is no option stands, or that a `--` was reached

 @example
 ```ts
 const after = skipOptions({ words, cursor, stopAtDoubleDash: true });
 ```
 */
export function skipOptions({
  words,
  cursor,
  stopAtDoubleDash,
}: {
  readonly words: readonly string[];
  readonly cursor: { at: number };
  readonly stopAtDoubleDash: boolean;
},): { readonly kind: 'at a word' } | {
  readonly kind: 'after a double dash';
  readonly at: number
} {
  while (cursor.at < words.length) {
    /**
     The word this step reads.
     */
    const word = words[cursor.at] ?? '';
    if (stopAtDoubleDash && (word === '--')) {
      return {
        kind: 'after a double dash',
        at: cursor.at + 1,
      };
    }
    if (!isAnOption(word,)) {
      return { kind: 'at a word', };
    }
    cursor.at += 1;
    // An option that names a value takes the word after it, which is not the program.
    if ((!word.includes('=',)) && VALUE_OPTIONS.has(word,)) {
      cursor.at += 1;
    }
  }
  return { kind: 'at a word', };
}
