import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  createLogger,
  type Level,
} from '@monochromatic-dev/logger-core/ts';
import {
  createFileSink,
  NO_LOG_RETENTION_SWEEP,
} from '@monochromatic-dev/logger-core/ts/node.ts';
import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';

import { scratchDir, } from '../scratch-dir.test-fixture.ts';

//region Run log write
// WRITES A RUN'S RECORDS THE WAY A RUN DOES: through the shared logger and its file sink, into the folder a pass keeps
// them in. Nothing here writes a record by hand, so a report test reads files whose shape the logger decided.
//
// A CASE STATES ITS LINES AS THE CONSOLE PRINTS THEM, `[level] [ISO instant] message`, because that is the text every
// report parser was written against and every case was first written in. A line in that form is logged at its level
// and its instant; a line that is not in that form (a bare line, a cut one, one with an instant the logger would
// never write) stands for a foreign or damaged line and is written into the file as it is, in its place.

/**
 A line in the form the console sink prints.
 */
const CONSOLE_LINE = /^\[(trace|debug|info|warn|error|fatal)\] \[(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z)\] (.*)$/u;

/**
 Builds the lines of a record file from lines stated as the console prints them.

 @param sinon - calling case's own sandbox (`ctx.sinon`), whose stub of `Date.now` gives each record its instant
 @param lines - lines the file holds, in order

 @returns Lines of the file: a record the logger wrote for each console-form line, the other lines as they stand

 @example
 ```ts
 const fileLines = await recordFileLines({ sinon: ctx.sinon, lines: ['[info] [2026-10-23T12:00:00.000Z] purr',], },);
 ```
 */
async function recordFileLines(
  {
    sinon,
    lines,
  }: {
    readonly sinon: DisposableSandbox;
    readonly lines: readonly string[];
  },
): Promise<readonly string[]> {
  await using scratch = await scratchDir({ prefix: 'run-log-build-', },);
  const { logger, } = createLogger({
    sinks: [createFileSink({
      directory: scratch.path,
      retention: NO_LOG_RETENTION_SWEEP,
      environment: {},
    },),],
  },);

  for (const line of lines) {
    /**
     Parts of the line when it is in the console's form.
     */
    const parts = CONSOLE_LINE.exec(line,);
    if (parts === null)
      continue;
    /**
     Pieces of the match, present whenever it matched.
     */
    const [, level = '', stamp = '', message = '',] = parts;
    /**
     Stub giving the record the instant its line names.
     */
    const now = sinon.stub(
      Date,
      'now',
    )
      .returns(Date.parse(stamp,),);
    try {
      logger[level as Level](message,);
    } finally {
      now.restore();
    }
  }
  await logger.flush();

  /**
   Day directories the sink made under the scratch directory.
   */
  const days = await readdir(scratch.path,);
  /**
   Lines the sink wrote, in order.
   */
  const written: string[] = [];
  for (const day of days.toSorted()) {
    /* oxlint-disable no-await-in-loop -- one directory per day, read in name order so the lines stay in order */
    for (const file of (await readdir(join(
      scratch.path,
      day,
    ),)).toSorted()) {
      written.push(...(await readFile(
        join(
          scratch.path,
          day,
          file,
        ),
        'utf8',
      )).split('\n',)
        .filter(function present(line,): boolean {
          return line !== '';
        },),);
    }
    /* oxlint-enable no-await-in-loop */
  }

  /**
   Records handed out in order as the console-form lines are met.
   */
  const records = [...written,];
  return lines.map(function placed(line,): string {
    return (CONSOLE_LINE.test(line,) ? records.shift() : undefined) ?? line;
  },);
}

/**
 Writes a record file of the given lines at an exact path.

 @param sinon - calling case's own sandbox (`ctx.sinon`)
 @param path - file to write, whose name ends `.log.jsonl`
 @param lines - lines it holds, as the console prints them

 @returns Path of the file

 @example
 ```ts
 const path = await writeRecordFile({ sinon: ctx.sinon, path: join(dir, 'a.log.jsonl'), lines: [], },);
 ```
 */
export async function writeRecordFile(
  {
    sinon,
    path,
    lines,
  }: {
    readonly sinon: DisposableSandbox;
    readonly path: string;
    readonly lines: readonly string[];
  },
): Promise<string> {
  await writeFile(
    path,
    `${(await recordFileLines({ sinon, lines, },)).join('\n',)}\n`,
  );
  return path;
}

/**
 Writes a run folder the way a pass leaves one: the records under `log/<UTC day>/`.

 @param sinon - calling case's own sandbox (`ctx.sinon`)
 @param dir - directory the run folder is made in
 @param name - run folder's name
 @param day - UTC day directory the file sits in
 @param lines - lines the one file holds, as the console prints them

 @returns Path of the run folder

 @example
 ```ts
 const runDir = await writeRunFolder({ sinon: ctx.sinon, dir, name: 'corpus-pass-tabby', day: '2026-10-23', lines, },);
 ```
 */
export async function writeRunFolder(
  {
    sinon,
    dir,
    name,
    day,
    lines,
  }: {
    readonly sinon: DisposableSandbox;
    readonly dir: string;
    readonly name: string;
    readonly day: string;
    readonly lines: readonly string[];
  },
): Promise<string> {
  /**
   Run folder.
   */
  const runDir = join(
    dir,
    name,
  );
  /**
   Directory of the one day.
   */
  const dayDir = join(
    runDir,
    'log',
    day,
  );
  await mkdir(
    dayDir,
    { recursive: true, },
  );
  await writeRecordFile({
    sinon,
    path: join(
      dayDir,
      `${day}T08-00-00.000Z.00000000-0000-4000-8000-000000000001.log.jsonl`,
    ),
    lines,
  },);
  return runDir;
}

//endregion Run log write
