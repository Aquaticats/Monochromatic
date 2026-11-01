/**
 Tests that the corpus pin dial overrides either half and REFUSES rather
 than falling back, because a mistyped override silently becoming the pin
 would run the wrong corpus and record fixture conclusions as pinned ones.

 EVERY CASE HANDS THE READER ITS ENVIRONMENT, and none writes the process's:
 the reader reads the environment it is handed, which an entry reads inside
 its refusal boundary (ledger B302, B381).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  CORPUS_CLONE_DIR_VAR,
  CORPUS_COMMIT_VAR,
  corpusPinOverrideNote,
  readCorpusPinSetting,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Pin used when the environment says nothing.
 */
const FALLBACK_PIN = {
  cloneDir: '/cats/corpus/clone',
  commitSha: 'a'.repeat(40,),
} as const;

/**
 Commits the reader refuses: abbreviated, uppercase, and not hexadecimal.
 */
const REFUSED_COMMITS = [
  'abc123',
  'B'.repeat(40,),
  'g'.repeat(40,),
] as const;

/**
 Reads the setting under one commit dial value and returns what was thrown,
 as its class check and its printed text.

 @param commit - commit value the handed environment writes for this reading

 @returns Whether the reading threw a stated refusal, and what it printed

 @example
 ```ts
 const [stated, text,] = commitRefusal({ commit: 'abc123', },);
 ```
 */
function commitRefusal(
  { commit, }: { readonly commit: string; },
): readonly [boolean, string,] {
  /**
   What the reader threw for this commit.
   */
  const thrown = caught(function readsUnderBadCommit() {
    readCorpusPinSetting({
      env: { [CORPUS_COMMIT_VAR]: commit, },
      fallback: FALLBACK_PIN,
    },);
  },);
  return [
    thrown instanceof StatedRefusalError,
    String(thrown,),
  ];
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: readCorpusPinSetting.name,
      children: [
        it({
          name: 'returns the fallback pin with both sources named fallback when unset',
          fn: async () => {
            expect(readCorpusPinSetting({
              env: {},
              fallback: FALLBACK_PIN,
            },),).toEqual({
              pin: FALLBACK_PIN,
              cloneDirSource: 'fallback',
              commitSource: 'fallback',
            },);
          },
        },),

        it({
          name: 'READS THE ENVIRONMENT IT IS HANDED, never the process\'s own, overriding both halves from it',
          fn: async () => {
            expect(readCorpusPinSetting({
              env: {
                [CORPUS_CLONE_DIR_VAR]: '/cats/fixture/clone',
                [CORPUS_COMMIT_VAR]: 'b'.repeat(40,),
              },
              fallback: FALLBACK_PIN,
            },),).toEqual({
              pin: {
                cloneDir: '/cats/fixture/clone',
                commitSha: 'b'.repeat(40,),
              },
              cloneDirSource: CORPUS_CLONE_DIR_VAR,
              commitSource: CORPUS_COMMIT_VAR,
            },);
          },
        },),

        it({
          name: 'overrides the clone dir alone and names its variable as the source',
          fn: async () => {
            expect(readCorpusPinSetting({
              env: { [CORPUS_CLONE_DIR_VAR]: '/cats/fixture/clone', },
              fallback: FALLBACK_PIN,
            },),).toEqual({
              pin: {
                cloneDir: '/cats/fixture/clone',
                commitSha: FALLBACK_PIN.commitSha,
              },
              cloneDirSource: CORPUS_CLONE_DIR_VAR,
              commitSource: 'fallback',
            },);
          },
        },),

        it({
          name: 'overrides the commit alone and names its variable as the source',
          fn: async () => {
            expect(readCorpusPinSetting({
              env: { [CORPUS_COMMIT_VAR]: 'b'.repeat(40,), },
              fallback: FALLBACK_PIN,
            },),).toEqual({
              pin: {
                cloneDir: FALLBACK_PIN.cloneDir,
                commitSha: 'b'.repeat(40,),
              },
              cloneDirSource: 'fallback',
              commitSource: CORPUS_COMMIT_VAR,
            },);
          },
        },),

        it({
          name: 'REFUSES a relative clone dir rather than falling back, naming the variable and the value',
          fn: async () => {
            /**
             Refusal thrown for a clone dir no read could pin down.
             */
            const thrown = caught(function readsUnderRelativeDir() {
              readCorpusPinSetting({
                env: { [CORPUS_CLONE_DIR_VAR]: 'cats/relative/clone', },
                fallback: FALLBACK_PIN,
              },);
            },);
            expect(thrown,).toBeInstanceOf(StatedRefusalError,);
            expect(String(thrown,),).toBe(
              'StatedRefusalError: TRANSLATION_REPAIR_CORPUS_CLONE_DIR must be an absolute path, and '
                + 'cats/relative/clone is not one',
            );
          },
        },),

        it({
          name: 'REFUSES an abbreviated, uppercase or non-hexadecimal commit rather than falling back, naming the '
            + 'variable and the value',
          fn: async () => {
            expect(REFUSED_COMMITS.map(function refusalOf(commit,): readonly [boolean, string,] {
              return commitRefusal({ commit, },);
            },),).toEqual(REFUSED_COMMITS.map(function expectedOf(commit,): readonly [boolean, string,] {
              return [
                true,
                'StatedRefusalError: TRANSLATION_REPAIR_CORPUS_COMMIT must be one full lowercase hexadecimal '
                  + `commit name, and ${commit} is not one`,
              ];
            },),);
          },
        },),
      ],
    },),

    describe({
      name: corpusPinOverrideNote.name,
      children: [
        it({
          name: 'SAYS NOTHING FOR THE BUILT-IN PIN, and names each overridden half with its source (ledger D13: the '
            + 'setting was kept for launch logs and no launch printed it)',
          fn: async () => {
            /**
             The note under each handed environment.
             */
            const notes = [
              {},
              { [CORPUS_COMMIT_VAR]: 'b'.repeat(40,), },
              { [CORPUS_CLONE_DIR_VAR]: '/cats/fixture/clone', },
            ].map(function noteUnder(env,): string {
              return corpusPinOverrideNote({
                setting: readCorpusPinSetting({
                  env,
                  fallback: FALLBACK_PIN,
                },),
              },);
            },);
            expect(notes,).toEqual([
              '',
              `CORPUS PIN OVERRIDDEN: clone /cats/corpus/clone from fallback, commit ${'b'.repeat(40,)} from ${CORPUS_COMMIT_VAR}`,
              `CORPUS PIN OVERRIDDEN: clone /cats/fixture/clone from ${CORPUS_CLONE_DIR_VAR}, commit ${'a'.repeat(40,)} from fallback`,
            ],);
          },
        },),
      ],
    },),
  ],
},);
