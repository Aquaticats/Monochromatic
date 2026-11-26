/**
 Run the live quota-deadline check in a started Pi session, then say whether it passed.
 @module

 `verify-live-quota-deadline.ts` is the Pi extension that reads the endpoints; this file starts it, keeps its record where the
 package's docs point, prints the record's lines, and fails when the record does not show what the check is for:

 - every member whose provider publishes a quota endpoint answered with evidence, inside the shipped deadline;
 - every member whose provider publishes none sent no request, which is the empty evidence the shipped read answers with;
 - the control arm was abandoned at the deadline, so the bound can fire at all;
 - Pi exited on its own, and the extension wrote the whole record.

 The record path is `--record=<file>`, and the default is the dated file under `doc/quota-read-latency-records/`. A file already
 there is never overwritten: a reader who wants a second check of the same day names the file it goes to.

 Run it with `mise run //package/pi-plugin/rate-limit-fallback:verify:quota-deadline`.
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  access,
  mkdir,
  readFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';
import { RECORD_PATH_VARIABLE, } from './verify-live-quota-deadline.ts';

/**
 The package folder this file lives in, which its records sit under.
 */
const PACKAGE_FOLDER = fileURLToPath(new URL(
  '..',
  import.meta.url,
),);

/**
 The option that names the file the record goes to.
 */
const RECORD_OPTION = '--record=';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 One line of the check's record, whose fields this runner reads by name.
 */
type RecordLine = Readonly<Record<string, unknown>>;

/**
 Reads a value as a record of fields, which is what one line of the record is.

 @param value - the value to read

 @returns whether the value is a record, and if so the fields it holds

 @example
 ```ts
 const line = JSON.parse('{"kind":"done"}');
 if (!isRecord(line)) { return; }
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads the code the operating system gave a failed call, when it gave one.

 @param error - the value a catch block caught

 @returns the code, or empty text when the value carries none

 @example
 ```ts
 const code = codeOf(error);
 ```
 */
function codeOf(error: unknown,): string {
  if ((Error.isError(error,)) && ('code' in error)
    && ((typeof error.code) === 'string')) {
    return error.code;
  }
  return '';
}

/**
 Says whether a file is there.

 @param path - the file to look for

 @returns whether it exists

 @example
 ```ts
 const there = await fileIsThere({ path: '/tmp/check.jsonl' });
 ```
 */
