/**
 Tests for the per-command deletion judgement.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  deletionFindingDrafts,
  type DeletionTarget,
  shellWordCarriesExpansion,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Home folder every case is judged against, so no case depends on this account.
 */
const FIXTURE_HOME = '/home/user';

//endregion Fixtures

//region Expectations

/**
 Reads the targets of one command from plain words.
 
 @param words - words the deletion program would receive
 
 @returns targets carrying the same value and spelling
 
 @example
 ```ts
 targets(['-rf', '/x']);
 ```
 */
function targets(words: readonly string[],): readonly DeletionTarget[] {
  return words
    .map(function targetOf(word,): DeletionTarget {
      return {
        value: word,
        sourceText: word,
      };
    },);
}

/**
 Reads the drafts of one command whose targets are all plain words.
 
 @param words - words the deletion program would receive
 
 @param unwrittenPaths - whether another program supplies the paths
 
 @returns drafts the judgement named
 
 @example
 ```ts
 draftsOf({ words: ['"$d"'], unwrittenPaths: false });
 ```
 */
function draftsOf({
  words,
  unwrittenPaths,
}: {
  readonly words: readonly string[];
  readonly unwrittenPaths: boolean;
},): ReturnType<typeof deletionFindingDrafts> {
  return deletionFindingDrafts({
    programWord: 'rm',
    targets: targets(words,),
    unwrittenPaths,
    homeFolder: FIXTURE_HOME,
    carriesExpansion: shellWordCarriesExpansion,
  },);
}

//endregion Expectations

await describe({
  name: deletionFindingDrafts.name,
  children: [
    //region Another program supplies the paths
    describe({
      name: 'when another program supplies the paths',
      children: [
        it({
          name: 'names the program word whatever the targets spell',
          fn: async function testSuppliedPaths() {
            expect(draftsOf({
              words: ['/home/user/temp/agent/x',],
              unwrittenPaths: true,
            },),).toEqual([{
              word: 'rm',
              check: 'another-program-supplies-the-paths',
            },],);
          },
        },),
      ],
    },),
    //endregion Another program supplies the paths

    //region Computed paths
    describe({
      name: 'when the shell computes a target',
      children: [
        it({
          name: 'names the computed word and says the shell computes it',
          fn: async function testComputedTarget() {
            expect(
              deletionFindingDrafts({
                programWord: 'rm',
                targets: [{ value: '$d', sourceText: '"$d"', },],
                unwrittenPaths: false,
                homeFolder: FIXTURE_HOME,
                carriesExpansion: shellWordCarriesExpansion,
              },),
            ).toEqual([{
              word: '"$d"',
              check: 'shell-computes-the-path',
            },],);
          },
        },),
        it({
          name: 'names every computed target in order',
          fn: async function testSeveralComputedTargets() {
            expect(
              deletionFindingDrafts({
                programWord: 'rm',
                targets: [
                  { value: '$a', sourceText: '"$a"', },
                  { value: '/home/user/temp/agent/x', sourceText: '/home/user/temp/agent/x', },
                  { value: '$b', sourceText: '"$b"', },
                ],
                unwrittenPaths: false,
                homeFolder: FIXTURE_HOME,
                carriesExpansion: shellWordCarriesExpansion,
              },)
                .map(function wordOf(draft,): string {
                  return draft.word;
                },),
            ).toEqual(['"$a"', '"$b"',],);
          },
        },),
      ],
    },),
    //endregion Computed paths

    //region Collapsed targets
    describe({
      name: 'when a target collapsed',
      children: [
        it({
          name: 'names the target and the reading that fired',
          fn: async function testCollapsedTarget() {
            expect(draftsOf({
              words: ['/home',],
              unwrittenPaths: false,
            },),).toEqual([{
              word: '/home',
              check: 'target-collapses-to-a-broader-path',
              detail: 'a top-level folder',
            },],);
          },
        },),
        it({
          name: 'names one readable target as no draft at all',
          fn: async function testReadableTarget() {
            expect(draftsOf({
              words: ['/home/user/temp/agent/x',],
              unwrittenPaths: false,
            },),).toEqual([],);
          },
        },),
      ],
    },),
    //endregion Collapsed targets

    //region Both readings
    describe({
      name: 'when one target is computed and another collapsed',
      children: [
        it({
          name: 'names each target with its own check',
          fn: async function testBothReadings() {
            expect(
              deletionFindingDrafts({
                programWord: 'rm',
                targets: [
                  { value: '$d', sourceText: '"$d"', },
                  { value: '/', sourceText: '/', },
                ],
                unwrittenPaths: false,
                homeFolder: FIXTURE_HOME,
                carriesExpansion: shellWordCarriesExpansion,
              },)
                .map(function checkOf(draft,): string {
                  return draft.check;
                },),
            ).toEqual(['shell-computes-the-path', 'target-collapses-to-a-broader-path',],);
          },
        },),
      ],
    },),
    //endregion Both readings
  ],
},);
