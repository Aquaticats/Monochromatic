/**
 The processes one Pi run has spawned, as `ps` reports them: how old each is, what state it is in, what the kernel has it
 waiting on, and the command it runs.

 Arguments are read and the environment is not: `ps` is asked for the command lines only, and no process's environment is read.

 The listing is the seam: a test hands over the text `ps` would have printed and never needs a real process table.

 @module
 */

import type { RunCommand, } from './run-command.ts';

/**
 The columns asked of `ps`, each with a `=` suffix so no header line is printed.

 `wchan` is the kernel function a process is waiting in, which tells a run waiting on a test suite from one waiting on a child of
 its own. Its field is wide enough for the longest name the kernel prints.
 */
const PROCESS_COLUMNS = 'pid=,ppid=,etimes=,stat=,wchan:18=,args=';

/**
 How many columns a line has before its command line starts.
 */
const COLUMNS_BEFORE_THE_COMMAND = 5;

/**
 One process, as the listing describes it.
 */
export type ProcessUnderARun = {
  /**
   The process number.
   */
  readonly pid: number;

  /**
   The process number of the process that started it.
   */
  readonly parentPid: number;

  /**
   How long it has been running, in seconds.
   */
  readonly elapsedSeconds: number;

  /**
   Its state, as `ps` prints it: `S` for sleeping, `R` for running, `D` for waiting on a device, and so on.
   */
  readonly state: string;

  /**
   The kernel function it is waiting in, or `-` when it is not waiting in one.
   */
  readonly waitingIn: string;

  /**
   The command and its arguments, with the environment left out.
   */
  readonly command: string;
};

/**
 A line of the listing that could not be read, as a symbol so no process's command can be mistaken for it.
 */
export const NOT_A_PROCESS_LINE: unique symbol = Symbol('ps line describes no process',);

/**
 A column that holds no whole number, as a symbol so no number can be mistaken for it.
 */
const NOT_A_NUMBER: unique symbol = Symbol('ps column holds no whole number',);

/**
 A line with too few columns to describe a process, as a symbol so no line's columns can be mistaken for it.
 */
const NOT_ENOUGH_COLUMNS: unique symbol = Symbol('ps line has fewer columns than a process needs',);

/**
 One line's first columns and the command that follows them.
 */
type ColumnsOfALine = {
  /**
   The columns read, in order.
   */
  readonly words: readonly string[];

  /**
   Everything after those columns, which is the command and its arguments.
   */
  readonly rest: string;
};

/**
 Reads whether a character separates columns.

 `ps` separates its columns with spaces, and a command line keeps its own whitespace after them.

 @param character - the character to read

 @returns whether it separates columns

 @example
 ```ts
 const separates = isBlank(' ',);
 ```
 */
function isBlank(character: string,): boolean {
  return (character === ' ') || (character === '\t');
}

/**
 A line of the listing that could not be read.
 */
export class ProcessListingUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ProcessListingUnreadableError';
}

/**
 Reads a line into its first columns and the rest, which is the command and holds spaces of its own.

 One pass over the line, so a command line of any length costs the time it takes to read once.

 @param line - the line to read

 @param columns - how many columns come before the rest

 @returns those columns and everything after them, or {@link NOT_ENOUGH_COLUMNS}

 @example
 ```ts
 const read = columnsOf({ line: ' 1 0 30 Ss - /usr/bin/init', columns: 5, },);
 ```
 */
function columnsOf({
  line,
  columns,
}: {
  readonly line: string;
  readonly columns: number;
},): ColumnsOfALine | typeof NOT_ENOUGH_COLUMNS {
  /**
   The columns read so far.
   */
  const words: string[] = [];
  /**
   Where the reader is in the line. One field of an object, so the helper that moves it is a function rather than a statement
   of this one.
   */
  const position = { at: 0, };
  /**
   Reads one column, leaving the reader after it.

   @returns whether the line had another column to read
   */
  function takeWord(): boolean {
    while ((position.at < line.length) && isBlank(line[position.at] ?? '',)) {
      position.at += 1;
    }
    /**
     Where the column's text starts.
     */
    const start = position.at;
    while ((position.at < line.length) && (!isBlank(line[position.at] ?? '',))) {
      position.at += 1;
    }
    if (start === position.at) {
      return false;
    }
    words.push(line.slice(
      start,
      position.at,
    ),);
    return true;
  }

  for (let taken = 0; taken < columns; taken += 1) {
    if (!takeWord()) {
      return NOT_ENOUGH_COLUMNS;
    }
  }
  return {
    words,
    rest: line.slice(position.at,)
      .trim(),
  };
}

/**
 Reads a column that should hold a whole number.

 @param word - the column's text

 @returns the number, or {@link NOT_A_NUMBER} when the column holds anything else

 @example
 ```ts
 const pid = numberOfWord('1234',);
 ```
 */
