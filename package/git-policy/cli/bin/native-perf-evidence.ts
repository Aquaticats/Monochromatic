/**
 The evidence a latency run keeps beyond its timings: the phases of every wrapped commit and the commit service's own account
 of each landing.

 A paired delta says that a commit was slow; the client's timing record splits it into preparation, build, submit, gate and
 handoff, and the service's `answered` line splits the landing inside submit into its lock wait, reads, objects, intent
 journal, ref update and index follow-up. Both live in the scratch tree, so they are read before it is removed.
 */
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

/**
 Name of the file in the scratch root that receives the standard error of every commit service of the run, where a service
 that is not under a service manager writes its lines.
 */
export const SERVICE_LOG_NAME = 'service.log';

/**
 Name of the commit timing file in a Git directory.
 */
const TIMING_FILE = 'cli-git-commit-timing.jsonl';

/**
 What a run keeps: per scenario the parsed timing records of its wrapped repository, and the service's landing lines.
 */
export type Evidence = {
  readonly commitTiming: Readonly<Record<string, readonly unknown[]>>;
  readonly serviceLandings: readonly string[];
};

/**
 Parse the lines of a JSON-lines file, skipping a line that is not a record.

 @param text - file content

 @returns the parsed records in file order

 @example
 ```ts
 parseRecords('{"a":1}\n');
 ```
 */
export function parseRecords(text: string): readonly unknown[] {
  return text
    .split('\n')
    .filter(function nonEmpty(line) {
      return line.trim() !== '';
    })
    .flatMap(function parse(line): readonly unknown[] {
      try {
        return [JSON.parse(line)];
      }
      catch (error) {
        // A torn or foreign line is not a record; the others stay evidence.
        console.error(`Skipping a line that is not a record: ${String(error)}`);
        return [];
      }
    });
}

/**
 Read a file, or nothing when it does not exist.

 @param path - file to read

 @returns its content, or an empty string
 */
async function readOrEmpty(path: string): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    // A scenario that never ran leaves no file; any other failure is worth seeing.
    if (!(Error.isError(error) && ('code' in error)
      && (error.code === 'ENOENT'))) {
      console.error(`Reading ${path} failed: ${String(error)}`);
    }
    return '';
  }
}

/**
 Collect the commit timing records and the service's landing lines of a finished run.

 @param root - scratch root of the run

 @param repositories - wrapped repository of each commit scenario, by scenario name

 @returns the evidence to store beside the report

 @example
 ```ts
 const evidence = await collectEvidence({ root, repositories: { 'commit-service': wrappedRepository } });
 ```
 */
export async function collectEvidence({
  root,
  repositories,
}: {
  readonly root: string;
  readonly repositories: Readonly<Record<string, string>>;
}): Promise<Evidence> {
  /**
   Timing records per scenario.
   */
  const entries = await Promise.all(Object.keys(repositories)
    .map(async function read(scenario) {
      return [
        scenario,
        parseRecords(
          await readOrEmpty(join(
            repositories[scenario] ?? '',
            '.git',
            TIMING_FILE,
          )),
        ),
      ] as const;
    }));
  /**
   The service's standard error.
   */
  const log = await readOrEmpty(join(
    root,
    SERVICE_LOG_NAME,
  ));
  return {
    commitTiming: Object.fromEntries(entries),
    serviceLandings: log
      .split('\n')
      .filter(function landing(line) {
        return line.includes('answered request=');
      }),
  };
}
