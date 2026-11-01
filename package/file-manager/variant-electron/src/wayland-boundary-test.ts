/**
 End-user boundary test for the sticky-flow file manager under pure Wayland.
 
 The test builds a throwaway fixture directory tree, hosts Electron inside
 this repo's nested Wayland compositor with `DISPLAY` unset, drives the strip
 with compositor keyboard input, and asserts the renderer-observed state:
 spawn, dedup, close, and, decisively, that the root pane pins to the
 scroller top while the strip is scrolled (`rootPinned`) with zero pane
 overlaps (`overlapCount`), the behavior CSS sticky flow must provide.
 
 @example
 ```ts
 await runFileManagerWaylandBoundaryTest();
 ```
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { runWaylandElectronBoundaryTest, } from '@monochromatic-dev/electron-infra-core/ts/wayland-test';

import {
  DEBUG_TINT_ENVIRONMENT_VARIABLE,
  ROOT_DIRECTORY_ENVIRONMENT_VARIABLE,
  STATE_PATH_ENVIRONMENT_VARIABLE,
} from './constants.js';

/**
 Fixture directory tree of known contents the app browses during the test.
 
 @example
 ```ts
 const fixture = await createFixtureTree();
 ```
 */
type FixtureTree = {
  /**
   Path of the `alpha` subdirectory (first entry, dirs sort first).
   */
  readonly alphaPath: string;

  /**
   Path of the `beta` subdirectory (second entry).
   */
  readonly betaPath: string;

  /**
   Fixture root the first pane lists.
   */
  readonly rootPath: string;

  /**
   Removes the fixture tree at end of scope.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates the throwaway fixture tree: `alpha/` (with one nested directory and
 one file), `beta/` (empty), and one root-level file, so the sorted root
 listing is deterministic: alpha, beta, readme.txt.
 
 @returns Fixture paths plus async disposal.
 
 @example
 ```ts
 await using fixture = await createFixtureTree();
 ```
 */
async function createFixtureTree(): Promise<FixtureTree> {
  /**
   Temporary fixture root for this run.
   */
  const rootPath = await mkdtemp(join(
    scratchRoot(),
    'file-manager-electron-fixture-',
  ),);

  /**
   First subdirectory, aligned to the root's row when descended into.
   */
  const alphaPath = join(
    rootPath,
    'alpha',
  );

  /**
   Second subdirectory, stacked one row below when descended into.
   */
  const betaPath = join(
    rootPath,
    'beta',
  );

  await mkdir(
    join(
      alphaPath,
      'nested-one',
    ),
    { recursive: true, },
  );
  await mkdir(
    betaPath,
    { recursive: true, },
  );
  await writeFile(
    join(
      alphaPath,
      'note.txt',
    ),
    'fixture note\n',
    'utf8',
  );
  await writeFile(
    join(
      rootPath,
      'readme.txt',
    ),
    'fixture readme\n',
    'utf8',
  );

  return {
    alphaPath,
    betaPath,
    rootPath,
    [Symbol.asyncDispose]: async function removeFixtureTree(): Promise<void> {
      await rm(
        rootPath,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Runs the complete pure-Wayland interaction test.
 
 @example
 ```ts
 await runFileManagerWaylandBoundaryTest();
 ```
 */
async function runFileManagerWaylandBoundaryTest(): Promise<void> {
  /**
   Throwaway fixture tree removed when the test scope ends.
   */
  await using fixture = await createFixtureTree();

  // The nested compositor and /usr/bin/env inherit this process's environment,
  // so the fixture root and debug tint reach the Electron main process.
  process.env[ROOT_DIRECTORY_ENVIRONMENT_VARIABLE] = fixture.rootPath;
  process.env[DEBUG_TINT_ENVIRONMENT_VARIABLE] = '1';

  await runWaylandElectronBoundaryTest({
    packageRoot: process.cwd(),
    screenshotName: 'sticky-rails.png',
    statePathEnvironmentVariable: STATE_PATH_ENVIRONMENT_VARIABLE,
    steps: [
      {
        expected: {
          activePath: fixture.rootPath,
          columnCount: 1,
          overlapCount: 0,
          paneCount: 1,
          ready: true,
        },
      },
      {
        commands: ['key enter',],
        expected: {
          activePath: fixture.alphaPath,
          columnCount: 2,
          overlapCount: 0,
          paneCount: 2,
        },
      },
      {
        commands: ['key left',],
        expected: {
          activePath: fixture.rootPath,
          paneCount: 2,
        },
      },
      {
        commands: ['key enter',],
        expected: {
          activePath: fixture.alphaPath,
          paneCount: 2,
        },
      },
      {
        commands: ['key left',],
        expected: { activePath: fixture.rootPath, },
      },
      {
        commands: [
          'key down',
          'key enter',
        ],
        expected: {
          activePath: fixture.betaPath,
          overlapCount: 0,
          paneCount: 3,
          rootPinned: true,
          scrolledDown: true,
        },
      },
      {
        commands: ['key backspace',],
        expected: {
          activePath: '',
          paneCount: 2,
        },
      },
    ],
  },);
}

await runFileManagerWaylandBoundaryTest();
