import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  commitPaths,
  createTempDirectory,
  initializeRepository,
  requireSuccess,
  runRealGit,
  captureWrapper,
} from './worktree-copy-fixture.unit.test.ts';

/**
 * One worktree-add form and destination used by built wrapper catalog.
 */
type WorktreeForm = Readonly<{
  /**
   * Human-readable form name.
   */
  name: string;
  /**
   * Complete arguments passed to built wrapper.
   */
  args: readonly string[];
  /**
   * Created worktree destination.
   */
  destinationRoot: string;
}>;

await describe({
  name: 'worktree creation form coverage',
  concurrency: 1,
  children: [
    it({
      name: 'copies ignored state for branch, force, existing, detached, orphan, convenience, and remote-guess forms',
      fn: async () => {
        await using fixture = await createTempDirectory();
        /**
         * Invoking source repository.
         */
        const repositoryRoot = join(fixture.path, 'repository',);
        /**
         * Local bare remote used by unique remote-guess form.
         */
        const remoteRoot = join(fixture.path, 'remote.git',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({
          repositoryRoot,
          message: 'ignore worktree state',
          paths: ['.gitignore',],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'all forms\n',);
        await Promise.all([
          runRealGit({
            cwd: repositoryRoot,
            args: [
              'branch',
              'existing-form',
            ],
          },),
          runRealGit({
            cwd: repositoryRoot,
            args: [
              'branch',
              'force-form',
            ],
          },),
          runRealGit({
            cwd: fixture.path,
            args: [
              'init',
              '--bare',
              remoteRoot,
            ],
          },),
        ],);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'remote',
            'add',
            'origin',
            remoteRoot,
          ],
        },);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'push',
            'origin',
            'main:refs/heads/remote-guess',
          ],
        },);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'fetch',
            'origin',
          ],
        },);
        /**
         * Destination roots for every accepted Git 2.55 creation family.
         */
        const destination = {
          branch: join(fixture.path, 'branch-form',),
          force: join(fixture.path, 'force-form-worktree',),
          existing: join(fixture.path, 'existing-form-worktree',),
          detached: join(fixture.path, 'detached-form',),
          orphan: join(fixture.path, 'orphan-form',),
          convenience: join(fixture.path, 'convenience-form',),
          remoteGuess: join(fixture.path, 'remote-guess',),
        } as const;
        /**
         * Complete creation-form catalog.
         */
        const forms: readonly WorktreeForm[] = [
          {
            name: 'new branch with implicit HEAD',
            args: ['worktree', 'add', '-b', 'branch-form', destination.branch,],
            destinationRoot: destination.branch,
          },
          {
            name: 'forced branch with explicit start point',
            args: ['worktree', 'add', '-B', 'force-form', destination.force, 'HEAD',],
            destinationRoot: destination.force,
          },
          {
            name: 'existing branch',
            args: ['worktree', 'add', destination.existing, 'existing-form',],
            destinationRoot: destination.existing,
          },
          {
            name: 'detached explicit start point',
            args: ['worktree', 'add', '--detach', destination.detached, 'HEAD',],
            destinationRoot: destination.detached,
          },
          {
            name: 'orphan branch',
            args: ['worktree', 'add', '--orphan', '-b', 'orphan-form', destination.orphan,],
            destinationRoot: destination.orphan,
          },
          {
            name: 'path-only convenience branch',
            args: ['worktree', 'add', destination.convenience,],
            destinationRoot: destination.convenience,
          },
          {
            name: 'unique remote guess',
            args: ['worktree', 'add', '--guess-remote', destination.remoteGuess,],
            destinationRoot: destination.remoteGuess,
          },
        ];
        for (const form of forms) {
          // oxlint-disable-next-line no-await-in-loop -- each Git form creates repository state consumed by following assertion
          requireSuccess(await captureWrapper({
            cwd: repositoryRoot,
            args: form.args,
          },),);
          // oxlint-disable-next-line no-await-in-loop -- each destination must be verified through built wrapper boundary
          expect(await readFile(join(form.destinationRoot, 'state.txt',), 'utf8',),)
            .toBe('all forms\n',);
        }
      },
    },),
  ],
},);
