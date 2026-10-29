/**
 Unit tests for the machine-global plan loader
 (`plan.load.ts`): data modules become typed plan entries, and malformed or
 inconsistent declarations fail with the file and tool named.
 */

import {
  mkdirSync,
  mkdtempSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  loadPlanFromDir,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Writes one disposable plan stack and returns its directory.
 
 @param options - Declarations source and artifacts source to write
 
 @returns Absolute fixture directory
 */
function makePlanDir(
  {
    toolsSource,
    artifactsSource = 'export const ARTIFACTS = {};',
  }: {
    readonly toolsSource: string;
    readonly artifactsSource?: string;
  },
): string {
  /**
   Disposable directory holding the two data modules.
   */
  const dir = mkdtempSync(join(
    tmpdir(),
    'toolchain-plan-load-',
  ),);
  mkdirSync(
    dir,
    { recursive: true },
  );
  writeFileSync(
    join(dir, 'plan.tools.data.ts',),
    toolsSource,
    'utf8',
  );
  writeFileSync(
    join(dir, 'plan.artifacts.data.ts',),
    artifactsSource,
    'utf8',
  );
  return dir;
}

/**
 One valid declaration, ready for concatenation with test-specific fields.
 
 @param overrides - Field text replacing the defaults
 
 @returns Object-literal text for one entry
 */
function entrySource(overrides: string,): string {
  return `{
    id: 'fixture',
    request: { track: 'latest' },
    version: '1.2.3',
    route: 'bin-sourced',
    manager: null,
    packageId: null,
    bins: ['fixture'],
    os: null,
    options: {},
    env: {},
    ${overrides}
  }`;
}

//endregion Fixtures

//region Loading

await describe({
  name: loadPlanFromDir.name,
  children: [
    it({
      name: 'merges generated artifact rows into typed entries in declaration order',
      fn: async () => {
        /**
         Fixture plan with two entries and rows for the first.
         */
        const dir = makePlanDir({
          toolsSource: `export const TOOLS = [
  ${entrySource('',)},
  {
    id: 'second',
    request: { pinned: '2.0.0' },
    version: '2.0.0',
    route: 'archive',
    manager: null,
    packageId: null,
    bins: ['second'],
    os: ['linux'],
    options: { assetPattern: '*linux*' },
    env: {},
  },
];`,
          artifactsSource: `export const ARTIFACTS = {
  'fixture': {
    'linux-x64': {
      url: 'https://example.invalid/fixture.tar.gz',
      sha256: 'a'.repeat(64),
    },
  },
};`,
        },);
        /**
         Loaded entries.
         */
        const entries = await loadPlanFromDir({ dir, },);
        expect(entries.length,)
          .toBe(2,);
        expect(entries[0]?.id,)
          .toBe('fixture',);
        expect(entries[0]?.artifacts['linux-x64']?.url,)
          .toBe('https://example.invalid/fixture.tar.gz',);
        expect(entries[1]?.id,)
          .toBe('second',);
        expect(entries[1]?.artifacts,)
          .toEqual({},);
      },
    },),
    it({
      name: 'reports the file and tool when a route is unknown',
      fn: async () => {
        /**
         Fixture plan declaring an unknown route.
         */
        const dir = makePlanDir({
          toolsSource: `export const TOOLS = [${
            entrySource(`route: 'teleport',`)
          }];`,
        },);
        /**
         Loader rejection.
         */
        let caught: unknown;
        try {
          await loadPlanFromDir({ dir, },);
        } catch (error) {
          caught = error;
        }
        expect(caught,)
          .toBeInstanceOf(Error,);
        expect((caught as Error).message.includes('plan.tools.data.ts',),)
          .toBe(true,);
        expect((caught as Error).message.includes('fixture',),)
          .toBe(true,);
        expect((caught as Error).message.includes('route',),)
          .toBe(true,);
      },
    },),
    it({
      name: 'accepts one entry naming the same exposure name as bin and alias',
      fn: async () => {
        /**
         Fixture plan whose entry lists `bunx` as a bin and as its own alias.
         */
        const dir = makePlanDir({
          toolsSource: `export const TOOLS = [${
            entrySource(`bins: ['bun', 'bunx'], options: { binAliases: { bunx: 'bun' } },`)
          }];`,
        },);
        /**
         Loaded entries.
         */
        const entries = await loadPlanFromDir({ dir, },);
        expect(entries.length,)
          .toBe(1,);
        expect(entries[0]?.bins,)
          .toEqual(['bun', 'bunx',],);
      },
    },),
    it({
      name: 'reports the tool when two entries share one exposure name',
      fn: async () => {
        /**
         Fixture plan whose two entries both declare `shared`.
         */
        const dir = makePlanDir({
          toolsSource: `export const TOOLS = [
  ${entrySource(`bins: ['shared'],`)},
  {
    id: 'second',
    request: { track: 'latest' },
    version: '1.0.0',
    route: 'archive',
    manager: null,
    packageId: null,
    bins: ['shared'],
    os: null,
    options: {},
    env: {},
  },
];`,
        },);
        /**
         Loader rejection.
         */
        let caught: unknown;
        try {
          await loadPlanFromDir({ dir, },);
        } catch (error) {
          caught = error;
        }
        expect(caught,)
          .toBeInstanceOf(Error,);
        expect((caught as Error).message.includes('shared',),)
          .toBe(true,);
      },
    },),
    it({
      name: 'reports the tool when two entries share one id',
      fn: async () => {
        /**
         Fixture plan declaring `fixture` twice.
         */
        const dir = makePlanDir({
          toolsSource: `export const TOOLS = [
  ${entrySource('',)},
  ${entrySource(`bins: ['other'],`)},
];`,
        },);
        /**
         Loader rejection.
         */
        let caught: unknown;
        try {
          await loadPlanFromDir({ dir, },);
        } catch (error) {
          caught = error;
        }
        expect(caught,)
          .toBeInstanceOf(Error,);
        expect((caught as Error).message.includes('declared twice',),)
          .toBe(true,);
      },
    },),
    it({
      name: 'rejects a request carrying two intents',
      fn: async () => {
        /**
         Fixture plan whose request names two intents.
         */
        const dir = makePlanDir({
          toolsSource: `export const TOOLS = [${
            entrySource(`request: { track: 'latest', pinned: '1.2.3' },`)
          }];`,
        },);
        /**
         Loader rejection.
         */
        let caught: unknown;
        try {
          await loadPlanFromDir({ dir, },);
        } catch (error) {
          caught = error;
        }
        expect(caught,)
          .toBeInstanceOf(Error,);
        expect((caught as Error).message.includes('request',),)
          .toBe(true,);
      },
    },),
    it({
      name: 'rejects a tools module without a TOOLS export',
      fn: async () => {
        /**
         Fixture directory whose tools module exports nothing.
         */
        const dir = makePlanDir({
          toolsSource: 'export const OTHER = [];',
        },);
        /**
         Loader rejection.
         */
        let caught: unknown;
        try {
          await loadPlanFromDir({ dir, },);
        } catch (error) {
          caught = error;
        }
        expect(caught,)
          .toBeInstanceOf(Error,);
        expect((caught as Error).message.includes('TOOLS',),)
          .toBe(true,);
      },
    },),
  ],
},);

//endregion Loading
