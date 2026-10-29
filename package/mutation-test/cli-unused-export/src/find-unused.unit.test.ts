import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  discoverWorkspacePackages,
  findUnusedExports,
  workspaceResolver,
} from '../dist/final/node/index.mjs';

/**
 Disposable fixture workspace rooted in a fresh temp directory.
 */
type FixtureWorkspace = AsyncDisposable & Readonly<{
  root: string;
}>;

/**
 Writes one disposable workspace with three scoped fixture packages.
 
 @returns Disposable fixture removing itself on dispose.
 
 @example
 ```ts
 await using fixture = await fixtureWorkspace();
 ```
 */
async function fixtureWorkspace(): Promise<FixtureWorkspace> {
  /**
   Fresh fixture root under the platform temp directory.
   */
  const root = await mkdtemp(join(
    tmpdir(),
    'unused-export-',
  ),);
  /**
   Fixture files as workspace-relative path to content.
   */
  const files: Readonly<Record<string, string>> = {
    'pnpm-workspace.yaml': "packages:\n- 'pkg/*'\n",
    'pkg/one/package.json': JSON.stringify({ name: '@monochromatic-dev/fixture-one', },),
    'pkg/one/src/lib.ts': 'export const used = 1;\nexport const dead = 2;\nexport type DeadShape = string;\n',
    'pkg/one/src/index.ts': "export { used, dead } from './lib.ts';\n",
    'pkg/two/package.json': JSON.stringify({ name: '@monochromatic-dev/fixture-two', },),
    'pkg/two/src/index.ts':
      "import { used } from '@monochromatic-dev/fixture-one/ts';\nexport const twoMain = used + 1;\n",
    'pkg/three/package.json': JSON.stringify({ name: '@monochromatic-dev/fixture-three', },),
    'pkg/three/src/index.ts': 'export const probed = 9;\n',
    'pkg/three/src/probed.unit.test.ts':
      "import { probed } from '../dist/final/node/index.mjs';\nconsole.log(probed);\n",
  };

  await Promise.all(Object.entries(files,)
    .map(async function writeEntry([
      path,
      content,
    ],): Promise<void> {
      /**
       Absolute fixture file path.
       */
      const target = join(
        root,
        path,
      );
      await mkdir(
        dirname(target,),
        { recursive: true, },
      );
      await writeFile(
        target,
        content,
      );
    },),);

  return {
    root,
    [Symbol.asyncDispose]: async function disposeFixture(): Promise<void> {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: discoverWorkspacePackages.name,
      children: [
        it({
          name: 'discovers scoped packages with sorted sources',
          fn: async () => {
            await using fixture = await fixtureWorkspace();
            /**
             Discovered fixture packages in directory order.
             */
            const packages = await discoverWorkspacePackages({ workspaceRoot: fixture.root, },);
            expect(packages.map(function toName(entry,) {
              return entry.name;
            },),).toEqual([
              '@monochromatic-dev/fixture-one',
              '@monochromatic-dev/fixture-three',
              '@monochromatic-dev/fixture-two',
            ],);
            expect(packages[0]?.sourceFiles,).toEqual([
              'pkg/one/src/index.ts',
              'pkg/one/src/lib.ts',
            ],);
          },
        },),
        it({
          name: 'rejects a root without a workspace manifest',
          fn: async () => {
            let caught: unknown;
            try {
              await discoverWorkspacePackages({ workspaceRoot: join(
                tmpdir(),
                'unused-export-absent',
              ), },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),
    describe({
      name: workspaceResolver.name,
      children: [
        it({
          name: 'resolves relative, workspace, and dist specifiers',
          fn: async () => {
            /**
             Resolver over one synthetic package layout.
             */
            const resolve = workspaceResolver({
              packageDirsByName: new Map([[
                '@monochromatic-dev/fixture-one',
                'pkg/one',
              ],],),
              fileSet: new Set([
                'pkg/one/src/index.ts',
                'pkg/one/src/lib.ts',
                'pkg/one/src/deep/index.ts',
              ],),
            },);
            expect(resolve(
              './lib.ts',
              'pkg/one/src/index.ts',
            ),).toBe('pkg/one/src/lib.ts',);
            expect(resolve(
              './lib',
              'pkg/one/src/index.ts',
            ),).toBe('pkg/one/src/lib.ts',);
            expect(resolve(
              './deep',
              'pkg/one/src/index.ts',
            ),).toBe('pkg/one/src/deep/index.ts',);
            expect(resolve(
              '@monochromatic-dev/fixture-one/ts',
              'pkg/two/src/index.ts',
            ),).toBe('pkg/one/src/index.ts',);
            expect(resolve(
              '@monochromatic-dev/fixture-one/ts/lib',
              'pkg/two/src/index.ts',
            ),).toBe('pkg/one/src/lib.ts',);
            expect(resolve(
              '../dist/final/node/index.mjs',
              'pkg/one/src/probe.unit.test.ts',
            ),).toBe('pkg/one/src/index.ts',);
            expect(resolve(
              'node:fs',
              'pkg/one/src/index.ts',
            ),).toBeNull();
            expect(resolve(
              '@monochromatic-dev/fixture-one/other',
              'pkg/two/src/index.ts',
            ),).toBeNull();
          },
        },),
      ],
    },),
    describe({
      name: findUnusedExports.name,
      children: [
        it({
          name: 'reports zero-reference exports at their declarations',
          fn: async () => {
            await using fixture = await fixtureWorkspace();
            /**
             Findings across the fixture workspace.
             */
            const findings = await findUnusedExports({ workspaceRoot: fixture.root, },);
            expect(findings.map(function toSummary(finding,) {
              return `${finding.file}:${String(finding.line,)} ${finding.typeOnly ? 'type ' : ''}${finding.name}`;
            },),).toEqual([
              'pkg/one/src/lib.ts:2 dead',
              'pkg/one/src/lib.ts:3 type DeadShape',
              'pkg/two/src/index.ts:2 twoMain',
            ],);
          },
        },),
        it({
          name: 'counts chained and dist-mapped test usage as used',
          fn: async () => {
            await using fixture = await fixtureWorkspace();
            /**
             Names reported unused across the fixture workspace.
             */
            const names = (await findUnusedExports({ workspaceRoot: fixture.root, },))
              .map(function toName(finding,) {
                return finding.name;
              },);
            expect(names,).not.toContain('used',);
            expect(names,).not.toContain('probed',);
          },
        },),
      ],
    },),
  ],
},);
