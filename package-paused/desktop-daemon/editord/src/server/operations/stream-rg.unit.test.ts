/**
 * Unit tests for the streaming `rg` helper.
 *
 * Covers:
 *
 * - a pattern `rg` rejects, which must reject the promise naming the failure
 * - a pattern that matches nothing, which stays a normal empty result
 * - the result cap, whose early kill must not be reported as a failure
 * - an aborted search, which resolves with what was collected
 *
 * Each test runs against a fixture directory under the test harness scratch
 * root, never the system temporary directory.
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import type { SearchResult, } from '../../protocol.ts';
import { streamRg, } from './stream-rg.ts';

/**
 * Files the fixture directory holds, each containing its own name.
 */
const FIXTURE_FILES = [
  'alpha.txt',
  'beta.txt',
  'gamma.txt',
];

/**
 * Pattern `rg` cannot compile because the group is never closed.
 */
const UNCLOSED_GROUP_PATTERN = 'alpha(';

/**
 * Pattern no fixture file contains.
 */
const ABSENT_PATTERN = 'nothing-matches-this';

/**
 * Result cap used to force the early-kill path.
 */
const CAP_FOR_ONE_RESULT = 1;

/**
 * Creates a fixture directory holding `FIXTURE_FILES`.
 *
 * @returns absolute path of the created directory
 *
 * @example
 * ```ts
 * const directory = await createFixtureDirectory();
 * ```
 */
async function createFixtureDirectory(): Promise<string> {
  const directory = await mkdtemp(join(scratchRoot(), 'stream-rg-',),);
  await Promise.all(FIXTURE_FILES.map(function writeOne(name: string,) {
    return writeFile(
      join(directory, name,),
      name,
    );
  },),);
  return directory;
}

/**
 * Converts one matched path into a file result.
 *
 * @param line - absolute path rg reported
 *
 * @returns file result for that path
 *
 * @example
 * ```ts
 * fileResult('/repo/alpha.txt');
 * ```
 */
function fileResult(line: string,): SearchResult {
  return {
    kind: 'file',
    path: line,
  };
}

await describe({
  name: streamRg.name,
  concurrency: 1,
  children: [
    it({
      name: 'rejects naming the failure when rg rejects the pattern',
      async fn() {
        const directory = await createFixtureDirectory();
        using _cleanup = {
          [Symbol.dispose](): void {
            void rm(directory, {
              recursive: true,
              force: true,
            },);
          },
        };
        await expect(streamRg({
          args: [
            '--',
            UNCLOSED_GROUP_PATTERN,
            directory,
          ],
          maxResults: CAP_FOR_ONE_RESULT,
          processLine: fileResult,
          signal: undefined,
        },),)
          .rejects
          .toThrow('rg',);
      },
    },),
    it({
      name: 'resolves an empty list when nothing matches',
      async fn() {
        const directory = await createFixtureDirectory();
        using _cleanup = {
          [Symbol.dispose](): void {
            void rm(directory, {
              recursive: true,
              force: true,
            },);
          },
        };
        const results = await streamRg({
          args: [
            '--files',
            directory,
          ],
          maxResults: CAP_FOR_ONE_RESULT,
          processLine(line: string,) {
            return line.includes(ABSENT_PATTERN,)
              ? fileResult(line,)
              : null;
          },
          signal: undefined,
        },);
        expect(results,).toEqual([],);
      },
    },),
    it({
      name: 'resolves the collected results once the cap is reached',
      async fn() {
        const directory = await createFixtureDirectory();
        using _cleanup = {
          [Symbol.dispose](): void {
            void rm(directory, {
              recursive: true,
              force: true,
            },);
          },
        };
        const results = await streamRg({
          args: [
            '--files',
            directory,
          ],
          maxResults: CAP_FOR_ONE_RESULT,
          processLine: fileResult,
          signal: undefined,
        },);
        expect(results.length,).toBe(CAP_FOR_ONE_RESULT,);
      },
    },),
    it({
      name: 'resolves instead of rejecting when the signal aborts',
      async fn() {
        const directory = await createFixtureDirectory();
        using _cleanup = {
          [Symbol.dispose](): void {
            void rm(directory, {
              recursive: true,
              force: true,
            },);
          },
        };
        /**
         * Controller aborted straight after the search starts, so the silent
         * cancellation path is taken rather than the failure path.
         */
        const controller = new AbortController();
        const pending = streamRg({
          args: [
            '--',
            UNCLOSED_GROUP_PATTERN,
            directory,
          ],
          maxResults: CAP_FOR_ONE_RESULT,
          processLine: fileResult,
          signal: controller.signal,
        },);
        controller.abort();
        await expect(pending,).resolves.toEqual([],);
      },
    },),
  ],
},);
