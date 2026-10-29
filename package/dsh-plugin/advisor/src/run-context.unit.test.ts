/**
 Unit tests for the shared Advisor run-context assembly.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createDshAdvisorRunContext,
} from '../dist/final/node/index.mjs';

/**
 Options shape of the run-context builder.
 */
type RunContextOptions = Parameters<typeof createDshAdvisorRunContext>[0];

await describe({
  name: createDshAdvisorRunContext.name,
  children: [
    it({
      name: 'exposes the catalog models as the live scope and carries the main route identity',
      fn: async () => {
        const models = [
          {
            id: 'model-id',
            name: 'Model',
            provider: 'route',
            api: 'openai-completions',
          },
        ];
        const catalog = {
          getAvailable: () => models,
        } as unknown as RunContextOptions['catalog'];
        const transcript = {
          buildContextEntries: () => [],
        } as unknown as RunContextOptions['transcript'];
        const ctx = createDshAdvisorRunContext({
          catalog,
          transcript,
          cwd: '/work',
          mainProvider: 'route',
          mainModel: 'model-id',
        },);
        expect(ctx.modelRegistry,).toBe(catalog,);
        expect(ctx.transcript,).toBe(transcript,);
        expect(ctx.cwd,).toBe('/work',);
        expect(ctx.model,).toEqual({
          provider: 'route',
          id: 'model-id',
        },);
        expect(ctx.getScopedModels?.(),).toBe(models,);
      },
    },),
    it({
      name: 'omits the main identity when the route is incomplete',
      fn: async () => {
        const models: unknown[] = [];
        const catalog = {
          getAvailable: () => models,
        } as unknown as RunContextOptions['catalog'];
        const transcript = {
          buildContextEntries: () => [],
        } as unknown as RunContextOptions['transcript'];
        const withoutModel = createDshAdvisorRunContext({
          catalog,
          transcript,
          cwd: '/work',
          mainProvider: 'route',
        },);
        expect(withoutModel.model,).toBe(undefined,);
        const withoutProvider = createDshAdvisorRunContext({
          catalog,
          transcript,
          cwd: '/work',
          mainModel: 'model-id',
        },);
        expect(withoutProvider.model,).toBe(undefined,);
        expect(withoutProvider.getScopedModels?.(),).toBe(models,);
      },
    },),
  ],
},);
