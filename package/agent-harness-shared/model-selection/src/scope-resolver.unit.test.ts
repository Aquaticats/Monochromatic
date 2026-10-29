/**
 Unit tests for effective scope resolution.
 
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

import { resolveEffectiveScope, } from '../dist/final/node/index.mjs';
import { fixtureModel, } from './test-fixtures.ts';

//region Fixtures

/** Cheap model fixture. */
const cheapModel = fixtureModel({
  provider: 'cheap',
  id: 'reviewer',
},);

/** Expensive model fixture. */
const expensiveModel = fixtureModel({
  provider: 'expensive',
  id: 'reviewer',
},);

/** Available models fixture. */
const availableModels = [
  cheapModel,
  expensiveModel,
] as const;

/** Minimal model registry fixture. */
const modelRegistry = {
  getAvailable() {
    return availableModels;
  },
};

/** Base context fixture. */
const baseContext = {
  cwd: '/tmp/repo',
  modelRegistry,
};

//endregion Fixtures

await describe({
  name: resolveEffectiveScope.name,
  children: [
    it({
      name: 'prefers live scope over argv',
      fn: async function testLiveScopePrecedence() {
        const scope = await resolveEffectiveScope({
          ctx: {
            ...baseContext,
            getScopedModels() {
              return [
                {
                  model: expensiveModel,
                  thinkingLevel: 'high',
                },
              ];
            },
          },
          argv: [
            'pi',
            '--models',
            'cheap/*',
          ],
        },);
        expect(scope.source,).toBe('live',);
        expect(scope.entries[0]?.canonicalSlug,).toBe('expensive/reviewer',);
        expect(scope.entries[0]?.thinkingLevel,).toBe('high',);
      },
    },),
    it({
      name: 'uses argv scope before settings',
      fn: async function testArgvScope() {
        const scope = await resolveEffectiveScope({
          ctx: baseContext,
          argv: [
            'pi',
            '--models',
            'cheap/*',
          ],
        },);
        expect(scope.source,).toBe('argv',);
        expect(scope.entries[0]?.canonicalSlug,).toBe('cheap/reviewer',);
      },
    },),
    it({
      name: 'uses settings scope before available fallback',
      fn: async function testSettingsScope() {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-shared-scope-test-',
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
          JSON.stringify({ enabledModels: ['expensive/*',], },),
        );
        const scope = await resolveEffectiveScope({
          ctx: {
            ...baseContext,
            cwd,
          },
          argv: ['pi',],
          home: join(
            root,
            'home',
          ),
        },);
        expect(scope.source,).toBe('settings',);
        expect(scope.entries[0]?.canonicalSlug,).toBe('expensive/reviewer',);
      },
    },),
    it({
      name: 'falls back to all available models',
      fn: async function testAvailableFallback() {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-shared-scope-test-',
        ),);
        const scope = await resolveEffectiveScope({
          ctx: {
            ...baseContext,
            cwd: join(
              root,
              'repo',
            ),
          },
          argv: ['pi',],
          home: join(
            root,
            'home',
          ),
        },);
        expect(scope.source,).toBe('available',);
        expect(scope.entries,).toHaveLength(2,);
      },
    },),
  ],
},);
