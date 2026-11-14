// Builds the baked input lists for the Optique measurement image.
// Run once on the host: node scripts/gen-inputs.ts
// Reads ctx/tracked-files-parent.txt (git ls-tree -r --name-only 4879a44e6a^)
// and writes data/*.json. Lawful inputs only: distinct short names, real
// repository paths, and the issue-988 path shape.

import { mkdir, readFile, writeFile, } from 'node:fs/promises';
import { dirname, join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

//region Limits

/**
 * Upper bound of the measured path counts; every workload is a prefix of a
 * baked list of at least this length.
 */
const MAXIMUM_PATH_COUNT = 1000;

/**
 * Number of `--option-N` declarations the upstream issue-988 reproduction uses.
 */
const ISSUE_988_OPTION_COUNT = 30;

//endregion Limits

//region Path list builders

/**
 * Generates distinct alphabetic names shortest-first, so a short-path prefix of
 * length n stays "a few characters" and always names unique files.
 *
 * @param count - How many names to generate.
 *
 * @returns Distinct names of one to three letters.
 */
function buildShortPaths(count: number,): readonly string[] {
  /** Names collected shortest-first. */
  const names: string[] = [];
  /** Alphabet used for name generation. */
  const alphabet = 'abcdefghijklmnopqrstuvwxyz';

  for (const first of alphabet) {
    names.push(first,);
  }
  for (const first of alphabet) {
    for (const second of alphabet) {
      names.push(`${first}${second}`,);
    }
  }
  for (const first of alphabet) {
    for (const second of alphabet) {
      for (const third of alphabet) {
        names.push(`${first}${second}${third}`,);
      }
    }
  }

  return names.slice(
    0,
    count,
  );
}

/**
 * Builds the distinct long paths the issue-988 reproduction feeds to the root
 * parser, matching the shape in the issue text.
 *
 * @param count - How many paths to generate.
 *
 * @returns Distinct `src/component-N/filename.ts` paths.
 */
function buildIssue988Paths(count: number,): readonly string[] {
  return Array.from(
    { length: count, },
    function path(_value, index,): string {
      return `src/component-${index}/filename.ts`;
    },
  );
}

/**
 * Takes the first `count` real repository paths, preserving tree order.
 *
 * @param tracked - Every tracked path at the migration parent commit.
 *
 * @param count - How many paths to take.
 *
 * @returns The prefix of the tracked list.
 */
function buildRealPaths(tracked: readonly string[], count: number,): readonly string[] {
  return tracked.slice(
    0,
    count,
  );
}

/**
 * Reports the length distribution of a path list as evidence about the set.
 *
 * @param name - List name used in the report line.
 *
 * @param paths - Paths to summarise.
 */
function reportLengths(name: string, paths: readonly string[],): void {
  /** Path lengths in characters. */
  const lengths = paths
    .map(function lengthOf(path,) {
      return path.length;
    },)
    .sort(function ascending(left, right,) {
      return left - right;
    },);
  /** Index of the middle length. */
  const middleIndex = Math.floor(lengths.length / 2,);
  /** Middle length. */
  const median = lengths[middleIndex];
  console.log(
    `[gen] ${name}: n=${lengths.length} min=${lengths[0]} median=${median} max=${lengths[lengths.length - 1]}`,
  );
}

//endregion Path list builders

//region Entry point

/** Directory holding this script. */
const scriptDirectory = dirname(fileURLToPath(import.meta.url,),);
/** Scratch root, one level above the scripts directory. */
const rootDirectory = dirname(scriptDirectory,);
/** Output directory for baked input lists. */
const dataDirectory = join(
  rootDirectory,
  'data',
);

/**
 * Reads the tracked-file list, writes every baked input list, and prints the
 * length evidence for each one.
 */
async function main(): Promise<void> {
  /** Raw tracked-file listing, one path per line. */
  const trackedRaw = await readFile(
    join(
      rootDirectory,
      'ctx/tracked-files-parent.txt',
    ),
    'utf8',
  );
  /** Tracked paths without empty trailing lines. */
  const tracked = trackedRaw
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },);

  if (tracked.length < MAXIMUM_PATH_COUNT) {
    throw new Error(
      `tracked list has ${tracked.length} entries, need ${MAXIMUM_PATH_COUNT}.`,
    );
  }

  for (const [index, path,] of tracked.entries()) {
    if (path.startsWith('-',)) {
      throw new Error(
        `tracked path ${index} starts with a dash, which would parse as an option: ${path}`,
      );
    }
  }

  /** Real repository paths, first MAXIMUM_PATH_COUNT in tree order. */
  const realPaths = buildRealPaths(tracked, MAXIMUM_PATH_COUNT,);
  /** Short generated names. */
  const shortPaths = buildShortPaths(MAXIMUM_PATH_COUNT,);
  /** Issue-988 long paths. */
  const issue988Paths = buildIssue988Paths(MAXIMUM_PATH_COUNT,);

  if (new Set(shortPaths,).size !== shortPaths.length) {
    throw new Error('short path list contains duplicates.',);
  }

  await mkdir(dataDirectory, { recursive: true, },);
  await writeFile(
    join(
      dataDirectory,
      'real-paths.json',
    ),
    `${JSON.stringify(realPaths,)}\n`,
  );
  await writeFile(
    join(
      dataDirectory,
      'short-paths.json',
    ),
    `${JSON.stringify(shortPaths,)}\n`,
  );
  await writeFile(
    join(
      dataDirectory,
      'issue988-paths.json',
    ),
    `${JSON.stringify(issue988Paths,)}\n`,
  );

  reportLengths('real', realPaths,);
  reportLengths('short', shortPaths,);
  reportLengths('issue988', issue988Paths,);
  console.log(`[gen] issue988 option count: ${ISSUE_988_OPTION_COUNT}`);
}

await main();

//endregion Entry point
