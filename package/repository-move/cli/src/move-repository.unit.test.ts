/**
 Unit tests of the built copy tool against a btrfs test double: the control flow of both passes, and the four defects
 of the first script (records keyed by position, freeze joined to the copy, no total deadline, external second pass
 that could not report deletions).

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  rm,
  stat,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { execFile, } from 'node:child_process';
import {
  basename,
  join,
} from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  addWorktree,
  type Fixture,
  makeFixture,
  removeWorktree,
  runCopyTool,
  writeInventory,
} from './move-fixture.ts';

/** A folder on a memory filesystem, which is never the filesystem of the scratch folder; the tests only look at it. */
const OTHER_FILESYSTEM_FOLDER = '/dev/shm';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
const execFileAsync = promisifyExecFile(execFile,);

type LinkedFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/** Makes a symbolic link to a folder beside it, and removes the link (never the folder) at the end of the scope. */
async function linkTo(target: string,): Promise<LinkedFolder> {
  const path = `${target}-link`;
  await symlink(target, path,);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { force: true, },);
    },
  };
}

async function records(fixture: Fixture,): Promise<string[]> {
  return (await readdir(join(fixture.base, 'state',),)).filter((name,) => name.endsWith('.json',)).toSorted();
}

async function exists(path: string,): Promise<boolean> {
  return await stat(path,).then(() => true, () => false);
}

async function sendCalls({ fixture, snapshot, }: { fixture: Fixture; snapshot: string; },): Promise<number> {
  return (await readFile(fixture.doubleLog, 'utf8',)).split('\n',).filter((line,) =>
    line.startsWith('["send"',) && line.endsWith(`/snap/${snapshot}"]`,)
  ).length;
}

