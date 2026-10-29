/**
 Tests for what the pre-launch cache check asks git: one command's output, and
 a commit log read into commits.

 Git runs inside a throwaway repository the cases build, so nothing reads this
 repository's history.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CacheAccountLogError,
  cacheAccountCommitsOf,
  cacheAccountGitOutput,
} from '../../dist/final/node/index.mjs';
import { makeCacheAccountRepo, } from './cache-account-repo.test-fixture.ts';

await describe({
  name: 'cache-account git',
  children: [
    describe({
      name: cacheAccountGitOutput.name,
      children: [
        it({
          name: 'RETURNS WHAT GIT PRINTED for a read-only command run in a repository, without its final newline',
          fn: async () => {
            await using repo = await makeCacheAccountRepo({ versions: 'one', },);

            expect(await cacheAccountGitOutput({
              args: [
                '-C',
                repo.path,
                'log',
                '--format=%H',
                '--max-count=1',
                '--',
                'src/stage-cache-version.ts',
              ],
            },),).toBe(repo.stageSet.hash,);
          },
        },),
      ],
    },),
    describe({
      name: cacheAccountCommitsOf.name,
      children: [
        it({
          name: 'READS EACH NON-EMPTY LINE INTO A COMMIT, in the order git printed them, and skips the empty lines',
          fn: async () => {
            expect(cacheAccountCommitsOf({
              output: 'aaa\t1788000000\tteach Tabby a longer nap\n\nbbb\t1787900000\tset the stage cache version\n',
            },),).toEqual([
              {
                hash: 'aaa',
                seconds: 1_788_000_000,
                subject: 'teach Tabby a longer nap',
              },
              {
                hash: 'bbb',
                seconds: 1_787_900_000,
                subject: 'set the stage cache version',
              },
            ],);
          },
        },),
        it({
          name: 'READS NO COMMIT FROM AN EMPTY LOG',
          fn: async () => {
            expect(cacheAccountCommitsOf({ output: '', },),).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES A LINE THAT IS NOT A COMMIT, naming the line, rather than skipping it',
          fn: async () => {
            /**
             What reading the log refused with.
             */
            const refusal = caught(function act(): unknown {
              return cacheAccountCommitsOf({ output: 'aaa\tnot-a-time\tsubject\n', },);
            },);

            expect(refusal,).toBeInstanceOf(CacheAccountLogError,);
            expect(String(refusal,),).toBe(
              'CacheAccountLogError: git log wrote a line the cache account audit cannot read: "aaa\tnot-a-time\tsubject"',
            );
          },
        },),
      ],
    },),
  ],
},);
