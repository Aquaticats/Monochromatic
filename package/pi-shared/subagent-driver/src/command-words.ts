/**
 Reads a command as a shell would split it, just far enough to tell which words are run: the program of each segment after the
 wrappers in front of it, the script an interpreter or a shell runs, the text `bash -c` runs, and the build task a `mise run` or
 `cargo` call asks for.

 A word that merely names a path is not run. `ls dist/x.js` names a program's path; `node dist/x.js`, `timeout 60 dist/x.js` and
 `systemd-run --user --scope -- timeout 60 dist/x.js` run it. Wrappers nest, which is why the program is read through every wrapper in
 turn rather than through one.

 This is a reading of the common shell forms, never a shell: pipes, `&&`, `||`, `;`, newlines, here-documents, `VAR=value` prefixes,
 and the wrappers a review command uses (`sudo`, `env`, `time`, `timeout`, `nice`, `nohup`, `exec`, `command`, `xargs`, `ionice`,
 `stdbuf`, `systemd-run`, and `mise exec --`) are understood; anything else is left as the program it is written as.

 @module
 */
import {
  VALUE_OPTIONS,
  basenameOf,
  isAssignment,
  isAnOption,
  segmentsIn,
  skipOptions,
  wordsIn,
} from './command-line.ts';

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
 The words of a `mise run` task that make it a build or a run rather than a formatting or listing task.
 */
const BUILD_TASK_WORDS: readonly string[] = [
  'build',
  'test',
  'bench',
  'run',
  'check',
  'conformance',
];

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
 The shells whose `-c` word is a command string this reader reads as a command of its own.
 */
const SHELLS: ReadonlySet<string> = new Set([
  'bash',
  'sh',
  'dash',
  'zsh',
  'fish',
]);

/**
 The wrappers whose own words come before the program a segment runs.
 */
const WRAPPERS: ReadonlySet<string> = new Set([
  'sudo',
  'env',
  'time',
  'timeout',
  'nice',
  'nohup',
  'exec',
  'command',
  'xargs',
  'ionice',
  'stdbuf',
  'systemd-run',
]);

/**
 How many wrappers one program word is read through before the reading stops.
 */
const WRAPPER_DEPTH = 8;

/**
 How deep a shell command inside a shell command is followed.
 */
const INLINE_DEPTH = 4;

/**
 No program in a segment, as a symbol so that no word can stand for it.
 */
const NO_PROGRAM: unique symbol = Symbol('segment runs no program',);

/**
 No script in an interpreter's words, as a symbol so that no word can stand for it.
 */
const NO_SCRIPT: unique symbol = Symbol('interpreter runs no script file',);

/**
 No command string in an interpreter's words, as a symbol so that no word can stand for it.
 */
const NO_INLINE_COMMAND: unique symbol = Symbol('interpreter runs no command string',);

/**
 Reads the program one segment runs, after every wrapper in front of it, and through `mise exec --`.

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
  for (let depth = 0; depth < WRAPPER_DEPTH; depth += 1) {
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
    /**
     The program's name, which decides whether it is a wrapper.
     */
    const name = basenameOf(head,);
    if ((name === 'mise') && ((words[cursor.at + 1] === 'exec') || (words[cursor.at + 1] === 'x'))) {
      // `mise exec -- node dist` runs the node program the rest of the words name.
      cursor.at += 2;
      if (words[cursor.at] === '--') {
        cursor.at += 1;
      }
      continue;
    }
    if (!WRAPPERS.has(name,)) {
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
      stopAtDoubleDash: name === 'systemd-run',
    },);
    if (afterOptions.kind === 'after a double dash') {
      cursor.at = afterOptions.at;
      continue;
    }
    if (name === 'timeout') {
      // The first word after the options is how long the command may take; the program follows it.
      cursor.at += 1;
      skipOptions({
        words,
        cursor,
        stopAtDoubleDash: false,
      },);
    }
    // Every other wrapper is now behind us; the word at the cursor may be another wrapper.
  }
  return NO_PROGRAM;
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
 Reads the command string a shell's `-c` word carries.

 @param words - the segment's words after the shell's own word

 @returns the command string, or nothing when the shell runs a script file

 @example
 ```ts
 const inline = inlineCommandIn({ words: ['-c', 'cd a && node b.ts'] });
 ```
 */