async function fileIsThere({ path, }: { readonly path: string },): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (codeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Reads the file the record goes to: the one the caller named, or the dated file under the package's records folder.

 @param words - the command line's own words

 @returns the path the record goes to

 @throws Error when the option holds no path, or when the file it names is already there

 @example
 ```ts
 const recordPath = await recordPathIn({ words: ['--record=/tmp/check.jsonl'], });
 ```
 */
async function recordPathIn({ words, }: { readonly words: readonly string[] },): Promise<string> {
  /**
   The path the caller named, when it named one.
   */
  const named = words.find(function isTheRecordOption(word,) {
    return word.startsWith(RECORD_OPTION,);
  },);
  /**
   The day the check runs, as its record's own name carries it.
   */
  const day = new Date()
    .toISOString()
    .slice(
    0,
    'YYYY-MM-DD'.length,
    );
  /**
   Where the record goes: the caller's file, or this day's file under the docs.
   */
  const path = (named === undefined)
    ? join(
      PACKAGE_FOLDER,
      'doc',
      'quota-read-latency-records',
      `live-check-${day}.jsonl`,
    )
    : named.slice(RECORD_OPTION.length,);
  if (path === '') {
    throw new Error(`${RECORD_OPTION} was given no file to write the record to`,);
  }
  if (await fileIsThere({ path, },)) {
    throw new Error(`${path} already holds a record: name another file with ${RECORD_OPTION}<file> rather than replacing one`,);
  }
  return path;
}

/**
 Starts Pi with the check's extension, and waits for it to end.

 @param recordPath - the file the record goes to, which the session is told

 @returns Pi's exit status, and what it wrote to its standard error

 @example
 ```ts
 const ended = await runTheCheck({ recordPath: '/tmp/check.jsonl' });
 ```
 */
async function runTheCheck({ recordPath, }: { readonly recordPath: string },): Promise<{
  readonly status: number;
  readonly stderr: string
}> {
  /**
   The extension Pi loads, which is this package's own source file.
   */
  const extension = fileURLToPath(new URL(
    'verify-live-quota-deadline.ts',
    import.meta.url,
  ),);
  /**
   The Pi process, whose standard input is closed so it takes no prompt and ends when its session starts.
   */
  const child = spawn(
    'pi',
    [
      '--no-session',
      '--no-mcp',
      '--no-skills',
      '--no-prompt-templates',
      '--no-context-files',
      '--mode',
      'rpc',
      '--extension',
      extension,
    ],
    {
      env: {
        ...process.env,
        [RECORD_PATH_VARIABLE]: recordPath,
      },
      stdio: [
        'ignore',
        'ignore',
        'pipe',
      ],
    },
  );
  /**
   What Pi wrote to its standard error, drained while this runner waits for it to end, so neither waits on the other.
   */
  const [ , stderr,] = await Promise.all([
    once(
      child,
      'close',
    ),
    text(child.stderr,),
  ],);
  return {
    status: child.exitCode ?? 1,
    stderr,
  };
}

/**
 Reads the record's lines.

 @param recordPath - the file the record went to

 @returns one entry per line, and nothing for a file that was never written

 @example
 ```ts
 const lines = await recordLinesIn({ recordPath: '/tmp/check.jsonl' });
 ```
 */
async function recordLinesIn({ recordPath, }: { readonly recordPath: string },): Promise<readonly RecordLine[]> {
  if (!(await fileIsThere({ path: recordPath, },))) {
    return [];
  }
  /**
   The record as it stands, whose lines are JSON of unknown shape until each is read.
   */
  const shown = await readFile(
    recordPath,
    'utf8',
  );
  /**
   The lines that hold a record, each as the fields it carries.
   */
  const lines: RecordLine[] = [];
  for (const line of shown.split('\n',)) {
    if (line === '') {
      continue;
    }
    /**
     One line of the record, as JSON parsing gives it: a value of no known shape.
     */
    const parsed: unknown = JSON.parse(line,);
    if (isRecord(parsed,)) {
      lines.push(parsed,);
    }
  }
  return lines;
}

/**
 Says what the record fails to show, which is nothing when the check passed.

 @param lines - the record's lines

 @returns one line per thing the record does not show

 @example
 ```ts
 const problems = problemsIn({ lines });
 ```
 */
function problemsIn({ lines, }: { readonly lines: readonly RecordLine[] },): readonly string[] {
  /**
   The deadline the record states, which every live read is compared with.
   */
  const deadline = lines.find(function isTheDeadline(line,) {
    return line.kind === 'deadline';
  },);
  if (deadline === undefined) {
    return ['the record holds no deadline line, so the check did not read anything',];
  }
  /**
   The milliseconds the shipped deadline allows.
   */
  const deadlineMs = Number(deadline.deadlineMs,);
  /**
   What the record does not show, one line per problem.
   */
  const problems: string[] = [];
  /**
   Every live read the record holds.
   */
  const reads = lines.filter(function isALiveRead(line,) {
    return line.kind === 'live';
  },);
  if (reads.length === 0) {
    problems.push('the record holds no live read',);
  }
  if (lines.some(function isMissing(line,) {
    return line.kind === 'missing';
  },)) {
    problems.push('a pool member was not in the session\'s registry, so it was never read',);
  }
  for (const read of reads) {
    /**
     The milliseconds that read took.
     */
    const elapsed = Number(read.elapsedMs,);
    /**
     Where the line says that read is from.
     */
    const where = `${String(read.member,)} round ${String(read.round,)}`;
    if ((read.endpointExpected === true) && (read.outcome !== 'evidence')) {
      problems.push(`${where} answered ${String(read.outcome,)} instead of evidence`,);
    }
    if ((read.endpointExpected === false) && (read.outcome !== 'no-endpoint')) {
      problems.push(`${where} answered ${String(read.outcome,)} although its provider publishes no endpoint`,);
    }
    if (!(elapsed < deadlineMs)) {
      problems.push(`${where} took ${String(elapsed,)} ms, at or past the ${String(deadlineMs,)} ms deadline`,);
    }
  }
  /**
   The control arm's own line, which is what shows the deadline can fire.
   */
  const control = lines.find(function isTheControl(line,) {
    return line.kind === 'control';
  },);
  if (control === undefined) {
    problems.push('the record holds no control line, so nothing shows the deadline can abandon a read',);
  }
  else if (control.abandoned !== true) {
    problems.push(`the control read was not abandoned at the deadline: ${JSON.stringify(control,)}`,);
  }
  if (!lines.some(function isDone(line,) {
    return line.kind === 'done';
  },)) {
    problems.push('the record holds no done line, so the check did not finish',);
  }
  return problems;
}

/**
 Runs the check and reports what it found: the record's lines, then the problems that make it a failure.

 @param words - the command line's own words

 @throws Error when the record shows something other than what the check is for

 @example
 ```ts
 await main({ words: process.argv.slice(2) });
 ```
 */
async function main({ words, }: { readonly words: readonly string[] },): Promise<void> {
  /**
   The file the record goes to.
   */
  const recordPath = await recordPathIn({ words, },);
  await mkdir(
    dirname(recordPath,),
    { recursive: true, },
  );
  /**
   What Pi ended with, and what it wrote to its standard error.
   */
  const ended = await runTheCheck({ recordPath, },);
  /**
   The record's lines.
   */
  const lines = await recordLinesIn({ recordPath, },);
  for (const line of lines) {
    process.stdout
      .write(`${JSON.stringify(line,)}\n`,);
  }
  process.stderr
    .write(`record: ${recordPath}\n`,);
  /**
   What the record fails to show.
   */
  const problems = [
    ...((ended.status === 0) ? [] : [`pi exited with status ${String(ended.status,)}: ${ended.stderr
      .trim()}`,]),
    ...problemsIn({ lines, },),
  ];
  if (problems.length > 0) {
    throw new Error(`the live quota-deadline check did not pass:\n  ${problems.join('\n  ',)}`,);
  }
  process.stderr
    .write('the live quota-deadline check passed: every read came back inside the deadline, and the control was abandoned at it\n',);
}

await main({ words: process.argv
  .slice(2,), },);
