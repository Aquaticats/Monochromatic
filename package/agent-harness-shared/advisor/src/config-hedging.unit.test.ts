/** Opt-in overlap and finite scheduling configuration at the real file-loading boundary. @module */
import { mkdir, mkdtemp, rm, writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { DEFAULT_CONFIG, loadMergedConfig, } from '../dist/final/node/index.mjs';

/** Load disposable global and project JSON files through the actual config merger. */
async function configured(global: Record<string, unknown> | string, project: Record<string, unknown> = {}) {
  const root = await mkdtemp(join(scratchRoot(), 'advisor-hedge-config-',),);
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
  const home = join(root, 'home',);
  const cwd = join(root, 'project',);
  const globalDir = join(home, '.pi/agent/extensions',);
  const projectDir = join(cwd, '.pi/extensions',);
  await Promise.all([mkdir(globalDir, { recursive: true, },), mkdir(projectDir, { recursive: true, },),],);
  await Promise.all([
    writeFile(join(globalDir, 'pi-advisor.json',), (typeof global) === 'string' ? global : JSON.stringify(global,),),
    writeFile(join(projectDir, 'pi-advisor.json',), JSON.stringify(project,),),
  ],);
  return await loadMergedConfig({ home, cwd, },);
}

await describe({ name: '', children: [
  it({ name: 'overlap starts disabled and collection grace defaults to thirty seconds', fn: async (): Promise<void> => {
    expect(DEFAULT_CONFIG.hedgingEnabled,).toBe(false,);
    expect(DEFAULT_CONFIG.hedgeDelayMs,).toBeUndefined();
    expect(DEFAULT_CONFIG.collectionGraceMs,).toBe(30_000,);
  }, },),
  it({ name: 'a configured delay alone does not enable duplicate work', fn: async (): Promise<void> => {
    const config = await configured({ hedgeDelayMs: 1_000, },);
    expect(config.hedgingEnabled,).toBe(false,);
  }, },),
  it({ name: 'enabled overlap requires an explicitly configured launch delay', fn: async (): Promise<void> => {
    let caught: unknown;
    try {
      await configured({ hedgingEnabled: true, },);
    }
    catch (error) {
      caught = error;
    }
    expect(caught,).toBeInstanceOf(Error,);
    expect((caught as Error).message,).toContain('requires an explicit positive hedgeDelayMs',);
  }, },),
  it({ name: 'project disablement overrides global opt-in without erasing inherited timing', fn: async (): Promise<void> => {
    const config = await configured({ hedgingEnabled: true, hedgeDelayMs: 1_000, }, { hedgingEnabled: false, collectionGraceMs: 50, },);
    expect(config.hedgingEnabled,).toBe(false,);
    expect(config.hedgeDelayMs,).toBe(1_000,);
    expect(config.collectionGraceMs,).toBe(50,);
  }, },),
  it({ name: 'project overlap settings retain unrelated global reviewer instructions', fn: async (): Promise<void> => {
    const config = await configured({ systemPrompt: 'Retain global reviewer instructions.', hedgeDelayMs: 1_000, }, { hedgingEnabled: true, },);
    expect(config.systemPrompt,).toBe('Retain global reviewer instructions.',);
    const cleared = await configured({ systemPrompt: 'Retain global reviewer instructions.', }, { systemPrompt: '', },);
    expect(cleared.systemPrompt,).toBe('',);
  }, },),
  it({ name: 'project delay and grace override global timing', fn: async (): Promise<void> => {
    const config = await configured({ hedgingEnabled: true, hedgeDelayMs: 1_000, }, { hedgeDelayMs: 2_000, collectionGraceMs: 100, },);
    expect(config.hedgingEnabled,).toBe(true,);
    expect(config.hedgeDelayMs,).toBe(2_000,);
    expect(config.collectionGraceMs,).toBe(100,);
  }, },),
  it({ name: 'rejects an operation timeout overflowing JSON numeric range', fn: async (): Promise<void> => {
    let caught: unknown;
    try {
      await configured('{"timeoutMs":1e400}',);
    }
    catch (error) {
      caught = error;
    }
    expect(caught,).toBeInstanceOf(Error,);
  }, },),
  ...['timeoutMs', 'hedgeDelayMs', 'collectionGraceMs',].flatMap(field => [0, -1, 0.5, 2_147_483_648,].map(value =>
    it({ name: `rejects invalid scheduling value ${field}=${value}`, fn: async (): Promise<void> => {
      let caught: unknown;
      try {
        await configured({ [field]: value, },);
      }
      catch (error) {
        caught = error;
      }
      expect(caught,).toBeInstanceOf(Error,);
      expect((caught as Error).message,).toContain('invalid global config',);
    }, },),
  )),
], },);