function inlineCommandIn({ words, }: { readonly words: readonly string[] },): string | typeof NO_INLINE_COMMAND {
  for (const [at, word,] of words.entries()) {
    if ((word === '-c') || (word === '--command')) {
      return words[at + 1] ?? NO_INLINE_COMMAND;
    }
    if ((word === '-e') || (word === '--eval')
      || (word === '-p')
      || (word === '--print')) {
      return NO_INLINE_COMMAND;
    }
    if (!isAnOption(word,)) {
      return NO_INLINE_COMMAND;
    }
  }
  return NO_INLINE_COMMAND;
}

/**
 Reads every word a command runs: each segment's program, the script an interpreter or shell runs, and the commands a shell's `-c`
 string carries.

 @param command - the command as written

 @param depth - how many shell command strings this reading is already inside

 @returns the executed words, each once, in the order they run

 @example
 ```ts
 const ran = executedWordsIn({ command: 'cd a && node repro.ts' });
 ```
 */
export function executedWordsIn({
  command,
  depth = 0,
}: {
  readonly command: string;
  readonly depth?: number;
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
    /**
     The program's name, which decides whether it is an interpreter.
     */
    const name = basenameOf(program,);
    if (!INTERPRETERS.has(name,)) {
      continue;
    }
    /**
     Where the interpreter's word stands in the segment.
     */
    const at = words.indexOf(program,);
    /**
     The words after the interpreter.
     */
    const after = words.slice(at + 1,);
    if ((SHELLS.has(name,)) && (depth < INLINE_DEPTH)) {
      /**
       The command string the shell's `-c` word carries.
       */
      const inline = inlineCommandIn({ words: after, },);
      if (inline !== NO_INLINE_COMMAND) {
        executed.push(...executedWordsIn({
          command: inline,
          depth: depth + 1,
        },),);
        continue;
      }
    }
    /**
     The script it runs, when it runs one.
     */
    const script = scriptIn({ words: after, },);
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
 Reads the task a `mise run` asks for.

 @param words - one segment's words

 @returns the task's word, or empty text when the segment is no `mise run`

 @example
 ```ts
 const task = miseRunTaskIn({ words: ['mise', 'run', '//package/a:build'] });
 ```
 */
function miseRunTaskIn({ words, }: { readonly words: readonly string[] },): string {
  /**
   The first word that is no assignment or wrapper before `mise`.
   */
  const start = words.findIndex(function isMise(word,) {
    return basenameOf(word,) === 'mise';
  },);
  if ((start === (-1)) || (words[start + 1] !== 'run')) {
    return '';
  }
  /**
   Where the task's word is read from.
   */
  const cursor = { at: start + 2, };
  skipOptions({
    words,
    cursor,
    stopAtDoubleDash: false,
  },);
  return words[cursor.at] ?? '';
}

/**
 Says whether a command runs a program or task that builds or runs something.

 @param command - the command as written

 @returns whether a build tool is a segment's program, and no wrapper hides it

 @example
 ```ts
 const builds = buildToolRanIn({ command: 'mise run //package/a:native:build' });
 ```
 */
export function buildToolRanIn({ command, }: { readonly command: string },): boolean {
  /**
   The segments of the command.
   */
  const segments = segmentsIn({ command, },);
  for (const segment of segments) {
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
    /**
     The program's name.
     */
    const name = basenameOf(program,);
    if (!BUILD_TOOLS.has(name,)) {
      continue;
    }
    if (name !== 'mise') {
      return true;
    }
    /**
     The task a `mise run` asks for, or empty text for any other `mise` word.
     */
    const task = miseRunTaskIn({ words, },);
    if (task === '') {
      // `mise exec -- cargo` is unwrapped to cargo, so a bare mise here is not a build by itself.
      continue;
    }
    /**
     The task's own text, which says whether it builds or runs.
     */
    const lowered = task.toLowerCase();
    if (BUILD_TASK_WORDS.some(function namesABuild(word,) {
      return lowered.includes(word,);
    },)) {
      return true;
    }
  }
  return false;
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
 The programs that build something, for a caller that reports which command ran one.

 @param program - a program word a command runs

 @returns whether the word names a build tool

 @example
 ```ts
 const builds = isABuildTool('cargo');
 ```
 */
export function isABuildTool(program: string,): boolean {
  return BUILD_TOOLS.has(basenameOf(program,),);
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
