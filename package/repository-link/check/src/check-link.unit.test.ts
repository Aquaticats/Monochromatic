/**
 The table of answers of the repository link check against an in-memory probe, plus the deadline against a probe that
 never answers. The real probe, the bins and the installer are in `bins.unit.test.ts`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  checkRepositoryLink,
  type EntryInfo,
  type LinkIdentity,
  type Probe,
  RepositoryLinkNotReadyError,
  openRepositoryRoot,
  writeIdentity,
} from '../dist/final/node/index.mjs';
import { mkdtemp, mkdir, readFile, rm, } from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

const LINK = '/home/u/Monochromatic';
const ROOT = '/mnt/860/repository/Monochromatic';
const MOUNT = '/mnt/860';

const AFTER: LinkIdentity = {
  linkPath: LINK,
  linkText: ROOT,
  canonicalRoot: ROOT,
  mountPoint: MOUNT,
  driveUuid: 'DRIVE',
  nvmeUuid: 'NVME',
  rootCommit: 'abc123',
  phase: 'after',
};
const BEFORE: LinkIdentity = { ...AFTER, phase: 'before', linkText: '', };

type World = {
  entries: Record<string, EntryInfo['kind']>;
  links: Record<string, string>;
  real: Record<string, string>;
  texts: Record<string, string>;
  uuids: Record<string, string>;
  mounted: readonly string[];
};

/** The world in which the repository is ready on the 860, with the root at the given path. */
function readyAfterAt(root: string,): World {
  return {
    entries: { [LINK]: 'link', [root]: 'directory', [`${root}/.git`]: 'directory', },
    links: { [LINK]: root, },
    real: { [LINK]: root, [MOUNT]: MOUNT, },
    texts: { [`${root}/.git/monochromatic-root-commit`]: 'abc123\n', },
    uuids: { [root]: 'DRIVE', [`${root}/.git`]: 'DRIVE', },
    mounted: [MOUNT,],
  };
}

function readyAfter(): World {
  return readyAfterAt(ROOT,);
}

/** A copy of a record without one key. */
function without<Value,>(record: Record<string, Value>, key: string,): Record<string, Value> {
  return Object.fromEntries(Object.entries(record,).filter(([name,],) => name !== key),);
}

type ScratchFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function makeFolder(): Promise<ScratchFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'repository-link-open-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

function probeOf(world: World,): Probe {
  return {
    lstat: async (path,) => {
      const kind = world.entries[path];
      return kind === undefined ? { exists: false, kind: 'missing', } : { exists: true, kind, };
    },
    readlink: async (path,) => world.links[path] ?? '',
    realpath: async (path,) => world.real[path] ?? '',
    readText: async (path,) => world.texts[path] ?? '',
    uuidOf: async (path,) => world.uuids[path] ?? '',
    isMountPoint: async (path,) => world.mounted.includes(path,),
  };
}

async function answerFor({ identity, world, }: { identity: LinkIdentity; world: World; },) {
  return await checkRepositoryLink({ read: { found: true, identity, }, probe: probeOf(world,), },);
}

// The rows below are shared with the Rust copy of the check in the Git wrapper: the Rust suite
// package/git-policy/cli/src/native/repository_link_check/repository_link_shared_table_tests.rs runs the same file.
// Edit a row in the JSON and both suites follow; the crate holds the file because the container gate copies only the crate.
const SHARED_TABLE_PATH = fileURLToPath(new URL(
  '../../../git-policy/cli/src/native/repository_link_check/repository_link_shared_table.json',
  import.meta.url,
),);

type SharedRow = {
  readonly name: string;
  readonly found: boolean;
  readonly identity: Partial<LinkIdentity>;
  readonly world: World;
  readonly expect: { readonly kind: string; readonly ready: boolean; readonly message?: string; readonly contains?: readonly string[]; };
};

type SharedTable = {
  readonly baseIdentity: LinkIdentity;
  readonly rowCount: number;
  readonly rows: readonly SharedRow[];
};

