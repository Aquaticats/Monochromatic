/**
 The real probe, the three bins and the outside-the-repository installer, against throwaway folders in a disposable
 home. Cases that need a filesystem UUID skip themselves when the scratch filesystem has none (a container, say).
 The stalled-drive case on a hung mount and a wrong filesystem mounted at the real mount point need root and are not
 covered here.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  type LinkIdentity,
  mountHolding,
  parseMountInfo,
  readIdentity,
  realProbe,
  resolveCommand,
  writeIdentity,
} from '../dist/final/node/index.mjs';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

const execFileAsync = promisifyExecFile(execFile,);
/** Milliseconds the wait test lets the command poll before the identity file appears (under the 15 s wait). */
const WRITE_DELAY_MILLISECONDS = 1_500;
const BIN_FOLDER = new URL('../dist/final/node/', import.meta.url,).pathname;
const INSTALLER = new URL('install-outside-repository.ts', import.meta.url,).pathname;

type Run = { status: number; stdout: string; stderr: string; };

async function run({ file, args, cwd = '/', }: { file: string; args: readonly string[]; cwd?: string; },): Promise<Run> {
  try {
    const result = await execFileAsync('node', [file, ...args,], { cwd, encoding: 'utf8', },);
    return { status: 0, stdout: result.stdout, stderr: result.stderr, };
  }
  catch (error) {
    const failure = error as { code: number; stdout: string; stderr: string; };
    return { status: failure.code, stdout: failure.stdout, stderr: failure.stderr, };
  }
}

type HomeFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

