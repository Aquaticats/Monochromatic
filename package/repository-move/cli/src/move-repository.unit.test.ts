/**
 Unit tests of the built copy tool against a btrfs test double: the control flow of both passes, and the four defects
 of the first script (records keyed by position, freeze joined to the copy, no total deadline, external second pass
 that could not report deletions).

 @module
 */

import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  addWorktree,
  type Fixture,
  makeFixture,
  removeFixture,
  removeWorktree,
  runCopyTool,
  writeInventory,
} from './move-fixture.ts';

function records(fixture: Fixture,): string[] {
  return readdirSync(join(fixture.base, 'state',),).filter((name,) => name.endsWith('.json',)).toSorted();
}

function sendCalls({ fixture, snapshot, }: { fixture: Fixture; snapshot: string; },): number {
  return readFileSync(fixture.doubleLog, 'utf8',).split('\n',).filter((line,) =>
    line.startsWith('["send"',) && line.endsWith(`/snap/${snapshot}"]`,)
  ).length;
}

async function withFixture(
  { names, fn, }: { names: readonly string[]; fn: (fixture: Fixture,) => void; },
): Promise<void> {
  const fixture = makeFixture({ names, },);
  try {
    fn(fixture,);
  }
  finally {
    removeFixture({ fixture, },);
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: 'first pass',
      children: [
        it({
          name: 'copies the checkout and every worktree and records each by name',
          fn: async () => {
            await withFixture({
              names: ['a', 'b',],
              fn: (fixture,) => {
                const result = runCopyTool({ fixture, pass: 'first', },);
                expect(result.status,).toBe(0,);
                expect(result.stderr,).toContain('pass first finished',);
                expect(records(fixture,),).toContain('s002.received.json',);
              },
            },);
          },
        },),
        it({
          name: 'stops when a send fails and records nothing for the failed subvolume',
          fn: async () => {
            await withFixture({
              names: ['a', 'b',],
              fn: (fixture,) => {
                const result = runCopyTool({ fixture, pass: 'first', env: { FAIL_SEND: 's001', }, },);
                expect(result.status,).toBe(1,);
                expect(records(fixture,),).toContain('s000.received.json',);
                expect(records(fixture,),).not.toContain('s001.received.json',);
                expect(existsSync(join(fixture.base, 'snap', 's002',),),).toBe(false,);
              },
            },);
          },
        },),
        it({
          name: 'refuses a received folder that has no completion record and names the delete for the owner',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                mkdirSync(join(fixture.base, 'recv', 's001',),);
                const result = runCopyTool({ fixture, pass: 'first', },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('exists without a completion record',);
                expect(result.stderr,).toContain('btrfs subvolume delete',);
                expect(existsSync(join(fixture.base, 'recv', 's001',),),).toBe(true,);
              },
            },);
          },
        },),
        it({
          name: 'stops when a received copy is not read-only',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                const result = runCopyTool({ fixture, pass: 'first', env: { FAIL_RO: 's001', }, },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('is not a complete read-only copy',);
              },
            },);
          },
        },),
        it({
          name: 'stops when the inventory lists a subvolume Git does not know',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                writeInventory({ fixture, rows: [['a', 'u-a',], ['hidden', 'u-hidden',],], },);
                const result = runCopyTool({ fixture, pass: 'first', },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('subvolumes Git does not know',);
              },
            },);
          },
        },),
        it({
          name: 'stops when the snapshot of a path has a different UUID than the inventory recorded',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                writeInventory({ fixture, rows: [['a', 'u-other',],], },);
                const result = runCopyTool({ fixture, pass: 'first', },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('was replaced',);
              },
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'resume after worktrees changed (records keyed by UUID and path)',
      children: [
        it({
          name: 'keeps the record of an unchanged worktree, gives a new worktree a new name, and states both changes',
          fn: async () => {
            await withFixture({
              names: ['a', 'b',],
              fn: (fixture,) => {
                expect(runCopyTool({ fixture, pass: 'first', env: { FAIL_SEND: 's002', }, },).status,).toBe(1,);
                removeWorktree({ fixture, name: 'b', },);
                addWorktree({ fixture, name: 'c', uuid: 'u-c', },);
                writeInventory({ fixture, rows: [['a', 'u-a',], ['c', 'u-c',],], },);
                const result = runCopyTool({ fixture, pass: 'first', },);
                expect(result.status,).toBe(0,);
                expect(result.stderr,).toContain('added since the pass started',);
                expect(result.stderr,).toContain('/wt/c',);
                expect(result.stderr,).toContain('no longer present',);
                expect(result.stderr,).toContain('/wt/b',);
                expect(sendCalls({ fixture, snapshot: 's001', },),).toBe(1,);
                const manifest = readFileSync(join(fixture.base, 'state', 'first-pass-manifest.tsv',), 'utf8',);
                expect(manifest,).toContain('s003\t',);
                expect(manifest,).not.toContain('/wt/b',);
                expect(records(fixture,),).toContain('s003.received.json',);
              },
            },);
          },
        },),
        it({
          name: 'treats a path whose subvolume was replaced (new UUID) as a new subvolume and leaves the old record alone',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                expect(runCopyTool({ fixture, pass: 'first', },).status,).toBe(0,);
                writeFileSync(join(fixture.main, 'wt', 'a', '.stub-subvolume',), 'u-a2\n',);
                writeInventory({ fixture, rows: [['a', 'u-a2',],], },);
                const result = runCopyTool({ fixture, pass: 'first', },);
                expect(result.status,).toBe(0,);
                expect(result.stderr,).toContain('replaced',);
                expect(records(fixture,),).toContain('s001.received.json',);
                expect(records(fixture,),).toContain('s002.received.json',);
                expect(sendCalls({ fixture, snapshot: 's001', },),).toBe(1,);
              },
            },);
          },
        },),
        it({
          name: 'sends nothing again when the same worktrees are present and the pass already finished',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                expect(runCopyTool({ fixture, pass: 'first', },).status,).toBe(0,);
                expect(runCopyTool({ fixture, pass: 'first', },).status,).toBe(0,);
                expect(sendCalls({ fixture, snapshot: 's001', },),).toBe(1,);
              },
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'freeze as its own step',
      children: [
        it({
          name: 'freezes every subvolume without taking a snapshot or sending, and thaw undoes exactly those',
          fn: async () => {
            await withFixture({
              names: ['a', 'b',],
              fn: (fixture,) => {
                const frozen = runCopyTool({ fixture, pass: 'freeze', },);
                expect(frozen.status,).toBe(0,);
                expect(existsSync(join(fixture.base, 'snap', 's000',),),).toBe(false,);
                expect(readFileSync(join(fixture.main, '.meta',), 'utf8',),).toContain('ro=true',);
                expect(readFileSync(join(fixture.main, 'wt', 'b', '.meta',), 'utf8',),).toContain('ro=true',);
                const thawed = runCopyTool({ fixture, pass: 'thaw', },);
                expect(thawed.status,).toBe(0,);
                expect(readFileSync(join(fixture.main, 'wt', 'a', '.meta',), 'utf8',),).toContain('ro=false',);
              },
            },);
          },
        },),
        it({
          name: 'refuses the final pass while nothing is frozen',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                expect(runCopyTool({ fixture, pass: 'first', },).status,).toBe(0,);
                const result = runCopyTool({ fixture, pass: 'final', },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('not frozen',);
                expect(existsSync(join(fixture.base, 'snap', 'f000',),),).toBe(false,);
              },
            },);
          },
        },),
        it({
          name: 'refuses the final pass when a worktree appeared after the freeze, naming it',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                expect(runCopyTool({ fixture, pass: 'first', },).status,).toBe(0,);
                expect(runCopyTool({ fixture, pass: 'freeze', },).status,).toBe(0,);
                addWorktree({ fixture, name: 'late', uuid: 'u-late', },);
                writeInventory({ fixture, rows: [['a', 'u-a',], ['late', 'u-late',],], },);
                const result = runCopyTool({ fixture, pass: 'final', },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('not frozen',);
                expect(result.stderr,).toContain('/wt/late',);
              },
            },);
          },
        },),
        it({
          name: 'runs the final pass after the freeze: incremental sends from the first-pass copies, then placement',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                expect(runCopyTool({ fixture, pass: 'first', },).status,).toBe(0,);
                expect(runCopyTool({ fixture, pass: 'freeze', },).status,).toBe(0,);
                const result = runCopyTool({ fixture, pass: 'final', env: { STUBS: 'wt/a', }, },);
                expect(result.status,).toBe(0,);
                expect(result.stderr,).toContain('pass final finished',);
                expect(records(fixture,),).toContain('f001.placed.json',);
                const sends = readFileSync(fixture.doubleLog, 'utf8',).split('\n',).filter((line,) =>
                  line.includes('"-p"',)
                );
                expect(sends,).toHaveLength(2,);
              },
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'total deadline',
      children: [
        it({
          name: 'stops with every earlier record kept when the total deadline passes during a transfer',
          fn: async () => {
            await withFixture({
              names: ['a', 'b',],
              fn: (fixture,) => {
                const result = runCopyTool({
                  fixture,
                  pass: 'first',
                  env: { SLOW_SEND: 's001', },
                  extra: ['--deadline-seconds=100', '--total-deadline-seconds=2',],
                },);
                expect(result.status,).toBe(1,);
                expect(result.stderr,).toContain('total deadline',);
                expect(records(fixture,),).toContain('s000.received.json',);
                expect(records(fixture,),).not.toContain('s001.received.json',);
              },
            },);
          },
        },),
        it({
          name: 'bounds each transfer by the smaller of the per-transfer and the remaining total deadline',
          fn: async () => {
            await withFixture({
              names: ['a',],
              fn: (fixture,) => {
                const result = runCopyTool({
                  fixture,
                  pass: 'first',
                  extra: ['--deadline-seconds=14400', '--total-deadline-seconds=50', '--no-scope', '--print',],
                },);
                expect(result.status,).toBe(0,);
                expect(result.stdout,).toMatch(/timeout --kill-after=30 (49|50) /,);
              },
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'external worktrees',
      children: [
        it({
          name: 'pass two reports a file deleted at the source after pass one and deletes nothing from the copy',
          fn: async () => {
            await withFixture({
              names: [],
              fn: (fixture,) => {
                const source = join(fixture.base, 'external',);
                mkdirSync(join(source, 'w1',), { recursive: true, },);
                writeFileSync(join(source, 'w1', 'keep.txt',), 'k',);
                writeFileSync(join(source, 'w1', 'gone.txt',), 'g',);
                const target = join(fixture.base, 'external-final',);
                mkdirSync(target,);
                const extra = [`--external-source=${source}`, `--external-final=${target}`,];
                expect(runCopyTool({ fixture, pass: 'external-first', extra, },).status,).toBe(0,);
                rmSync(join(source, 'w1', 'gone.txt',),);
                const result = runCopyTool({ fixture, pass: 'external-final', extra, },);
                expect(result.status,).toBe(0,);
                expect(result.stderr,).toContain('deleted at the source after pass one: 1 in w1',);
                expect(existsSync(join(target, 'w1', 'gone.txt',),),).toBe(true,);
                expect(readFileSync(join(fixture.base, 'state', 'external-w1.deleted.txt',), 'utf8',),).toContain(
                  'gone.txt',
                );
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
