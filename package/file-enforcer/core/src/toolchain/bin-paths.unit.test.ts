/**
 Unit tests for the `bin-paths.ts` JSON contract.
 */

import {
  mkdirSync,
  mkdtempSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { buildBinPathsOutput, } from './bin-paths.ts';
import {
  resolveToolchainPaths,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';
import { TOOLS, } from './plan.ts';

/**
 Disposable worktree fixture with an isolated home.
 */
type Fixture = {
  readonly paths: ToolchainPaths;
  readonly env: PathEnv;
};

/**
 Builds a disposable worktree fixture.
 
 @param options - Optional pre-created exposure links and exposure dir creation
 
 @returns Resolved paths and isolated environment for the fixture
 */
function fixtureWorktree(
  {
    links,
    withExposureDir = true,
  }: {
    readonly links?: Readonly<Record<string, string>>;
    readonly withExposureDir?: boolean;
  } = {},
): Fixture {
  /**
   Disposable fixture root.
   */
  const root = mkdtempSync(join(
    tmpdir(),
    'toolchain-bin-paths-',
  ),);
  /**
   Isolated environment pointing every home-derived path at the fixture.
   */
  const env: PathEnv = {
    HOME: root,
    XDG_DATA_HOME: join(
      root,
      'data',
    ),
  };
  if (withExposureDir) {
    mkdirSync(
      join(
        root,
        '.cache',
        'toolchain-bin',
      ),
      { recursive: true },
    );
  }
  for (const target of Object.values(links ?? {},))
    writeFileSync(
      target,
      '#!/bin/sh\n',
      'utf8',
    );
  for (const [name, target,] of Object.entries(links ?? {},)) {
    symlinkSync(
      target,
      join(
        root,
        '.cache',
        'toolchain-bin',
        name,
      ),
    );
  }
  return {
    paths: resolveToolchainPaths({
      worktreeRoot: root,
      env,
    },),
    env,
  };
}

await describe({
  name: 'bin-paths contract',
  children: [
    describe({
      name: buildBinPathsOutput.name,
      children: [
        it({
          name: 'returns the pinned shape with empty maps when unprovisioned',
          fn: async () => {
            /**
             Fixture without an exposure directory.
             */
            const fixture = fixtureWorktree({ withExposureDir: false, },);
            /**
             Output for the fixture.
             */
            const output = await buildBinPathsOutput({
              paths: fixture.paths,
              env: fixture.env,
            },);
            expect(output.exposureDir,)
              .toBe('.cache/toolchain-bin',);
            expect(output.pathDirs,)
              .toEqual([],);
            expect(output.bins,)
              .toEqual({},);
          },
        },),
        it({
          name: 'lists only links that resolve, as absolute exposure paths',
          fn: async () => {
            /**
             Fixture with one real target and one dangling link.
             */
            const fixture = fixtureWorktree({
              links: {
                taplo: join(
                  tmpdir(),
                  'toolchain-bin-paths-target-taplo',
                ),
              },
            },);
            symlinkSync(
              join(
                tmpdir(),
                'toolchain-bin-paths-missing',
              ),
              join(
                fixture.paths.exposureDir,
                'dangling',
              ),
            );
            /**
             Output for the fixture.
             */
            const output = await buildBinPathsOutput({
              paths: fixture.paths,
              env: fixture.env,
            },);
            expect(Object.keys(output.bins,)
              .toSorted(),)
              .toEqual(['taplo'],);
            expect(output.bins.taplo,)
              .toBe(join(
                fixture.paths.exposureDir,
                'taplo',
              ),);
            expect(output.bins.taplo?.startsWith('/',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'lists the exposure directory only when it exists',
          fn: async () => {
            /**
             Fixture whose exposure directory exists.
             */
            const fixture = fixtureWorktree();
            expect(
              (await buildBinPathsOutput({
                paths: fixture.paths,
                env: fixture.env,
              },))
                .pathDirs,
            )
              .toEqual([fixture.paths.exposureDir],);
          },
        },),
        it({
          name: 'throws when two plan entries declare the same binary name',
          fn: async () => {
            /**
             Two plan-shaped entries sharing one binary name.
             */
            const duplicate = TOOLS.filter(
              (entry,) => entry.bins.length > 0,
            )
              .slice(
              0,
              2,
              )
              .map(
              (entry, index,) => ({
                ...entry,
                bins: ['same-name'],
                id: `duplicate-${index}`,
              }),
              );
            /**
             Fixture for the collision check.
             */
            const fixture = fixtureWorktree();
            await expect(
              buildBinPathsOutput({
                paths: fixture.paths,
                env: fixture.env,
                entries: duplicate,
              },),
            )
              .rejects
              .toThrow();
          },
        },),
      ],
    },),
  ],
},);
