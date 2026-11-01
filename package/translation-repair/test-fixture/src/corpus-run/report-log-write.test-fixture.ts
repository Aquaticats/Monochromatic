import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

//region Report log write
// Writes a log file the way a run's own log ends, with a newline after the
// last line, for the cases of the reports that total logs.

/**
 Writes a log of the given lines, ended by a newline as a run's own log is.

 @param dir - directory to write in

 @param name - log's file name

 @param lines - lines it holds

 @returns Path of the log

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
   Where the log goes.
   */
  const path = join(
    dir,
    name,
  );
  await writeFile(
    path,
    `${lines.join('\n',)}\n`,
  );
  return path;
}

//endregion Report log write