async function readSharedTable(): Promise<SharedTable> {
  return JSON.parse(await readFile(SHARED_TABLE_PATH, 'utf8',),) as SharedTable;
}

/**
 The table, or nothing when the path no longer resolves: the test of the path then fails by name instead of the whole file
 failing at load.
 */
async function readSharedTableOrNothing(): Promise<SharedTable | undefined> {
  try {
    return await readSharedTable();
  }
  catch {
    return undefined;
  }
}

const sharedTable = await readSharedTableOrNothing();

await describe({
  name: checkRepositoryLink.name,
  children: [
    describe({
      name: 'phase after',
      children: [
        it({
          name: 'is ready for a link to the recorded root on the recorded filesystem with the recorded marker',
          fn: async () => {
            const answer = await answerFor({ identity: AFTER, world: readyAfter(), },);
            expect(answer.kind,).toBe('ready-after',);
            expect(answer.ready,).toBe(true,);
          },
        },),
        it({
          name: 'says the drive is not mounted, with the plan message, when the target is missing and the mount point is not mounted',
          fn: async () => {
            const world = { ...readyAfter(), real: {}, mounted: [], };
            const answer = await answerFor({ identity: AFTER, world, },);
            expect(answer.kind,).toBe('drive-not-mounted',);
            expect(answer.message,).toBe(
              `the 860 drive is not mounted yet (${MOUNT} is empty), so the repository at ${LINK} is not there; `
                + 'unlock it (the mount-860 service does that at login) and run this again',
            );
          },
        },),
        it({
          name: 'says the folder is missing when the drive is mounted and the target is missing',
          fn: async () => {
            const world = { ...readyAfter(), real: { [MOUNT]: MOUNT, }, };
            const answer = await answerFor({ identity: AFTER, world, },);
            expect(answer.kind,).toBe('mounted-folder-missing',);
            expect(answer.message,).toBe(
              `the 860 drive is mounted but ${ROOT} does not exist; nothing was written`,
            );
          },
        },),
        it({
          name: 'says a real folder replaced the link, with the plan message',
          fn: async () => {
            const world = readyAfter();
            world.entries[LINK] = 'directory';
            const answer = await answerFor({ identity: AFTER, world, },);
            expect(answer.kind,).toBe('real-folder-replaced-link',);
            expect(answer.message,).toBe(`${LINK} is a real folder, not the link to the 860; nothing was written`,);
          },
        },),
        it({
          name: 'says the link is gone when nothing is at the path',
          fn: async () => {
            const world = readyAfter();
            world.entries = without(world.entries, LINK);
            expect((await answerFor({ identity: AFTER, world, },)).kind,).toBe('link-missing',);
          },
        },),
        it({
          name: 'refuses a link whose text differs from the recorded text, naming both',
          fn: async () => {
            const world = readyAfter();
            world.links[LINK] = '/elsewhere';
            const answer = await answerFor({ identity: AFTER, world, },);
            expect(answer.kind,).toBe('wrong-repository',);
            expect(answer.message,).toContain('/elsewhere',);
            expect(answer.message,).toContain(ROOT,);
          },
        },),
        it({
          name: 'refuses a path that is a file, not a link',
          fn: async () => {
            const world = readyAfter();
            world.entries[LINK] = 'file';
            expect((await answerFor({ identity: AFTER, world, },)).kind,).toBe('wrong-repository',);
          },
        },),
        it({
          name: 'refuses a link that resolves to another root than the approved one',
          fn: async () => {
            const world = readyAfter();
            world.real[LINK] = '/mnt/860/repository/Other';
            expect((await answerFor({ identity: AFTER, world, },)).kind,).toBe('wrong-repository',);
          },
        },),
        it({
          name: 'refuses a target on another filesystem and names both UUIDs',
          fn: async () => {
            const world = readyAfter();
            world.uuids[ROOT] = 'OTHER';
            const answer = await answerFor({ identity: AFTER, world, },);
            expect(answer.kind,).toBe('wrong-filesystem',);
            expect(answer.message,).toContain('OTHER',);
            expect(answer.message,).toContain('DRIVE',);
          },
        },),
        it({
          name: 'refuses a wrong filesystem mounted at the mount point (a target with no UUID at all)',
          fn: async () => {
            const world = readyAfter();
            world.uuids = without(world.uuids, ROOT);
            const answer = await answerFor({ identity: AFTER, world, },);
            expect(answer.kind,).toBe('wrong-filesystem',);
            expect(answer.message,).toContain('unknown',);
          },
        },),
        it({
          name: 'refuses a root without a .git folder, and a .git on another filesystem',
          fn: async () => {
            const noGit = readyAfter();
            noGit.entries = without(noGit.entries, `${ROOT}/.git`);
            expect((await answerFor({ identity: AFTER, world: noGit, },)).kind,).toBe('not-a-repository',);
            const otherGit = readyAfter();
            otherGit.uuids[`${ROOT}/.git`] = 'OTHER';
            expect((await answerFor({ identity: AFTER, world: otherGit, },)).kind,).toBe('wrong-filesystem',);
          },
        },),
        it({
          name: 'refuses a root whose marker is absent or differs, naming both values',
          fn: async () => {
            const absent = readyAfter();
            absent.texts = without(absent.texts, `${ROOT}/.git/monochromatic-root-commit`);
            expect((await answerFor({ identity: AFTER, world: absent, },)).kind,).toBe('wrong-repository',);
            const differs = readyAfter();
            differs.texts[`${ROOT}/.git/monochromatic-root-commit`] = 'zzz';
            const answer = await answerFor({ identity: AFTER, world: differs, },);
            expect(answer.message,).toContain('zzz',);
            expect(answer.message,).toContain('abc123',);
          },
        },),
        it({
          name: 'refuses an identity that recorded no drive UUID or no root commit',
          fn: async () => {
            expect((await answerFor({ identity: { ...AFTER, driveUuid: '', }, world: readyAfter(), },)).ready,).toBe(false,);
            expect((await answerFor({ identity: { ...AFTER, rootCommit: '', }, world: readyAfter(), },)).ready,).toBe(false,);
          },
        },),
        it({
          name: 'handles adversarial paths: a target with a newline, a space, dot-dot segments, and a link chain',
          fn: async () => {
            await Promise.all(['/mnt/860/a\nb', '/mnt/860/a b', '/mnt/860/x/../y', '/mnt/860/$(touch z)',].map(async (target,) => {
              const world = readyAfter();
              const identity = { ...AFTER, linkText: target, canonicalRoot: '/mnt/860/real', };
              world.links[LINK] = target;
              world.real[LINK] = '/mnt/860/real';
              world.entries['/mnt/860/real'] = 'directory';
              world.entries['/mnt/860/real/.git'] = 'directory';
              world.uuids['/mnt/860/real'] = 'DRIVE';
              world.uuids['/mnt/860/real/.git'] = 'DRIVE';
              world.texts['/mnt/860/real/.git/monochromatic-root-commit'] = 'abc123';
              expect((await answerFor({ identity, world, },)).kind,).toBe('ready-after',);
            },),);
          },
        },),
      ],
    },),
    describe({
      name: 'phase before',
      children: [
        it({
          name: 'is ready for a real folder with a .git folder on the recorded NVMe',
          fn: async () => {
            const world: World = {
              entries: { [LINK]: 'directory', [`${LINK}/.git`]: 'directory', },
              links: {},
              real: {},
              texts: {},
              uuids: { [LINK]: 'NVME', },
              mounted: [],
            };
            expect((await answerFor({ identity: BEFORE, world, },)).kind,).toBe('ready-before',);
          },
        },),
        it({
          name: 'refuses a link, another filesystem, and a missing .git',
          fn: async () => {
            const base: World = {
              entries: { [LINK]: 'directory', [`${LINK}/.git`]: 'directory', },
              links: {},
              real: {},
              texts: {},
              uuids: { [LINK]: 'NVME', },
              mounted: [],
            };
            expect((await answerFor({ identity: BEFORE, world: { ...base, entries: { [LINK]: 'link', }, }, },)).kind,).toBe(
              'not-a-repository',
            );
            expect((await answerFor({ identity: BEFORE, world: { ...base, uuids: { [LINK]: 'OTHER', }, }, },)).kind,).toBe(
              'wrong-filesystem',
            );
            expect(
              (await answerFor({ identity: BEFORE, world: { ...base, entries: { [LINK]: 'directory', }, }, },)).kind,
            ).toBe('not-a-repository',);
          },
        },),
      ],
    },),
    describe({
      name: 'the table shared with the Git wrapper',
      children: [
        it({
          name: 'resolves from this suite to the file inside the Git wrapper crate, with the row count it declares',
          fn: async () => {
            const table = await readSharedTable();
            expect(table.rows.length,).toBe(table.rowCount,);
            expect(table.rowCount,).toBeGreaterThan(0,);
          },
        },),
        ...(sharedTable?.rows ?? []).map(function rowTest(row,) {
          return it({
            name: row.name,
            fn: async () => {
              if (sharedTable === undefined) {
                throw new Error(`the shared table ${SHARED_TABLE_PATH} does not resolve`,);
              }
              const identity: LinkIdentity = { ...sharedTable.baseIdentity, ...row.identity, };
              const answer = await checkRepositoryLink({
                read: { found: row.found, identity, },
                probe: probeOf(row.world,),
              },);
              expect(answer.kind,).toBe(row.expect.kind,);
              expect(answer.ready,).toBe(row.expect.ready,);
              if (row.expect.message !== undefined) {
                expect(answer.message,).toBe(row.expect.message,);
              }
              for (const part of row.expect.contains ?? []) {
                expect(answer.message,).toContain(part,);
              }
            },
          },);
        },),
      ],
    },),
    describe({
      name: 'identity and deadline',
      children: [
        it({
          name: 'says the identity file is missing when none was read',
          fn: async () => {
            const answer = await checkRepositoryLink({
              read: { found: false, identity: AFTER, },
              probe: probeOf(readyAfter(),),
            },);
            expect(answer.kind,).toBe('identity-missing',);
          },
        },),
        it({
          name: 'answers that the drive did not answer when the filesystem never replies, within the deadline',
          fn: async () => {
            const stalled: Probe = { ...probeOf(readyAfter(),), lstat: () => new Promise(() => {}), };
            const started = performance.now();
            const answer = await checkRepositoryLink({
              read: { found: true, identity: AFTER, },
              probe: stalled,
              deadlineSeconds: 0.3,
            },);
            expect(answer.kind,).toBe('drive-did-not-answer',);
            expect(performance.now() - started,).toBeLessThan(3_000,);
          },
        },),
      ],
    },),
    describe({
      name: openRepositoryRoot.name,
      children: [
        it({
          name: 'opens the resolved root as a directory descriptor when ready, and refuses with the plain message when not',
          fn: async () => {
            await using folder = await makeFolder();
            await mkdir(join(folder.path, 'root',),);
            const identityFile = join(folder.path, 'identity.json',);
            const root = join(folder.path, 'root',);
            const identity = { ...AFTER, linkText: root, canonicalRoot: root, };
            await writeIdentity({ path: identityFile, identity, },);
            const world = readyAfterAt(root,);
            const ready = probeOf(world,);
            const opened = await openRepositoryRoot({ identityFile, probe: ready, },);
            expect(opened.descriptorPath.startsWith('/proc/self/fd/',),).toBe(true,);
            await opened.handle.close();
            let caught: unknown;
            try {
              await openRepositoryRoot({ identityFile, probe: probeOf({ ...world, entries: { [LINK]: 'directory', }, },), },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(RepositoryLinkNotReadyError,);
            expect((caught as Error).message,).toContain('real folder',);
          },
        },),
      ],
    },),
  ],
},);
