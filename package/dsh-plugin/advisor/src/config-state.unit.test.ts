/**
 Unit tests for unified config loading and session state.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { join, } from 'node:path';

import {
  createDshAdvisorSessionState,
  loadDshAdvisorConfig,
} from '../dist/final/node/index.mjs';

/**
 Build one disposable config fixture home and project directory.

 @param globalJson - global `pi-advisor.json` document text

 @param projectJson - project `pi-advisor.json` document text

 @returns home directory and project working directory paths

 @example
 ```ts
 const fixture = await configFixture('{"enabled":false}', '{"enabled":true}');
 ```
 */
async function configFixture(
  globalJson: string,
  projectJson: string,
): Promise<{
  readonly home: string;
  readonly cwd: string;
}> {
  /**
   Disposable fixture root under the system temp directory.
   */
  const root = await mkdtemp(join(scratchRoot(), 'advisor-config-',),);
  /**
   Global extension directory holding the user-level config file.
   */
  const globalDir = join(root, 'home', '.pi', 'agent', 'extensions',);
  await mkdir(globalDir, {
    recursive: true,
  },);
  await writeFile(join(globalDir, 'pi-advisor.json',), globalJson, 'utf8',);
  /**
   Project extension directory holding the project-level config file.
   */
  const projectDir = join(root, 'project', '.pi', 'extensions',);
  await mkdir(projectDir, {
    recursive: true,
  },);
  await writeFile(join(projectDir, 'pi-advisor.json',), projectJson, 'utf8',);
  return {
    home: join(root, 'home',),
    cwd: join(root, 'project',),
  };
}

await describe({
  name: loadDshAdvisorConfig.name,
  children: [
    it({
      name: 'merges the project config over the global config',
      fn: async () => {
        const fixture = await configFixture(
          '{"enabled":false,"defaultReviewerModel":"p/global"}',
          '{"enabled":true}',
        );
        const config = await loadDshAdvisorConfig(fixture,);
        expect(config.enabled,).toBe(true,);
        expect(config.defaultReviewerModel,).toBe('p/global',);
      },
    },),
    it({
      name: 'falls back to schema defaults when no config file exists',
      fn: async () => {
        const config = await loadDshAdvisorConfig({
          cwd: '/definitely/no/extension/file',
          home: '/definitely/no/home',
        },);
        expect(typeof config.enabled,).toBe('boolean',);
      },
    },),
  ],
},);

await describe({
  name: createDshAdvisorSessionState.name,
  children: [
    it({
      name: 'tracks the toggle and the transcript degradation reason',
      fn: async () => {
        const state = createDshAdvisorSessionState({
          initialEnabled: true,
        },);
        expect(state.getEnabled(),).toBe(true,);
        state.setEnabled(false,);
        expect(state.getEnabled(),).toBe(false,);
        expect(state.getTranscriptDegradationReason(),).toBe('',);
        state.recordTranscriptDegradation('derive failed',);
        expect(state.getTranscriptDegradationReason(),).toBe('derive failed',);
        state.recordTranscriptDegradation('',);
        expect(state.getTranscriptDegradationReason(),).toBe('',);
      },
    },),
    it({
      name: 'replaces adopted tool restrictions and disposes them on clear',
      fn: async () => {
        const state = createDshAdvisorSessionState({
          initialEnabled: true,
        },);
        /**
         Disposal calls recorded by the fixture disposers.
         */
        const disposed: string[] = [];
        state.adoptToolRestriction(() => {
          disposed.push('first',);
        },);
        state.adoptToolRestriction(() => {
          disposed.push('second',);
        },);
        expect(disposed,).toEqual(['first',],);
        state.clearToolRestriction();
        expect(disposed,).toEqual([
          'first',
          'second',
        ],);
        state.clearToolRestriction();
        expect(disposed.length,).toBe(2,);
      },
    },),
  ],
},);