async function metaOf(path: string,): Promise<string> {
  return await readFile(join(path, '.meta',), 'utf8',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'first pass',
      children: [
        it({
          name: 'copies the checkout and every worktree and records each by name',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('pass first finished',);
            expect(await records(fixture,),).toContain('s002.received.json',);
          },
        },),
        it({
          name: 'stops when a send fails and records nothing for the failed subvolume',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            const result = await runCopyTool({ fixture, pass: 'first', env: { FAIL_SEND: 's001', }, },);
            expect(result.status,).toBe(1,);
            expect(await records(fixture,),).toContain('s000.received.json',);
            expect(await records(fixture,),).not.toContain('s001.received.json',);
            expect(
              await exists(join(fixture.base, 'snap', 's002',),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'refuses a received folder that has no completion record and names the delete for the owner',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await mkdir(join(fixture.base, 'recv', 's001',),);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('exists without a completion record',);
            expect(result.stderr,).toContain('btrfs subvolume delete',);
            expect(
              await exists(join(fixture.base, 'recv', 's001',),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'stops when a received copy is not read-only',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            const result = await runCopyTool({ fixture, pass: 'first', env: { FAIL_RO: 's001', }, },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('is not a complete read-only copy',);
          },
        },),
        it({
          name: 'stops when the inventory lists a subvolume Git does not know',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['hidden', 'u-hidden',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('subvolumes Git does not know',);
          },
        },),
        it({
          name: 'stops when the snapshot of a path has a different UUID than the inventory recorded',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-other',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('was replaced',);
          },
        },),
      ],
    },),
    describe({
      name: 'one folder spelled in different ways (links, dot-dot, trailing slash)',
      children: [
        it({
          name: 'agrees when the main checkout and the inventory mapping are spelled through a link and Git holds the physical paths',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            await using linked = await linkTo(fixture.base,);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['b', 'u-b',],], },);
            const result = await runCopyTool({
              fixture,
              pass: 'first',
              mainPath: join(linked.path, 'main',),
              inventoryBase: linked.path,
            },);
            expect(result.stderr,).toContain('inventory agrees: 2 nested subvolumes',);
            expect(result.status,).toBe(0,);
          },
        },),
        it({
          name: 'agrees when only the inventory mapping goes through a link, or the main path has a trailing slash or a dot-dot segment',
          fn: async () => {
            const spellings: readonly ((fixture: Fixture, linked: string,) => { mainPath: string; inventoryBase: string; })[] = [
              (fixture, linked,) => ({ mainPath: fixture.main, inventoryBase: linked, }),
              (fixture,) => ({ mainPath: `${fixture.main}/`, inventoryBase: fixture.base, }),
              (fixture,) => ({ mainPath: join(fixture.base, 'main', 'wt', '..',), inventoryBase: fixture.base, }),
            ];
            const outcomes = await Promise.all(spellings.map(async (spell,) => {
              await using fixture = await makeFixture({ names: ['a',], },);
              await using linked = await linkTo(fixture.base,);
              await writeInventory({ fixture, rows: [['a', 'u-a',],], },);
              const result = await runCopyTool({ fixture, pass: 'first', ...spell(fixture, linked.path,), },);
              return { status: result.status, agrees: result.stderr.includes('inventory agrees: 1 nested subvolumes',), };
            },),);
            expect(outcomes,).toEqual(spellings.map(() => ({ status: 0, agrees: true, }),),);
          },
        },),
        it({
          name: 'agrees when a worktree was registered through a link',
          fn: async () => {
            await using fixture = await makeFixture({ names: [], },);
            await using linked = await linkTo(fixture.base,);
            await mkdir(join(fixture.main, 'wt',), { recursive: true, },);
            await execFileAsync('/usr/bin/git', ['-c', 'user.name=t', '-c', 'user.email=t@t', '-C', join(linked.path, 'main',), 'worktree', 'add', '--quiet', join(linked.path, 'main', 'wt', 'x',), '-b', 'x',],);
            await writeFile(join(fixture.main, 'wt', 'x', '.stub-subvolume',), 'u-x\n',);
            await writeInventory({ fixture, rows: [['x', 'u-x',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', mainPath: join(linked.path, 'main',), inventoryBase: linked.path, },);
            expect(result.stderr,).toContain('inventory agrees: 1 nested subvolumes',);
            expect(result.status,).toBe(0,);
          },
        },),
        it({
          name: 'a real difference is still refused, naming the canonical path and the spelling it was listed in',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await using linked = await linkTo(fixture.base,);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['hidden', 'u-hidden',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', mainPath: join(linked.path, 'main',), inventoryBase: linked.path, },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('subvolumes Git does not know',);
            expect(result.stderr,).toContain(join(fixture.base, 'main', 'wt', 'hidden',),);
            expect(result.stderr,).toContain(`listed as ${join(linked.path, 'main', 'wt', 'hidden',)}`,);
          },
        },),
      ],
    },),
    describe({
      name: 'send plan: which earlier copy a send may start from',
      children: [
        it({
          name: 'sends a worktree snapshotted from main as an increment on the snapshot of main, an unrelated one in full, and says which',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a', 'u-main',], ['b', 'u-b',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(0,);
            expect(await metaOf(join(fixture.base, 'recv', 's001',),),).toContain('sentParent=s000',);
            expect(await metaOf(join(fixture.base, 'recv', 's002',),),).toContain('sentParent=-',);
            expect(await metaOf(join(fixture.base, 'recv', 's002',),),).toContain('sentClones=\n',);
            expect(result.stderr,).toContain('s001',);
            expect(result.stderr,).toContain('incremental on s000',);
            expect(result.stderr,).toContain('full send',);
          },
        },),
        it({
          name: 'finds the related copy through a chain of parents, and never starts from a copy that is not sent yet',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a', 'u-b',], ['b', 'u-b', 'u-main',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(0,);
            expect(await metaOf(join(fixture.base, 'recv', 's001',),),).toContain('sentParent=s000',);
            expect(await metaOf(join(fixture.base, 'recv', 's002',),),).toContain('sentParent=s000',);
          },
        },),
        it({
          name: 'print mode names the choice of every row and shows a send without clone sources for an unrelated worktree',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a', 'u-main',], ['b', 'u-b',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', run: false, },);
            expect(result.status,).toBe(0,);
            expect(result.stdout,).toContain(`btrfs send -p ${join(fixture.base, 'snap', 's000',)} ${join(fixture.base, 'snap', 's001',)} |`,);
            expect(result.stdout,).toContain(`btrfs send ${join(fixture.base, 'snap', 's002',)} |`,);
            expect(result.stderr,).toContain('incremental on s000',);
            expect(result.stderr,).toContain('full send',);
          },
        },),
        it({
          name: 'refuses to run when the inventory was taken without the parent column, naming the command that has it',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a',],], withParents: false, },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('btrfs subvolume list -u -q',);
            expect(await exists(join(fixture.base, 'snap', 's000',),),).toBe(false,);
          },
        },),
        it({
          name: 'a rerun after a failed send of a related worktree sends it again from the same parent and keeps the earlier records',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a', 'u-main',],], },);
            const failed = await runCopyTool({ fixture, pass: 'first', env: { FAIL_SEND: 's001', }, },);
            expect(failed.status,).toBe(1,);
            const again = await runCopyTool({ fixture, pass: 'first', },);
            expect(again.status,).toBe(0,);
            expect(await sendCalls({ fixture, snapshot: 's000', },),).toBe(1,);
            expect(await metaOf(join(fixture.base, 'recv', 's001',),),).toContain('sentParent=s000',);
          },
        },),
      ],
    },),
    describe({
      name: 'filesystems a btrfs snapshot and a placement need to share',
      children: [
        it({
          name: 'refuses before any snapshot when the snapshot folder is on another filesystem than the sources, naming both paths',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a',],], },);
            const otherFilesystem = join(OTHER_FILESYSTEM_FOLDER, `repository-move-test-${basename(fixture.base,)}`, 'snap',);
            const result = await runCopyTool({ fixture, pass: 'first', snapshotFolder: otherFilesystem, },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('same filesystem',);
            expect(result.stderr,).toContain(otherFilesystem,);
            expect(result.stderr,).toContain(fixture.main,);
            expect(await exists(fixture.doubleLog,) ? await readFile(fixture.doubleLog, 'utf8',) : '',).not.toContain('snapshot',);
            expect(await exists(otherFilesystem,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses before any snapshot when the received folder and the final path are on different filesystems, naming both paths',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a',],], },);
            const otherFilesystem = join(OTHER_FILESYSTEM_FOLDER, `repository-move-test-${basename(fixture.base,)}`, 'recv',);
            const result = await runCopyTool({ fixture, pass: 'first', receivedFolder: otherFilesystem, },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('same filesystem',);
            expect(result.stderr,).toContain(otherFilesystem,);
            expect(result.stderr,).toContain(join(fixture.base, 'final',),);
            expect(await exists(fixture.doubleLog,) ? await readFile(fixture.doubleLog, 'utf8',) : '',).not.toContain('snapshot',);
          },
        },),
        it({
          name: 'accepts a snapshot folder that does not exist yet when its nearest existing folder shares the sources filesystem',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            await writeInventory({ fixture, rows: [['a', 'u-a',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', snapshotFolder: join(fixture.base, 'not', 'made', 'yet',), },);
            expect(result.stderr,).not.toContain('same filesystem',);
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
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            expect((await runCopyTool({ fixture, pass: 'first', env: { FAIL_SEND: 's002', }, },)).status,).toBe(1,);
            await removeWorktree({ fixture, name: 'b', },);
            await addWorktree({ fixture, name: 'c', uuid: 'u-c', },);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['c', 'u-c',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('added since the pass started',);
            expect(result.stderr,).toContain('/wt/c',);
            expect(result.stderr,).toContain('no longer present',);
            expect(result.stderr,).toContain('/wt/b',);
            expect(await sendCalls({ fixture, snapshot: 's001', },),).toBe(1,);
            const manifest = await readFile(join(fixture.base, 'state', 'first-pass-manifest.tsv',), 'utf8',);
            expect(manifest,).toContain('s003\t',);
            expect(manifest,).not.toContain('/wt/b',);
            expect(await records(fixture,),).toContain('s003.received.json',);
          },
        },),
        it({
          name: 'treats a path whose subvolume was replaced (new UUID) as a new subvolume and leaves the old record alone',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            await writeFile(join(fixture.main, 'wt', 'a', '.stub-subvolume',), 'u-a2\n',);
            await writeInventory({ fixture, rows: [['a', 'u-a2',],], },);
            const result = await runCopyTool({ fixture, pass: 'first', },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('replaced',);
            expect(await records(fixture,),).toContain('s001.received.json',);
            expect(await records(fixture,),).toContain('s002.received.json',);
            expect(await sendCalls({ fixture, snapshot: 's001', },),).toBe(1,);
          },
        },),
        it({
          name: 'sends nothing again when the same worktrees are present and the pass already finished',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            expect(await sendCalls({ fixture, snapshot: 's001', },),).toBe(1,);
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
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            expect((await runCopyTool({ fixture, pass: 'freeze', },)).status,).toBe(0,);
            expect(
              await exists(join(fixture.base, 'snap', 's000',),),
            ).toBe(false,);
            expect(await metaOf(fixture.main,),).toContain('ro=true',);
            expect(
              await metaOf(join(fixture.main, 'wt', 'b',),),
            ).toContain('ro=true',);
            expect((await runCopyTool({ fixture, pass: 'thaw', },)).status,).toBe(0,);
            expect(
              await metaOf(join(fixture.main, 'wt', 'a',),),
            ).toContain('ro=false',);
          },
        },),
        it({
          name: 'thaws exactly the frozen subvolumes even when worktrees changed since, without reading the inventory',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'freeze', },)).status,).toBe(0,);
            await addWorktree({ fixture, name: 'late', uuid: 'u-late', },);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['hidden', 'u-hidden',],], },);
            const result = await runCopyTool({ fixture, pass: 'thaw', },);
            expect(result.status,).toBe(0,);
            expect(await metaOf(fixture.main,),).toContain('ro=false',);
            expect(
              await metaOf(join(fixture.main, 'wt', 'a',),),
            ).toContain('ro=false',);
          },
        },),
        it({
          name: 'refuses thaw when nothing was frozen',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            const result = await runCopyTool({ fixture, pass: 'thaw', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('froze nothing',);
          },
        },),
        it({
          name: 'refuses the final pass while nothing is frozen',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            const result = await runCopyTool({ fixture, pass: 'final', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('not frozen',);
            expect(
              await exists(join(fixture.base, 'snap', 'f000',),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'refuses the final pass when a worktree appeared after the freeze, naming it',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            expect((await runCopyTool({ fixture, pass: 'freeze', },)).status,).toBe(0,);
            await addWorktree({ fixture, name: 'late', uuid: 'u-late', },);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['late', 'u-late',],], },);
            const result = await runCopyTool({ fixture, pass: 'final', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('not frozen',);
            expect(result.stderr,).toContain('/wt/late',);
          },
        },),
        it({
          name: 'refuses the final pass when a subvolume was thawed after the freeze',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            expect((await runCopyTool({ fixture, pass: 'freeze', },)).status,).toBe(0,);
            await writeFile(join(fixture.main, '.meta',), 'uuid=u-main\nparent=-\nreceived=-\nro=false\n',);
            const result = await runCopyTool({ fixture, pass: 'final', },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('thawed after the freeze',);
          },
        },),
        it({
          name: 'runs the final pass after the freeze: incremental sends from the first-pass copies, then placement',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            expect((await runCopyTool({ fixture, pass: 'freeze', },)).status,).toBe(0,);
            const result = await runCopyTool({ fixture, pass: 'final', env: { STUBS: 'wt/a', }, },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('pass final finished',);
            expect(await records(fixture,),).toContain('f001.placed.json',);
            const parented = (await readFile(fixture.doubleLog, 'utf8',)).split('\n',).filter((line,) =>
              line.includes('"-p"',)
            );
            expect(parented,).toHaveLength(2,);
          },
        },),
        it({
          name: 'sends a worktree that has no first-pass copy in full and says so',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            expect((await runCopyTool({ fixture, pass: 'first', },)).status,).toBe(0,);
            await addWorktree({ fixture, name: 'late', uuid: 'u-late', },);
            await writeInventory({ fixture, rows: [['a', 'u-a',], ['late', 'u-late',],], },);
            expect((await runCopyTool({ fixture, pass: 'freeze', },)).status,).toBe(0,);
            const result = await runCopyTool({ fixture, pass: 'final', env: { STUBS: 'wt/a,wt/late', }, },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('has no first-pass copy',);
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
            await using fixture = await makeFixture({ names: ['a', 'b',], },);
            const result = await runCopyTool({
              fixture,
              pass: 'first',
              env: { SLOW_SEND: 's001', },
              extra: ['--deadline-seconds=100', '--total-deadline-seconds=8',],
            },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('total deadline',);
            expect(await records(fixture,),).toContain('s000.received.json',);
            expect(await records(fixture,),).not.toContain('s001.received.json',);
          },
        },),
        it({
          name: 'bounds each transfer by the smaller of the per-transfer and the remaining total deadline',
          fn: async () => {
            await using fixture = await makeFixture({ names: ['a',], },);
            const result = await runCopyTool({
              fixture,
              pass: 'first',
              extra: ['--deadline-seconds=14400', '--total-deadline-seconds=50', '--print',],
            },);
            expect(result.status,).toBe(0,);
            expect(result.stdout.includes('timeout --kill-after=30 49 ',) || result.stdout.includes('timeout --kill-after=30 50 ',),).toBe(true,);
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
            await using fixture = await makeFixture({ names: [], },);
            const source = join(fixture.base, 'external',);
            await mkdir(join(source, 'w1',), { recursive: true, },);
            await writeFile(join(source, 'w1', 'keep.txt',), 'k',);
            await writeFile(join(source, 'w1', 'gone.txt',), 'g',);
            const target = join(fixture.base, 'external-final',);
            await mkdir(target,);
            const extra = [`--external-source=${source}`, `--external-final=${target}`,];
            expect((await runCopyTool({ fixture, pass: 'external-first', extra, },)).status,).toBe(0,);
            await rm(join(source, 'w1', 'gone.txt',),);
            const result = await runCopyTool({ fixture, pass: 'external-final', extra, },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('deleted at the source after pass one: 1 in w1',);
            expect(
              await exists(join(target, 'w1', 'gone.txt',),),
            ).toBe(true,);
            expect(await readFile(join(fixture.base, 'state', 'external-w1.deleted.txt',), 'utf8',),).toContain(
              'gone.txt',
            );
          },
        },),
        it({
          name: 'pass two copies a folder that appeared after pass one in full',
          fn: async () => {
            await using fixture = await makeFixture({ names: [], },);
            const source = join(fixture.base, 'external',);
            await mkdir(join(source, 'late',), { recursive: true, },);
            await writeFile(join(source, 'late', 'f.txt',), 'f',);
            const target = join(fixture.base, 'external-final',);
            await mkdir(target,);
            const extra = [`--external-source=${source}`, `--external-final=${target}`,];
            const result = await runCopyTool({ fixture, pass: 'external-final', extra, },);
            expect(result.status,).toBe(0,);
            expect(result.stderr,).toContain('has no first-pass copy',);
            expect(
              await exists(join(target, 'late', 'f.txt',),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'refuses a folder name it would have to quote into a command',
          fn: async () => {
            await using fixture = await makeFixture({ names: [], },);
            const source = join(fixture.base, 'external',);
            await mkdir(join(source, 'has space',), { recursive: true, },);
            const target = join(fixture.base, 'external-final',);
            await mkdir(target,);
            const result = await runCopyTool({
              fixture,
              pass: 'external-first',
              extra: [`--external-source=${source}`, `--external-final=${target}`,],
            },);
            expect(result.status,).toBe(1,);
            expect(result.stderr,).toContain('has space',);
          },
        },),
      ],
    },),
  ],
},);
