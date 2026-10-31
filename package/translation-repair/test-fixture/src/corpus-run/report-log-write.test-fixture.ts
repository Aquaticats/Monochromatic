import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { parseConsoleLine, } from './console-line-parse.test-fixture.ts';

//region Report log write
// Writes a record file the way a run's own ends, with a newline after the last line, for the cases of the reports that
// total a run's records.
//
// A CASE STATES ITS LINES AS THE CONSOLE PRINTS THEM, `[level] [ISO instant] message`, which is the text every report
// parser was written against. A line in that form is written as the record the logger writes for it, a JSON object of
// `level`, `message` and `timestamp` in that order, which is the bytes the file sink writes (`report-log-write.unit.test.ts`
// holds the two side by side); any other line (a bare line, a cut one, one whose instant is not as the logger writes
// one) stands for a foreign or damaged line and is written as it is. A case that wants the logger itself to write
// the records uses `run-log-write.test-fixture.ts`.
//
// THE FILE IS NAMED AS THE RECORD FILE IT IS: a name ending `.log` gains `.jsonl`, so a case keeps naming the run it
// writes by the stem it always used.

/**
 Renders one case line as the line of a record file.

 @param line - line as the console prints it, or a foreign or cut line

 @returns Record the logger writes for it, or the line as it stands

 @example
 ```ts
 const written = recordFileLine({ line: '[info] [2026-10-23T12:00:00.000Z] purr', },);
 ```
 */
function recordFileLine({ line, }: { readonly line: string; },): string {
  /**
   Parts of the line when it is in the console's form.
   */
  const parts = parseConsoleLine({ line, },);
  if (parts === 'not-console-form')
    return line;
  return JSON.stringify({
    level: parts.level,
    message: parts.message,
    timestamp: parts.at,
  },);
}

/**
 Writes a record file of the given lines, ended by a newline as a run's own is.

 @param dir - directory to write in

 @param name - file's name, which gains `.jsonl` where it ends `.log`

 @param lines - lines it holds

 @returns Path of the file

 @example
 ```ts
 const path = await writeLog({ dir, name: 'a.log', lines: ['the cat slept',], },);
 ```
 */
export async function writeLog(
  {
    dir,
    name,
    lines,
  }: {
    readonly dir: string;
    readonly name: string;
    readonly lines: readonly string[];
  },
): Promise<string> {
  /**
   Where the file goes.
   */
  const path = join(
    dir,
    name.endsWith('.log',) ? `${name}.jsonl` : name,
  );
  await writeFile(
    path,
    `${lines.map(function written(line,): string {
      return recordFileLine({ line, },);
    },)
      .join('\n',)}\n`,
  );
  return path;
}

//endregion Report log write
