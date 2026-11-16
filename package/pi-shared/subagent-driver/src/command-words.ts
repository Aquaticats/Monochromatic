/**
 Reads a command as a shell would split it, just far enough to tell which words are run: the program of each segment, and the script
 an interpreter or a shell runs.

 A word that merely names a path is not run. `ls dist/x.js` names a program's path; `node dist/x.js` and `dist/x.js --version` run
 it, and a script that a command runs is itself run, so its own text is what says whether it starts a built program.

 This is a reading of the common shell forms, never a shell: pipes, `&&`, `||`, `;`, newlines, `VAR=value` prefixes, and the
 wrappers a review command uses (`sudo`, `env`, `timeout`, `nice`, `nohup`, `exec`, `command`, `xargs`, `systemd-run`) are
 understood, and anything else is left as the program it is written as.

 @module
 */

/**
 No program in a segment, as a symbol so that no word can stand for it.
 */
const NO_PROGRAM: unique symbol = Symbol('segment runs no program',);

/**
 No script in an interpreter's words, as a symbol so that no word can stand for it.
 */
const NO_SCRIPT: unique symbol = Symbol('interpreter runs no script file',);

/**
 One word a command runs, and whether it is the program of its segment or a script an interpreter runs.
 */
export type ExecutedWord = {
  /**
   The word, with its quotes taken off.
   */
  readonly word: string;

  /**
   What the word is: a segment's program, or the script an interpreter runs.
   */
  readonly role: 'program' | 'script';
};

/**
 The programs that build something, which a command that runs one is a build command.

 `mise` is here because every build task of this repository runs through it; its own arguments say which task, and a reading that
 did not count it would find no build in the commands this repository's reviews actually run.
 */
const BUILD_TOOLS: ReadonlySet<string> = new Set([
  'cargo',
  'cargo-nextest',
  'mise',
  'rolldown',
  'tsc',
  'pnpm',
  'npm',
  'yarn',
  'bun',
  'nextest',
]);

/**
 The interpreters whose next word is a script file the command runs.
 */
const INTERPRETERS: ReadonlySet<string> = new Set([
  'bash',
  'sh',
  'dash',
  'zsh',
  'fish',
  'node',
  'nodejs',
  'bun',
  'deno',
  'python',
  'python3',
  'perl',
  'ruby',
]);

/**
 The wrappers whose own words come before the program a segment runs.
 */
const WRAPPERS: ReadonlySet<string> = new Set([
  'sudo',
  'env',
  'timeout',
  'nice',
  'nohup',
  'exec',
  'command',
  'xargs',
  'systemd-run',
]);

/**
 The wrapper options that take the word after them as their value, which is not the program.
 */
const VALUE_OPTIONS: ReadonlySet<string> = new Set([
  '-s',
  '--signal',
  '-k',
  '--kill-after',
  '-u',
  '--unset',
  '-n',
  '-I',
  '-P',
  '-L',
  '--unit',
  '--description',
  '--slice',
  '--working-directory',
]);

/**
 Splits a command at the shell's own separators, outside quotes.

 @param command - the command as written

 @returns its segments, in order, each with its words as written

 @example
 ```ts
 const segments = segmentsIn({ command: 'cd a && node b.ts | tee c' });
 ```
 */
