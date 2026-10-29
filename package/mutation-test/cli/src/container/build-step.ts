/**
 In-container package build via rolldown, once per mutant.
 
 Tests exercise built output (repo convention), so every mutant must be
 built before tests can observe it; the build also emits declaration
 files (oxc isolatedDeclarations), which the tsgo gate needs for
 package self-reference imports. Packages without rolldown configs (like
 the fixture) skip the step: their tests import source directly.
 
 @example
 ```ts
 await runBuildStep({ packageCwd: '/work/packages/module/fs-path' });
 ```
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { diagnosticsFromError, } from './tsgo-check.ts';
import { WORK_MOUNT, } from '../mounts.ts';

/**
 Module logger for the container-side build step.
 */
const l = tagged({ tag: 'mutation-test-container', },);

/**
 Root rolldown bin inside the work tree; shims resolve through the
 symlinked pnpm store into the baked layer.
 */
const ROLLDOWN_BIN = join(
  WORK_MOUNT,
  'node_modules/.bin/rolldown',
);

/**
 Outcome of one package build.
 */
export type BuildOutcome = {
  readonly built: boolean;
  readonly clean: boolean;
  readonly durationMs: number;
  readonly detail: string;
};

/**
 Lists rolldown config files in one package directory.
 
 @param packageCwd - Package working directory.
 
 @returns Config file names, sorted.
 
 @example
 ```ts
 await listRolldownConfigs('/work/packages/module/fs-path');
 // ['rolldown.browser.config.ts']
 ```
 */
export async function listRolldownConfigs(packageCwd: string,): Promise<readonly string[]> {
  /**
   Directory entries of the package root.
   */
  const entries = await readdir(packageCwd,);
  return entries
    .filter(function isRolldownConfig(name,): boolean {
      return name.startsWith('rolldown.',) && name.endsWith('.config.ts',);
    },)
    .toSorted();
}

/**
 Builds one package by running every rolldown config sequentially.
 
 @param options - Package working directory.
 
 @returns Build outcome; `built` false means no configs exist.
 
 @example
 ```ts
 const outcome = await runBuildStep({ packageCwd });
 ```
 */
export async function runBuildStep(options: {
  readonly packageCwd: string;
},): Promise<BuildOutcome> {
  /**
   Logger scoped to this build invocation.
   */
  const rl = tagged({
    tag: runBuildStep.name,
    l,
  },);
  /**
   Start timestamp for duration measurement.
   */
  const startedAt = performance.now();
  /**
   rolldown config files declared by the package.
   */
  const configs = await listRolldownConfigs(options.packageCwd,);

  if (configs.length === 0)
    return {
      built: false,
      clean: true,
      durationMs: performance.now() - startedAt,
      detail: 'no rolldown configs; build step skipped',
    };

  /* oxlint-disable no-await-in-loop */
  // Configs of one package share output directories; sequential by design.
  for (const config of configs) {
    try {
      await spawn(
        ROLLDOWN_BIN,
        [
          '--configLoader',
          'native',
          '--config',
          config,
        ],
        { cwd: options.packageCwd, },
      );
    }
    catch (error) {
      rl.debug(`build rejected mutant via ${config}: ${String(error,)}`,);
      return {
        built: true,
        clean: false,
        durationMs: performance.now() - startedAt,
        detail: `${String(error,)} ${diagnosticsFromError(error,)}`.trim(),
      };
    }
  }
  /* oxlint-enable no-await-in-loop */

  return {
    built: true,
    clean: true,
    durationMs: performance.now() - startedAt,
    detail: '',
  };
}