function numberOfWord(word: string,): number | typeof NOT_A_NUMBER {
  if (word === '') {
    return NOT_A_NUMBER;
  }
  for (const character of word) {
    if ((character < '0') || (character > '9')) {
      return NOT_A_NUMBER;
    }
  }
  return Number(word,);
}

/**
 Reads one line of a `ps` listing.

 @param line - one process line

 @returns the process, or {@link NOT_A_PROCESS_LINE}

 @example
 ```ts
 const entry = processOfLine(' 1 0 30 Ss - /usr/bin/init',);
 ```
 */
function processOfLine(line: string,): ProcessUnderARun | typeof NOT_A_PROCESS_LINE {
  /**
   The line broken into its columns and its command.
   */
  const read = columnsOf({
    line,
    columns: COLUMNS_BEFORE_THE_COMMAND,
  },);
  if ((typeof read) === 'symbol') {
    return NOT_A_PROCESS_LINE;
  }
  /**
   The columns in the order `ps` was asked for them, with a column the line leaves out read as empty text.
   */
  const [pidWord = '', parentWord = '', elapsedWord = '', state = '', waitingIn = '',] = read.words;
  /**
   The process number.
   */
  const pid = numberOfWord(pidWord,);
  /**
   The process number of the process that started it.
   */
  const parentPid = numberOfWord(parentWord,);
  /**
   How long it has been running.
   */
  const elapsedSeconds = numberOfWord(elapsedWord,);
  if (
    ((typeof pid) === 'symbol')
      || ((typeof parentPid) === 'symbol')
      || ((typeof elapsedSeconds) === 'symbol')
  ) {
    return NOT_A_PROCESS_LINE;
  }
  return {
    pid,
    parentPid,
    elapsedSeconds,
    state,
    waitingIn,
    command: read.rest,
  };
}

/**
 Reads a whole `ps` listing.

 @param listing - what `ps` printed

 @returns every process the listing described

 @example
 ```ts
 const table = parseProcessListing({ listing: '    1       0  30 Ss   -      /usr/bin/init', },);
 ```
 */
export function parseProcessListing({
  listing,
}: {
  readonly listing: string;
},): readonly ProcessUnderARun[] {
  /**
   The processes read so far.
   */
  const table: ProcessUnderARun[] = [];
  for (const line of listing.split(
    '\n',
  )) {
    /**
     The entry this line describes, when it describes one.
     */
    const entry = processOfLine(line,);
    if ((typeof entry) !== 'symbol') {
      table.push(entry,);
    }
  }
  return table;
}

/**
 Reads the processes under one process, in the order they were listed.

 @param table - the listing

 @param rootPid - the process whose descendants are wanted; it is not one of them

 @returns every descendant, and not the process itself

 @example
 ```ts
 const under = processesUnder({ table, rootPid: 10, },);
 ```
 */
export function processesUnder({
  table,
  rootPid,
}: {
  readonly table: readonly ProcessUnderARun[];
  readonly rootPid: number;
},): readonly ProcessUnderARun[] {
  /**
   The processes found so far.
   */
  const found: ProcessUnderARun[] = [];
  /**
   The process numbers already taken, so a table that names a process as its own parent, or two as each other's, is read once
   rather than followed for ever.
   */
  const taken = new Set([rootPid,],);
  /**
   The processes whose children are still to be looked for. A work stack, so a deep tree costs no stack frames of its own.
   */
  const parents = [rootPid,];
  while ((parents.length > 0) && (found.length < table.length)) {
    /**
     The process whose children are looked for next.
     */
    const parent = parents.pop();
    for (const candidate of table.filter(function isChildOf(entry,) {
      return entry.parentPid === parent;
    },)) {
      if (taken.has(candidate.pid,)) {
        continue;
      }
      taken.add(candidate.pid,);
      found.push(candidate,);
      parents.push(candidate.pid,);
    }
  }
  return found;
}

/**
 Reads the process table, asking `ps`.

 @param run - how a command is run, which a test replaces

 @returns what to await for the table at that moment

 @throws ProcessListingUnreadableError when `ps` could not be run, or refused

 @example
 ```ts
 const table = startProcessTable({ run: runCommandWithChildProcess, },);
 const processes = await table();
 ```
 */
export function startProcessTable({
  run,
}: {
  readonly run: RunCommand;
},): () => Promise<readonly ProcessUnderARun[]> {
  return async function processTable(): Promise<readonly ProcessUnderARun[]> {
    /**
     What `ps` printed.
     */
    const listing = await run({
      command: 'ps',
      words: [
        '-e',
        '--no-headers',
        '-o',
        PROCESS_COLUMNS,
      ],
    },);
    if (listing.status !== 0) {
      throw new ProcessListingUnreadableError(
        `ps could not list the processes: ${listing.problem
          ?? listing.stderr
            .trim()}`,
      );
    }
    return parseProcessListing({ listing: listing.stdout, },);
  };
}
