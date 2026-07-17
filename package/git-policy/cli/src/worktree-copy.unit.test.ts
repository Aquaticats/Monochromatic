import {
  chmod,
  lstat,
  mkdir,
  readFile,
  readlink,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import nanoSpawn from 'nano-spawn';

import {
  captureWrapper,
  commitPaths,
  copySummaryLines,
  createTempDirectory,
  initializeRepository,
  permissionMode,
  requireFailure,
  requireSuccess,
  runRealGit,
  WRAPPER_PATH,
  writePostCheckoutHook,
} from './worktree-copy-fixture.unit.test.ts';

await describe({
  name: 'automatic ignored-state worktree copying',
  concurrency: 1,
  children: [
    it({
      name: 'copies the standard ignore stack, structure, symlinks, and permission bits',
      fn: async () => {
        await using fixture = await createTempDirectory();
        /**
         * Source repository root.
         */
        const repositoryRoot = join(fixture.path, 'repository',);
        /**
         * New linked-worktree root.
         */
        const destinationRoot = join(fixture.path, 'topic',);
        await initializeRepository(repositoryRoot,);
        await Promise.all([
          writeFile(
            join(repositoryRoot, '.gitignore',),
            'cache/\nfrom-tree.txt\nlink.txt\n',
          ),
          writeFile(
            join(repositoryRoot, '.git', 'info', 'exclude',),
            'from-info.txt\n',
          ),
          writeFile(
            join(fixture.path, 'global-excludes',),
            'from-global.txt\n',
          ),
          mkdir(
            join(repositoryRoot, 'cache',),
            { mode: 0o700, },
          ),
        ],);
        await commitPaths({
          repositoryRoot,
          message: 'add ignore contract',
          paths: ['.gitignore',],
        },);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'config',
            'core.excludesFile',
            join(fixture.path, 'global-excludes',),
          ],
        },);
        await Promise.all([
          writeFile(join(repositoryRoot, 'cache', 'nested.txt',), 'nested\n', { mode: 0o640, },),
          writeFile(join(repositoryRoot, 'from-tree.txt',), 'tree\n', { mode: 0o751, },),
          writeFile(join(repositoryRoot, 'from-info.txt',), 'info\n',),
          writeFile(join(repositoryRoot, 'from-global.txt',), 'global\n',),
          symlink('cache/nested.txt', join(repositoryRoot, 'link.txt',),),
        ],);
        await Promise.all([
          chmod(join(repositoryRoot, 'cache',), 0o500,),
          chmod(join(repositoryRoot, 'cache', 'nested.txt',), 0o640,),
          chmod(join(repositoryRoot, 'from-tree.txt',), 0o751,),
        ],);

        /**
         * Successful built-wrapper worktree creation.
         */
        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'topic',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'cache', 'nested.txt',), 'utf8',),).toBe('nested\n',);
        expect(await readFile(join(destinationRoot, 'from-tree.txt',), 'utf8',),).toBe('tree\n',);
        expect(await readFile(join(destinationRoot, 'from-info.txt',), 'utf8',),).toBe('info\n',);
        expect(await readFile(join(destinationRoot, 'from-global.txt',), 'utf8',),).toBe('global\n',);
        expect(
          await readlink(join(destinationRoot, 'link.txt',),),
        ).toBe('cache/nested.txt',);
        expect(
          await permissionMode(join(destinationRoot, 'cache',),),
        ).toBe(0o500,);
        expect(
          await permissionMode(join(destinationRoot, 'cache', 'nested.txt',),),
        ).toBe(0o640,);
        expect(
          await permissionMode(join(destinationRoot, 'from-tree.txt',),),
        ).toBe(0o751,);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'detects an ordinary Git alias and copies after no-checkout registration',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'alias-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({
          repositoryRoot,
          message: 'ignore state',
          paths: ['.gitignore',],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'source state\n',);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'config',
            'alias.new-worktree',
            'worktree add --no-checkout -b alias-topic',
          ],
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'new-worktree',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('source state\n',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'copies source-ignored state even when destination reports it untracked',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'old-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'branch',
            'old',
          ],
        },);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({
          repositoryRoot,
          message: 'ignore local state',
          paths: ['.gitignore',],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'source only\n',);

        requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            destinationRoot,
            'old',
          ],
        },),);
        const status = await runRealGit({
          cwd: destinationRoot,
          args: [
            'status',
            '--short',
            '--',
            'state.txt',
          ],
        },);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('source only\n',);
        expect(status.stdout,).toContain('?? state.txt',);
      },
    },),

    it({
      name: 'retains differing tracked destination collision and exits two',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'collision-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, 'state.txt',), 'tracked old\n',);
        await commitPaths({
          repositoryRoot,
          message: 'track old state',
          paths: ['state.txt',],
        },);
        await runRealGit({ cwd: repositoryRoot, args: ['branch', 'old',], },);
        await rm(join(repositoryRoot, 'state.txt',),);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await runRealGit({ cwd: repositoryRoot, args: ['add', '--', '.gitignore', 'state.txt',], },);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'commit',
            '--no-gpg-sign',
            '-m',
            'replace tracked state with ignore',
          ],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'ignored new\n',);

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            destinationRoot,
            'old',
          ],
        },),);

        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('would overwrite differing destination entry',);
        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('tracked old\n',);
      },
    },),

    it({
      name: 'accepts an exact hook-created destination entry',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'hook-match',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore hook state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'same\n',);
        await writePostCheckoutHook({
          repositoryRoot,
          body: "require('node:fs').writeFileSync('state.txt', 'same\\n');",
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'hook-match',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('same\n',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'settles recursive hook worktree creation through inherited outer lease',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'outer-topic',);
        const nestedRoot = join(fixture.path, 'nested-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore recursive hook state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'outer source\n',);
        await writePostCheckoutHook({
          repositoryRoot,
          body: `if (process.env.CLI_GIT_RECURSIVE_HOOK !== '1') {
  const { spawnSync } = require('node:child_process');
  const result = spawnSync(process.execPath, [${JSON.stringify(WRAPPER_PATH,)}, 'worktree', 'add', '-b', 'nested-topic', ${JSON.stringify(nestedRoot,)}], {
    cwd: process.cwd(),
    env: { ...process.env, CLI_GIT_RECURSIVE_HOOK: '1' },
    stdio: 'inherit',
  });
  if (result.status !== 0) process.exitCode = result.status ?? 1;
}`,
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'outer-topic',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('outer source\n',);
        expect(await readFile(join(nestedRoot, 'state.txt',), 'utf8',),).toBe('outer source\n',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'copies into a worktree retained after post-checkout failure and preserves Git status',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'hook-failure',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'copied after failure\n',);
        await writePostCheckoutHook({
          repositoryRoot,
          body: 'process.exitCode = 7;',
        },);

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'hook-failure',
            destinationRoot,
          ],
        },),);

        expect(error.exitCode,).toBe(7,);
        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('copied after failure\n',);
        expect(copySummaryLines(error.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'uses fallback status one when signaled Git and copy both fail',
      skip: process.platform === 'win32',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'signaled-collision',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'source\n',);
        await writePostCheckoutHook({
          repositoryRoot,
          body: "require('node:fs').writeFileSync('state.txt', 'hook\\n'); process.kill(process.ppid, 'SIGTERM');",
        },);

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'signaled-collision',
            destinationRoot,
          ],
        },),);

        expect(error.exitCode,).toBe(1,);
        expect(error.stderr,).toContain('would overwrite differing destination entry',);
        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('hook\n',);
      },
    },),

    it({
      name: 'excludes a nested registered worktree while copying its ignored parent tree',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(repositoryRoot, 'sandboxes', 'nested-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'sandboxes/\n',);
        await commitPaths({ repositoryRoot, message: 'ignore sandboxes', paths: ['.gitignore',], },);
        await mkdir(join(repositoryRoot, 'sandboxes',),);
        await writeFile(join(repositoryRoot, 'sandboxes', 'local.txt',), 'local sibling\n',);

        requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'nested-topic',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'sandboxes', 'local.txt',), 'utf8',),).toBe('local sibling\n',);
        expect(await readFile(join(destinationRoot, '.git',), 'utf8',),).toContain('gitdir:',);
      },
    },),

    it({
      name: 'ignores a stale missing registered root while detecting the new worktree',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const staleRoot = join(fixture.path, 'stale',);
        const destinationRoot = join(fixture.path, 'healthy',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'healthy\n',);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'stale',
            staleRoot,
          ],
        },);
        await rm(staleRoot, { recursive: true, force: true, },);

        requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'healthy',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('healthy\n',);
      },
    },),

    it({
      name: 'uses an empty source set for bare repository creation',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const bareRoot = join(fixture.path, 'repository.git',);
        const destinationRoot = join(fixture.path, 'bare-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: fixture.path,
          args: [
            'clone',
            '--bare',
            repositoryRoot,
            bareRoot,
          ],
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: fixture.path,
          args: [
            '--git-dir',
            bareRoot,
            'worktree',
            'add',
            '-b',
            'bare-topic',
            destinationRoot,
            'main',
          ],
        },),);

        expect(result.stderr,).toContain('bare repository with an empty source set',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'rejects an ignored FIFO without opening it and retains new worktree',
      skip: process.platform === 'win32',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'fifo-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'pipe\n',);
        await commitPaths({ repositoryRoot, message: 'ignore pipe', paths: ['.gitignore',], },);
        await nanoSpawn(
          'mkfifo',
          [join(repositoryRoot, 'pipe',),],
        );

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'fifo-topic',
            destinationRoot,
          ],
        },),);

        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('unsupported filesystem type',);
        expect((await lstat(join(repositoryRoot, 'pipe',),)).isFIFO(),).toBe(true,);
        expect(await readFile(join(destinationRoot, '.git',), 'utf8',),).toContain('gitdir:',);
      },
    },),

    it({
      name: 'recovers a staged journal before forwarding the next command',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'recovery-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'recovery-topic',
            destinationRoot,
          ],
        },);
        /**
         * Private staged payload sibling to destination.
         */
        const stageContainer = join(
          dirname(destinationRoot,),
          '.cli-git-worktree-copy-recovery',
        );
        const stageRoot = join(stageContainer, 'payload',);
        await mkdir(stageRoot, { recursive: true, mode: 0o700, },);
        await Promise.all([
          chmod(stageContainer, 0o700,),
          chmod(stageRoot, 0o700,),
          writeFile(join(stageRoot, 'state.txt',), 'recovered\n', { mode: 0o640, },),
        ],);
        await chmod(join(stageRoot, 'state.txt',), 0o640,);
        /**
         * Common Git directory returned by real Git.
         */
        const commonDir = (await runRealGit({
          cwd: repositoryRoot,
          args: [
            'rev-parse',
            '--path-format=absolute',
            '--git-common-dir',
          ],
        },)).stdout.trim();
        const journalRoot = join(commonDir, 'cli-git-worktree-copy', 'v1',);
        const journalPath = join(journalRoot, 'recovery.json',);
        await mkdir(journalRoot, { recursive: true, mode: 0o700, },);
        await chmod(journalRoot, 0o700,);
        await writeFile(
          journalPath,
          `${JSON.stringify({
            createdEntries: [],
            destinationRoot,
            intendedEntries: [],
            phase: 'staged',
            selectedRoots: ['state.txt',],
            sourceRoot: repositoryRoot,
            stageContainer,
            stageRoot,
            version: 1,
          },)}\n`,
          { mode: 0o600, },
        );

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: ['status', '--short',],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('recovered\n',);
        expect(result.stderr,).toContain('recovered ignored-state copies for 1 worktree transaction',);
        /**
         * Next invocation proving no completed journal remains to recover.
         */
        const nextResult = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: ['status', '--short',],
        },),);
        expect(nextResult.stderr,).not.toContain('recovered ignored-state copies',);
      },
    },),

    it({
      name: 'rejects a journal whose stage path escapes destination sibling boundary',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'malicious-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'malicious-topic',
            destinationRoot,
          ],
        },);
        const commonDir = (await runRealGit({
          cwd: repositoryRoot,
          args: [
            'rev-parse',
            '--path-format=absolute',
            '--git-common-dir',
          ],
        },)).stdout.trim();
        const journalRoot = join(commonDir, 'cli-git-worktree-copy', 'v1',);
        const journalPath = join(journalRoot, 'malicious.json',);
        await mkdir(journalRoot, { recursive: true, mode: 0o700, },);
        await chmod(journalRoot, 0o700,);
        await writeFile(
          journalPath,
          `${JSON.stringify({
            createdEntries: [],
            destinationRoot,
            intendedEntries: [],
            phase: 'complete',
            selectedRoots: [],
            sourceRoot: repositoryRoot,
            stageContainer: fixture.path,
            stageRoot: join(fixture.path, 'payload',),
            version: 1,
          },)}\n`,
          { mode: 0o600, },
        );

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: ['status', '--short',],
        },),);

        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('unsafe private stage relation',);
        expect(await readFile(journalPath, 'utf8',),).toContain('malicious-topic',);
      },
    },),
  ],
},);
