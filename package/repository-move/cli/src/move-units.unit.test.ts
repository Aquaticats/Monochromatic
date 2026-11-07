/**
 Unit tests of the pure parts of the move tools: option parsing, inventory parsing, copy naming and the deadline clock,
 and the table of uncommitted work against a real throwaway repository.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assignNames,
  boundedSeconds,
  buildUncommittedWorkTable,
  MoveStopError,
  parseInventory,
  parseMoveOptions,
  startDeadlineClock,
} from '../dist/final/node/index.mjs';
import { makeFixture, } from './move-fixture.ts';

const REQUIRED = [
  '--pass=first',
  '--main=/m',
  '--snapshots=/s',
  '--received=/r',
  '--final=/f',
  '--state=/t',
];

await describe({
  name: '',
  children: [
    describe({
      name: parseMoveOptions.name,
      children: [
        it({
          name: 'reads defaults, a limit, and lets --print win over --run',
          fn: async () => {
            const options = parseMoveOptions([...REQUIRED, '--run', '--print', '--limit=3',],);
            expect(options.run,).toBe(false,);
            expect(options.limit,).toBe(3,);
            expect(options.inventoryPrefixFrom,).toBe('home/',);
            expect(options.limit,).not.toBe(Number.POSITIVE_INFINITY,);
          },
        },),
        it({
          name: 'throws a stop error for a missing required flag, an unknown pass and a negative number',
          fn: async () => {
            expect(function missing() {
              parseMoveOptions(['--pass=first',],);
            },).toThrow(MoveStopError,);
            expect(function unknownPass() {
              parseMoveOptions([...REQUIRED.slice(1,), '--pass=sideways',],);
            },).toThrow(MoveStopError,);
            expect(function negative() {
              parseMoveOptions([...REQUIRED, '--total-deadline-seconds=-1',],);
            },).toThrow(MoveStopError,);
            expect(function badClone() {
              parseMoveOptions([...REQUIRED, '--clone-sources=some',],);
            },).toThrow(MoveStopError,);
          },
        },),
      ],
    },),
    describe({
      name: parseInventory.name,
      children: [
        it({
          name: 'maps rows below the main checkout, finds the main row, and skips unrelated rows',
          fn: async () => {
            const options = parseMoveOptions([
              ...REQUIRED,
              '--inventory-main=home/user/repo',
              '--inventory-prefix=home/=/var/home/',
            ],);
            const parsed = parseInventory({
              text: [
                'ID 99 gen 1 top level 5 parent_uuid - received_uuid - uuid u-x path other/place',
                'ID 6 gen 1 top level 5 uuid u-main path home/user/repo',
                'ID 8 gen 1 top level 6 uuid u-b path home/user/repo/b w',
                'ID 7 gen 1 top level 6 uuid u-a path home/user/repo/a',
                '',
              ].join('\n',),
              options,
            },);
            expect(parsed.main,).toEqual({ uuid: 'u-main', path: 'home/user/repo', },);
            expect(parsed.nested.map((row,) => row.path),).toEqual([
              '/var/home/user/repo/a',
              '/var/home/user/repo/b w',
            ],);
          },
        },),
        it({
          name: 'rebuilds full paths from the real list, where the path column is relative to the parent named by top level',
          fn: async () => {
            const options = parseMoveOptions([
              ...REQUIRED,
              '--inventory-main=home/user/Monochromatic',
              '--inventory-prefix=home/=/var/home/',
            ],);
            const parsed = parseInventory({
              text: [
                'ID 256 gen 815068 top level 5 uuid 1e647278-be05-1b4a-9277-4bd783bf076f path root',
                'ID 257 gen 815137 top level 5 uuid 3c579a80-eb6a-d742-8cab-f0447fa31d28 path home',
                'ID 3569 gen 815138 top level 257 uuid 61d41cbe-834d-1c41-b18d-65f75135642a path user/Monochromatic',
                'ID 3600 gen 815138 top level 3569 uuid u-a path .claude/worktrees/a',
                'ID 3601 gen 815138 top level 3569 uuid u-b path user/Monochromatic/.claude/worktrees/b',
                'ID 3602 gen 815138 top level 3600 uuid u-c path .claude/worktrees/a/inner',
                'ID 3700 gen 815138 top level 257 uuid u-other path user/other',
              ].join('\n',),
              options,
            },);
            expect(parsed.main.uuid,).toBe('61d41cbe-834d-1c41-b18d-65f75135642a',);
            expect(parsed.nested.map((row,) => row.path),).toEqual([
              '/var/home/user/Monochromatic/.claude/worktrees/a',
              '/var/home/user/Monochromatic/.claude/worktrees/a/inner',
              '/var/home/user/Monochromatic/.claude/worktrees/b',
            ],);
          },
        },),
        it({
          name: 'refuses a row without a UUID, and a path outside the prefix',
          fn: async () => {
            const options = parseMoveOptions([
              ...REQUIRED,
              '--inventory-main=elsewhere/repo',
              '--inventory-prefix=home/=/var/home/',
            ],);
            expect(function withoutUuid() {
              parseInventory({ text: 'ID 6 gen 1 top level 5 path home/user/repo\n', options, },);
            },).toThrow(MoveStopError,);
            expect(function outsidePrefix() {
              parseInventory({ text: 'ID 6 gen 1 top level 5 uuid u path elsewhere/repo/a\n', options, },);
            },).toThrow(MoveStopError,);
          },
        },),
      ],
    },),
    describe({
      name: assignNames.name,
      children: [
        it({
          name: 'numbers a first run in order, and reuses names for the same UUID and path',
          fn: async () => {
            const current = [{ path: '/m', uuid: 'u-m', }, { path: '/m/a', uuid: 'u-a', },];
            const first = await assignNames({ current, ledger: [], prefix: 's', folders: [], },);
            expect(first.named.map((row,) => row.name),).toEqual(['s000', 's001',],);
            const again = await assignNames({
              current: [...current, { path: '/m/z', uuid: 'u-z', },],
              ledger: [{ name: 's000', uuid: 'u-m', path: '/m', }, { name: 's001', uuid: 'u-a', path: '/m/a', },],
              prefix: 's',
              folders: [],
            },);
            expect(again.named.map((row,) => row.name),).toEqual(['s000', 's001', 's002',],);
            expect(again.changes.map((change,) => change.kind),).toEqual(['added',],);
          },
        },),
        it({
          name: 'tells moved, replaced and removed apart and never reuses a number',
          fn: async () => {
            const result = await assignNames({
              current: [{ path: '/m', uuid: 'u-m', }, { path: '/m/new-place', uuid: 'u-a', }, {
                path: '/m/b',
                uuid: 'u-b2',
              },],
              ledger: [
                { name: 's000', uuid: 'u-m', path: '/m', },
                { name: 's001', uuid: 'u-a', path: '/m/a', },
                { name: 's002', uuid: 'u-b', path: '/m/b', },
                { name: 's003', uuid: 'u-gone', path: '/m/gone', },
              ],
              prefix: 's',
              folders: [],
            },);
            expect(result.named.map((row,) => row.name),).toEqual(['s000', 's004', 's005',],);
            expect(result.changes.map((change,) => change.kind).toSorted(),).toEqual([
              'moved',
              'removed',
              'replaced',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: startDeadlineClock.name,
      children: [
        it({
          name: 'counts down from the total and bounds a step by what remains',
          fn: async () => {
            let milliseconds = 0;
            const clock = startDeadlineClock({ totalSeconds: 10, now: () => milliseconds, },);
            expect(clock.remainingSeconds(),).toBe(10,);
            milliseconds = 9_500;
            expect(boundedSeconds({ stepSeconds: 100, clock, },),).toBe(1,);
            expect(boundedSeconds({ stepSeconds: 0.2, clock, },),).toBe(1,);
            milliseconds = 11_000;
            expect(clock.remainingSeconds(),).toBeLessThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: buildUncommittedWorkTable.name,
      children: [
        it({
          name: 'counts a worktree with an untracked file as dirty and one without as clean',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            await writeFile(join(fixture.main, 'wt', 'a', 'extra.txt',), 'x',);
            await mkdir(join(fixture.main, 'wt', 'b', 'sub',),);
            const table = await buildUncommittedWorkTable({ mainWorktree: fixture.main, perWorktreeSeconds: 60, },);
            expect(table.worktrees,).toBe(3,);
            expect(table.text,).toContain('/wt/a\tok\t',);
            expect(table.text.split('\n',)[0],).toContain('indexSha256',);
            expect(table.dirty,).toBeGreaterThanOrEqual(1,);
          },
        },),
        it({
          name: 'throws a stop error when the folder is not a repository',
          fn: async () => {
            let caught: unknown;
            try {
              await buildUncommittedWorkTable({ mainWorktree: '/nonexistent-folder-for-test', perWorktreeSeconds: 5, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(MoveStopError,);
          },
        },),
      ],
    },),
  ],
},);
