/**
 Unit tests for Advisor configuration loading.
 
 @module
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
import { loadMergedConfig, } from '../dist/final/node/index.mjs';

//region Constants

/** User-selected default operation timeout. */
const EXPECTED_DEFAULT_TIMEOUT_MS = 600_000;

/** User-selected default Advisor output token budget. */
const EXPECTED_DEFAULT_OUTPUT_TOKENS = 32_000;

/** Global timeout fixture. */
const GLOBAL_TIMEOUT_MS = 10;

/** Project timeout fixture. */
const PROJECT_TIMEOUT_MS = 20;

//endregion Constants

await describe({
  name: loadMergedConfig.name,
  children: [
    it({
      name: 'uses defaults when config files are absent',
      fn: async () => {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-advisor-test-',
        ),);
        const config = await loadMergedConfig({
          cwd: join(
            root,
            'repo',
          ),
          home: join(
            root,
            'home',
          ),
        },);
        expect(config.enabled,).toBe(true,);
        expect(config.timeoutMs,).toBe(EXPECTED_DEFAULT_TIMEOUT_MS,);
        expect(config.maxAdvisorOutputTokens,).toBe(EXPECTED_DEFAULT_OUTPUT_TOKENS,);
      },
    },),
    it({
      name: 'merges project config over global config',
      fn: async () => {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-advisor-test-',
        ),);
        const home = join(
          root,
          'home',
        );
        const cwd = join(
          root,
          'repo',
        );
        const globalConfigDir = join(
          home,
          '.pi',
          'agent',
          'extensions',
        );
        const projectConfigDir = join(
          cwd,
          '.pi',
          'extensions',
        );
        await Promise.all([
          mkdir(
            globalConfigDir,
            { recursive: true, },
          ),
          mkdir(
            projectConfigDir,
            { recursive: true, },
          ),
        ],);
        await writeFile(
          join(
            globalConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ timeoutMs: GLOBAL_TIMEOUT_MS, },),
        );
        await writeFile(
          join(
            projectConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ timeoutMs: PROJECT_TIMEOUT_MS, },),
        );
        const config = await loadMergedConfig({ cwd, home, },);
        expect(config.timeoutMs,).toBe(PROJECT_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'reports invalid config path',
      fn: async () => {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-advisor-test-',
        ),);
        const home = join(
          root,
          'home',
        );
        const globalConfigDir = join(
          home,
          '.pi',
          'agent',
          'extensions',
        );
        await mkdir(
          globalConfigDir,
          { recursive: true, },
        );
        await writeFile(
          join(
            globalConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ timeoutMs: 0, },),
        );
        let caught: unknown;
        try {
          await loadMergedConfig({
            cwd: join(
              root,
              'repo',
            ),
            home,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('pi-advisor.json',);
      },
    },),
    it({
      name: 'merges the configured default reviewer with project override and explicit clear',
      fn: async () => {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-advisor-test-',
        ),);
        const home = join(
          root,
          'home',
        );
        const cwd = join(
          root,
          'repo',
        );
        const globalConfigDir = join(
          home,
          '.pi',
          'agent',
          'extensions',
        );
        const projectConfigDir = join(
          cwd,
          '.pi',
          'extensions',
        );
        await Promise.all([
          mkdir(
            globalConfigDir,
            { recursive: true, },
          ),
          mkdir(
            projectConfigDir,
            { recursive: true, },
          ),
        ],);
        await writeFile(
          join(
            globalConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ defaultReviewerModel: 'global-provider/global-model', },),
        );
        const mergedGlobal = await loadMergedConfig({ cwd, home, },);
        expect(mergedGlobal.defaultReviewerModel,).toBe(
          'global-provider/global-model',
        );
        await writeFile(
          join(
            projectConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ defaultReviewerModel: 'project-provider/project-model', },),
        );
        const mergedProject = await loadMergedConfig({ cwd, home, },);
        expect(mergedProject.defaultReviewerModel,).toBe(
          'project-provider/project-model',
        );
        await writeFile(
          join(
            projectConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ defaultReviewerModel: '', },),
        );
        const mergedCleared = await loadMergedConfig({ cwd, home, },);
        expect(mergedCleared.defaultReviewerModel,).toBe('',);
      },
    },),
    it({
      name: 'rejects a non-string default reviewer key',
      fn: async () => {
        const root = await mkdtemp(join(
          scratchRoot(),
          'pi-advisor-test-',
        ),);
        const home = join(
          root,
          'home',
        );
        const globalConfigDir = join(
          home,
          '.pi',
          'agent',
          'extensions',
        );
        await mkdir(
          globalConfigDir,
          { recursive: true, },
        );
        await writeFile(
          join(
            globalConfigDir,
            'pi-advisor.json',
          ),
          JSON.stringify({ defaultReviewerModel: 42, },),
        );
        let caught: unknown;
        try {
          await loadMergedConfig({
            cwd: join(
              root,
              'repo',
            ),
            home,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('pi-advisor.json',);
      },
    },),
  ],
},);