async function makeHome(): Promise<HomeFolder> {
  const path = await mkdtemp(join(scratchRoot(), 'repository-link-home-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

async function withHome(fn: (home: string,) => Promise<void>,): Promise<void> {
  await using home = await makeHome();
  await fn(home.path,);
}

type IdentityInput = Omit<LinkIdentity, 'canonicalRoot' | 'driveUuid' | 'linkPath' | 'linkText' | 'mountPoint' | 'nvmeUuid' | 'phase' | 'rootCommit'> & {
  readonly canonicalRoot?: string;
  readonly driveUuid?: string;
  readonly linkPath?: string;
  readonly linkText?: string;
  readonly mountPoint?: string;
  readonly nvmeUuid?: string;
  readonly phase?: LinkIdentity['phase'];
  readonly rootCommit?: string;
};

function identityOf(overrides: IdentityInput,): LinkIdentity {
  return {
    linkPath: '',
    linkText: '',
    canonicalRoot: '',
    mountPoint: '',
    driveUuid: '',
    nvmeUuid: '',
    rootCommit: 'abc123',
    phase: 'after',
    ...overrides,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseMountInfo.name,
      children: [
        it({
          name: 'reads mount points and sources, undoing octal escapes, and finds the longest holding mount',
          fn: async () => {
            const mounts = parseMountInfo(
              [
                '24 1 0:21 / / rw - btrfs /dev/mapper/root rw',
                String.raw`25 24 0:22 / /var/mnt/with\040space rw shared:1 - btrfs /dev/mapper/crypt_sda rw`,
                '',
              ].join('\n',),
            );
            expect(mounts.map((mount,) => mount.mountPoint),).toEqual(['/', '/var/mnt/with space',],);
            expect(mounts[1]?.source,).toBe('/dev/mapper/crypt_sda',);
            expect(mountHolding({ mounts, path: '/var/mnt/with space/repository', },).source,).toBe(
              '/dev/mapper/crypt_sda',
            );
            expect(mountHolding({ mounts, path: '/home/x', },).source,).toBe('/dev/mapper/root',);
            expect(mountHolding({ mounts: [], path: '/x', },).source,).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: 'real probe',
      children: [
        it({
          name: 'tells links, folders, files and missing paths apart without following a final link, and resolves chains',
          fn: async () => {
            await withHome(async (home,) => {
              await mkdir(join(home, 'dir',),);
              await writeFile(join(home, 'file',), 'x',);
              await symlink(join(home, 'dir',), join(home, 'link1',),);
              await symlink(join(home, 'link1',), join(home, 'link2',),);
              await symlink(join(home, 'nowhere',), join(home, 'dangling',),);
              expect((await realProbe.lstat(join(home, 'link2',),)).kind,).toBe('link',);
              expect((await realProbe.lstat(join(home, 'dir',),)).kind,).toBe('directory',);
              expect((await realProbe.lstat(join(home, 'file',),)).kind,).toBe('file',);
              expect((await realProbe.lstat(join(home, 'absent',),)).kind,).toBe('missing',);
              expect((await realProbe.lstat(join(home, 'file', 'under',),)).kind,).toBe('missing',);
              expect(
                await realProbe.realpath(join(home, 'link2',),),
              ).toBe(
                await realProbe.realpath(join(home, 'dir',),),
              );
              expect(
                await realProbe.realpath(join(home, 'dangling',),),
              ).toBe('',);
              expect(
                await realProbe.readText(join(home, 'absent',),),
              ).toBe('',);
              expect(
                await realProbe.readText(join(home, 'file',),),
              ).toBe('x',);
              expect(
                await realProbe.readlink(join(home, 'link1',),),
              ).toBe(join(home, 'dir',),);
              expect(await realProbe.isMountPoint('/',),).toBe(true,);
              expect(
                await realProbe.isMountPoint(join(home, 'dir',),),
              ).toBe(false,);
              expect(
                await realProbe.uuidOf(join(home, 'absent',),),
              ).toBe('',);
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'repository-link-check',
      children: [
        it({
          name: 'prints the missing-identity message and exits 75, or 128 for the Git wrapper',
          fn: async () => {
            await withHome(async (home,) => {
              const identity = join(home, 'none.json',);
              const tempfail = await run({ file: join(BIN_FOLDER, 'check.mjs',), args: [`--identity=${identity}`,], },);
              expect(tempfail.status,).toBe(75,);
              expect(tempfail.stderr,).toContain('identity file is missing',);
              const git = await run({ file: join(BIN_FOLDER, 'check.mjs',), args: [`--identity=${identity}`, '--for=git',], },);
              expect(git.status,).toBe(128,);
              expect(await readIdentity(identity,),).toMatchObject({ found: false, },);
            },);
          },
        },),
        it({
          name: 'says a real folder replaced the link, a missing target on an unmounted drive, and a mounted drive without the folder',
          fn: async () => {
            await withHome(async (home,) => {
              const link = join(home, 'Monochromatic',);
              const identityFile = join(home, 'identity.json',);
              const check = async () => await run({ file: join(BIN_FOLDER, 'check.mjs',), args: [`--identity=${identityFile}`,], },);
              await mkdir(link,);
              await writeIdentity({
                path: identityFile,
                identity: identityOf({ linkPath: link, linkText: '/t', canonicalRoot: '/t', mountPoint: home, },),
              },);
              const folder = await check();
              expect(folder.status,).toBe(75,);
              expect(folder.stderr,).toContain('is a real folder, not the link to the 860; nothing was written',);
              await rm(link, { recursive: true, },);
              await symlink('/does/not/exist', link,);
              await writeIdentity({
                path: identityFile,
                identity: identityOf({ linkPath: link, linkText: '/does/not/exist', canonicalRoot: '/does/not/exist', mountPoint: home, },),
              },);
              const unmounted = await check();
              expect(unmounted.stderr,).toContain('is not mounted yet',);
              await writeIdentity({
                path: identityFile,
                identity: identityOf({ linkPath: link, linkText: '/does/not/exist', canonicalRoot: '/does/not/exist', mountPoint: '/dev/shm', },),
              },);
              const missing = await check();
              expect(missing.stderr,).toContain('is mounted but /does/not/exist does not exist; nothing was written',);
              expect(await realProbe.isMountPoint('/dev/shm',),).toBe(true,);
            },);
          },
        },),
        it({
          name: 'refuses a link to a folder on the wrong filesystem (a tmpfs target where another UUID was recorded)',
          fn: async () => {
            await withHome(async (home,) => {
              const link = join(home, 'Monochromatic',);
              const identityFile = join(home, 'identity.json',);
              await symlink('/dev/shm', link,);
              await writeIdentity({
                path: identityFile,
                identity: identityOf({ linkPath: link, linkText: '/dev/shm', canonicalRoot: '/dev/shm', mountPoint: '/dev/shm', driveUuid: 'RECORDED-UUID', },),
              },);
              const result = await run({ file: join(BIN_FOLDER, 'check.mjs',), args: [`--identity=${identityFile}`,], },);
              expect(result.status,).toBe(75,);
              expect(result.stderr,).toContain('RECORDED-UUID',);
            },);
          },
        },),
        it({
          name: 'is ready for a real repository folder in phase before and for a link chain with awkward names in phase after',
          fn: async () => {
            await withHome(async (home,) => {
              const uuid = await realProbe.uuidOf(home,);
              if (uuid === '') {
                return;
              }
              const identityFile = join(home, 'identity.json',);
              const folder = join(home, 'Monochromatic',);
              await mkdir(join(folder, '.git',), { recursive: true, },);
              await writeIdentity({ path: identityFile, identity: identityOf({ linkPath: folder, nvmeUuid: uuid, phase: 'before', },), },);
              const before = await run({ file: join(BIN_FOLDER, 'check.mjs',), args: [`--identity=${identityFile}`,], },);
              expect(before.status,).toBe(0,);
              expect(before.stdout,).toContain('ready (before the move)',);
              await Promise.all(['with space', 'with\nnewline', 'dots',].map(async (name, index,) => {
                const root = join(home, 'real', name,);
                await mkdir(join(root, '.git',), { recursive: true, },);
                await writeFile(join(root, '.git', 'monochromatic-root-commit',), 'abc123\n',);
                const hop = join(home, `hop ${String(index,)}`,);
                const link = join(home, `link-${String(index,)}`,);
                const linkText = join(home, 'real', '..', `hop ${String(index,)}`,);
                await symlink(root, hop,);
                await symlink(linkText, link,);
                const canonical = await realProbe.realpath(link,);
                const ownIdentityFile = join(home, `identity-${String(index,)}.json`,);
                await writeIdentity({
                  path: ownIdentityFile,
                  identity: identityOf({
                    linkPath: link,
                    linkText,
                    canonicalRoot: canonical,
                    mountPoint: home,
                    driveUuid: uuid,
                  },),
                },);
                const after = await run({ file: join(BIN_FOLDER, 'check.mjs',), args: [`--identity=${ownIdentityFile}`,], },);
                expect(after.status,).toBe(0,);
                expect(after.stdout,).toContain('ready (on the 860)',);
              },),);
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'repository-link-run',
      children: [
        it({
          name: 'does not run the command when the check is not ready, and exits 75',
          fn: async () => {
            await withHome(async (home,) => {
              const marker = join(home, 'ran',);
              const result = await run({
                file: join(BIN_FOLDER, 'run.mjs',),
                args: [`--identity=${join(home, 'none.json',)}`, '--', 'node', '-e', `require('node:fs').writeFileSync(${JSON.stringify(marker,)}, 'x')`,],
              },);
              expect(result.status,).toBe(75,);
              await expect(execFileAsync('test', ['-e', marker,],),).rejects.toBeDefined();
            },);
          },
        },),
        it({
          name: 'with --wait-seconds waits for the identity to appear and then runs the command',
          fn: async () => {
            await withHome(async (home,) => {
              const uuid = await realProbe.uuidOf(home,);
              if (uuid === '') {
                return;
              }
              const identityFile = join(home, 'identity.json',);
              const folder = join(home, 'Monochromatic',);
              await mkdir(join(folder, '.git',), { recursive: true, },);
              const pending = run({
                file: join(BIN_FOLDER, 'run.mjs',),
                args: [`--identity=${identityFile}`, '--wait-seconds=15', '--', 'node', '-e', "console.log('command ran')",],
              },);
              await setTimeout(WRITE_DELAY_MILLISECONDS,);
              await writeIdentity({ path: identityFile, identity: identityOf({ linkPath: folder, nvmeUuid: uuid, phase: 'before', },), },);
              const result = await pending;
              expect(result.status,).toBe(0,);
              expect(result.stdout,).toContain('command ran',);
            },);
          },
        },),
        it({
          name: 'gives up at the wait deadline with the last answer, and refuses an empty command',
          fn: async () => {
            await withHome(async (home,) => {
              const gaveUp = await run({
                file: join(BIN_FOLDER, 'run.mjs',),
                args: [`--identity=${join(home, 'none.json',)}`, '--wait-seconds=2', '--', 'node',],
              },);
              expect(gaveUp.status,).toBe(75,);
              expect(gaveUp.stderr,).toContain('identity file is missing',);
              const empty = await run({ file: join(BIN_FOLDER, 'run.mjs',), args: [`--identity=${join(home, 'none.json',)}`,], },);
              expect(empty.status,).toBe(75,);
              expect(empty.stderr,).toContain('usage',);
            },);
          },
        },),
        it({
          name: resolveCommand.name,
          fn: async () => {
            expect((await resolveCommand({ command: 'node', pathVariable: process.env.PATH ?? '', },)).endsWith('/node',),).toBe(true,);
            expect(await resolveCommand({ command: './x', pathVariable: '', },),).toBe('./x',);
            expect(await resolveCommand({ command: 'no-such-program-here', pathVariable: '/usr/bin', },),).toBe('no-such-program-here',);
          },
        },),
      ],
    },),
    describe({
      name: 'repository-link-record',
      children: [
        it({
          name: 'records phase before for a real folder, refuses phase after for a real folder, and records phase after for a link with its marker',
          fn: async () => {
            await withHome(async (home,) => {
              const uuid = await realProbe.uuidOf(home,);
              if (uuid === '') {
                return;
              }
              const identityFile = join(home, 'identity.json',);
              const folder = join(home, 'Monochromatic',);
              await mkdir(folder,);
              await execFileAsync('/usr/bin/git', ['init', '--quiet', '--initial-branch=main', folder,],);
              await execFileAsync('/usr/bin/git', ['-C', folder, '-c', 'user.name=t', '-c', 'user.email=t@t', 'commit', '--quiet', '--allow-empty', '--message', 'first',],);
              const common = [`--identity=${identityFile}`, `--link-path=${folder}`, `--mount-point=/dev/shm`,];
              const before = await run({ file: join(BIN_FOLDER, 'record.mjs',), args: ['--phase=before', ...common,], },);
              expect(before.status,).toBe(0,);
              expect((await readIdentity(identityFile,)).identity.phase,).toBe('before',);
              const refused = await run({ file: join(BIN_FOLDER, 'record.mjs',), args: ['--phase=after', ...common,], },);
              expect(refused.status,).toBe(75,);
              expect(refused.stderr,).toContain('not a symbolic link',);
              const target = join(home, 'target',);
              await execFileAsync('mv', [folder, target,],);
              await symlink(target, folder,);
              const after = await run({ file: join(BIN_FOLDER, 'record.mjs',), args: ['--phase=after', ...common,], },);
              expect(after.status,).toBe(0,);
              const written = (await readIdentity(identityFile,)).identity;
              expect(written.phase,).toBe('after',);
              expect(written.linkText,).toBe(target,);
              expect(written.nvmeUuid,).toBe(uuid,);
              expect(written.rootCommit,).toHaveLength(40,);
              const again = await run({ file: join(BIN_FOLDER, 'record.mjs',), args: ['--phase=after', ...common,], },);
              expect(again.status,).toBe(0,);
              const badPhase = await run({ file: join(BIN_FOLDER, 'record.mjs',), args: ['--phase=sideways',], },);
              expect(badPhase.status,).toBe(75,);
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'install outside the repository',
      children: [
        it({
          name: 'copies the bins so they run from the copy with nothing of the repository reachable',
          fn: async () => {
            await withHome(async (home,) => {
              const installed = await run({ file: INSTALLER, args: [`--data-home=${home}`,], },);
              expect(installed.status,).toBe(0,);
              const copy = join(home, 'monochromatic-dev', 'repository-link', 'repository-link-check',);
              const result = await run({ file: copy, args: [`--identity=${join(home, 'none.json',)}`,], cwd: home, },);
              expect(result.status,).toBe(75,);
              expect(result.stderr,).toContain('identity file is missing',);
              const again = await run({ file: INSTALLER, args: [`--data-home=${home}`,], },);
              expect(again.status,).toBe(0,);
            },);
          },
        },),
      ],
    },),
  ],
},);
