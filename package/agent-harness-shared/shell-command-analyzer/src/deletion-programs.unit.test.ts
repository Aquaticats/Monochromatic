/**
 Tests for the deletion-program table and the targets it hands the reader.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  deletionProgramOf,
  deletionTargetsOf,
  type DeletionTarget,
  FIND_DELETE_ACTION,
  findDeletesThePathsItFound,
  NO_DELETION_PROGRAM,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 Every program the literal-path rule covers.
 */
const DELETING_PROGRAM_WORDS: readonly string[] = [
  'rm',
  'rmdir',
  'unlink',
  'shred',
  '/usr/bin/rm',
  '/usr/bin/rmdir',
  './unlink',
  '/bin/shred',
];

/**
 Words that name a program this rule does not cover.
 */
const OTHER_PROGRAM_WORDS: readonly string[] = [
  'git',
  'podman',
  'docker',
  'npm',
  'pnpm',
  'cargo',
  'find',
  'cp',
  '',
  '/usr/bin/git',
];

//endregion Fixtures

//region Expectations

/**
 Reads words as the reader receives them, each naming itself in source too.
 
 @param words - plain words of one command
 
 @returns targets carrying the same value and spelling
 
 @example
 ```ts
 targetsOf(['-rf', '/x']);
 ```
 */
function targetsOf(words: readonly string[],): readonly DeletionTarget[] {
  return words
    .map(function targetOf(word,): DeletionTarget {
      return {
        value: word,
        sourceText: word,
      };
    },);
}

//endregion Expectations

await describe({
  name: '',
  children: [
    describe({
      name: deletionProgramOf.name,
      children: [
        ...DELETING_PROGRAM_WORDS.map(function deletingProgramWord(word,) {
          return it({
            name: `reads \`${word}\` as a program that deletes`,
            fn: async function testDeletingProgramWord() {
              expect(deletionProgramOf(word,),).not.toBe(NO_DELETION_PROGRAM,);
            },
          },);
        },),
        ...OTHER_PROGRAM_WORDS.map(function otherProgramWord(word,) {
          return it({
            name: `reads \`${word}\` as a program that deletes nothing`,
            fn: async function testOtherProgramWord() {
              expect(deletionProgramOf(word,),).toBe(NO_DELETION_PROGRAM,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: deletionTargetsOf.name,
      children: [
        it({
          name: 'drops flags before the targets',
          fn: async function testFlagsDropped() {
            expect(
              deletionTargetsOf({
                words: targetsOf(['-rf', '/home/user/temp/agent/x',],),
                optionsWithValue: new Set(),
              },)
                .map(function valueOf(target,): string {
                  return target.value;
                },),
            ).toEqual(['/home/user/temp/agent/x',],);
          },
        },),
        it({
          name: 'drops the value of an option that takes one',
          fn: async function testOptionValueDropped() {
            expect(
              deletionTargetsOf({
                words: targetsOf(['-n', '3', '-s', '10', '/home/user/temp/agent/x',],),
                optionsWithValue: new Set(['-n', '-s',],),
              },)
                .map(function valueOf(target,): string {
                  return target.value;
                },),
            ).toEqual(['/home/user/temp/agent/x',],);
          },
        },),
        it({
          name: 'reads every word after the options terminator as a target',
          fn: async function testOptionsTerminator() {
            expect(
              deletionTargetsOf({
                words: targetsOf(['-rf', '--', '-weird', '--',],),
                optionsWithValue: new Set(),
              },)
                .map(function valueOf(target,): string {
                  return target.value;
                },),
            ).toEqual(['-weird', '--',],);
          },
        },),
        it({
          name: 'reads a lone dash as a target',
          fn: async function testLoneDash() {
            expect(
              deletionTargetsOf({
                words: targetsOf(['-',],),
                optionsWithValue: new Set(),
              },)
                .map(function valueOf(target,): string {
                  return target.value;
                },),
            ).toEqual(['-',],);
          },
        },),
        it({
          name: 'keeps the source spelling of every target',
          fn: async function testSourceSpelling() {
            expect(
              deletionTargetsOf({
                words: [{ value: '$d', sourceText: '"$d"', },],
                optionsWithValue: new Set(),
              },),
            ).toEqual([{ value: '$d', sourceText: '"$d"', },],);
          },
        },),
        it({
          name: 'reads no target from an empty word list',
          fn: async function testNoWords() {
            expect(
              deletionTargetsOf({
                words: [],
                optionsWithValue: new Set(),
              },),
            ).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: findDeletesThePathsItFound.name,
      children: [
        it({
          name: 'reads a find that deletes what it matched',
          fn: async function testFindDelete() {
            expect(
              findDeletesThePathsItFound(['find', '/home/user/temp/agent/x', FIND_DELETE_ACTION,],),
            ).toBe(true,);
          },
        },),
        it({
          name: 'reads a find spelled by path',
          fn: async function testFindByPath() {
            expect(
              findDeletesThePathsItFound(['/usr/bin/find', '.', FIND_DELETE_ACTION,],),
            ).toBe(true,);
          },
        },),
        it({
          name: 'reads a find that only prints',
          fn: async function testFindPrint() {
            expect(findDeletesThePathsItFound(['find', '.', '-print',],),).toBe(false,);
          },
        },),
        it({
          name: 'reads no delete action from another program',
          fn: async function testOtherProgram() {
            expect(findDeletesThePathsItFound(['ls', '-delete',],),).toBe(false,);
          },
        },),
        it({
          name: 'reads no delete action from an empty word list',
          fn: async function testNoWords() {
            expect(findDeletesThePathsItFound([],),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
