/**
 Drives the platform-neutral artifact in a real browser through the
 Playwright harness page, which imports `dist/final/neutral/index.mjs`
 onto `window.moduleFsPath`. The test writes repository markers into the
 origin private file system, runs findRoot with each preset marker, and
 checks the pure-JS path operations. A browser that refuses OPFS writes
 (headless WebKit exposes `getDirectory` but no `createWritable`) reports
 itself skipped, never failed.

 Local podman run only until a browser CI job exists:
 `mise run //package/fs-path/core:build`, then
 `mise run test:browser -- package/fs-path/core/src/opfs-root-discovery.browser.test.ts`.

 @module
 */

import {
  expect,
  test,
} from '@playwright/test';

declare global {
  // oxlint-disable-next-line typescript/consistent-type-imports -- typeof import() cannot use import type syntax
  var moduleFsPath: typeof import('@monochromatic-dev/fs-path-core');
}

/**
 Outcome of the in-page fixture setup: the repository root that was
 written, or the reason the platform refused to write it.
 */
type FixtureOutcome = {
  readonly root: string;
  readonly skipReason: string;
};

/**
 Settled outcome of one finder call in the page.
 */
type FinderOutcome = {
  readonly status: 'fulfilled' | 'rejected';
  readonly value: string;
  readonly errorName: string;
};

test.describe('root discovery over the origin private file system', () => {
  test.beforeEach(async ({ page, },) => {
    await page.goto('/',);
    await page.waitForFunction(() => globalThis.moduleFsPath !== undefined);
  },);

  test('path operations run on the pure-JS backend', async ({ page, },) => {
    /**
     Results of the path operations computed in the page.
     */
    const results = await page.evaluate(() => {
      const {
        dirname,
        isAbsolute,
        join,
        normalize,
        resolve,
        sep,
        trimTrailingSlash,
      } = globalThis.moduleFsPath;
      return {
        dirname: dirname('/repo/pkg/src/',),
        isAbsolute: isAbsolute('/repo',),
        join: join(['/repo', 'pkg', '../lib', 'index.ts',],),
        normalize: normalize('a//b/./c/../d',),
        resolveRelative: resolve(['pkg', 'src',],),
        sep,
        trimmed: trimTrailingSlash('/repo/',),
      };
    },);
    expect(results,).toEqual({
      dirname: '/repo/pkg',
      isAbsolute: true,
      join: '/repo/lib/index.ts',
      normalize: 'a/b/d',
      resolveRelative: '/pkg/src',
      sep: '/',
      trimmed: '/repo',
    },);
  },);

  test('findRoot locates markers written into OPFS and fails cleanly without them', async ({ page, },) => {
    /**
     Repository fixture written into OPFS by the page, or the skip reason.
     */
    const fixture: FixtureOutcome = await page.evaluate(async () => {
      /**
       Unique root name so reruns on the same origin never collide.
       */
      const rootName = `repo-${crypto.randomUUID()}`;
      /**
       Writes text into a file under a directory handle.
       */
      async function writeText({
        directory,
        name,
        text,
      }: {
        readonly directory: FileSystemDirectoryHandle;
        readonly name: string;
        readonly text: string;
      },): Promise<void> {
        /**
         File handle created on demand.
         */
        const file = await directory.getFileHandle(
          name,
          { create: true, },
        );
        /**
         Writable stream; absent in browsers that refuse OPFS writes.
         */
        const writable = await file.createWritable();
        await writable.write(text,);
        await writable.close();
      }
      try {
        /**
         OPFS root for this origin.
         */
        const opfsRoot = await navigator.storage
          .getDirectory();
        /**
         Repository directory holding every marker.
         */
        const repo = await opfsRoot.getDirectoryHandle(
          rootName,
          { create: true, },
        );
        await writeText(
          {
            directory: repo,
            name: 'mise.toml',
            text: '[tools]\nnode = "24"\n\n[monorepo]\nroot = true\n',
          },
        );
        await writeText(
          {
            directory: repo,
            name: 'pnpm-workspace.yaml',
            text: 'packages:\n  - pkg\n',
          },
        );
        /**
         Git administrative directory with the signatures the finder validates.
         */
        const gitDirectory = await repo.getDirectoryHandle(
          '.git',
          { create: true, },
        );
        await writeText(
          {
            directory: gitDirectory,
            name: 'HEAD',
            text: 'ref: refs/heads/main\n',
          },
        );
        await gitDirectory.getDirectoryHandle(
          'objects',
          { create: true, },
        );
        await gitDirectory.getDirectoryHandle(
          'refs',
          { create: true, },
        );
        /**
         Nested package directory the walk starts from.
         */
        const pkg = await repo.getDirectoryHandle(
          'pkg',
          { create: true, },
        );
        await pkg.getDirectoryHandle(
          'src',
          { create: true, },
        );
        return {
          root: `/${rootName}`,
          skipReason: '',
        };
      }
      catch (error: unknown) {
        return {
          root: '',
          skipReason: `origin private file system refused: ${String(error,)}`,
        };
      }
    },);
    test.skip(
      fixture.skipReason !== '',
      fixture.skipReason,
    );

    /**
     Outcomes of the finder calls, in the order issued.
     */
    const outcomes: readonly FinderOutcome[] = await page.evaluate(async (root,) => {
      const {
        findRoot,
        GIT_REPOSITORY,
        MISE_MONOREPO,
        PNPM_WORKSPACE,
      } = globalThis.moduleFsPath;
      /**
       Deepest directory of the fixture, forcing an upward walk.
       */
      const cwd = `${root}/pkg/src`;
      /**
       Directory with no marker on any ancestor.
       */
      const nowhere = `${root}-absent/deep/er`;
      /**
       Settles one finder call into a serializable outcome.
       */
      async function settle(call: Promise<string>,): Promise<FinderOutcome> {
        try {
          return {
            errorName: '',
            status: 'fulfilled',
            value: await call,
          };
        }
        catch (error: unknown) {
          return {
            errorName: (Error.isError(error,)) ? error.name : 'non-error',
            status: 'rejected',
            value: '',
          };
        }
      }
      return [
        await settle(findRoot({ cwd, marker: MISE_MONOREPO, },),),
        await settle(findRoot({ cwd, marker: GIT_REPOSITORY, },),),
        await settle(findRoot({ cwd, marker: PNPM_WORKSPACE, },),),
        await settle(findRoot({ cwd: nowhere, marker: MISE_MONOREPO, },),),
        await settle(findRoot({ cwd: nowhere, marker: GIT_REPOSITORY, },),),
      ];
    }, fixture.root,);

    // Precise terminal output: the dot reporter shows nothing else about
    // which browser found which marker.
    console.log(`${test.info().project.name}: ${outcomes
      .map((outcome,) => `${outcome.status}${(outcome.value === '') ? '' : ` ${outcome.value}`}${(outcome.errorName === '') ? '' : ` ${outcome.errorName}`}`)
      .join(' | ',)}`,);
    expect(outcomes,).toEqual([
      {
        errorName: '',
        status: 'fulfilled',
        value: fixture.root,
      },
      {
        errorName: '',
        status: 'fulfilled',
        value: fixture.root,
      },
      {
        errorName: '',
        status: 'fulfilled',
        value: fixture.root,
      },
      {
        errorName: 'RootNotFoundError',
        status: 'rejected',
        value: '',
      },
      {
        errorName: 'RootNotFoundError',
        status: 'rejected',
        value: '',
      },
    ],);
  },);
},);
