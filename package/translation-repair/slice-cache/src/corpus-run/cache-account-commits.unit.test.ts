/**
 Tests the commit reading behind the pre-launch cache version check (ledger
 M28): a log line read into a commit, a subject's own tab kept and a short
 line refused; the setting commit as the newest that added the value on
 balance; the commits no account cites by nine characters, each with the
 versions set strictly before it; and a time written as UTC to the minute.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CacheAccountLogError,
  type CacheVersion,
  settingCommit,
  type SourceCommit,
  sourceCommitOf,
  unaccountedCommits,
  utcMinutes,
} from '../../dist/final/node/index.mjs';

/**
 Builds a commit whose hash repeats one character.

 @param mark - character the hash repeats

 @param seconds - committer time

 @returns Commit

 @example
 ```ts
 commit({ mark: 'a', seconds: 1, },);
 ```
 */
function commit({
  mark,
  seconds,
}: {
  readonly mark: string;
  readonly seconds: number;
},): SourceCommit {
  return {
    hash: mark.repeat(40,),
    seconds,
    subject: `nap ${mark}`,
  };
}

/**
 Builds a version of one name.

 @param name - constant's name

 @returns Version at value 1

 @example
 ```ts
 version({ name: 'NAP_CACHE_VERSION', },);
 ```
 */
function version({ name, }: { readonly name: string; },): CacheVersion {
  return {
    name,
    value: 1,
    path: 'src/nap-key.ts',
    declaration: `${name} = 1`,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sourceCommitOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS HASH, TIME AND SUBJECT, keeping a tab the subject holds',
          fn: async () => {
            expect(sourceCommitOf({ line: 'abc\t1790000000\tfix: nap\tpurr', },),).toEqual({
              hash: 'abc',
              seconds: 1_790_000_000,
              subject: 'fix: nap\tpurr',
            },);
          },
        },),
        it({
          name: 'REFUSES A LINE WITHOUT A HASH, A WHOLE-SECOND TIME OR A SUBJECT',
          fn: async () => {
            for (const line of [
              'abc',
              'abc\t1790000000',
              'abc\tsoon\tfix: nap',
              'abc\t1.5\tfix: nap',
              'abc\t\tfix: nap',
              '\t1790000000\tfix: nap',
            ]) {
              expect(function read(): void {
                sourceCommitOf({ line, },);
              },).toThrow(CacheAccountLogError,);
            }
          },
        },),
        it({
          name: 'REFUSES A TIME PAST THE LARGEST WHOLE NUMBER A DOUBLE HOLDS EXACTLY, which `Number` reads as a '
            + 'neighbouring second nobody wrote (ledger B73)',
          fn: async () => {
            /**
             Line whose time lies two past the exact range.
             */
            const line = `abc\t${String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,)}\tfix: nap`;
            /**
             What the reader threw.
             */
            const refusal = caught(function read(): void {
              sourceCommitOf({ line, },);
            },);
            expect(refusal,).toBeInstanceOf(CacheAccountLogError,);
            expect((refusal as Error).message,).toBe(`git log wrote a line the cache account audit cannot read: "${line}"`,);
          },
        },),
      ],
    },),

    describe({
      name: settingCommit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE NEWEST COMMIT THAT ADDED THE VALUE ON BALANCE, passing over a move',
          fn: async () => {
            expect(settingCommit({
              candidates: [
                {
                  commit: commit({
                    mark: 'c',
                    seconds: 3,
                  },),
                  added: 1,
                  removed: 1,
                },
                {
                  commit: commit({
                    mark: 'b',
                    seconds: 2,
                  },),
                  added: 1,
                  removed: 0,
                },
                {
                  commit: commit({
                    mark: 'a',
                    seconds: 1,
                  },),
                  added: 1,
                  removed: 0,
                },
              ],
            },),).toEqual({
              kind: 'set',
              commit: commit({
                mark: 'b',
                seconds: 2,
              },),
            },);
          },
        },),
        it({
          name: 'FINDS NONE WHERE NO COMMIT ADDED THE VALUE, which is an uncommitted value',
          fn: async () => {
            expect(settingCommit({
              candidates: [{
                commit: commit({
                  mark: 'a',
                  seconds: 1,
                },),
                added: 0,
                removed: 1,
              },],
            },),).toEqual({ kind: 'uncommitted', },);
          },
        },),
      ],
    },),

    describe({
      name: unaccountedCommits.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE COMMITS NO ACCOUNT CITES BY NINE CHARACTERS, with the versions set strictly before each',
          fn: async () => {
            /**
             Two versions, set at times 10 and 20, whose accounts cite commit d by
             nine characters and commit e by eight only.
             */
            const settings = [
              {
                version: version({ name: 'NAP_CACHE_VERSION', },),
                commit: commit({
                  mark: '1',
                  seconds: 10,
                },),
                account: `rides inside: ${'d'.repeat(9,)}`,
              },
              {
                version: version({ name: 'PURR_CACHE_VERSION', },),
                commit: commit({
                  mark: '2',
                  seconds: 20,
                },),
                account: `rides inside: ${'e'.repeat(8,)}`,
              },
            ];
            /**
             Commits after the earliest setting, one at the second setting's time.
             */
            const commits = [
              commit({
                mark: 'd',
                seconds: 30,
              },),
              commit({
                mark: 'e',
                seconds: 25,
              },),
              commit({
                mark: 'f',
                seconds: 20,
              },),
            ];
            expect(unaccountedCommits({
              settings,
              commits,
            },).map(function summary(unnamed,): readonly [string, readonly string[],] {
              return [unnamed.commit.subject, unnamed.setBefore,];
            },),).toEqual([
              ['nap e', ['NAP_CACHE_VERSION', 'PURR_CACHE_VERSION',],],
              ['nap f', ['NAP_CACHE_VERSION',],],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: utcMinutes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES A TIME AS UTC TO THE MINUTE, dropping the seconds rather than rounding them up',
          fn: async () => {
            expect(utcMinutes({ seconds: 0, },),).toBe('1970-01-01T00:00Z',);
            expect(utcMinutes({
              seconds: Date.UTC(
                2_026,
                8,
                27,
                14,
                5,
                59,
              ) / 1_000,
            },),).toBe('2026-09-27T14:05Z',);
          },
        },),
      ],
    },),
  ],
},);
