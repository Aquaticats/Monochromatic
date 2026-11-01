/**
 Unit tests for pi settings scope loading.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { loadSettingsScopePatterns, } from '../dist/final/node/index.mjs';
import { captureAsyncError, } from './test-fixtures.ts';

//region Fixtures

/** Global enabled model pattern fixture. */
const GLOBAL_PATTERN = 'cheap/*';

/** Project enabled model pattern fixture. */
const PROJECT_PATTERN = 'expensive/reviewer';

//endregion Fixtures

await describe({
  name: loadSettingsScopePatterns.name,
  children: [
    it({
      name: 'returns unrestricted scope when settings are absent',
      fn: async function testAbsentSettings() {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        expect(await loadSettingsScopePatterns({
          cwd: join(
            root,
            'repo',
          ),
          home: join(
            root,
            'home',
          ),
        },),)
          .toEqual({},);
      },
    },),
    it({
      name: 'prefers project enabledModels over global enabledModels',
      fn: async function testProjectSettingsPrecedence() {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        const home = join(
          root,
          'home',
        );
        const cwd = join(
          root,
          'repo',
        );
        await Promise.all([
          mkdir(
            join(
              home,
              '.pi',
              'agent',
            ),
            { recursive: true, },
          ),
          mkdir(
            join(
              cwd,
              '.pi',
            ),
            { recursive: true, },
          ),
        ],);
        await writeFile(
          join(
            home,
            '.pi',
            'agent',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [GLOBAL_PATTERN,], },),
        );
        await writeFile(
          join(
            cwd,
            '.pi',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [
            '',
            PROJECT_PATTERN,
          ], },),
        );
        expect((await loadSettingsScopePatterns({ cwd, home, },)).patterns,).toEqual([
          PROJECT_PATTERN,
        ],);
      },
    },),
    it({
      name: 'throws for invalid enabledModels',
      fn: async function testInvalidSettings() {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        const cwd = join(
          root,
          'repo',
        );
        await mkdir(
          join(
            cwd,
            '.pi',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            cwd,
            '.pi',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [1,], },),
        );
        const error = await captureAsyncError(function loadInvalidSettings() {
          return loadSettingsScopePatterns({
            cwd,
            home: join(
              root,
              'home',
            ),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain('invalid project settings',);
      },
    },),
  ],
},);