function segmentsIn({ command, }: { readonly command: string },): readonly string[] {
  /**
   The segments read so far, and the one being read.
   */
  const state = {
    segments: [] as string[],
    current: '',
    quote: '',
    escaped: false,
  };
  for (const character of command) {
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
function wordsIn({ segment, }: { readonly segment: string },): readonly string[] {
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
function isAssignment(word: string,): boolean {
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
function isAnOption(word: string,): boolean {
  return word.startsWith('-',);
}

/**
 Skips the options at the cursor, with the values of the options that take one.

 @param words - the segment's words

 @param cursor - where the next word is read from, advanced past the options

 @param stopAtDoubleDash - whether a `--` ends the options and leaves the word after it to the caller

 @returns where the next word that is no option stands, or nothing when a `--` was reached

 @example
 ```ts
 const at = skipOptions({ words, cursor, stopAtDoubleDash: true });
 ```
 */
function skipOptions({
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

/**
 Reads the program one segment runs, after its assignments and the wrappers before it.

 @param words - the segment's words

 @returns the program's word, or nothing when the segment runs no program this reader can name

 @example
 ```ts
 const program = programIn({ words: ['timeout', '60', 'node', 'a.ts'] });
 ```
 */
function programIn({ words, }: { readonly words: readonly string[] },): string | typeof NO_PROGRAM {
  /**
   Where the next word is read from.
   */
  const cursor = { at: 0, };
  while ((cursor.at < words.length) && (isAssignment(words[cursor.at] ?? '',))) {
    cursor.at += 1;
  }
  /**
   The first word that is no assignment.
   */
  const head = words[cursor.at];
  if ((head === undefined) || (head === '')) {
    return NO_PROGRAM;
  }
  if (!WRAPPERS.has(head,)) {
    return head;
  }
  cursor.at += 1;
  // `env HOME=/tmp cargo build` sets variables after the wrapper and before the program.
  while ((cursor.at < words.length) && (isAssignment(words[cursor.at] ?? '',))) {
    cursor.at += 1;
  }
  /**
   Where the wrapper's own options end.
   */
  const afterOptions = skipOptions({
    words,
    cursor,
    stopAtDoubleDash: head === 'systemd-run',
  },);
  if (afterOptions.kind === 'after a double dash') {
    return words[afterOptions.at] ?? NO_PROGRAM;
  }
  if (head === 'timeout') {
    // The first word after the options is how long the command may take; the program follows it.
    cursor.at += 1;
    skipOptions({
      words,
      cursor,
      stopAtDoubleDash: false,
    },);
  }
  return words[cursor.at] ?? NO_PROGRAM;
}

/**
 Reads the script an interpreter word runs, when the word after an interpreter is a script file rather than inline code.

 @param words - the segment's words after the interpreter's own word

 @returns the script's word, or nothing when the interpreter runs inline code

 @example
 ```ts
 const script = scriptIn({ words: ['--input-type=module', 'repro.ts'] });
 ```
 */
function scriptIn({ words, }: { readonly words: readonly string[] },): string | typeof NO_SCRIPT {
  /**
   Where the next word is read from.
   */
  const cursor = { at: 0, };
  while (cursor.at < words.length) {
    /**
     The word this step reads.
     */
    const word = words[cursor.at] ?? '';
    if ((word === '-e') || (word === '--eval')
      || (word === '-c')
      || (word === '-p')
      || (word === '--print')) {
      return NO_SCRIPT;
    }
    if (!isAnOption(word,)) {
      return word;
    }
    cursor.at += 1;
    if ((!word.includes('=',)) && VALUE_OPTIONS.has(word,)) {
      cursor.at += 1;
    }
  }
  return NO_SCRIPT;
}

/**
 Reads every word a command runs: each segment's program, and the script an interpreter or shell runs.

 @param command - the command as written

 @returns the executed words, each once, in the order they run

 @example
 ```ts
 const ran = executedWordsIn({ command: 'cd a && node repro.ts' });
 ```
 */
export function executedWordsIn({
  command,
}: {
  readonly command: string;
},): readonly ExecutedWord[] {
  /**
   The executed words read so far.
   */
  const executed: ExecutedWord[] = [];
  for (const segment of segmentsIn({ command, },)) {
    /**
     The segment's words.
     */
    const words = wordsIn({ segment, },);
    /**
     The program this segment runs.
     */
    const program = programIn({ words, },);
    if (program === NO_PROGRAM) {
      continue;
    }
    executed.push({
      word: program,
      role: 'program',
    },);
    if (!INTERPRETERS.has(basenameOf(program,),)) {
      continue;
    }
    /**
     Where the interpreter's word stands in the segment.
     */
    const at = words.indexOf(program,);
    /**
     The script it runs, when it runs one.
     */
    const script = scriptIn({ words: words.slice(at + 1,), },);
    if (script !== NO_SCRIPT) {
      executed.push({
        word: script,
        role: 'script',
      },);
    }
  }
  return executed;
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
function basenameOf(word: string,): string {
  /**
   Where the last separator stands.
   */
  const at = word.lastIndexOf('/',);
  return (at === (-1)) ? word : word.slice(at + 1,);
}

/**
 Says whether a command runs a program that builds something.

 @param command - the command as written

 @returns whether a build tool is a segment's program

 @example
 ```ts
 const builds = buildToolRanIn({ command: 'mise run //package/a:native:build' });
 ```
 */
export function buildToolRanIn({ command, }: { readonly command: string },): boolean {
  return executedWordsIn({ command, },)
    .some(function isABuildTool(ran,) {
      return (ran.role === 'program') && BUILD_TOOLS.has(basenameOf(ran.word,),);
    },);
}

/**
 Says whether a script's own text runs a build tool.

 @param text - a script's text

 @returns whether a build tool is named in a position this reader reads as a program

 @example
 ```ts
 const builds = scriptRunsABuildTool({ text: 'exec cargo build' });
 ```
 */
export function scriptRunsABuildTool({ text, }: { readonly text: string },): boolean {
  return text.split('\n',)
    .some(function lineBuilds(line,) {
      return buildToolRanIn({ command: line, },);
    },);
}

/**
 How much of a command a reason quotes, so one line stays readable while still naming the command.
 */
export const COMMAND_IN_A_REASON_CHARACTERS = 160;

/**
 Collapses the runs of whitespace in a command, so a reviewer's line breaks and a session's single line compare. A backslash that
 ends a line joins it to the next in the shell, so it is dropped with the line break it ends.

 @param text - the command as written

 @returns the text with each run of whitespace as one space

 @example
 ```ts
 const same = collapsed('  cd a \\\n  && git status ');
 ```
 */
export function collapsed(text: string,): string {
  /* oxlint-disable-next-line no-restricted-syntax/no-regex -- a run of whitespace in one command line; the pattern has one
     quantifier, no backreference and no alternation, and the input is one command's own text */
  return text.replaceAll(
    /\\\s+/gu,
    ' ',
  )
    .replaceAll(
    /\s+/gu,
    ' ',
    )
    .trim();
}

/**
 Cuts a command to the room a reason line has.

 @param command - the command as written

 @returns the command, cut with an ellipsis when it is longer

 @example
 ```ts
 const short = quoted({ command: 'mise run //package/a:native:build' });
 ```
 */
export function quoted({ command, }: { readonly command: string },): string {
  return (command.length <= COMMAND_IN_A_REASON_CHARACTERS)
    ? command
    : `${command.slice(
      0,
      COMMAND_IN_A_REASON_CHARACTERS,
    )}...`;
}
