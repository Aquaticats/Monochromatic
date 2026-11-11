import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import {
  checkoutMarkerName,
  defaultGateLogDirectory,
  defaultGateWorktreeName,
  gateLogDirectoryOf,
  gateLogPath,
  gateRepositoryOf,
  gateWorktreeNameOf,
  gateWorktreePath,
  logDirectoryEnvironmentVariable,
  promptlessGitSettings,
  repositoryEnvironmentVariable,
  pathKindOf,
  repositoryHoldingCheckout,
} from './native-gate-paths.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';

await describe({
  name: 'where a gate runs and where it writes',
  children: [
    it({
      name: 'keeps the log folder of every recorded gate, under the home directory and never in the system temporary directory',
      fn: async () => {
        expect(defaultGateLogDirectory({ home: '/home/a', }),).toBe('/home/a/temp/agent/gate',);
      },
    }),
    it({
      name: 'reads a bare worktree name as a worktree of the checkout it was given',
      fn: async () => {
        expect(gateWorktreePath({
          repository: '/repo',
          worktree: 'gate-wrapper-next-alpha',
        }),).toBe('/repo/.claude/worktrees/gate-wrapper-next-alpha',);
      },
    }),
    it({
      name: 'takes an absolute worktree path as given, so a gate can run outside this repository folder',
      fn: async () => {
        expect(gateWorktreePath({
          repository: '/repo',
          worktree: '/elsewhere/w',
        }),).toBe('/elsewhere/w',);
      },
    }),
    it({
      name: 'names the log after the unit, which is what makes a verdict findable after the unit is collected',
      fn: async () => {
        expect(gateLogPath({
          logDirectory: '/l',
          unitName: 'gpc-gate-a-20261110T180203-0500',
        }),).toBe('/l/gpc-gate-a-20261110T180203-0500.log',);
      },
    }),
    it({
      name: 'finds the checkout of a folder in the tree, at the nearest folder holding a checkout marker',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-paths-', });
        /**
         The checkout's root.
         */
        const repository = join(
          scratch.path,
          'repo',
        );
        /**
         A folder deep inside it, which is where a gate command starts.
         */
        const inside = join(
          repository,
          'package',
          'git-policy',
          'cli',
        );
        await mkdir(
          inside,
          { recursive: true, },
        );
        await mkdir(join(
          repository,
          checkoutMarkerName,
        ));
        expect(await pathKindOf({ path: join(repository, checkoutMarkerName), }),).toBe('folder',);
        expect(await pathKindOf({ path: join(inside, 'nothing'), }),).toBe('missing',);
        expect(await pathKindOf({ path: inside, }),).toBe('folder',);
        expect(await repositoryHoldingCheckout({ from: inside, }),).toEqual({
          kind: 'found',
          checkout: repository,
        },);
        expect(await gateRepositoryOf({
          environment: {},
          from: inside,
        }),).toBe(repository,);
      },
    }),
    it({
      name: 'skips a worktree, whose marker is a file, and answers the main checkout above it',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-paths-', });
        /**
         The main checkout, whose marker is a folder.
         */
        const repository = join(scratch.path, 'repo');
        /**
         A worktree of it, whose marker is a file pointing at the main checkout's git folder.
         */
        const worktree = join(repository, '.claude', 'worktrees', 'gate-under-test');
        await mkdir(join(repository, checkoutMarkerName), { recursive: true, });
        await mkdir(worktree, { recursive: true, });
        await writeFile(join(worktree, checkoutMarkerName), 'gitdir: /repo/.git/worktrees/w\n');
        expect(await pathKindOf({ path: join(worktree, checkoutMarkerName), }),).toBe('other',);
        expect(await repositoryHoldingCheckout({ from: worktree, }),).toEqual({
          kind: 'found',
          checkout: repository,
        },);
      },
    }),
    it({
      name: 'answers no checkout when no folder above the one it starts at holds a marker',
      fn: async () => {
        expect(await repositoryHoldingCheckout({ from: '/no-such-checkout-gate-fixture/repo/package', }),).toEqual({ kind: 'none', },);
        await expect(gateRepositoryOf({
          environment: {},
          from: '/no-such-checkout-gate-fixture/repo/package',
        }),)
          .rejects
          .toThrow('no folder above',);
        await expect(gateRepositoryOf({
          environment: {},
          from: '/no-such-checkout-gate-fixture/repo/package',
        }),)
          .rejects
          .toThrow(repositoryEnvironmentVariable,);
      },
    }),
    it({
      name: 'lets the option name the checkout, and the environment after it, without reading the folder tree',
      fn: async () => {
        expect(await gateRepositoryOf({
          environment: {},
          named: '/given/repo',
        }),).toBe('/given/repo',);
        expect(await gateRepositoryOf({
          environment: { [repositoryEnvironmentVariable]: '/from/env', },
          from: '/no-such-checkout-gate-fixture/repo',
        }),).toBe('/from/env',);
        expect(await gateRepositoryOf({
          environment: { [repositoryEnvironmentVariable]: '/from/env', },
          named: '/given/repo',
        }),).toBe('/given/repo',);
      },
    }),
    it({
      name: 'turns every git credential prompt off, so a checkout never waits for a password',
      fn: async () => {
        expect(promptlessGitSettings,).toEqual({
          GIT_ASKPASS: '',
          GIT_TERMINAL_PROMPT: '0',
        },);
      },
    }),
  ],
});
