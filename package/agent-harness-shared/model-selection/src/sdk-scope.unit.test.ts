/** Empty SDK cycle lists must not hide configured model restrictions. @module */
import { mkdir, mkdtemp, rm, writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { resolveEffectiveScope, } from '../dist/final/node/index.mjs';
import { fixtureModel, } from './test-fixtures.ts';

/** Registry candidates with distinct identities. */
const models = ['allowed', 'outside',].map(id => fixtureModel({ provider: 'fixture', id, },));
/** Each configured restriction stays authoritative even when it matches nothing. */
const cases = [
  { name: 'argv precedes settings', argv: ['--models', 'fixture/allowed',], settings: ['fixture/outside',], source: 'argv', ids: ['allowed',], },
  { name: 'settings constrain candidates', argv: [], settings: ['fixture/allowed',], source: 'settings', ids: ['allowed',], },
  { name: 'unmatched argv fails closed', argv: ['--models', 'fixture/missing',], settings: ['fixture/allowed',], source: 'argv', ids: [], },
  { name: 'empty argv fails closed', argv: ['--models', '',], settings: ['fixture/allowed',], source: 'argv', ids: [], },
  { name: 'unmatched settings fail closed', argv: [], settings: ['fixture/missing',], source: 'settings', ids: [], },
  { name: 'empty configured restriction fails closed', argv: [], settings: [], source: 'settings', ids: [], },
  { name: 'absent restrictions use available models', argv: [], settings: undefined, source: 'available', ids: ['allowed', 'outside',], },
] as const;

await describe({ name: 'SDK empty live scope fallback', children: cases.map(testCase => it({
  name: testCase.name,
  fn: async (): Promise<void> => {
    /** Private fixture home prevents real user settings from influencing selection. */
    const root = await mkdtemp(join(scratchRoot(), 'sdk-scope-',),);
    await using cleanup = { [Symbol.asyncDispose]: async (): Promise<void> => {
      await rm(root, { recursive: true, force: true, },);
    }, };
    await mkdir(join(root, '.pi',),);
    await writeFile(join(root, '.pi', 'settings.json',), JSON.stringify({ enabledModels: testCase.settings, },),);
    /** Empty live list mirrors createAgentSession without scopedModels. */
    const scope = await resolveEffectiveScope({
      ctx: { cwd: root, scopedModels: [], modelRegistry: { getAvailable: () => models, }, },
      argv: ['pi', ...testCase.argv,], home: join(root, 'home',),
    },);
    expect(scope.source,).toBe(testCase.source,);
    expect(scope.entries.map(entry => entry.model.id),).toEqual(testCase.ids,);
  },
},)), },);
